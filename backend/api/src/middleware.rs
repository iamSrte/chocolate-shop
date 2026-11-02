//! HTTP middleware stack shared by `main` and tests.

use std::{any::Any, time::Duration};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Request},
    http::{HeaderName, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use tower_http::{
    catch_panic::CatchPanicLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::{
    config::HttpConfig,
    error::{AppError, ErrorBody},
};

pub const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Wraps `router` with every layer. Outermost first:
/// request ID, tracing span, error normalisation, panic catcher, timeout, body limit.
pub fn apply(router: Router, http: &HttpConfig) -> Router {
    let timeout = http.request_timeout;
    // `Router::layer` wraps outside what is already there, so add innermost first.
    router
        // Applies to body extractors (`Json`, `Bytes`). Raise per route with another
        // `DefaultBodyLimit::max` for uploads.
        .layer(DefaultBodyLimit::max(http.body_limit_bytes))
        .layer(middleware::from_fn(move |req, next| {
            with_timeout(timeout, req, next)
        }))
        .layer(CatchPanicLayer::custom(panic_response))
        .layer(middleware::from_fn(normalize_errors))
        .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER))
        .layer(TraceLayer::new_for_http().make_span_with(|req: &Request| {
            let request_id = req
                .headers()
                .get(&REQUEST_ID_HEADER)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default();
            tracing::info_span!(
                "request",
                method = %req.method(),
                uri = %req.uri(),
                request_id,
            )
        }))
        // Keeps an incoming `x-request-id` from the reverse proxy, else makes a UUID.
        .layer(SetRequestIdLayer::new(REQUEST_ID_HEADER, MakeRequestUuid))
}

async fn with_timeout(timeout: Duration, req: Request, next: Next) -> Response {
    match tokio::time::timeout(timeout, next.run(req)).await {
        Ok(res) => res,
        Err(_) => AppError::Timeout.into_response(),
    }
}

fn panic_response(panic: Box<dyn Any + Send + 'static>) -> Response {
    let detail = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("unknown panic");
    AppError::Internal(anyhow::anyhow!("handler panicked: {detail}")).into_response()
}

/// Makes every error response an [`ErrorBody`]:
/// - plain-text errors from axum (404 route, 405) get a JSON body;
/// - 5xx bodies get the request ID.
///
/// Error responses that are already JSON but not from [`AppError`], such as the health
/// check's 503, pass through untouched.
async fn normalize_errors(req: Request, next: Next) -> Response {
    let request_id = req
        .extensions()
        .get::<RequestId>()
        .and_then(|id| id.header_value().to_str().ok())
        .map(str::to_owned);
    let res = next.run(req).await;
    let status = res.status();
    if !(status.is_client_error() || status.is_server_error()) {
        return res;
    }

    let mut body = match res.extensions().get::<ErrorBody>() {
        Some(body) if status.is_server_error() => body.clone(),
        Some(_) => return res,
        None if is_json(&res) => return res,
        None => ErrorBody::for_status(status),
    };
    if status.is_server_error() {
        body.request_id = request_id;
    }

    let (mut parts, _) = res.into_parts();
    parts.headers.remove(header::CONTENT_LENGTH);
    parts.headers.remove(header::CONTENT_TYPE);
    parts.extensions.insert(body.clone());
    (parts, Json(body)).into_response()
}

fn is_json(res: &Response) -> bool {
    res.headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/json"))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::{get, post},
    };
    use http_body_util::BodyExt;
    use serde::Deserialize;
    use tower::ServiceExt;

    use super::*;
    use crate::extract::ValidJson;

    #[derive(Deserialize, garde::Validate)]
    struct Signup {
        #[garde(length(min = 1))]
        name: String,
    }

    async fn boom() -> &'static str {
        panic!("boom")
    }

    fn app() -> Router {
        let router = Router::new()
            .route("/panic", get(boom))
            .route(
                "/slow",
                get(|| async { tokio::time::sleep(Duration::from_secs(5)).await }),
            )
            .route(
                "/signup",
                post(|ValidJson(form): ValidJson<Signup>| async move { form.name }),
            );
        let http = HttpConfig {
            request_timeout: Duration::from_millis(50),
            body_limit_bytes: 64,
        };
        apply(router, &http)
    }

    async fn send(req: Request<Body>) -> (StatusCode, serde_json::Value) {
        let res = app().oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    fn post_json(body: &'static str) -> Request<Body> {
        Request::post("/signup")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap()
    }

    #[tokio::test]
    async fn panic_becomes_internal_with_request_id() {
        let (status, json) = send(Request::get("/panic").body(Body::empty()).unwrap()).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(json["code"], "internal");
        assert!(json["request_id"].is_string());
        assert!(!json["message"].as_str().unwrap().contains("boom"));
    }

    #[tokio::test]
    async fn slow_handler_times_out() {
        let (status, json) = send(Request::get("/slow").body(Body::empty()).unwrap()).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(json["code"], "timeout");
        assert!(json["request_id"].is_string());
    }

    #[tokio::test]
    async fn invalid_body_lists_fields() {
        let (status, json) = send(post_json(r#"{"name":""}"#)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json["code"], "validation_failed");
        assert!(json["details"]["fields"]["name"].is_array());
        assert!(json.get("request_id").is_none());
    }

    #[tokio::test]
    async fn malformed_json_is_bad_request() {
        let (status, json) = send(post_json("{not json")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["code"], "bad_request");
    }

    #[tokio::test]
    async fn oversized_body_is_rejected() {
        let big = Box::leak(format!(r#"{{"name":"{}"}}"#, "x".repeat(100)).into_boxed_str());
        let (status, json) = send(post_json(big)).await;
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(json["code"], "payload_too_large");
    }
}

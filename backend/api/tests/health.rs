use api::AppState;
use axum::{body::Body, http::Request, http::StatusCode};
use http_body_util::BodyExt;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

fn app() -> axum::Router {
    // Lazy pool never connects unless a handler queries it.
    let db = PgPoolOptions::new()
        .connect_lazy("postgres://unused@localhost/unused")
        .unwrap();
    api::router(AppState { db })
}

#[tokio::test]
async fn health_returns_ok() {
    let res = app()
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn openapi_lists_health_under_api_prefix() {
    let spec = api::openapi();
    assert!(spec.paths.paths.contains_key("/api/health"));
}

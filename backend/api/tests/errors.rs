//! Every non-2xx response has the `ErrorBody` shape, and every response has a request ID.

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::TestApp;

#[tokio::test]
async fn unknown_route_is_json_not_found() {
    let res = TestApp::without_db().get("/api/nope").await;

    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.json["code"], "not_found");
}

#[tokio::test]
async fn wrong_method_is_json_method_not_allowed() {
    let res = TestApp::without_db()
        .post_json("/api/health", &serde_json::json!({}))
        .await;

    assert_eq!(res.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(res.json["code"], "method_not_allowed");
}

#[tokio::test]
async fn response_gets_request_id() {
    let res = TestApp::without_db().get("/api/health").await;

    assert!(res.headers.contains_key("x-request-id"));
}

#[tokio::test]
async fn incoming_request_id_is_kept() {
    let req = Request::get("/api/health")
        .header("x-request-id", "from-proxy")
        .body(Body::empty())
        .unwrap();
    let res = TestApp::without_db().send(req).await;

    assert_eq!(res.headers["x-request-id"], "from-proxy");
}

#[tokio::test]
async fn ready_without_db_keeps_its_own_body() {
    let res = TestApp::without_db().get("/api/health/ready").await;

    assert_eq!(res.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(res.json["status"], "unavailable");
}

#[tokio::test]
async fn openapi_exposes_error_codes() {
    let spec = serde_json::to_value(api::openapi()).unwrap();
    let codes = &spec["components"]["schemas"]["ErrorCode"]["enum"];

    assert!(
        codes
            .as_array()
            .unwrap()
            .contains(&"validation_failed".into())
    );
}

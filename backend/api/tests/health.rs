mod common;

use axum::http::StatusCode;
use common::TestApp;
use sqlx::PgPool;

#[tokio::test]
async fn health_returns_ok() {
    let res = TestApp::without_db().get("/api/health").await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json["status"], "ok");
}

#[sqlx::test]
async fn ready_returns_ok_with_db(pool: PgPool) {
    let res = TestApp::new(pool).get("/api/health/ready").await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json["status"], "ok");
}

#[tokio::test]
async fn openapi_lists_health_under_api_prefix() {
    let spec = api::openapi();
    assert!(spec.paths.paths.contains_key("/api/health"));
}

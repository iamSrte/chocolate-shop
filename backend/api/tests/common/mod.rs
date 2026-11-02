//! Shared test harness: the real router and middleware, driven in-process.
//!
//! With a DB: `#[sqlx::test] async fn t(pool: PgPool) { let app = TestApp::new(pool); }`.
//! Without:   `let app = TestApp::without_db();`.

// Each test binary uses a different subset of these helpers.
#![allow(dead_code)]

use std::time::Duration;

use api::{AppState, config::HttpConfig};
use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;

pub struct TestApp {
    router: Router,
}

pub struct TestResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    /// Parsed JSON body, or `Null` when the body is empty or not JSON.
    pub json: serde_json::Value,
}

impl TestApp {
    pub fn new(db: PgPool) -> Self {
        Self {
            router: api::router(AppState { db }, &HttpConfig::default()),
        }
    }

    /// For tests that never touch the DB. The lazy pool fails only if a handler queries it.
    pub fn without_db() -> Self {
        let db = PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(1))
            .connect_lazy("postgres://unused@localhost/unused")
            .unwrap();
        Self::new(db)
    }

    pub async fn get(&self, uri: &str) -> TestResponse {
        self.send(Request::get(uri).body(Body::empty()).unwrap())
            .await
    }

    pub async fn post_json(&self, uri: &str, body: &impl Serialize) -> TestResponse {
        let req = Request::post(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(body).unwrap()))
            .unwrap();
        self.send(req).await
    }

    pub async fn send(&self, req: Request<Body>) -> TestResponse {
        let res = self.router.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let headers = res.headers().clone();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        TestResponse {
            status,
            headers,
            json,
        }
    }
}

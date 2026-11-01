pub mod config;
pub mod error;
pub mod extract;
mod health;
pub mod middleware;
pub mod telemetry;

use axum::{Json, Router, routing::get};
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    config::HttpConfig,
    error::{ErrorBody, ErrorCode, ErrorDetails},
};

#[derive(OpenApi)]
#[openapi(
    info(title = "Chocolate Shop API"),
    components(schemas(ErrorBody, ErrorCode, ErrorDetails))
)]
struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

/// Every route lives under this prefix; the reverse proxy sends `/api/*` here.
const PREFIX: &str = "/api";

/// HTTP app: all API routes, the spec at `/api/openapi.json`, and the full middleware stack.
pub fn router(state: AppState, http: &HttpConfig) -> Router {
    let (router, spec) = api().split_for_parts();
    let router = router
        .route(
            &format!("{PREFIX}/openapi.json"),
            get(move || async move { Json(spec) }),
        )
        .with_state(state);
    middleware::apply(router, http)
}

/// OpenAPI spec for the API. Source of truth for `frontend/api-client`.
pub fn openapi() -> utoipa::openapi::OpenApi {
    api().into_openapi()
}

fn api() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi()).nest(PREFIX, health::router())
}

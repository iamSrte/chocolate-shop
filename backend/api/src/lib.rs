pub mod config;
pub mod error;
mod health;

use axum::{Json, Router, routing::get};
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::error::ErrorBody;

#[derive(OpenApi)]
#[openapi(info(title = "Chocolate Shop API"), components(schemas(ErrorBody)))]
struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

/// Every route lives under this prefix; the reverse proxy sends `/api/*` here.
const PREFIX: &str = "/api";

/// HTTP router with all API routes, plus the spec served at `/api/openapi.json`.
pub fn router(state: AppState) -> Router {
    let (router, spec) = api().split_for_parts();
    router
        .route(
            &format!("{PREFIX}/openapi.json"),
            get(move || async move { Json(spec) }),
        )
        .with_state(state)
}

/// OpenAPI spec for the API. Source of truth for `frontend/api-client`.
pub fn openapi() -> utoipa::openapi::OpenApi {
    api().into_openapi()
}

fn api() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi()).nest(PREFIX, health::router())
}

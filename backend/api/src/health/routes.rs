use axum::{Json, extract::State, http::StatusCode};

use super::dto::Health;
use crate::AppState;

/// Liveness: the process is up. Does not touch the database.
#[utoipa::path(get, path = "/health", tag = "health", responses((status = 200, body = Health)))]
pub async fn live() -> Json<Health> {
    Json(Health { status: "ok" })
}

/// Readiness: the database answers.
#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "health",
    responses((status = 200, body = Health), (status = 503, body = Health))
)]
pub async fn ready(State(state): State<AppState>) -> (StatusCode, Json<Health>) {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => (StatusCode::OK, Json(Health { status: "ok" })),
        Err(err) => {
            tracing::warn!(%err, "database not ready");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(Health {
                    status: "unavailable",
                }),
            )
        }
    }
}

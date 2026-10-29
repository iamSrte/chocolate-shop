mod dto;
mod routes;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(routes::live))
        .routes(routes!(routes::ready))
}

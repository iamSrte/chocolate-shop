use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

/// Error type for every handler. Maps to an HTTP status and an [`ErrorBody`].
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

/// JSON error body. The frontend branches on `code`; `message` is for logs and debugging.
#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    /// Stable snake_case key, e.g. `not_found`, `box_not_full`.
    pub code: &'static str,
    pub message: String,
}

impl AppError {
    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            Self::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();
        let message = if status.is_server_error() {
            tracing::error!(err = %self, "request failed");
            // Never leak internals to clients.
            "internal error".to_owned()
        } else {
            self.to_string()
        };
        (status, Json(ErrorBody { code, message })).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_maps_to_404() {
        let res = AppError::NotFound.into_response();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn database_error_maps_to_500() {
        let res = AppError::Database(sqlx::Error::RowNotFound).into_response();
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}

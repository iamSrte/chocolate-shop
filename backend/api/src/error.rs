//! One error type for every handler. See ADR 0004.
//!
//! Domain module errors get their own `AppError` variant via `#[from]`, and are mapped to a
//! status and [`ErrorCode`] in [`AppError::status_and_code`]. Never map elsewhere.

use std::collections::BTreeMap;

use axum::{
    Json,
    extract::rejection::{JsonRejection, PathRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

/// Stable machine-readable error key. The frontend branches on it.
///
/// Append-only: once shipped, never rename or reuse a variant. Name domain codes with
/// `GLOSSARY.md` terms, e.g. `box_not_full`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    NotFound,
    MethodNotAllowed,
    PayloadTooLarge,
    /// The request took longer than `REQUEST_TIMEOUT_SECS`.
    Timeout,
    /// Malformed request: bad JSON, path or query.
    BadRequest,
    /// Well-formed request that fails input validation. See `details.fields`.
    ValidationFailed,
    /// Unexpected server failure. Details are only in the logs.
    Internal,
}

/// JSON body of every non-2xx response.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ErrorBody {
    pub code: ErrorCode,
    /// For logs and debugging. Never shown to customers as-is.
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ErrorDetails>,
    /// Set on 5xx only, so a customer can quote it. Matches the `x-request-id` header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ErrorBody {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            request_id: None,
        }
    }

    /// Body for an error response that did not come from [`AppError`], e.g. axum's 405.
    pub fn for_status(status: StatusCode) -> Self {
        let code = match status {
            StatusCode::NOT_FOUND => ErrorCode::NotFound,
            StatusCode::METHOD_NOT_ALLOWED => ErrorCode::MethodNotAllowed,
            StatusCode::PAYLOAD_TOO_LARGE => ErrorCode::PayloadTooLarge,
            s if s.is_server_error() => ErrorCode::Internal,
            _ => ErrorCode::BadRequest,
        };
        let message = status.canonical_reason().unwrap_or("error").to_lowercase();
        Self::new(code, message)
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ErrorDetails {
    /// Messages per invalid field, keyed by field path, e.g. `address.city`.
    pub fields: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("payload too large")]
    PayloadTooLarge,
    #[error("request timed out")]
    Timeout,
    #[error("{0}")]
    BadRequest(String),
    #[error("validation failed")]
    Validation(#[from] garde::Report),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn status_and_code(&self) -> (StatusCode, ErrorCode) {
        match self {
            Self::NotFound => (StatusCode::NOT_FOUND, ErrorCode::NotFound),
            Self::PayloadTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, ErrorCode::PayloadTooLarge),
            Self::Timeout => (StatusCode::SERVICE_UNAVAILABLE, ErrorCode::Timeout),
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, ErrorCode::BadRequest),
            Self::Validation(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorCode::ValidationFailed,
            ),
            Self::Database(_) | Self::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::Internal)
            }
        }
    }

    fn details(&self) -> Option<ErrorDetails> {
        let Self::Validation(report) = self else {
            return None;
        };
        let mut fields = BTreeMap::<String, Vec<String>>::new();
        for (path, error) in report.iter() {
            fields
                .entry(path.to_string())
                .or_default()
                .push(error.message().to_owned());
        }
        Some(ErrorDetails { fields })
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();
        let body = if matches!(code, ErrorCode::Internal) {
            // `{:#}` prints the full anyhow context chain.
            tracing::error!(
                error = format!("{:#}", anyhow::Error::from(self)),
                "request failed"
            );
            // Never leak internals to clients.
            ErrorBody::new(code, "internal error")
        } else {
            ErrorBody {
                details: self.details(),
                ..ErrorBody::new(code, self.to_string())
            }
        };
        // The middleware reads this to add `request_id` to 5xx bodies.
        let mut res = (status, Json(body.clone())).into_response();
        res.extensions_mut().insert(body);
        res
    }
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
            Self::PayloadTooLarge
        } else {
            Self::BadRequest(rejection.body_text())
        }
    }
}

impl From<PathRejection> for AppError {
    fn from(rejection: PathRejection) -> Self {
        Self::BadRequest(rejection.body_text())
    }
}

impl From<QueryRejection> for AppError {
    fn from(rejection: QueryRejection) -> Self {
        Self::BadRequest(rejection.body_text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status_and_body(err: AppError) -> (StatusCode, ErrorBody) {
        let res = err.into_response();
        let body = res.extensions().get::<ErrorBody>().cloned().unwrap();
        (res.status(), body)
    }

    #[test]
    fn not_found_maps_to_404() {
        let (status, body) = status_and_body(AppError::NotFound);
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body.code, ErrorCode::NotFound);
    }

    #[test]
    fn database_error_maps_to_500_without_leaking() {
        let (status, body) = status_and_body(AppError::Database(sqlx::Error::RowNotFound));
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body.code, ErrorCode::Internal);
        assert_eq!(body.message, "internal error");
    }

    #[test]
    fn internal_error_hides_context() {
        let err = anyhow::anyhow!("secret detail").context("loading box");
        let (_, body) = status_and_body(AppError::Internal(err));
        assert!(!body.message.contains("secret"));
    }

    #[test]
    fn validation_lists_fields() {
        #[derive(garde::Validate)]
        struct Form {
            #[garde(length(min = 1))]
            name: String,
        }
        let report = garde::Validate::validate(&Form {
            name: String::new(),
        })
        .unwrap_err();
        let (status, body) = status_and_body(AppError::Validation(report));
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body.code, ErrorCode::ValidationFailed);
        assert!(body.details.unwrap().fields.contains_key("name"));
    }

    #[test]
    fn code_serializes_snake_case() {
        let json = serde_json::to_string(&ErrorCode::ValidationFailed).unwrap();
        assert_eq!(json, "\"validation_failed\"");
    }
}

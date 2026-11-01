//! Extractors that reject with [`AppError`], so bad input renders as `{ code, message }`.
//! Use these instead of the `axum::extract` / `axum::Json` versions.

use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};

use crate::error::AppError;

#[derive(Debug, FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct Json<T>(pub T);

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

#[derive(Debug, FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(AppError))]
pub struct Path<T>(pub T);

#[derive(Debug, FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(AppError))]
pub struct Query<T>(pub T);

/// JSON body checked with `garde::Validate`. Fails with `validation_failed` and per-field details.
#[derive(Debug)]
pub struct ValidJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + garde::Validate,
    T::Context: Default,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(Self(value))
    }
}

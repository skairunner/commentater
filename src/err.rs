use anyhow::Error;
use axum::http::StatusCode;
use axum::response::{ErrorResponse, IntoResponse, Response};
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("internal error: {0}")]
    InternalError(Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong!".to_owned(),
            ),
        };

        (status, message).into_response()
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::InternalError(value.into())
    }
}

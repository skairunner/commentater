use crate::response::AppJsonError;
use anyhow::Error;
use axum::http::StatusCode;
use axum::response::{ErrorResponse, IntoResponse, Response};
use axum::Json;
use base64;
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("internal error: {0}")]
    InternalError(Error),
    #[error("bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::InternalError(error) => {
                log::error!("Internal error: {error}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong!".to_owned(),
                )
            }
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        (status, Json(AppJsonError { error: message })).into_response()
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::InternalError(value.into())
    }
}

impl From<base64::DecodeError> for AppError {
    fn from(value: base64::DecodeError) -> Self {
        let msg = value.to_string();
        Self::BadRequest(format!("Invalid base64 parameter: {msg}"))
    }
}

impl From<std::str::Utf8Error> for AppError {
    fn from(value: std::str::Utf8Error) -> Self {
        let i = value.valid_up_to();
        Self::BadRequest(format!("Decoded string was not valid utf-8 after byte {i}"))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self::InternalError(Error::from(value))
    }
}

impl From<url::ParseError> for AppError {
    fn from(value: url::ParseError) -> Self {
        Self::BadRequest(format!("Invalid url: {value}"))
    }
}

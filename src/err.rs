use anyhow::Error;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use base64;
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("internal error: {0}")]
    InternalError(Error),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not found")]
    GenericNotFound,
    #[error("not found")]
    NotFound(String, i64),
}

impl AppError {
    pub fn from_sql(object: &str, object_id: &i64) -> impl FnOnce(sqlx::Error) -> Self {
        let o = object.to_string();
        let i = *object_id;
        move |e| match e {
            sqlx::Error::RowNotFound => Self::NotFound(o, i),
            e => Self::InternalError(e.into()),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::InternalError(error) => {
                log::error!("Internal error: {error:?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong!".to_owned(),
                )
            }
            Self::GenericNotFound => (
                StatusCode::NOT_FOUND,
                "The object you requested was not found".to_owned(),
            ),
            Self::NotFound(o, id) => (StatusCode::NOT_FOUND, format!("{o} {id}")),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let reason = status.canonical_reason().unwrap_or("Error");
        (
            status,
            format!("<h1>{reason}</h1><div>{message}</div>"),
        ).into_response()
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

impl From<Error> for AppError {
    fn from(value: Error) -> Self {
        Self::InternalError(value)
    }
}

impl From<tera::Error> for AppError {
    fn from(value: tera::Error) -> Self {
        Self::InternalError(Error::from(value))
    }
}

impl From<url::ParseError> for AppError {
    fn from(value: url::ParseError) -> Self {
        Self::BadRequest(format!("Invalid url: {value}"))
    }
}

impl From<tower_sessions::session::Error> for AppError {
    fn from(value: tower_sessions::session::Error) -> Self {
        Self::InternalError(Error::from(value))
    }
}

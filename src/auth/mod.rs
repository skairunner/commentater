use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

/// The user id of the active user
#[derive(Default, Deserialize, Serialize)]
pub struct UserState {
    pub user_id: Option<i64>,
}

impl UserState {
    pub const KEY: &'static str = "USER_STATE";
}

impl UserState {
    fn log_error_and_500<E: std::error::Error>(e: E) -> (StatusCode, &'static str) {
        log::error!("{e:?}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    }
}

impl<S> FromRequestParts<S> for UserState
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;
        let user_state: UserState = session
            .get(Self::KEY)
            .await
            .map_err(Self::log_error_and_500)?
            .unwrap_or_default();
        Ok(user_state)
    }
}

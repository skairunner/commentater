use axum::extract::State;
use crate::auth::UserState;
use crate::err::AppError;
use crate::templates::TEMPLATES;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{debug_handler, Form};
use serde::Deserialize;
use sqlx::PgPool;
use tera::Context;
use tower_sessions::Session;
use crate::db::user::get_user_id_or_insert;
use crate::req::{get_wa_client_builder};
use crate::worldanvil_api::get_user_identity;
use crate::worldanvil_api::schema::IdentityResult;

#[derive(Deserialize)]
pub struct ApiKeyForm {
    pub api_key: String,
}

/// This is the login page.
pub async fn login_get(UserState { user_id }: UserState) -> Result<Response, AppError> {
    if user_id.is_some() {
        Ok(Redirect::temporary("/").into_response())
    } else {
        let html = TEMPLATES.render("login.html", &Default::default())?;
        Ok(Html(html).into_response())
    }
}

/// Handle the submission of the login
pub async fn login_post(
    session: Session,
    State(pool): State<PgPool>,
    Form(ApiKeyForm { api_key }): Form<ApiKeyForm>,
) -> Result<Response, AppError> {
    let client = get_wa_client_builder(&api_key).build()?;
    // Try using the API key.
    let info = match get_user_identity(&client).await? {
        IdentityResult::Identified(i) => i,
        IdentityResult::NotIdentified(e) => {
            let mut context = Context::new();
            context.insert("error", &format!("The API key wasn't recognized: {0}", e.error));
            let html = TEMPLATES.render("login.html", &context)?;
            return Ok(Html(html).into_response())
        }
    };
    // Find the right user id for the api key, or insert it
    let user = get_user_id_or_insert(&pool, &api_key, &info.username, &info.id).await?;
    // Then update the user's session
    session.insert(UserState::KEY, UserState {
        user_id: Some(user.id),
    }).await?;
    // Finally render the response
    let mut context = Context::new();
    context.insert("username", &info.username);
    let html = TEMPLATES.render("loggedin.html", &context)?;
    Ok(Html(html).into_response())
}

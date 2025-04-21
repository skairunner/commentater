use crate::auth::UserState;
use crate::db::article::{
    get_article, get_article_conn, get_article_details, get_unqueued_article_ids, register_articles,
};
use crate::db::comments::get_comments;
use crate::db::queue::{article_is_queued, insert_tasks};
use crate::db::user::get_user;
use crate::db::world::get_world;
use crate::err::AppError;
use crate::req::get_wa_client_builder;
use crate::templates::TEMPLATES;
use crate::worldanvil_api::world_list_articles;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use sqlx::{Acquire, PgPool};
use tera::Context;

pub async fn list_comments(
    State(pool): State<PgPool>,
    Path((world_id, article_id)): Path<(i64, i64)>,
    user_state: UserState,
) -> Result<Response, AppError> {
    let user_id = match user_state.user_id.clone() {
        Some(id) => id,
        None => return Ok(Redirect::to("/login").into_response()),
    };
    let mut context = Context::new();
    user_state.insert_context(&mut context);
    let world = get_world(&pool, &user_id, &world_id).await?;
    context.insert("world", &world);
    let article = get_article_details(&pool, &article_id, &user_id).await?;
    context.insert("article", &article);
    let comments = get_comments(&pool, article_id, user_id).await?;
    context.insert("comments", &comments);
    let html = TEMPLATES.render("article.html", &context)?;
    Ok(Html(html).into_response())
}

pub async fn fetch_articles(
    Path(world_id): Path<i64>,
    State(pool): State<PgPool>,
    user_state: UserState,
) -> Result<Response, AppError> {
    if user_state.user_id.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }

    let mut context = Context::new();
    user_state.insert_context(&mut context);

    let user_id = match user_state.user_id.clone() {
        Some(id) => id,
        None => {
            let html = TEMPLATES.render("base.html", &context)?;
            return Ok(Html(html).into_response());
        }
    };
    let user_info = get_user(&pool, &user_id)
        .await
        .map_err(AppError::from_sql("user", &user_id))?;

    let world = get_world(&pool, &user_id, &world_id)
        .await
        .map_err(AppError::from_sql("world", &world_id))?;

    let client = get_wa_client_builder(&user_info.api_key).build()?;
    // TODO: Cooldown on re-fetching articles
    let articles = world_list_articles(&client, &world.worldanvil_id).await?;
    let mut urls = Vec::new();
    let mut titles = Vec::new();
    let mut wa_ids = Vec::new();
    articles.into_iter().for_each(|a| {
        urls.push(a.url);
        titles.push(a.title);
        wa_ids.push(a.id);
    });
    register_articles(user_id, world.id, &urls, &titles, &wa_ids, &pool).await?;
    Ok(Redirect::to(&format!("/world/{world_id}/")).into_response())
}

pub async fn queue_all_articles(
    State(pool): State<PgPool>,
    Path(world_id): Path<i64>,
    user_state: UserState,
) -> Result<Response, AppError> {
    let mut context = Context::new();
    user_state.insert_context(&mut context);
    let user_id = match user_state.user_id {
        Some(id) => id,
        None => return Ok(Redirect::to("/login").into_response()),
    };
    let article_ids = get_unqueued_article_ids(&pool, &user_id, &world_id).await?;
    let mut db_conn = pool.acquire().await?;
    let conn = db_conn.acquire().await?;
    insert_tasks(&user_id, &article_ids, conn).await?;
    let len = article_ids.len();
    context.insert("world_id", &world_id);
    context.insert("count", &len);
    let html = TEMPLATES.render("queue_all_articles.html", &context)?;
    Ok(Html(html).into_response())
}

/// Queue a specific article for re-indexing.
pub async fn queue_one_article(
    State(pool): State<PgPool>,
    Path((world_id, article_id)): Path<(i64, i64)>,
    user_state: UserState,
) -> Result<Response, AppError> {
    let mut context = Context::new();
    user_state.insert_context(&mut context);
    let user_id = match user_state.user_id {
        Some(id) => id,
        None => return Ok(Redirect::to("/login").into_response()),
    };
    let mut tx = pool.begin().await?;
    // Check that the article exists first
    get_article_conn(tx.acquire().await?, article_id, user_id)
        .await
        .map_err(AppError::from_sql("article", &article_id))?;
    // Check that it isn't already queued
    if article_is_queued(&user_id, &article_id, tx.acquire().await?).await? {
        context.insert("already_queued", &true);
    } else {
        insert_tasks(&user_id, &[article_id], tx.acquire().await?).await?;
        tx.commit().await?;
    }
    let html = TEMPLATES.render("queue_one_article.html", &context)?;
    Ok(Html(html).into_response())
}

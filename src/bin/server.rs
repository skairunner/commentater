use anyhow;
use axum::extract::{Path, Request, State};
use axum::http::Method;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::post;
use axum::{response::Html, routing::get, Router, ServiceExt};
use dotenv::dotenv;
use libtater::auth::UserState;
use libtater::db::article::get_articles_and_status;
use libtater::db::get_connection_options;
use libtater::db::queue::get_queue_length;
use libtater::db::schema::WorldInsert;
use libtater::db::user::get_user;
use libtater::db::world::{get_world, get_worlds, upsert_worlds};
use libtater::err::AppError;
use libtater::req::get_wa_client_builder;
use libtater::routes::article;
use libtater::routes::login::{login_get, login_post};
use libtater::setup_logging;
use libtater::templates::TEMPLATES;
use libtater::worldanvil_api::get_worlds_for_user;
use sqlx::PgPool;
use tera::Context;
use time::Duration;
use tokio::task::AbortHandle;
use tower::Layer;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::services::ServeDir;
use tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    use tokio::signal;
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { deletion_task_abort_handle.abort() },
        _ = terminate => { deletion_task_abort_handle.abort() },
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    setup_logging("log/server.log")?;

    let pool = PgPool::connect_with(get_connection_options()).await?;

    // Session stuff
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;
    let session_deleter = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(2)));

    let app = Router::new()
        .route("/", get(list_worlds).post(list_worlds))
        .route("/world/{world_id}", get(list_articles))
        .route(
            "/world/{world_id}/fetch_articles",
            get(article::fetch_articles),
        )
        .route(
            "/world/{world_id}/article/{article_id}",
            get(article::list_comments),
        )
        .route(
            "/world/{world_id}/article/{article_id}/enqueue",
            post(article::queue_one_article),
        )
        .route("/session", get(check_session))
        .route(
            "/world/{world_id}/queue_all",
            get(article::queue_all_articles),
        )
        .route("/login", get(login_get).post(login_post))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(pool)
        .layer(session_layer);
    let app = NormalizePathLayer::trim_trailing_slash().layer(app);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        .with_graceful_shutdown(shutdown_signal(session_deleter.abort_handle()))
        .await?;

    session_deleter.await??;
    Ok(())
}

/// The home page.
/// List the worlds that are currently known to Commentater.
async fn list_worlds(
    State(pool): State<PgPool>,
    user_state: UserState,
    method: Method,
) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    if let Some(user_id) = &user_state.user_id {
        // If we're in the POST method, update the worlds before fetching them.
        if method == Method::POST {
            let user = get_user(&pool, user_id).await?;
            let client = get_wa_client_builder(&user.api_key).build()?;
            let worlds = get_worlds_for_user(&client, &user.worldanvil_id).await?;
            let worlds = worlds
                .into_iter()
                .map(|world| WorldInsert {
                    worldanvil_id: world.id,
                    name: world.title,
                })
                .collect();
            upsert_worlds(&pool, user_id, worlds).await?;
        }
        let worlds = get_worlds(&pool, user_id).await?;
        context.insert("worlds", &worlds);
    }
    user_state.insert_context(&mut context);
    let queue_length = get_queue_length(&mut *pool.acquire().await?).await?;
    context.insert("queue_length", &queue_length);

    let html = TEMPLATES.render("home.html", &context)?;
    Ok(Html(html))
}

/// List the articles for the world.
async fn list_articles(
    State(pool): State<PgPool>,
    Path(world_id): Path<i64>,
    user_state: UserState,
) -> Result<Response, AppError> {
    if user_state.user_id.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }

    let mut context = Context::new();
    user_state.insert_context(&mut context);
    let queue_length = get_queue_length(&mut *pool.acquire().await?).await?;
    context.insert("queue_length", &queue_length);

    if let Some(user_id) = &user_state.user_id {
        let world = get_world(&pool, user_id, &world_id)
            .await
            .map_err(AppError::from_sql("world", &world_id))?;
        let articles = get_articles_and_status(user_id, &world_id, &pool).await?;
        context.insert("world", &world);
        context.insert("articles", &articles);
    }

    let html = TEMPLATES.render("list_articles.html", &context)?;
    Ok(Html(html).into_response())
}

async fn check_session(UserState { user_id, .. }: UserState) -> Result<Html<String>, AppError> {
    match user_id {
        Some(id) => Ok(Html(format!("You are user {id}"))),
        None => Ok(Html("You are not logged in".to_string())),
    }
}

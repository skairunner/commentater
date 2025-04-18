use std::fs::File;
use anyhow;
use axum::extract::{Path, State};
use axum::{response::Html, routing::get, Router};
use dotenv::dotenv;
use lazy_static::lazy_static;
use libtater::db::article::{get_article_ids, register_articles};
use libtater::db::get_connection_options;
use libtater::db::queue::insert_tasks;
use libtater::err::AppError;
use libtater::req::get_wa_client_builder;
use libtater::worldanvil_api::world_list_articles;
use libtater::{TEST_USER_ID, TEST_WORLD_ID};
use simplelog::{CombinedLogger, TermLogger, WriteLogger};
use sqlx::PgPool;
use tera::{Context, Tera};
use time::Duration;
use tokio::task::AbortHandle;
use tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use libtater::auth::UserState;
use libtater::log_config::default_log_config;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html"]);
        tera
    };
}

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
    CombinedLogger::init(vec![
        TermLogger::new(
            simplelog::LevelFilter::Info,
            default_log_config(),
            Default::default(),
            Default::default(),
        ),
        WriteLogger::new(
            simplelog::LevelFilter::Info,
            Default::default(),
            File::create("log/server.log").unwrap(),
        )
    ])?;

    let pool = PgPool::connect_with(get_connection_options()).await?;

    // Session stuff
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;
    let session_deleter = tokio::task::spawn(
        session_store.clone().continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(2)));

    let app = Router::new()
        .route("/", get(hello))
        .route("/session", get(check_session))
        .route("/register_world/{world_id}", get(register_world))
        .route("/articles/queue_all", get(queue_all_articles))
        .with_state(pool)
        .layer(session_layer);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(session_deleter.abort_handle()))
        .await?;

    session_deleter.await??;
    Ok(())
}

async fn hello() -> Result<Html<String>, AppError> {
    Ok(Html("<h1>Commentater</h1>".to_string()))
}

async fn register_world(
    Path(world_id): Path<String>,
    State(pool): State<PgPool>,
) -> Result<Html<String>, AppError> {
    let test_user_token = std::env::var("TEST_USER_KEY").unwrap();
    let client = get_wa_client_builder(&test_user_token).build()?;
    // TODO: Filter articles to only get public
    // Also TODO: Cooldown on re-fetching articles
    let articles = world_list_articles(&client, &world_id).await?;
    let mut urls = Vec::new();
    let mut titles = Vec::new();
    articles.iter().for_each(|a| {
        urls.push(a.url.clone());
        titles.push(a.title.clone());
    });
    register_articles(TEST_USER_ID, TEST_WORLD_ID, &urls, &titles, &pool).await?;
    let mut context = Context::new();
    context.insert("articles", &articles);
    let html = TEMPLATES.render("list_articles.html", &context)?;
    Ok(Html(html))
}

async fn queue_all_articles(State(pool): State<PgPool>) -> Result<Html<String>, AppError> {
    let article_ids = get_article_ids(&pool, &TEST_USER_ID).await?;
    insert_tasks(&TEST_USER_ID, &article_ids, &pool).await?;
    let len = article_ids.len();
    Ok(Html(format!("Queued {len} articles")))
}

async fn check_session(UserState { user_id }: UserState) -> Result<Html<String>, AppError> {
    match user_id {
        Some(id) => Ok(Html(format!("You are user {id}"))),
        None => Ok(Html("You are not logged in".to_string())),
    }
}

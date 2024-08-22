use anyhow;
use axum::extract::{Path, State};
use axum::routing::post;
use axum::{
    response::{Html, Json},
    routing::get,
    Router,
};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use dotenv::dotenv;
use libtater::db::{get_connection_options, query};
use libtater::err::AppError;
use libtater::req::{check_url_valid, get_default_reqwest};
use libtater::response::RegisterArticleResponse;
use libtater::{TEST_USER_ID, TEST_WORLD_ID};
use simplelog::TermLogger;
use sqlx::PgPool;
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    TermLogger::init(
        simplelog::LevelFilter::Info,
        Default::default(),
        Default::default(),
        Default::default(),
    )?;

    let pool = PgPool::connect_with(get_connection_options()).await?;

    let app = Router::new()
        .route("/", get(hello))
        .route("/api/article/:article_url/register", post(register_article))
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn hello() -> Result<Html<String>, AppError> {
    Ok(Html("<h1>Hello World</h1>".to_string()))
}

async fn register_article(
    Path(article_url): Path<String>,
    State(pool): State<PgPool>,
) -> Result<Json<RegisterArticleResponse>, AppError> {
    let article_url = URL_SAFE.decode(article_url)?;
    let article_url = std::str::from_utf8(&article_url)?;
    check_url_valid(article_url)?;
    let r = get_default_reqwest().get(article_url).send().await?;
    if r.status() != 200 {
        return Err(AppError::BadRequest(format!(
            "Could not find article at url {article_url}"
        )));
    }
    let id = query::register_article(TEST_USER_ID, TEST_WORLD_ID, article_url, &pool).await?;
    Ok(Json(RegisterArticleResponse { id }))
}

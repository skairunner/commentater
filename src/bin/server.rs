use anyhow;
use axum::extract::{Path, State};
use axum::{
    response::{Html, Json},
    routing::get,
    Router,
};
use dotenv::dotenv;
use lazy_static::lazy_static;
use libtater::db::query::{get_article_ids, register_articles};
use libtater::db::queue::insert_tasks;
use libtater::db::{get_connection_options, query};
use libtater::err::AppError;
use libtater::req::get_wa_client_builder;
use libtater::worldanvil_api::world_list_articles;
use libtater::{TEST_USER_ID, TEST_WORLD_ID};
use simplelog::TermLogger;
use sqlx::PgPool;
use tera::{Context, Tera};
use tokio;

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
        .route("/register_world/:world_id", get(register_world))
        .route("/articles/queue_all", get(queue_all_articles))
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app).await?;

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

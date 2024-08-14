use anyhow;
use axum::{response::Html, routing::get, Router};
use dotenv::dotenv;
use libtater::err::AppError;
use libtater::req::get_default_reqwest;
use simplelog::TermLogger;
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

    let app = Router::new().route("/", get(hello));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn hello() -> Result<Html<String>, AppError> {
    let req = get_default_reqwest();
    let r = req
        .get("https://webhook.site/7f497a13-aba0-4887-b445-6f11bcca3c99")
        .send()
        .await?;
    let body = r.text().await?;
    Ok(Html(body))
}

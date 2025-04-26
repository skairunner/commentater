use dotenv::var as envvar;
use simplelog::{CombinedLogger, TermLogger, WriteLogger};
use std::fs::OpenOptions;

pub mod article_updater;
pub mod auth;
mod dateutil;
pub mod db;
pub mod err;
pub mod log_config;
pub mod parser;
pub mod req;
pub mod response;
pub mod routes;
pub mod templates;
pub mod worldanvil_api;

pub static TEST_USER_ID: i64 = 5;
pub static TEST_WORLD_ID: i64 = 5;

pub fn setup_logging(file_name: &str) -> anyhow::Result<()> {
    let log_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(file_name)?;

    let debug = envvar("DEBUG").map(|v| !v.is_empty()).unwrap_or(false);
    let log_level = if debug {
        simplelog::LevelFilter::Debug
    } else {
        simplelog::LevelFilter::Info
    };

    CombinedLogger::init(vec![
        TermLogger::new(
            log_level,
            Default::default(),
            Default::default(),
            Default::default(),
        ),
        WriteLogger::new(log_level, Default::default(), log_file),
    ])?;

    Ok(())
}

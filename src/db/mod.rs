use dotenv::var as envvar;
use sqlx::postgres::PgConnectOptions;

pub mod article;
pub mod comments;
mod pgacquire;
pub mod query;
pub mod queue;
pub mod schema;
pub mod test_queries;
pub mod user;
pub mod world;

pub fn get_connection_options() -> PgConnectOptions {
    PgConnectOptions::new()
        .host(&envvar("DATABASE_HOST").ok().unwrap())
        .port(5432)
        .username(&envvar("DATABASE_USER").ok().unwrap())
        .password(&envvar("DATABASE_PASSWORD").ok().unwrap())
        .database(&envvar("DATABASE_DB").ok().unwrap())
}

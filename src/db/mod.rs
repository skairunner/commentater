use dotenv::var as envvar;
use sqlx::postgres::PgConnectOptions;

pub mod schema;

pub fn get_connection_options() -> PgConnectOptions {
    PgConnectOptions::new()
        .host(&envvar("DATABASE_HOST").ok().unwrap())
        .port(5432)
        .username(&envvar("DATABASE_USER").ok().unwrap())
        .password(&envvar("DATABASE_PASSWORD").ok().unwrap())
}

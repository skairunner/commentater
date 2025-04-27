use dotenv::dotenv;
use influxdb::{Client, InfluxDbWriteable, Timestamp};
use libtater::db::get_connection_options;
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;

struct WrappedOffsetDateTime(OffsetDateTime);

impl From<OffsetDateTime> for WrappedOffsetDateTime {
    fn from(value: OffsetDateTime) -> Self {
        Self(value)
    }
}

impl From<WrappedOffsetDateTime> for Timestamp {
    fn from(value: WrappedOffsetDateTime) -> Self {
        Self::Seconds(value.0.unix_timestamp() as u128)
    }
}

#[derive(InfluxDbWriteable, FromRow)]
struct ArticleQueueInfo {
    time: WrappedOffsetDateTime,
    total_entries: i64,
    done_count: i64,
    pending_count: i64,
    errored_count: i64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let pool = PgPool::connect_with(get_connection_options()).await?;
    let results = sqlx::query_as!(
        ArticleQueueInfo,
        r#"
        SELECT
            NOW() as "time!",
            COUNT(1) as "total_entries!",
            COUNT(1) FILTER (where done=true) as "done_count!",
            COUNT(1) FILTER (where done<>true) as "pending_count!",
            COUNT(1) FILTER (where error=true) as "errored_count!"
        FROM article_queue;
        "#
    )
    .fetch_one(&pool)
    .await?;
    let client = Client::new("http://localhost:8086", "stats")
        .with_token(&std::env::var("INFLUXDB_TOKEN")?);
    client.query(results.into_query("article_queue")).await?;

    Ok(())
}

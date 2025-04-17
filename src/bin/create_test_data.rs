// Insert a test user and world.

use dotenv::dotenv;
use libtater::db::get_connection_options;
use libtater::db::test_queries::add_test_data;
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let pool = PgPool::connect_with(get_connection_options())
        .await
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    add_test_data(&mut tx).await;
    tx.commit().await.unwrap()
}

// Insert a test user and world.

use libtater::db::get_connection_options;
use libtater::db::query::add_test_user_and_world;
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    let pool = PgPool::connect_with(get_connection_options())
        .await
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    add_test_user_and_world(&mut tx).await;
    tx.commit().await.unwrap()
}

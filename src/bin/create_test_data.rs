// Insert a test user and world and other one-off scripts

use dotenv::dotenv;
use libtater::db::get_connection_options;
use libtater::db::schema::CommentaterUser;
use libtater::db::test_queries::add_test_data;
use libtater::db::world::get_worlds;
use libtater::req::get_wa_client_builder;
use libtater::worldanvil_api::world_list_articles;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let pool = PgPool::connect_with(get_connection_options())
        .await
        .unwrap();
    let mut tx = pool.begin().await?;
    // Lock entire table for update
    sqlx::query("SELECT * FROM article FOR UPDATE;")
        .execute(&mut *tx)
        .await?;

    // Ensure only one copy of each article is preserved
    sqlx::query!(
        "
DELETE FROM article USING (
    SELECT MIN(id) as id, url
    FROM article
    GROUP BY url
    HAVING count(*) > 1
) as dupes
WHERE article.url = dupes.url AND article.id <> dupes.id
    "
    )
    .execute(&mut *tx)
    .await?;

    // For each user, merge worldanvil ids based on the url
    let users = sqlx::query_as!(
        CommentaterUser,
        "SELECT id, display_name, api_key, last_seen, worldanvil_id
        FROM commentater_user
        "
    )
    .fetch_all(&mut *tx)
    .await?;
    for user in users {
        let client = get_wa_client_builder(&user.api_key).build()?;
        for world in get_worlds(&mut *tx, &user.id).await? {
            let articles = world_list_articles(&client, &world.worldanvil_id).await?;
            let mut article_wa_ids = vec![];
            let mut article_urls = vec![];
            articles.into_iter().for_each(|article| {
                article_wa_ids.push(article.id);
                article_urls.push(article.url);
            });
            sqlx::query!(
                "WITH update AS (
                    SELECT $1::bigint as user_id, *
                    FROM UNNEST($2::text[], $3::text[])
                    AS t(wa_id, url)
                )
                UPDATE article
                SET worldanvil_id=update.wa_id
                FROM update
                WHERE article.url = update.url
                AND article.user_id = update.user_id
                ",
                &user.id,
                &article_wa_ids,
                &article_urls,
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    Ok(())
}

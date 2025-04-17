use crate::db::pgacquire::PgAcquire;
use crate::db::schema::Article;

pub async fn register_article<'a, A: PgAcquire<'a>>(
    user_id: i64,
    world_id: i64,
    url: &str,
    conn: A,
) -> Result<i64, sqlx::Error> {
    let mut conn = conn.acquire().await?;
    sqlx::query!(
        "INSERT INTO article(user_id, world_id, url)
   VALUES ($1, $2, $3) ON CONFLICT DO NOTHING
   RETURNING id;",
        user_id,
        world_id,
        url
    )
    .fetch_one(&mut *conn)
    .await
    .map(|r| r.id)
}

pub async fn register_articles<'a, A: PgAcquire<'a>>(
    user_id: i64,
    world_id: i64,
    urls: &[String],
    titles: &[String],
    conn: A,
) -> Result<i64, sqlx::Error> {
    let mut conn = conn.acquire().await?;
    sqlx::query!(
        "
        INSERT INTO article(user_id, world_id, url, title)
        SELECT $1 as user_id, $2 as world_id, *
        FROM UNNEST($3::text[], $4::text[])
        ON CONFLICT DO NOTHING
        RETURNING id;",
        user_id,
        world_id,
        urls,
        titles
    )
    .fetch_one(&mut *conn)
    .await
    .map(|r| r.id)
}

/// Create or update the article content entry
pub async fn update_article_content<'a, A: PgAcquire<'a>>(
    conn: A,
    article_id: i64,
    worldanvil_id: &str,
    title: &str,
) -> sqlx::Result<()> {
    let mut conn = conn.acquire().await?;
    sqlx::query!(
        "INSERT INTO article_content(article_id, worldanvil_id, title) VALUES ($1, $2, $3)
        ON CONFLICT(article_id)
        DO UPDATE SET worldanvil_id=$2, title=$3;",
        article_id,
        worldanvil_id,
        title,
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}

pub async fn get_article_ids<'a, A: PgAcquire<'a>>(
    conn: A,
    user_id: &i64,
) -> sqlx::Result<Vec<i64>> {
    let mut conn = conn.acquire().await?;
    let result: Vec<(i64,)> = sqlx::query_as("SELECT id FROM article WHERE user_id=$1")
        .bind(user_id)
        .fetch_all(&mut *conn)
        .await?;
    Ok(result.into_iter().map(|row| row.0).collect())
}

pub async fn get_article<'a, A: PgAcquire<'a>>(
    conn: A,
    article_id: i64,
    user_id: i64,
) -> sqlx::Result<Article> {
    let mut conn = conn.acquire().await?;
    sqlx::query_as!(
        Article,
        "SELECT id, user_id, world_id, url, last_checked FROM article WHERE id=$1 AND user_id=$2;",
        article_id,
        user_id,
    )
    .fetch_one(&mut *conn)
    .await
}

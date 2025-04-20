use crate::db::pgacquire::PgAcquire;
use crate::db::schema::{Article, ArticleAndStatus, RawArticleAndStatus};

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
    worldanvil_ids: &[String],
    conn: A,
) -> Result<i64, sqlx::Error> {
    let mut conn = conn.acquire().await?;
    // Delete articles that no longer exist
    sqlx::query!(
        "
        DELETE FROM article
        WHERE worldanvil_id <> ANY($1::text[]);
        ",
        worldanvil_ids,
    )
    .execute(&mut *conn)
    .await?;
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
    world_id: &i64,
) -> sqlx::Result<Vec<i64>> {
    let mut conn = conn.acquire().await?;
    let result: Vec<(i64,)> = sqlx::query_as("
    SELECT id
    FROM article
    WHERE user_id=$1 AND world_id=$2
    ")
        .bind(user_id)
        .bind(world_id)
        .fetch_all(&mut *conn)
        .await?;
    Ok(result.into_iter().map(|row| row.0).collect())
}

/// Find all article IDs which are not currently in the job queue
pub async fn get_unqueued_article_ids<'a, A: PgAcquire<'a>>(
    conn: A,
    user_id: &i64,
    world_id: &i64,
) -> sqlx::Result<Vec<i64>> {
    let mut conn = conn.acquire().await?;
    let result: Vec<(i64,)> = sqlx::query_as("
        SELECT article.id, article_queue.done
        FROM article
        LEFT JOIN (
            -- Select all of the most recent queue jobs for each article
            SELECT MAX(id) as id, article_id
            FROM article_queue
            WHERE user_id=$1
            GROUP BY article_id
        ) as max_aq
        ON article.id=max_aq.article_id
        -- Then join on the article status
        LEFT JOIN article_queue
        ON article_queue.id=max_aq.id
        WHERE (article_queue.done is NULL or article_queue.done=true)
            AND article.user_id=$1 AND article.world_id=$2
    ")
        .bind(user_id)
        .bind(world_id)
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

pub async fn get_articles_and_status<'a, A: PgAcquire<'a>>(
    user_id: &i64,
    world_id: &i64,
    conn: A,
) -> sqlx::Result<Vec<ArticleAndStatus>> {
    let mut conn = conn.acquire().await?;
    let res = sqlx::query_as!(
        RawArticleAndStatus,
        r#"SELECT
            article.id AS article_id, title, url, last_checked,
            done as "done?", error as "error?", error_msg as "error_msg?", comments.count as unanswered_comments
        FROM article
        LEFT JOIN (
            SELECT MAX(id) AS id, article_id
            FROM article_queue
            GROUP by article_id
        ) AS max_aq
        ON article.id = max_aq.article_id
        LEFT JOIN (
            SELECT id, done, error, error_msg
            FROM article_queue
        ) AS aq
        ON max_aq.id = aq.id
        LEFT JOIN (
            SELECT COUNT(*) as count, article_id
            FROM comment
            GROUP BY article_id
        ) as comments
        ON comments.article_id = article.id
        WHERE article.user_id=$1 AND article.world_id=$2
        ORDER BY unanswered_comments DESC NULLS LAST"#,
        user_id,
        world_id,
    )
    .fetch_all(&mut *conn)
    .await?
        .into_iter()
        .map(|a| a.into_article_and_status())
        .collect();
    Ok(res)
}

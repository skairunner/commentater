use crate::db::pgacquire::PgAcquire;
use crate::db::schema::{
    Article, ArticleQueueEntry, Comment, CommentInsert, WorldAnvilUser, WorldAnvilUserInsert,
};
use sqlx::Postgres;
use std::collections::HashSet;
use std::env;

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

pub async fn add_test_data<'a, A: PgAcquire<'a>>(conn: A) {
    let mut conn = conn.acquire().await.unwrap();
    // Insert a WA user
    sqlx::query!(
        "INSERT INTO wa_user(id, worldanvil_id, name) OVERRIDING SYSTEM VALUE VALUES(5, '225bd01d-124c-4aa2-885b-0fc4bdf41bd8', 'nnie');",
    )
        .execute(&mut *conn)
        .await
        .unwrap();
    // Insert a commentator user
    sqlx::query!(
        "INSERT INTO commentater_user(id, worldanvil_id, api_key) OVERRIDING SYSTEM VALUE VALUES (5, 5, $1);",
        Some(env::var("TEST_USER_KEY").unwrap())
    )
    .execute(&mut *conn)
    .await
    .unwrap();
    sqlx::query!("INSERT INTO user_queue(user_id) VALUES (5);")
        .execute(&mut *conn)
        .await
        .unwrap();
    // Insert a world
    sqlx::query!("INSERT INTO world(id, user_id, worldanvil_id, name) OVERRIDING SYSTEM VALUE VALUES (5, 5, 'e69d6a36-2d22-4bf2-80f9-456a9b0d909e', 'Solaris');")
        .execute(&mut *conn)
        .await
        .unwrap();
    // Insert a few articles
    sqlx::query!(
        "
        INSERT INTO article(user_id, world_id, title, url)
        SELECT 5, 5, *
        FROM UNNEST($1::text[], $2::text[]);
        ",
        &[
            "A Day in the Caloris Basin".to_string(),
            "About Solaris".to_string()
        ],
        &[
            "https://www.worldanvil.com/w/solaris-nnie/a/a-day-in-the-caloris-basin-law"
                .to_string(),
            "https://www.worldanvil.com/w/solaris-nnie/a/about-solaris-article".to_string()
        ]
    )
    .execute(&mut *conn)
    .await
    .unwrap();
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

/// Fetch comments on a specified article
pub async fn get_comments<'a, A: PgAcquire<'a>>(
    conn: A,
    article_id: i64,
    user_id: i64,
) -> sqlx::Result<Vec<Comment>> {
    let mut conn = conn.acquire().await?;
    sqlx::query_as!(
        Comment,
        "SELECT id, user_id, author_id, article_id, content, date, starred, deleted
        FROM comment
        WHERE article_id=$1 AND user_id=$2;",
        article_id,
        user_id,
    )
    .fetch_all(&mut *conn)
    .await
}

/// Insert unanswered comments
pub async fn insert_comments<'a, A: PgAcquire<'a>>(
    conn: A,
    article_id: i64,
    user_id: i64,
    comments: Vec<CommentInsert>,
) -> sqlx::Result<()> {
    let mut author_ids = vec![];
    let mut contents = vec![];
    let mut dates = vec![];
    comments.into_iter().for_each(|comment| {
        author_ids.push(comment.author_id);
        contents.push(comment.content);
        dates.push(comment.date);
    });
    let mut conn = conn.acquire().await?;
    sqlx::query!(
        "INSERT INTO comment(user_id, article_id, author_id, content, date)
        SELECT $1, $2, * FROM UNNEST($3::bigint[], $4::text[], $5::timestamp with time zone[])",
        user_id,
        article_id,
        &author_ids,
        &contents,
        &dates,
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Delete all db comments for an article
pub async fn delete_comments<'a, A: PgAcquire<'a>>(
    conn: A,
    article_id: i64,
    user_id: i64,
) -> sqlx::Result<()> {
    let mut conn = conn.acquire().await?;
    sqlx::query!(
        "DELETE FROM comment WHERE article_id=$1 AND user_id=$2;",
        article_id,
        user_id
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Update or insert WorldAnvil users, returning a list of tuples from internal id to worldanvil id
pub async fn update_wa_users<'a, A: PgAcquire<'a>>(
    conn: A,
    users: Vec<WorldAnvilUserInsert>,
) -> sqlx::Result<Vec<(i64, Option<String>)>> {
    // Filter out duplicates
    let mut worldanvil_id_set = HashSet::new();
    let mut worldanvil_ids = vec![];
    let mut worldanvil_names = vec![];
    let mut worldanvil_avatar_urls = vec![];
    users.into_iter().for_each(|user| {
        if worldanvil_id_set.contains(&user) {
            return;
        }
        worldanvil_id_set.insert(user.clone());
        worldanvil_ids.push(user.worldanvil_id);
        worldanvil_names.push(user.name);
        worldanvil_avatar_urls.push(user.avatar_url);
    });
    let mut conn = conn.acquire().await?;
    sqlx::query!(
        r#"INSERT INTO wa_user(worldanvil_id, name, avatar_url)
    SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])
    ON CONFLICT (worldanvil_id) DO UPDATE SET
        name=EXCLUDED.name,
        avatar_url=EXCLUDED.avatar_url
    RETURNING id, worldanvil_id"#,
        &worldanvil_ids as _,
        &worldanvil_names,
        &worldanvil_avatar_urls as _,
    )
    .fetch_all(&mut *conn)
    .await
    .map(|records| {
        records
            .into_iter()
            .map(|record| (record.id, record.worldanvil_id))
            .collect()
    })
}

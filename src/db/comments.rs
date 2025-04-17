use crate::db::pgacquire::PgAcquire;
use crate::db::schema::{Comment, CommentInsert};

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

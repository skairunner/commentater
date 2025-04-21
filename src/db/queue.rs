use crate::db::pgacquire::PgAcquire;
use crate::db::schema::{ArticleQueueEntry, UserQueue};
use sqlx::{PgConnection, Postgres};

/// Lock a user for work
pub async fn get_next_user(
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> sqlx::Result<Option<UserQueue>> {
    sqlx::query_as!(
        UserQueue,
        "SELECT id, user_id, last_updated
        FROM user_queue
        WHERE last_updated < NOW() - interval '2 seconds'
        ORDER BY last_updated ASC
        FOR UPDATE SKIP LOCKED
        LIMIT 1;"
    )
    .fetch_optional(&mut **tx)
    .await
}

pub async fn update_user_queue(
    id: &i64,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> sqlx::Result<()> {
    sqlx::query_as!(
        UserQueue,
        "UPDATE user_queue
        SET last_updated = NOW()
        WHERE id = $1;",
        id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn insert_tasks(
    user_id: &i64,
    article_ids: &[i64],
    conn: &mut PgConnection,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO article_queue(user_id, article_id)
        SELECT $1, *
        FROM UNNEST($2::bigint[]);",
        user_id,
        article_ids,
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Get the next task from the task queue
pub async fn get_next_task(
    user_id: &i64,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> sqlx::Result<Option<ArticleQueueEntry>> {
    sqlx::query_as!(
        ArticleQueueEntry,
        "SELECT id, user_id, article_id
        FROM article_queue
        WHERE done=false AND user_id=$1
        ORDER BY id ASC
        FOR UPDATE SKIP LOCKED
        LIMIT 1;",
        user_id
    )
    .fetch_optional(&mut **tx)
    .await
}

/// Return true if there is a pending task for the article
pub async fn article_is_queued(
    user_id: &i64,
    article_id: &i64,
    conn: &mut PgConnection,
) -> sqlx::Result<bool> {
    let res = sqlx::query!(
        "SELECT id
        FROM article_queue
        WHERE done <> true AND user_id=$1 AND article_id=$2
        LIMIT 1",
        user_id,
        article_id,
    )
    .fetch_optional(&mut *conn)
    .await?;
    Ok(res.is_some())
}

/// Mark a task as done
pub async fn complete_task(
    id: i64,
    error: Option<&str>,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE article_queue SET done=true, error=$2, error_msg=$3 WHERE id=$1;",
        id,
        error.is_some(),
        error
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

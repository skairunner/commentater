use crate::db::pgacquire::PgAcquire;
use crate::db::schema::{ArticleQueueEntry, UserQueue};
use influxdb::InfluxDbWriteable;
use sqlx::{FromRow, PgConnection, Postgres};
use time::OffsetDateTime;

/// Lock a user for work.
/// It has to be a user which has at least one task ready.
pub async fn get_next_user(
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> sqlx::Result<Option<UserQueue>> {
    sqlx::query_as!(
        UserQueue,
        "SELECT user_queue.id, user_queue.user_id, user_queue.last_updated
        FROM user_queue
        JOIN (
            SELECT DISTINCT user_id
            FROM article_queue
            WHERE done <> true
        ) as aq
        ON user_queue.user_id = aq.user_id
        WHERE last_updated < NOW() - interval '2 seconds'
        ORDER BY last_updated ASC
        FOR UPDATE OF user_queue SKIP LOCKED
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

async fn update_user_queue_to(
    id: &i64,
    interval: &sqlx::postgres::types::PgInterval,
    conn: &mut PgConnection,
) -> sqlx::Result<()> {
    sqlx::query_as!(
        UserQueue,
        "UPDATE user_queue
        SET last_updated = NOW() - $2::interval
        WHERE id = $1;",
        id,
        interval,
    )
    .execute(&mut *conn)
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

#[derive(FromRow)]
struct ArticleQueueInfo {
    queue_length: i64,
}

pub async fn get_queue_length(conn: &mut PgConnection) -> sqlx::Result<i64> {
    let res = sqlx::query_as!(
        ArticleQueueInfo,
        r#"
        SELECT COUNT(*) as "queue_length!"
        FROM article_queue
        WHERE done <> true;
        "#
    )
    .fetch_one(conn)
    .await?;
    Ok(res.queue_length)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::article::register_article;
    use crate::db::schema::WorldInsert;
    use crate::db::user::{get_user_id_or_insert, insert_user_queue};
    use crate::db::world::{get_world, upsert_worlds};
    use sqlx::postgres::types::PgInterval;
    use sqlx::{Acquire, PgPool};

    /// Ensure that users are only selected for update if they have at least one queue entry.
    #[sqlx::test]
    async fn test_user_queue_logic(pool: PgPool) -> anyhow::Result<()> {
        let mut conn = pool.acquire().await?;
        // Insert two users into the db.
        let user1 = get_user_id_or_insert(&mut conn, "key1", "user1", "id1").await?;
        insert_user_queue(&mut conn, &user1.id).await?;
        let user2 = get_user_id_or_insert(&mut conn, "key2", "user2", "id2").await?;
        insert_user_queue(&mut conn, &user2.id).await?;
        // Make it so user1 has an earlier last update time than user1
        update_user_queue_to(
            &user1.id,
            &PgInterval {
                months: 0,
                days: 2,
                microseconds: 0,
            },
            &mut conn,
        )
        .await?;
        update_user_queue_to(
            &user2.id,
            &PgInterval {
                months: 0,
                days: 1,
                microseconds: 0,
            },
            &mut conn,
        )
        .await?;
        // Insert work for user 2
        let worlds = upsert_worlds(
            &mut conn,
            &user2.id,
            vec![WorldInsert {
                worldanvil_id: "worldid".to_string(),
                name: "testworld".to_string(),
            }],
        )
        .await?;
        let article_id =
            register_article(user2.id, worlds[0], "myurl", "mytitle", &mut conn).await?;
        insert_tasks(&user2.id, &[article_id], &mut conn).await?;

        // Finally, attempt to select a user for work.
        // It should select user 2.
        let mut tx = conn.begin().await?;
        let task = get_next_user(&mut tx).await?;
        let task_user_id = task.map(|t| t.user_id);
        assert_eq!(task_user_id, Some(user2.id));

        Ok(())
    }
}

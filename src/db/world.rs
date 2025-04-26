use crate::db::pgacquire::PgAcquire;
use crate::db::schema::{World, WorldInsert};

pub async fn get_worlds<'a, A: PgAcquire<'a>>(conn: A, user_id: &i64) -> sqlx::Result<Vec<World>> {
    let mut conn = conn.acquire().await?;
    sqlx::query_as!(
        World,
        "SELECT id, user_id, worldanvil_id, name
        FROM world
        WHERE user_id=$1",
        user_id,
    )
    .fetch_all(&mut *conn)
    .await
}

pub async fn get_world<'a, A: PgAcquire<'a>>(
    conn: A,
    user_id: &i64,
    world_id: &i64,
) -> sqlx::Result<World> {
    let mut conn = conn.acquire().await?;
    sqlx::query_as!(
        World,
        "
    SELECT id, user_id, worldanvil_id, name
    FROM world
    WHERE user_id=$1 AND id=$2
    LIMIT 1;",
        user_id,
        world_id,
    )
    .fetch_one(&mut *conn)
    .await
}

pub async fn upsert_worlds<'a, A: PgAcquire<'a>>(
    conn: A,
    user_id: &i64,
    worlds: Vec<WorldInsert>,
) -> sqlx::Result<Vec<i64>> {
    let mut conn = conn.acquire().await?;
    let mut world_ids = Vec::new();
    let mut world_names = Vec::new();
    worlds.into_iter().for_each(|world| {
        let WorldInsert {
            worldanvil_id,
            name,
        } = world;
        world_ids.push(worldanvil_id);
        world_names.push(name);
    });
    // First delete all worlds not in worlds, then upsert
    sqlx::query!(
        "
        DELETE FROM world
        WHERE user_id=$1 AND worldanvil_id <> ANY($2::text[]);
        ",
        user_id,
        &world_ids,
    )
    .execute(&mut *conn)
    .await?;
    let returning = sqlx::query!(
        "
        INSERT INTO world(user_id, worldanvil_id, name)
        SELECT $1, * FROM UNNEST($2::text[], $3::text[])
        ON CONFLICT (user_id, worldanvil_id) DO UPDATE SET
            name=EXCLUDED.name
        RETURNING id;
        ",
        user_id,
        &world_ids,
        &world_names,
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(returning.into_iter().map(|record| record.id).collect())
}

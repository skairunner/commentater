use sqlx::Postgres;

// Little bit of magic to make a shorthand for sqlx::Acquire
pub trait PgAcquire<'a>: sqlx::Acquire<'a, Database = Postgres> {}
impl<'a, T: sqlx::Acquire<'a, Database = Postgres>> PgAcquire<'a> for T {}

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

pub async fn add_test_user_and_world<'a, A: PgAcquire<'a>>(conn: A) {
    let mut conn = conn.acquire().await.unwrap();
    sqlx::query!("INSERT INTO wa_user(id, worldanvil_id, name) OVERRIDING SYSTEM VALUE VALUES(5, '225bd01d-124c-4aa2-885b-0fc4bdf41bd8', 'nnie');")
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::query!(
        "INSERT INTO commentater_user(id, worldanvil_id) OVERRIDING SYSTEM VALUE VALUES (5, 5);"
    )
    .execute(&mut *conn)
    .await
    .unwrap();
    sqlx::query!("INSERT INTO world(id, user_id, worldanvil_id, name) OVERRIDING SYSTEM VALUE VALUES (5, 5, 'e69d6a36-2d22-4bf2-80f9-456a9b0d909e', 'Solaris');")
        .execute(&mut *conn)
        .await
        .unwrap();
}

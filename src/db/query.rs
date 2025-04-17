use crate::db::pgacquire::PgAcquire;
use crate::db::schema::WorldAnvilUserInsert;
use std::collections::HashSet;
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

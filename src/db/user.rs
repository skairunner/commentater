use crate::db::pgacquire::PgAcquire;
use crate::db::schema::CommentaterUser;

pub async fn get_user_id_or_insert<'a, A: PgAcquire<'a>>(
    conn: A,
    api_key: &str,
    display_name: &str,
    worldanvil_id: &str,
) -> sqlx::Result<CommentaterUser> {
    let mut conn = conn.acquire().await?;
    sqlx::query_as!(CommentaterUser, "
    INSERT INTO commentater_user(api_key, display_name, worldanvil_id)
    VALUES($1, $2, $3)
    ON CONFLICT (api_key) DO UPDATE
    SET api_key=$1, display_name=$2, worldanvil_id=$3
    RETURNING id, display_name, api_key, last_seen, worldanvil_id",
        api_key,
        display_name,
        worldanvil_id
    )
        .fetch_one(&mut *conn)
        .await
}

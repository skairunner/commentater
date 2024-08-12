use sqlx;
use sqlx::FromRow;
use time::OffsetDateTime;

/// A user seen in comments.
#[derive(FromRow)]
pub struct User {
    id: i64,
    worldanvil_id: String,
    name: String,
    avatar_url: Option<String>,
    last_seen: OffsetDateTime,
}

#[derive(FromRow)]
pub struct World {
    id: i64,
    user_id: i64,
    worldanvil_id: String,
    name: String,
}

#[derive(FromRow)]
pub struct Article {
    id: i64,
    user_id: i64,
    world_id: i64,
    worldanvil_id: String,
    title: String,
    url: String,
    last_checked: Option<OffsetDateTime>,
}

#[derive(FromRow)]
pub struct Comment {
    id: i64,
    user_id: i64,
    article_id: i64,
    content: String,
    date: OffsetDateTime,
    starred: bool,
    parent: Option<i64>,
}

#[derive(FromRow)]
pub struct ArticleQueue {
    id: i64,
    article_id: i64,
}

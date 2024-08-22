use sqlx;
use sqlx::FromRow;
use time::{OffsetDateTime, PrimitiveDateTime};

/// A user seen in comments.
#[derive(FromRow)]
pub struct WorldAnvilUser {
    id: i64,
    worldanvil_id: Option<String>,
    name: String,
    avatar_url: Option<String>,
}

#[derive(FromRow)]
pub struct CommentaterUser {
    id: i64,
    worldanvil_id: Option<String>,
    last_seen: PrimitiveDateTime,
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
    url: String,
    last_checked: Option<OffsetDateTime>,
}

#[derive(FromRow)]
pub struct ArticleContent {
    id: i64,
    article_id: i64,
    worldanvil_id: String,
    title: String,
}

#[derive(FromRow)]
pub struct Comment {
    id: i64,
    user_id: i64,
    author_id: i64,
    article_id: i64,
    content: String,
    date: OffsetDateTime,
    starred: bool,
    deleted: bool,
}

pub struct CommentReplies {
    id: i64,
    user_id: i64,
    article_id: i64,
    content: String,
    date: OffsetDateTime,
    starred: bool,
    parent: i64,
    deleted: bool,
}

#[derive(FromRow)]
pub struct ArticleQueue {
    id: i64,
    article_id: i64,
}

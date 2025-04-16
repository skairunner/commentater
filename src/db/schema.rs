use sqlx;
use sqlx::FromRow;
use time::{OffsetDateTime, PrimitiveDateTime};

/// A user seen in comments.
#[derive(FromRow)]
pub struct WorldAnvilUser {
    pub id: i64,
    pub worldanvil_id: Option<String>,
    pub name: String,
    pub avatar_url: Option<String>,
}

/// The relevant info for inserting a worldanvil user
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct WorldAnvilUserInsert {
    pub worldanvil_id: Option<String>,
    pub name: String,
    pub avatar_url: Option<String>,
}

#[derive(FromRow)]
pub struct CommentaterUser {
    pub id: i64,
    pub worldanvil_id: Option<String>,
    pub last_seen: PrimitiveDateTime,
    pub api_key: Option<String>,
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
    pub id: i64,
    pub user_id: i64,
    pub world_id: i64,
    pub url: String,
    pub last_checked: Option<OffsetDateTime>,
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
    pub id: i64,
    pub user_id: i64,
    pub author_id: Option<i64>,
    pub article_id: i64,
    pub content: String,
    pub date: OffsetDateTime,
    pub starred: bool,
    pub deleted: bool,
}

impl Comment {
    /// The unique key for this comment
    pub fn key(&self) -> Option<(i64, OffsetDateTime)> {
        match self.author_id {
            Some(author_id) => Some((author_id, self.date)),
            None => None,
        }
    }
}

/// A comment struct for inserting into the db.
pub struct CommentInsert {
    pub user_id: i64,
    pub author_id: i64,
    pub article_id: i64,
    pub content: String,
    pub date: OffsetDateTime,
}

pub struct CommentReplies {
    id: i64,
    user_id: i64,
    article_id: i64,
    parent: i64,
    content: String,
    date: OffsetDateTime,
    starred: bool,
    deleted: bool,
}

#[derive(FromRow, Clone)]
pub struct ArticleQueueEntry {
    pub id: i64,
    pub user_id: i64,
    pub article_id: i64,
}

#[derive(FromRow)]
pub struct UserQueue {
    pub id: i64,
    pub user_id: i64,
    pub last_updated: OffsetDateTime,
}

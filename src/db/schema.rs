use sqlx;
use sqlx::FromRow;
use time::OffsetDateTime;

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
    pub display_name: Option<String>,
    pub worldanvil_id: Option<String>,
    pub last_seen: OffsetDateTime,
    pub api_key: String,
}

#[derive(FromRow)]
pub struct World {
    pub id: i64,
    pub user_id: i64,
    pub worldanvil_id: String,
    pub name: String,
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
    pub id: i64,
    pub article_id: i64,
    pub worldanvil_id: String,
    pub title: String,
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
        self.author_id.map(|author_id| (author_id, self.date))
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
    pub id: i64,
    pub user_id: i64,
    pub article_id: i64,
    pub parent: i64,
    pub content: String,
    pub date: OffsetDateTime,
    pub starred: bool,
    pub deleted: bool,
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

#[derive(FromRow)]
pub struct ArticleAndStatus {
    pub article_id: i64,
    pub title: String,
    pub url: String,
    pub last_checked: Option<OffsetDateTime>,
    pub done: bool,
    pub error: Option<bool>,
    pub error_msg: Option<String>,
}

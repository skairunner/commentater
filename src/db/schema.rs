use crate::dateutil::{date_as_human_friendly, date_option_as_human_friendly};
use serde::Serialize;
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
    pub worldanvil_id: String,
    pub last_seen: OffsetDateTime,
    pub api_key: String,
}

#[derive(FromRow, Serialize)]
pub struct World {
    pub id: i64,
    pub user_id: i64,
    pub worldanvil_id: String,
    pub name: String,
}

pub struct WorldInsert {
    pub worldanvil_id: String,
    pub name: String,
}

#[derive(FromRow, Serialize)]
pub struct Article {
    pub id: i64,
    pub user_id: i64,
    pub world_id: i64,
    pub url: String,
    #[serde(with = "time::serde::iso8601::option")]
    pub last_checked: Option<OffsetDateTime>,
}

#[derive(FromRow, Serialize)]
pub struct ArticleDetails {
    pub id: i64,
    pub user_id: i64,
    pub world_id: i64,
    pub url: String,
    #[serde(serialize_with = "date_option_as_human_friendly")]
    pub last_checked: Option<OffsetDateTime>,
    pub title: String,
    pub worldanvil_id: Option<String>,
}

#[derive(FromRow)]
pub struct ArticleContent {
    pub id: i64,
    pub article_id: i64,
    pub worldanvil_id: String,
    pub title: String,
}

#[derive(FromRow, Serialize)]
pub struct Comment {
    pub id: i64,
    pub user_id: i64,
    pub author_id: Option<i64>,
    pub article_id: i64,
    pub content: String,
    #[serde(serialize_with = "date_as_human_friendly")]
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

#[derive(FromRow, Serialize)]
pub struct RawArticleAndStatus {
    pub article_id: i64,
    pub title: String,
    pub url: String,
    pub last_checked: Option<OffsetDateTime>,
    pub done: Option<bool>,
    pub error: Option<bool>,
    pub error_msg: Option<String>,
    pub unanswered_comments: Option<i64>,
}

impl RawArticleAndStatus {
    pub fn into_article_and_status(self) -> ArticleAndStatus {
        let RawArticleAndStatus {
            article_id,
            title,
            url,
            last_checked,
            done,
            error,
            error_msg,
            unanswered_comments,
        } = self;
        // If done exists, all the others must exist
        let status = if done.is_some() {
            Some(ArticleStatus {
                done: done.unwrap(),
                error,
                error_msg,
            })
        } else {
            None
        };
        ArticleAndStatus {
            article_id,
            title,
            url,
            last_checked,
            status,
            unanswered_comments: unanswered_comments.unwrap_or(0),
        }
    }
}

#[derive(Serialize)]
pub struct ArticleAndStatus {
    pub article_id: i64,
    pub title: String,
    pub url: String,
    #[serde(serialize_with = "date_option_as_human_friendly")]
    pub last_checked: Option<OffsetDateTime>,
    pub status: Option<ArticleStatus>,
    pub unanswered_comments: i64,
}

#[derive(Serialize)]
pub struct ArticleStatus {
    pub done: bool,
    pub error: Option<bool>,
    pub error_msg: Option<String>,
}

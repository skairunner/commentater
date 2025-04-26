use crate::db::article::{get_article, set_article_checked_time, update_article_content};
use crate::db::comments::{delete_comments, insert_comments};
use crate::db::query::update_wa_users;
use crate::db::queue::{complete_task, get_next_task, get_next_user, update_user_queue};
use crate::db::schema::{ArticleQueueEntry, CommentInsert};
use crate::parser::{get_page, parse_page, ParseError, RootComment};
use sqlx::{Acquire, Postgres};
use std::collections::HashMap;

pub struct TaskError {
    pub error: anyhow::Error,
    pub unhandled: bool,
    /// This may be displayed to the user.
    pub message: String,
    pub user_queue_id: i64,
    pub task_id: i64,
}

pub enum TaskOutcome {
    Completed,
    NoUser,
    NoTasks,
    Error(TaskError),
}

impl ParseError {
    fn into_parse_error(self, user_queue_id: i64, task_id: i64) -> TaskOutcome {
        let message = self.to_string();
        TaskOutcome::Error(TaskError {
            error: self.into(),
            unhandled: false,
            message,
            user_queue_id,
            task_id,
        })
    }
}

/// Find comment indices which have no replies
fn find_comments_without_replies(comments: &[RootComment]) -> Vec<usize> {
    comments
        .iter()
        .enumerate()
        .filter_map(|(i, comment)| {
            if comment.replies.is_empty() {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

/// Fetch a task and update the article and its comments.
/// Returns NoTasks if there are no valid tasks to do so as to signal the task caller to sleep a bit.
pub async fn update_task_inner(
    task: &ArticleQueueEntry,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> anyhow::Result<TaskOutcome> {
    let ArticleQueueEntry {
        id: task_id,
        user_id,
        article_id,
    } = task.clone();

    let article = get_article(&mut *tx, article_id, user_id).await?;

    let page = get_page(&article.url).await?;
    let parsed = match parse_page(&page) {
        Ok(article) => article,
        Err(e) => return Ok(e.into_parse_error(user_id, task_id)),
    };
    update_article_content(&mut *tx, article_id, &parsed.worldanvil_id, &parsed.title).await?;
    // Clean old comments
    delete_comments(&mut *tx, article_id, user_id).await?;
    let potential_users = parsed
        .comments
        .iter()
        .map(|comment| comment.as_worldanvil_user())
        .collect();
    let users = update_wa_users(&mut *tx, potential_users).await?;
    // Turn users into a map from worldanvil id to internal id
    let user_map: HashMap<_, _> = users
        .into_iter()
        .filter_map(|(id, wa_id)| wa_id.map(|i| (i, id)))
        .collect();
    // Transform potential users into insertable comments only if they have no replies
    let comments: Vec<_> = parsed
        .comments
        .iter()
        .filter_map(
            |comment| match user_map.get(&comment.author_worldanvil_id) {
                Some(internal_id) => {
                    if comment.replies.is_empty() {
                        Some(CommentInsert {
                            user_id,
                            author_id: *internal_id,
                            article_id,
                            content: comment.comment.content.clone(),
                            date: comment.datetime().assume_utc(),
                        })
                    } else {
                        None
                    }
                }
                None => {
                    log::info!(
                        "Could not find internal id for user {}",
                        comment.author_worldanvil_id
                    );
                    None
                }
            },
        )
        .collect();

    if !comments.is_empty() {
        let n = comments.len();
        insert_comments(&mut *tx, article_id, user_id, comments).await?;
        log::info!("Inserted {n} comments for article {article_id} of user {user_id}")
    }
    Ok(TaskOutcome::Completed)
}

pub async fn update_task(mut tx: sqlx::Transaction<'_, Postgres>) -> anyhow::Result<TaskOutcome> {
    // Lock a valid user
    let user_queue_entry = get_next_user(&mut tx).await?;
    let user_queue_entry = match user_queue_entry {
        Some(entry) => entry,
        None => return Ok(TaskOutcome::NoUser),
    };
    let task = get_next_task(&user_queue_entry.user_id, &mut tx).await?;
    let task = match task {
        Some(task) => task,
        None => return Ok(TaskOutcome::NoTasks),
    };
    log::info!("Working on {}", task.article_id);
    // Use inner transaction to discard partial updates.
    let mut inner_tx = tx.begin().await?;
    match update_task_inner(&task, &mut inner_tx).await {
        Ok(TaskOutcome::NoTasks | TaskOutcome::NoUser) => {
            // This should never be returned.
            panic!("No tasks returned from inner update task");
        }
        Ok(TaskOutcome::Error(task_error)) => {
            let TaskError {
                error,
                message,
                unhandled,
                ..
            } = task_error;
            // Log error
            if unhandled {
                log::error!("{error:?}");
            }
            // Discard any changes
            inner_tx.rollback().await?;
            // Mark task as errored
            complete_task(task.id, Some(&message), &mut tx).await?;
            // Mark user as touched
            update_user_queue(&user_queue_entry.id, &mut tx).await?;
        }
        Ok(TaskOutcome::Completed) => {
            // Mark task as complete, user as touched
            inner_tx.commit().await?;
            set_article_checked_time(&task.user_id, &task.article_id, &mut tx).await?;
            complete_task(task.id, None, &mut tx).await?;
            update_user_queue(&user_queue_entry.id, &mut tx).await?;
        }
        Err(e) => {
            // Log error and continue
            log::error!("{e:?}");
            inner_tx.rollback().await?;
        }
    }
    tx.commit().await?;
    Ok(TaskOutcome::Completed)
}

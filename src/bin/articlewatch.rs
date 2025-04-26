// Runs in the background to do article updates.

use dotenv::dotenv;
use libtater::article_updater::{update_task, TaskError, TaskOutcome};
use libtater::db::get_connection_options;
use libtater::setup_logging;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    setup_logging("log/articlewatch.log")?;
    let pool = PgPool::connect_with(get_connection_options()).await?;
    loop {
        let tx = pool.begin().await?;
        match update_task(tx).await? {
            TaskOutcome::NoTasks => {
                log::debug!("No tasks!");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            TaskOutcome::NoUser => {
                log::debug!("No user!");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            TaskOutcome::Error(task_err) => {
                let TaskError { error, .. } = task_err;
                log::error!("{error:?}");
            }
            _ => {}
        }
    }
}

// Runs in the background to do article updates.

use dotenv::dotenv;
use libtater::article_updater::{update_task, TaskError, TaskOutcome};
use libtater::db::get_connection_options;
use simplelog::TermLogger;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    TermLogger::init(
        simplelog::LevelFilter::Info,
        Default::default(),
        Default::default(),
        Default::default(),
    )?;
    let pool = PgPool::connect_with(get_connection_options()).await?;
    loop {
        let tx = pool.begin().await?;
        match update_task(tx).await? {
            TaskOutcome::NoTasks => {
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

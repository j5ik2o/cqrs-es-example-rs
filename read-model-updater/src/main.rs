extern crate log;

use anyhow::Result;
use lambda_runtime::{Error, service_fn};
use sqlx::MySqlPool;

use cqrs_es_example_read_model_updater::{load_app_config, update_read_model};
use cqrs_es_example_read_model_updater::thread_read_model_dao::ThreadReadModelDaoImpl;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .init();

    tracing::info!("main: start");
    let app_settings = load_app_config().unwrap();
    let pool = MySqlPool::connect(&app_settings.database.url).await?;
    let dao = ThreadReadModelDaoImpl::new(pool);
    let func = service_fn(|event| async { update_read_model(&dao, event).await });
    lambda_runtime::run(func).await?;
    tracing::info!("main: finished");

    Ok(())
}

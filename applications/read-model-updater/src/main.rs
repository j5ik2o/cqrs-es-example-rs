extern crate log;

use anyhow::Result;
use std::time::Duration;

use command_interface_adaptor_impl::gateways::group_chat_read_model_dao_impl::GroupChatReadModelUpdateDaoImpl;
use lambda_runtime::{service_fn, Error};
use read_model_updater::load_app_config;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode};

use rmu::update_read_model;

#[tokio::main]
async fn main() -> Result<(), Error> {
  tracing_subscriber::fmt()
    .json()
    .with_max_level(tracing::Level::DEBUG)
    .with_current_span(false)
    .with_ansi(false)
    .without_time()
    .with_target(false)
    .init();
  env_logger::init();

  tracing::info!("main: start");
  let app_settings = load_app_config().unwrap();
  tracing::info!("main: load_app_config");
  let database_url = app_settings.database.url;
  tracing::info!("main: database url: {:?}", database_url);
  let op: MySqlConnectOptions = database_url.parse()?;
  let op = op.ssl_mode(MySqlSslMode::Disabled);
  let pool = MySqlPoolOptions::new()
    .acquire_timeout(Duration::from_secs(60))
    .max_connections(2)
    .min_connections(1)
    .connect_with(op)
    .await?;

  tracing::info!("main: connect");
  let dao = GroupChatReadModelUpdateDaoImpl::new(pool);

  tracing::info!("main: start");
  lambda_runtime::run(service_fn(|event| async {
    tracing::info!("function: start");
    let result = update_read_model(&dao, event).await;
    tracing::info!("function: finished: {:?}", result);
    result
  }))
  .await?;
  tracing::info!("main: finished");

  Ok(())
}

extern crate log;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use aws_sdk_dynamodb::Client as DynamoDBClient;
use lambda_runtime::{service_fn, Error};
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode};

use command_interface_adaptor_impl::gateways::group_chat_read_model_dao_impl::GroupChatReadModelUpdateDaoImpl;
use command_interface_adaptor_impl::gateways::recovery_task_store_impl::MySqlRecoveryTaskStore;
use read_model_updater::{create_aws_config, load_app_config};
use rmu::{update_read_model, Applier, DynamoDbHeadScanner, DynamoDbJournalReader, ReadModelUpdater, RecoveryService};

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
  let database_url = app_settings.database.url;
  let op: MySqlConnectOptions = database_url.parse()?;
  let op = op.ssl_mode(MySqlSslMode::Disabled);
  let pool = MySqlPoolOptions::new()
    .acquire_timeout(Duration::from_secs(60))
    .max_connections(2)
    .min_connections(1)
    .connect_with(op)
    .await?;

  let stream_settings = app_settings
    .stream
    .ok_or_else(|| anyhow::anyhow!("stream settings are not configured"))?;
  let aws_config = create_aws_config(&app_settings.aws).await;
  let dynamodb_client = DynamoDBClient::new(&aws_config);

  let dao = Arc::new(GroupChatReadModelUpdateDaoImpl::new(pool.clone()));
  let recovery_store = Arc::new(MySqlRecoveryTaskStore::new(pool));
  let journal_reader = Arc::new(DynamoDbJournalReader::new(
    dynamodb_client.clone(),
    stream_settings.journal_table_name.clone(),
  ));
  let head_scanner = Arc::new(DynamoDbHeadScanner::new(
    dynamodb_client,
    stream_settings.head_table_name.clone(),
  ));
  let applier = Arc::new(Applier::new(
    dao,
    journal_reader,
    stream_settings.max_events_per_transaction,
  ));
  let recovery = Arc::new(RecoveryService::new(recovery_store, head_scanner, applier.clone()));
  let updater = ReadModelUpdater::new(applier, Some(recovery), stream_settings.retry_policy());

  tracing::info!("main: start");
  lambda_runtime::run(service_fn(|event| {
    let updater = &updater;
    async move { update_read_model(updater, event).await }
  }))
  .await?;
  tracing::info!("main: finished");

  Ok(())
}

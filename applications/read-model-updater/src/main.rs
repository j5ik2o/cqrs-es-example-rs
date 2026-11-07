extern crate log;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use aws_sdk_cloudwatch::Client as CloudWatchClient;
use aws_sdk_dynamodb::Client as DynamoDBClient;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode};

use command_interface_adaptor_impl::gateways::group_chat_read_model_dao_impl::GroupChatReadModelUpdateDaoImpl;
use command_interface_adaptor_impl::gateways::recovery_task_store_impl::MySqlRecoveryTaskStore;
use read_model_updater::recovery_worker::{handle_recovery_event, CloudWatchRecoveryMetrics, RMU_METRIC_NAMESPACE};
use read_model_updater::{create_aws_config, load_app_config};
use rmu::{
  update_read_model, Applier, DynamoDbHeadScanner, DynamoDbJournalReader, ReadModelUpdater, RecoveryService, Resyncer,
};

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
  let recovery_worker = app_settings.recovery_worker;
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
  let recovery = Arc::new(RecoveryService::new(
    recovery_store,
    head_scanner.clone(),
    applier.clone(),
  ));
  let resyncer = Resyncer::new(applier.clone(), head_scanner, Some(recovery.clone()));
  let updater = ReadModelUpdater::new(applier, Some(recovery.clone()), stream_settings.retry_policy());

  tracing::info!("main: start");
  if recovery_worker {
    let metrics = CloudWatchRecoveryMetrics::new(CloudWatchClient::new(&aws_config), RMU_METRIC_NAMESPACE);
    lambda_runtime::run(service_fn(move |event: LambdaEvent<serde_json::Value>| {
      let resyncer = resyncer.clone();
      let recovery = recovery.clone();
      let metrics = metrics.clone();
      async move { Ok::<_, Error>(handle_recovery_event(event.payload, &resyncer, &recovery, &metrics).await) }
    }))
    .await?;
  } else {
    lambda_runtime::run(service_fn(|event| {
      let updater = &updater;
      async move { update_read_model(updater, event).await }
    }))
    .await?;
  }
  tracing::info!("main: finished");

  Ok(())
}

use std::sync::Arc;

use anyhow::Result;
use aws_sdk_dynamodb::Client as DynamoDBClient;
use sqlx::MySqlPool;

use command_interface_adaptor_impl::gateways::group_chat_read_model_dao_impl::GroupChatReadModelUpdateDaoImpl;
use command_interface_adaptor_impl::gateways::recovery_task_store_impl::MySqlRecoveryTaskStore;
use read_model_updater::{create_aws_config, load_app_config};
use rmu::{Applier, DynamoDbHeadScanner, DynamoDbJournalReader, RecoveryService, Resyncer};

/// Replays the pending recovery tasks recorded by the read model updater.
///
/// Run this from the SQS OnFailure worker or by hand after repairing a read
/// model constraint. It first reconciles the head table (DY-15) and then
/// replays each pending task from the journal. The task is only marked complete
/// after the position reaches the task's seq_nr.
#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .with_target(false)
    .with_ansi(false)
    .without_time()
    .init();

  let app_settings = load_app_config()?;
  let stream_settings = app_settings
    .stream
    .clone()
    .ok_or_else(|| anyhow::anyhow!("stream_settings is not found"))?;
  let pool = MySqlPool::connect(&app_settings.database.url).await?;
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
  let resyncer = Resyncer::new(applier, head_scanner, Some(recovery.clone()));

  let mut failed = false;
  match resyncer.resync().await {
    Ok(result) => tracing::info!(
      heads = result.heads,
      advanced = result.advanced,
      skipped = result.skipped,
      failed = result.failed,
      "resync finished"
    ),
    Err(error) => {
      tracing::error!(error = %error, "resync did not complete");
      failed = true;
    }
  }

  match recovery.recover_pending().await {
    Ok(result) => {
      tracing::info!(
        completed = result.completed,
        failures = result.failures.len(),
        "recovery run finished"
      );
      if !result.failures.is_empty() {
        for failure in &result.failures {
          tracing::error!(failure = %failure, "recovery task remains unresolved");
        }
        failed = true;
      }
    }
    Err(error) => {
      tracing::error!(error = %error, "recovery run did not complete");
      failed = true;
    }
  }

  if failed {
    anyhow::bail!("some heads or recovery tasks remain unresolved");
  }
  Ok(())
}

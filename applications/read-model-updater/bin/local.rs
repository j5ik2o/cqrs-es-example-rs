use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use aws_sdk_dynamodb::Client as DynamoDBClient;
use aws_sdk_dynamodbstreams::Client as DynamoDBStreamsClient;
use sqlx::MySqlPool;

use command_interface_adaptor_impl::gateways::group_chat_read_model_dao_impl::GroupChatReadModelUpdateDaoImpl;
use command_interface_adaptor_impl::gateways::recovery_task_store_impl::MySqlRecoveryTaskStore;
use read_model_updater::{create_aws_config, load_app_config, StreamSettings};
use rmu::{
  poll_shards, Applier, Backoff, DynamoDbHeadScanner, DynamoDbHeadStreamClient, DynamoDbJournalReader,
  HeadStreamClient, ReadModelUpdater, RecoveryService, Resyncer, StreamDriverError, StreamState,
};

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
  let streams_client = DynamoDBStreamsClient::new(&aws_config);

  let components = build_components(&pool, &stream_settings, dynamodb_client.clone());
  let mut resync_backoff = Backoff::new(Duration::from_millis(500), Duration::from_secs(30));

  // The first start has no positions at all, and a restart may follow a stop
  // longer than the 24-hour stream retention. Reconcile every head before
  // consuming the stream; the stream then covers the gap between the scan and
  // the first read.
  resync_until_resolved(&components.resyncer, &mut resync_backoff, "initial reconciliation").await;
  match components.recovery.recover_pending().await {
    Ok(result) => {
      if !result.failures.is_empty() {
        tracing::warn!(
          completed = result.completed,
          failures = result.failures.len(),
          "some pending recovery tasks did not complete"
        );
      } else if result.completed > 0 {
        tracing::info!(completed = result.completed, "completed pending recovery tasks");
      }
    }
    Err(error) => tracing::warn!(error = %error, "failed to read pending recovery tasks"),
  }

  let stream_client = DynamoDbHeadStreamClient::new(dynamodb_client, streams_client);
  let mut error_backoff = Backoff::new(Duration::from_millis(500), Duration::from_secs(30));
  let mut stream_arn = loop {
    match stream_client.stream_arn(&stream_settings.head_table_name).await {
      Ok(stream_arn) => break stream_arn,
      Err(error) => {
        tracing::warn!(error = %error, "failed to describe the head table stream; backing off");
        tokio::time::sleep(error_backoff.next_delay()).await;
      }
    }
  };
  let mut state = StreamState::default();
  let mut poll_backoff = stream_settings.poll_backoff();

  loop {
    match poll_shards(
      &stream_client,
      &components.updater,
      &stream_arn,
      &mut state,
      stream_settings.max_item_count,
    )
    .await
    {
      Ok(outcome) => {
        if outcome.processed > 0 {
          poll_backoff.reset();
          error_backoff.reset();
        }
        if outcome.idle {
          tokio::time::sleep(poll_backoff.next_delay()).await;
        }
      }
      Err(StreamDriverError::Trimmed) => {
        tracing::warn!("the reader fell behind the stream retention window; running a full resync");
        resync_until_resolved(&components.resyncer, &mut resync_backoff, "resync after trimmed stream").await;
        state = StreamState::default();
        poll_backoff.reset();
        // A new stream may have been created after a table replacement.
        stream_arn = match stream_client.stream_arn(&stream_settings.head_table_name).await {
          Ok(stream_arn) => stream_arn,
          Err(error) => {
            tracing::warn!(error = %error, "failed to describe the head table stream; backing off");
            tokio::time::sleep(error_backoff.next_delay()).await;
            continue;
          }
        };
      }
      Err(StreamDriverError::Other(error)) => {
        tracing::warn!(error = %error, "stream processing failed; backing off");
        tokio::time::sleep(error_backoff.next_delay()).await;
      }
    }
  }
}

struct Components {
  updater: ReadModelUpdater,
  recovery: Arc<RecoveryService>,
  resyncer: Resyncer,
}

fn build_components(pool: &MySqlPool, stream_settings: &StreamSettings, dynamodb_client: DynamoDBClient) -> Components {
  let dao = Arc::new(GroupChatReadModelUpdateDaoImpl::new(pool.clone()));
  let recovery_store = Arc::new(MySqlRecoveryTaskStore::new(pool.clone()));
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
  let updater = ReadModelUpdater::new(applier.clone(), Some(recovery.clone()), stream_settings.retry_policy());
  let resyncer = Resyncer::new(applier, head_scanner, Some(recovery.clone()));
  Components {
    updater,
    recovery,
    resyncer,
  }
}

async fn resync_until_resolved(resyncer: &Resyncer, backoff: &mut Backoff, label: &str) {
  backoff.reset();
  loop {
    match resyncer.resync().await {
      Ok(result) => {
        tracing::info!(
          label = label,
          heads = result.heads,
          advanced = result.advanced,
          skipped = result.skipped,
          failed = result.failed,
          "resync finished"
        );
        return;
      }
      Err(error) => {
        tracing::error!(label = label, error = %error, "resync did not complete");
        if !error.is_retryable() {
          // Permanent per-aggregate failures were recorded as recovery tasks.
          // Continuing keeps the stream moving while they are repaired.
          return;
        }
        tokio::time::sleep(backoff.next_delay()).await;
      }
    }
  }
}

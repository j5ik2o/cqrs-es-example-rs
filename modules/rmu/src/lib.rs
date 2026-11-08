use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use aws_lambda_events::dynamodb;
use aws_lambda_events::event::streams::{DynamoDbBatchItemFailure, DynamoDbEventResponse};
use chrono::{DateTime, Utc};
use lambda_runtime::LambdaEvent;
use serde_dynamo::AttributeValue;
use thiserror::Error;

use command_domain::group_chat::{GroupChatEvent, GroupChatId};
use command_interface_adaptor_if::{GroupChatReadModelUpdateDao, GroupChatReadModelUpdateDaoError, ReadModelEvent};

pub mod applier;
pub mod failure;
pub mod head_scanner;
pub mod journal;
pub mod recovery;
pub mod recovery_pass;
pub mod resync;
pub mod retry;
pub mod stream;

pub use applier::Applier;
pub use failure::{classify_update_error, is_retryable_update_error, FailureKind};
pub use head_scanner::{DynamoDbHeadScanner, Head, HeadScanError, HeadScanner, StoreIdProvider};
pub use journal::{DynamoDbJournalReader, JournalReadError, JournalReader};
pub use recovery::{RecoveryPendingResult, RecoveryService, PROJECTION_NAME, PROJECTION_VERSION};
pub use recovery_pass::{run_recovery_pass, RecoveryPassResult};
pub use resync::{ResyncError, ResyncResult, Resyncer};
pub use retry::{full_jitter, Backoff, RetryError, RetryPolicy, Sleeper};
pub use stream::{
  catch_up_blocked, poll_shards, CatchUpOutcome, DynamoDbHeadStreamClient, HeadStreamClient, StreamDriverError,
  StreamReadError, StreamState,
};

/// 設定項目を保持する予約済みの集約 ID。ヘッドの Streams に一度だけ流れる。
pub const CONFIG_AGGREGATE_ID: &str = "__config__";

/// 購読対象の集約型名。
pub const GROUP_CHAT_TYPE_NAME: &str = "GroupChat";

#[derive(Debug, Error)]
pub enum UpdateReadModelError {
  #[error("Unexpected stream event name: {0}")]
  UnexpectedEventName(String),
  #[error("The stream record has no new image")]
  MissingNewImage,
  #[error("The head item is missing the {0} attribute")]
  MissingAttribute(&'static str),
  #[error("The head attribute {0} has an unexpected type")]
  UnexpectedAttributeType(&'static str),
  #[error("The head attribute {name} is not a valid number: {value}")]
  InvalidNumber { name: &'static str, value: String },
  #[error("The head attribute occurred_at is out of range: {0}")]
  InvalidOccurredAt(String),
  #[error("The aggregate id is invalid: {aggregate_id}: {reason}")]
  InvalidAggregateId { aggregate_id: String, reason: String },
  #[error("Failed to deserialize the event payload: {0}")]
  Payload(String),
  #[error("Journal read failed: {0}")]
  JournalRead(#[from] JournalReadError),
  #[error("The journal is missing the contiguous seq_nr range {from_seq_nr}..={to_seq_nr} for {aggregate_id}")]
  JournalGap {
    aggregate_id: String,
    from_seq_nr: u64,
    to_seq_nr: u64,
  },
  #[error("Read model update failed: {0}")]
  ReadModelUpdate(#[from] GroupChatReadModelUpdateDaoError),
  #[error("The position for {aggregate_id} stayed at {position} while replaying to {target_seq_nr}")]
  PositionStalled {
    aggregate_id: String,
    position: u64,
    target_seq_nr: u64,
  },
  #[error("Recovery task store failed: {0}")]
  RecoveryTaskStore(String),
  #[error("Failed to record a recovery task after an apply failure: apply={apply}, record={record}")]
  RecoveryRecord { apply: String, record: String },
  #[error("Recovery failed: {0}")]
  Recovery(String),
  #[error("Head scan failed: {0}")]
  HeadScan(String),
  #[error("Retry exhausted after {attempts} attempts in {elapsed:?}: {source}")]
  RetryExhausted {
    attempts: usize,
    elapsed: std::time::Duration,
    #[source]
    source: Box<UpdateReadModelError>,
  },
}

impl From<RetryError<UpdateReadModelError>> for UpdateReadModelError {
  fn from(error: RetryError<UpdateReadModelError>) -> Self {
    if error.permanent {
      // The policy does not retry a permanent failure, so keep the original
      // error and its classification instead of wrapping it as exhausted.
      return error.source;
    }
    UpdateReadModelError::RetryExhausted {
      attempts: error.attempts,
      elapsed: error.elapsed,
      source: Box::new(error.source),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordOutcome {
  pub event_id: String,
  pub aggregate_id: Option<String>,
  pub permanent: bool,
}

/// Applies head stream records to the read model with bounded retries and
/// durable recovery tasks.
pub struct ReadModelUpdater {
  applier: Arc<Applier>,
  recovery: Option<Arc<RecoveryService>>,
  retry: RetryPolicy,
}

impl ReadModelUpdater {
  pub fn new(applier: Arc<Applier>, recovery: Option<Arc<RecoveryService>>, retry: RetryPolicy) -> Self {
    Self {
      applier,
      recovery,
      retry,
    }
  }

  pub fn applier(&self) -> &Arc<Applier> {
    &self.applier
  }

  /// Applies one head stream record. A permanent failure writes a recovery task
  /// and is returned; a transient failure is returned after the retry budget.
  pub async fn process_record(&self, record: dynamodb::EventRecord) -> Result<RecordOutcome, UpdateReadModelError> {
    let event_id = record.event_id.clone();
    let parsed = match parse_head_record(&record) {
      Ok(parsed) => parsed,
      Err(parse_error) => {
        let permanent = classify_update_error(&parse_error.error).is_some();
        if permanent {
          if let (Some(recovery), Some(aggregate_id)) = (&self.recovery, &parse_error.aggregate_id) {
            if let Err(record_error) = recovery
              .record(aggregate_id, parse_error.seq_nr, &parse_error.error)
              .await
            {
              return Err(UpdateReadModelError::RecoveryRecord {
                apply: parse_error.error.to_string(),
                record: record_error.to_string(),
              });
            }
          }
        }
        return Err(parse_error.error);
      }
    };
    let Some(aggregate_id) = parsed.aggregate_id else {
      return Ok(RecordOutcome {
        event_id,
        aggregate_id: None,
        permanent: false,
      });
    };
    for event in parsed.events {
      let seq_nr = event.seq_nr();
      let result = self
        .retry
        .run(
          || self.applier.apply_head_event(&aggregate_id, event.clone()),
          is_retryable_update_error,
        )
        .await;
      if let Err(retry_error) = result {
        let error = UpdateReadModelError::from(retry_error);
        let permanent = classify_update_error(&error).is_some();
        if permanent {
          if let Some(recovery) = &self.recovery {
            if let Err(record_error) = recovery.record(&aggregate_id.to_string(), seq_nr, &error).await {
              return Err(UpdateReadModelError::RecoveryRecord {
                apply: error.to_string(),
                record: record_error.to_string(),
              });
            }
          }
        }
        return Err(error);
      }
    }
    Ok(RecordOutcome {
      event_id,
      aggregate_id: Some(aggregate_id.to_string()),
      permanent: false,
    })
  }

  /// Applies all records and stops at the first failure. The Lambda entry point
  /// uses [`ReadModelUpdater::handle_event`] so successful records in a batch are
  /// not replayed.
  pub async fn process_records(&self, records: Vec<dynamodb::EventRecord>) -> Result<(), UpdateReadModelError> {
    for record in records {
      self.process_record(record).await?;
    }
    Ok(())
  }

  /// Lambda entry point. Returns the records that must be redelivered as a
  /// partial batch response. Records of an aggregate whose earlier event failed
  /// are also left for redelivery: the position rule forbids advancing past an
  /// unapplied event, so applying them would fail again.
  pub async fn handle_event(&self, event: dynamodb::Event) -> Result<DynamoDbEventResponse, UpdateReadModelError> {
    let records = event.records;
    tracing::info!(records = records.len(), "processing head stream batch");
    let mut failures = Vec::new();
    let mut blocked: HashSet<String> = HashSet::new();
    for (index, record) in records.iter().enumerate() {
      let event_id = record.event_id.clone();
      let aggregate_id = head_record_aggregate_id(record);
      if let Some(aggregate_id) = &aggregate_id {
        if blocked.contains(aggregate_id) {
          tracing::warn!(
            aggregate_id = %aggregate_id,
            event_id = %event_id,
            "skip record of an aggregate with an unapplied event"
          );
          failures.push(batch_failure(event_id));
          continue;
        }
      }
      match self.process_record(record.clone()).await {
        Ok(_) => {}
        Err(error) => {
          let permanent = classify_update_error(&error).is_some();
          failures.push(batch_failure(event_id));
          if let Some(aggregate_id) = aggregate_id {
            if permanent {
              blocked.insert(aggregate_id);
            }
          }
          if !permanent {
            // A storage outage has already consumed the retry budget. The
            // remaining records would spend the same budget and risk the
            // Lambda timeout, so hand them all back for redelivery.
            for rest in &records[index + 1..] {
              failures.push(batch_failure(rest.event_id.clone()));
            }
            break;
          }
        }
      }
    }
    Ok(DynamoDbEventResponse {
      batch_item_failures: failures,
    })
  }
}

/// Lambda entry point wrapper.
pub async fn update_read_model(
  updater: &ReadModelUpdater,
  event: LambdaEvent<dynamodb::Event>,
) -> Result<DynamoDbEventResponse, UpdateReadModelError> {
  updater.handle_event(event.payload).await
}

/// Applies records directly with a borrowed DAO and journal. This keeps the
/// existing test seam; production uses [`ReadModelUpdater`].
pub async fn process_records(
  group_chat_read_model_dao: &dyn GroupChatReadModelUpdateDao,
  journal_reader: &dyn JournalReader,
  records: Vec<dynamodb::EventRecord>,
) -> Result<(), UpdateReadModelError> {
  for record in records {
    let parsed = parse_head_record(&record).map_err(|error| error.error)?;
    let Some(aggregate_id) = parsed.aggregate_id else {
      continue;
    };
    for event in parsed.events {
      applier::apply_head_event_with(group_chat_read_model_dao, journal_reader, 1, &aggregate_id, event).await?;
    }
  }
  Ok(())
}

fn batch_failure(event_id: String) -> DynamoDbBatchItemFailure {
  DynamoDbBatchItemFailure {
    item_identifier: Some(event_id),
  }
}

struct HeadRecord {
  aggregate_id: Option<GroupChatId>,
  events: Vec<ReadModelEvent>,
}

struct HeadParseError {
  error: UpdateReadModelError,
  aggregate_id: Option<String>,
  seq_nr: u64,
}

impl HeadParseError {
  fn without_identity(error: UpdateReadModelError) -> Self {
    Self {
      error,
      aggregate_id: None,
      seq_nr: 0,
    }
  }
}

/// Parses a head record into the aggregate and the events it carries.
///
/// Configuration items, other aggregate types, and REMOVE records are skipped.
fn parse_head_record(record: &dynamodb::EventRecord) -> Result<HeadRecord, HeadParseError> {
  match record.event_name.as_str() {
    "INSERT" | "MODIFY" => {}
    "REMOVE" => return Ok(skip_record()),
    other => {
      return Err(HeadParseError::without_identity(
        UpdateReadModelError::UnexpectedEventName(other.to_string()),
      ))
    }
  }
  let image = &record.change.new_image;
  if image.is_empty() {
    return Err(HeadParseError::without_identity(UpdateReadModelError::MissingNewImage));
  }
  let aid = match image.get("aid") {
    Some(AttributeValue::S(value)) => value.clone(),
    Some(_) => {
      return Err(HeadParseError::without_identity(
        UpdateReadModelError::UnexpectedAttributeType("aid"),
      ))
    }
    None => {
      return Err(HeadParseError::without_identity(
        UpdateReadModelError::MissingAttribute("aid"),
      ))
    }
  };
  if aid == CONFIG_AGGREGATE_ID {
    return Ok(skip_record());
  }
  let seq_nr = match image.get("seq_nr") {
    Some(AttributeValue::N(value)) => value.parse::<u64>().unwrap_or(0),
    _ => 0,
  };
  let type_name = match image.get("type_name") {
    Some(AttributeValue::S(value)) => value.clone(),
    Some(_) => {
      return Err(HeadParseError {
        error: UpdateReadModelError::UnexpectedAttributeType("type_name"),
        aggregate_id: Some(aid),
        seq_nr,
      })
    }
    None => aid
      .split_once('-')
      .map(|(name, _)| name.to_string())
      .unwrap_or_default(),
  };
  if type_name != GROUP_CHAT_TYPE_NAME {
    return Ok(skip_record());
  }
  let aggregate_id = match GroupChatId::from_str(&aid) {
    Ok(aggregate_id) => aggregate_id,
    Err(error) => {
      return Err(HeadParseError {
        error: UpdateReadModelError::InvalidAggregateId {
          aggregate_id: aid.clone(),
          reason: error.to_string(),
        },
        aggregate_id: Some(aid),
        seq_nr,
      })
    }
  };
  let events = match parse_head_events(image) {
    Ok(events) => events,
    Err(error) => {
      return Err(HeadParseError {
        error,
        aggregate_id: Some(aid),
        seq_nr,
      })
    }
  };
  Ok(HeadRecord {
    aggregate_id: Some(aggregate_id),
    events,
  })
}

fn skip_record() -> HeadRecord {
  HeadRecord {
    aggregate_id: None,
    events: Vec::new(),
  }
}

/// Returns the aggregate id of a head record without parsing the payload.
pub fn head_record_aggregate_id(record: &dynamodb::EventRecord) -> Option<String> {
  if !matches!(record.event_name.as_str(), "INSERT" | "MODIFY") {
    return None;
  }
  let image = &record.change.new_image;
  let aid = match image.get("aid") {
    Some(AttributeValue::S(value)) => value.clone(),
    _ => return None,
  };
  if aid == CONFIG_AGGREGATE_ID {
    return None;
  }
  let type_name = match image.get("type_name") {
    Some(AttributeValue::S(value)) => value.clone(),
    Some(_) => return None,
    None => aid
      .split_once('-')
      .map(|(name, _)| name.to_string())
      .unwrap_or_default(),
  };
  if type_name != GROUP_CHAT_TYPE_NAME {
    return None;
  }
  Some(aid)
}

fn parse_head_events(image: &HashMap<String, AttributeValue>) -> Result<Vec<ReadModelEvent>, UpdateReadModelError> {
  let items = match image.get("events") {
    Some(AttributeValue::L(items)) => items,
    Some(_) => return Err(UpdateReadModelError::UnexpectedAttributeType("events")),
    None => return Err(UpdateReadModelError::MissingAttribute("events")),
  };
  let mut events = Vec::with_capacity(items.len());
  for item in items {
    match item {
      AttributeValue::M(fields) => events.push(parse_head_event(fields)?),
      _ => return Err(UpdateReadModelError::UnexpectedAttributeType("events")),
    }
  }
  events.sort_by_key(ReadModelEvent::seq_nr);
  Ok(events)
}

fn parse_head_event(fields: &HashMap<String, AttributeValue>) -> Result<ReadModelEvent, UpdateReadModelError> {
  let seq_nr_value = number_attribute(fields, "seq_nr")?;
  let seq_nr = u64::try_from(seq_nr_value).map_err(|_| UpdateReadModelError::InvalidNumber {
    name: "seq_nr",
    value: seq_nr_value.to_string(),
  })?;
  let occurred_at = number_attribute(fields, "occurred_at")?;
  let _manifest = string_attribute(fields, "manifest")?;
  let payload = bytes_attribute(fields, "payload")?;
  let event =
    serde_json::from_slice::<GroupChatEvent>(&payload).map_err(|e| UpdateReadModelError::Payload(e.to_string()))?;
  Ok(ReadModelEvent::new(seq_nr, nanos_to_datetime(occurred_at)?, event))
}

fn number_attribute(item: &HashMap<String, AttributeValue>, name: &'static str) -> Result<i64, UpdateReadModelError> {
  match item.get(name) {
    None => Err(UpdateReadModelError::MissingAttribute(name)),
    Some(AttributeValue::N(value)) => value.parse::<i64>().map_err(|_| UpdateReadModelError::InvalidNumber {
      name,
      value: value.clone(),
    }),
    Some(_) => Err(UpdateReadModelError::UnexpectedAttributeType(name)),
  }
}

fn string_attribute(
  item: &HashMap<String, AttributeValue>,
  name: &'static str,
) -> Result<String, UpdateReadModelError> {
  match item.get(name) {
    None => Err(UpdateReadModelError::MissingAttribute(name)),
    Some(AttributeValue::S(value)) => Ok(value.clone()),
    Some(_) => Err(UpdateReadModelError::UnexpectedAttributeType(name)),
  }
}

fn bytes_attribute(
  item: &HashMap<String, AttributeValue>,
  name: &'static str,
) -> Result<Vec<u8>, UpdateReadModelError> {
  match item.get(name) {
    None => Err(UpdateReadModelError::MissingAttribute(name)),
    Some(AttributeValue::B(value)) => Ok(value.clone()),
    Some(AttributeValue::S(value)) => Ok(value.as_bytes().to_vec()),
    Some(_) => Err(UpdateReadModelError::UnexpectedAttributeType(name)),
  }
}

fn nanos_to_datetime(nanos: i64) -> Result<DateTime<Utc>, UpdateReadModelError> {
  let seconds = nanos.div_euclid(1_000_000_000);
  let subsec_nanos = nanos.rem_euclid(1_000_000_000) as u32;
  DateTime::from_timestamp(seconds, subsec_nanos).ok_or(UpdateReadModelError::InvalidOccurredAt(nanos.to_string()))
}

#[cfg(test)]
mod tests;

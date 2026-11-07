use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use async_trait::async_trait;
use aws_lambda_events::dynamodb;
use aws_lambda_events::dynamodb::{StreamRecord, StreamViewType};
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_dynamodbstreams::error::SdkError;
use aws_sdk_dynamodbstreams::operation::get_records::GetRecordsError;
use aws_sdk_dynamodbstreams::operation::get_shard_iterator::GetShardIteratorError;
use aws_sdk_dynamodbstreams::types::ShardIteratorType;
use aws_sdk_dynamodbstreams::Client as DynamoDbStreamsClient;
use chrono::Utc;
use command_domain::group_chat::GroupChatId;
use thiserror::Error;

use crate::failure::classify_update_error;
use crate::{head_record_aggregate_id, Applier, ReadModelUpdater, RecordOutcome, UpdateReadModelError};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StreamReadError {
  /// The 15-minute iterator expired. The records are still in the stream.
  #[error("the stream iterator expired")]
  ExpiredIterator,
  /// The requested position is older than the 24-hour retention window.
  #[error("the stream records were trimmed")]
  TrimmedData,
  #[error("{0}")]
  Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamShard {
  pub shard_id: String,
}

#[derive(Debug)]
pub struct StreamRecordPage {
  pub records: Vec<dynamodb::EventRecord>,
  pub next_iterator: Option<String>,
}

/// The subset of DynamoDB Streams the reader needs. The driver is written
/// against this trait so ExpiredIterator and TrimmedData handling can be tested
/// without constructing AWS SDK errors.
#[async_trait]
pub trait HeadStreamClient: Send + Sync {
  async fn stream_arn(&self, head_table_name: &str) -> Result<String, StreamReadError>;
  async fn shards(&self, stream_arn: &str) -> Result<Vec<StreamShard>, StreamReadError>;
  async fn shard_iterator(
    &self,
    stream_arn: &str,
    shard_id: &str,
    after_sequence_number: Option<&str>,
  ) -> Result<String, StreamReadError>;
  async fn get_records(&self, iterator: &str, limit: i32) -> Result<StreamRecordPage, StreamReadError>;
}

/// Processes one head record. `ReadModelUpdater` is the production
/// implementation; tests use a fake.
#[async_trait]
pub trait RecordProcessor: Send + Sync {
  async fn process_record(&self, record: dynamodb::EventRecord) -> Result<RecordOutcome, UpdateReadModelError>;
}

#[async_trait]
impl RecordProcessor for ReadModelUpdater {
  async fn process_record(&self, record: dynamodb::EventRecord) -> Result<RecordOutcome, UpdateReadModelError> {
    ReadModelUpdater::process_record(self, record).await
  }
}

#[derive(Debug, Error)]
pub enum StreamDriverError {
  #[error("The head stream records were trimmed; run a full resync")]
  Trimmed,
  #[error("Head stream read failed: {0}")]
  Other(String),
}

#[derive(Debug, Default)]
pub struct StreamState {
  /// The last successfully processed Streams sequence number per shard. It is
  /// used to recreate an iterator after ExpiredIterator, not to skip records.
  pub last_sequence_numbers: HashMap<String, String>,
  /// Aggregates with an unapplied event, mapped to the head seq_nr that must be
  /// reached before they are unblocked. Their later records are skipped until
  /// the journal replay catches up to that position.
  pub blocked_aggregates: HashMap<String, u64>,
  /// Shards whose iterator returned no next iterator.
  pub closed_shards: HashSet<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PollOutcome {
  pub processed: usize,
  pub idle: bool,
}

/// Reads every shard once until each one has no more records. The caller owns
/// the backoff and the full-resync decision.
pub async fn poll_shards(
  client: &dyn HeadStreamClient,
  processor: &dyn RecordProcessor,
  stream_arn: &str,
  state: &mut StreamState,
  max_item_count: i32,
) -> Result<PollOutcome, StreamDriverError> {
  let shards = client
    .shards(stream_arn)
    .await
    .map_err(|error| StreamDriverError::Other(error.to_string()))?;
  let mut outcome = PollOutcome::default();
  for shard in shards {
    if state.closed_shards.contains(&shard.shard_id) {
      continue;
    }
    let mut iterator = client
      .shard_iterator(
        stream_arn,
        &shard.shard_id,
        state.last_sequence_numbers.get(&shard.shard_id).map(String::as_str),
      )
      .await
      .map_err(map_shard_iterator_error)?;
    loop {
      let page = match client.get_records(&iterator, max_item_count).await {
        Ok(page) => page,
        Err(StreamReadError::ExpiredIterator) => {
          // The iterator expired; the records are still retained. Recreate the
          // iterator after the last processed sequence number.
          tracing::warn!(
            shard_id = %shard.shard_id,
            sequence_number = ?state.last_sequence_numbers.get(&shard.shard_id),
            "stream iterator expired; resuming after the last processed sequence number"
          );
          iterator = client
            .shard_iterator(
              stream_arn,
              &shard.shard_id,
              state.last_sequence_numbers.get(&shard.shard_id).map(String::as_str),
            )
            .await
            .map_err(map_shard_iterator_error)?;
          continue;
        }
        Err(StreamReadError::TrimmedData) => return Err(StreamDriverError::Trimmed),
        Err(StreamReadError::Other(error)) => return Err(StreamDriverError::Other(error)),
      };
      if page.records.is_empty() {
        if page.next_iterator.is_none() {
          state.closed_shards.insert(shard.shard_id.clone());
        }
        outcome.idle = true;
        break;
      }
      for record in &page.records {
        let head_seq_nr = head_record_seq_nr(record);
        let aggregate_id = head_record_aggregate_id(record);
        if let Some(aggregate_id) = &aggregate_id {
          if state.blocked_aggregates.contains_key(aggregate_id) {
            let target = state
              .blocked_aggregates
              .entry(aggregate_id.clone())
              .or_insert(head_seq_nr);
            *target = (*target).max(head_seq_nr);
            let target = *target;
            tracing::warn!(
              aggregate_id = %aggregate_id,
              event_id = %record.event_id,
              target_seq_nr = target,
              "skip head record of an aggregate with an unapplied event"
            );
            update_last_sequence(state, &shard.shard_id, record);
            continue;
          }
        }
        match processor.process_record(record.clone()).await {
          Ok(_) => {
            outcome.processed += 1;
            update_last_sequence(state, &shard.shard_id, record);
          }
          Err(error) => {
            if let Some(kind) = classify_update_error(&error) {
              if let Some(aggregate_id) = &aggregate_id {
                let target = state
                  .blocked_aggregates
                  .entry(aggregate_id.clone())
                  .or_insert(head_seq_nr);
                *target = (*target).max(head_seq_nr);
              }
              tracing::error!(
                aggregate_id = ?aggregate_id,
                event_id = %record.event_id,
                failure = ?kind,
                "head record failed permanently; recorded a recovery task"
              );
              update_last_sequence(state, &shard.shard_id, record);
              continue;
            }
            return Err(StreamDriverError::Other(error.to_string()));
          }
        }
      }
      match page.next_iterator {
        Some(next_iterator) => iterator = next_iterator,
        None => {
          state.closed_shards.insert(shard.shard_id.clone());
          break;
        }
      }
    }
  }
  Ok(outcome)
}

fn update_last_sequence(state: &mut StreamState, shard_id: &str, record: &dynamodb::EventRecord) {
  if let Some(sequence_number) = &record.change.sequence_number {
    state
      .last_sequence_numbers
      .insert(shard_id.to_string(), sequence_number.clone());
  }
}

fn map_shard_iterator_error(error: StreamReadError) -> StreamDriverError {
  match error {
    StreamReadError::TrimmedData => StreamDriverError::Trimmed,
    other => StreamDriverError::Other(other.to_string()),
  }
}

fn head_record_seq_nr(record: &dynamodb::EventRecord) -> u64 {
  record
    .change
    .new_image
    .get("seq_nr")
    .and_then(|value| match value {
      serde_dynamo::AttributeValue::N(value) => value.parse().ok(),
      _ => None,
    })
    .unwrap_or(0)
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CatchUpOutcome {
  pub unblocked: usize,
  pub still_blocked: usize,
}

/// Replays the journal for blocked aggregates. It is called between polls so a
/// permanently failing aggregate can be unblocked even when no later head
/// record arrives. A transient failure is returned to the caller for backoff;
/// a permanent failure keeps the aggregate blocked for the next attempt.
pub async fn catch_up_blocked(
  applier: &Applier,
  state: &mut StreamState,
) -> Result<CatchUpOutcome, UpdateReadModelError> {
  let blocked = state
    .blocked_aggregates
    .iter()
    .map(|(aggregate_id, target_seq_nr)| (aggregate_id.clone(), *target_seq_nr))
    .collect::<Vec<_>>();
  let mut outcome = CatchUpOutcome::default();
  for (aggregate_id, target_seq_nr) in blocked {
    let parsed = match GroupChatId::from_str(&aggregate_id) {
      Ok(parsed) => parsed,
      Err(error) => {
        tracing::error!(
          aggregate_id = %aggregate_id,
          "blocked aggregate id is invalid; dropping the block: {}",
          error
        );
        state.blocked_aggregates.remove(&aggregate_id);
        continue;
      }
    };
    match applier.apply_through(&parsed, target_seq_nr).await {
      Ok(()) => {
        state.blocked_aggregates.remove(&aggregate_id);
        outcome.unblocked += 1;
      }
      Err(error) if classify_update_error(&error).is_some() => {
        tracing::error!(
          aggregate_id = %aggregate_id,
          target_seq_nr = target_seq_nr,
          failure = ?classify_update_error(&error),
          "blocked aggregate still cannot be applied"
        );
        outcome.still_blocked += 1;
      }
      Err(error) => return Err(error),
    }
  }
  Ok(outcome)
}

/// Production adapter over the DynamoDB and DynamoDB Streams SDK clients.
pub struct DynamoDbHeadStreamClient {
  dynamodb: DynamoDbClient,
  streams: DynamoDbStreamsClient,
}

impl DynamoDbHeadStreamClient {
  pub fn new(dynamodb: DynamoDbClient, streams: DynamoDbStreamsClient) -> Self {
    Self { dynamodb, streams }
  }
}

#[async_trait]
impl HeadStreamClient for DynamoDbHeadStreamClient {
  async fn stream_arn(&self, head_table_name: &str) -> Result<String, StreamReadError> {
    let output = self
      .dynamodb
      .describe_table()
      .table_name(head_table_name)
      .send()
      .await
      .map_err(|error| StreamReadError::Other(error.to_string()))?;
    output
      .table()
      .and_then(|table| table.latest_stream_arn())
      .map(ToString::to_string)
      .ok_or_else(|| {
        StreamReadError::Other(format!(
          "the head table {} has no DynamoDB Streams; enable Streams with NEW_IMAGE",
          head_table_name
        ))
      })
  }

  async fn shards(&self, stream_arn: &str) -> Result<Vec<StreamShard>, StreamReadError> {
    let mut shards = Vec::new();
    let mut last_evaluated_shard_id: Option<String> = None;
    loop {
      let mut request = self.streams.describe_stream().stream_arn(stream_arn);
      if let Some(shard_id) = last_evaluated_shard_id.clone() {
        request = request.exclusive_start_shard_id(shard_id);
      }
      let output = request
        .send()
        .await
        .map_err(|error| StreamReadError::Other(error.to_string()))?;
      let description = output
        .stream_description()
        .ok_or_else(|| StreamReadError::Other("the stream description is missing".to_string()))?;
      for shard in description.shards() {
        if let Some(shard_id) = shard.shard_id() {
          shards.push(StreamShard {
            shard_id: shard_id.to_string(),
          });
        }
      }
      match description.last_evaluated_shard_id() {
        Some(shard_id) if !shard_id.is_empty() => last_evaluated_shard_id = Some(shard_id.to_string()),
        _ => break,
      }
    }
    Ok(shards)
  }

  async fn shard_iterator(
    &self,
    stream_arn: &str,
    shard_id: &str,
    after_sequence_number: Option<&str>,
  ) -> Result<String, StreamReadError> {
    let mut request = self
      .streams
      .get_shard_iterator()
      .stream_arn(stream_arn)
      .shard_id(shard_id);
    request = match after_sequence_number {
      Some(sequence_number) => request
        .shard_iterator_type(ShardIteratorType::AfterSequenceNumber)
        .sequence_number(sequence_number),
      None => request.shard_iterator_type(ShardIteratorType::TrimHorizon),
    };
    let output = request.send().await.map_err(map_get_shard_iterator_error)?;
    output
      .shard_iterator()
      .map(ToString::to_string)
      .ok_or_else(|| StreamReadError::Other("the shard iterator is missing".to_string()))
  }

  async fn get_records(&self, iterator: &str, limit: i32) -> Result<StreamRecordPage, StreamReadError> {
    let output = self
      .streams
      .get_records()
      .shard_iterator(iterator)
      .limit(limit)
      .send()
      .await
      .map_err(map_get_records_error)?;
    Ok(StreamRecordPage {
      records: event_records_from_stream_records(output.records())?,
      next_iterator: output.next_shard_iterator().map(ToString::to_string),
    })
  }
}

fn map_get_shard_iterator_error<R>(error: SdkError<GetShardIteratorError, R>) -> StreamReadError
where
  R: std::fmt::Debug, {
  match &error {
    SdkError::ServiceError(service_error) => match service_error.err() {
      // AfterSequenceNumber past the retention window means the reader must
      // reconcile the whole head table, not just resume from a later position.
      GetShardIteratorError::TrimmedDataAccessException(_) => StreamReadError::TrimmedData,
      other => StreamReadError::Other(other.to_string()),
    },
    _ => StreamReadError::Other(error.to_string()),
  }
}

fn map_get_records_error<R>(error: SdkError<GetRecordsError, R>) -> StreamReadError
where
  R: std::fmt::Debug, {
  match &error {
    SdkError::ServiceError(service_error) => match service_error.err() {
      GetRecordsError::ExpiredIteratorException(_) => StreamReadError::ExpiredIterator,
      GetRecordsError::TrimmedDataAccessException(_) => StreamReadError::TrimmedData,
      other => StreamReadError::Other(other.to_string()),
    },
    _ => StreamReadError::Other(error.to_string()),
  }
}

fn event_records_from_stream_records(
  records: &[aws_sdk_dynamodbstreams::types::Record],
) -> Result<Vec<dynamodb::EventRecord>, StreamReadError> {
  records
    .iter()
    .map(|record| {
      let stream_record = record
        .dynamodb()
        .ok_or_else(|| StreamReadError::Other("the stream record has no dynamodb payload".to_string()))?;
      let new_image: serde_dynamo::Item = convert_item(stream_record.new_image().unwrap_or(&HashMap::new())).into();
      let old_image: serde_dynamo::Item = convert_item(stream_record.old_image().unwrap_or(&HashMap::new())).into();
      let keys: serde_dynamo::Item = convert_item(stream_record.keys().unwrap_or(&HashMap::new())).into();
      let event_name = record
        .event_name()
        .map(|name| name.as_str().to_string())
        .unwrap_or_else(|| "INSERT".to_string());
      Ok(dynamodb::EventRecord {
        aws_region: record.aws_region().unwrap_or("ap-northeast-1").to_string(),
        change: StreamRecord {
          approximate_creation_date_time: Utc::now(),
          keys,
          new_image,
          old_image,
          sequence_number: stream_record.sequence_number().map(ToString::to_string),
          size_bytes: stream_record.size_bytes().unwrap_or_default(),
          stream_view_type: Some(StreamViewType::NewImage),
        },
        event_id: record
          .event_id()
          .map(ToString::to_string)
          .unwrap_or_else(|| command_domain::id_generate().to_string()),
        event_name,
        event_source: record.event_source().map(ToString::to_string),
        event_version: record.event_version().map(ToString::to_string),
        event_source_arn: None,
        user_identity: None,
        record_format: None,
        table_name: None,
      })
    })
    .collect()
}

fn convert_to(src: aws_sdk_dynamodbstreams::types::AttributeValue) -> serde_dynamo::AttributeValue {
  match src {
    aws_sdk_dynamodbstreams::types::AttributeValue::B(b) => serde_dynamo::AttributeValue::B(b.into_inner()),
    aws_sdk_dynamodbstreams::types::AttributeValue::Bool(b) => serde_dynamo::AttributeValue::Bool(b),
    aws_sdk_dynamodbstreams::types::AttributeValue::Bs(bs) => {
      serde_dynamo::AttributeValue::Bs(bs.iter().map(|e| e.clone().into_inner()).collect())
    }
    aws_sdk_dynamodbstreams::types::AttributeValue::L(l) => {
      serde_dynamo::AttributeValue::L(l.iter().map(|v| convert_to(v.clone())).collect())
    }
    aws_sdk_dynamodbstreams::types::AttributeValue::M(m) => {
      serde_dynamo::AttributeValue::M(m.iter().map(|(k, v)| (k.clone(), convert_to(v.clone()))).collect())
    }
    aws_sdk_dynamodbstreams::types::AttributeValue::N(n) => serde_dynamo::AttributeValue::N(n.clone()),
    aws_sdk_dynamodbstreams::types::AttributeValue::Ns(ns) => serde_dynamo::AttributeValue::Ns(ns.clone()),
    aws_sdk_dynamodbstreams::types::AttributeValue::Null(b) => serde_dynamo::AttributeValue::Null(b),
    aws_sdk_dynamodbstreams::types::AttributeValue::S(s) => serde_dynamo::AttributeValue::S(s.clone()),
    aws_sdk_dynamodbstreams::types::AttributeValue::Ss(ss) => serde_dynamo::AttributeValue::Ss(ss.clone()),
    _ => panic!("not supported"),
  }
}

fn convert_item(
  item: &HashMap<String, aws_sdk_dynamodbstreams::types::AttributeValue>,
) -> HashMap<String, serde_dynamo::AttributeValue> {
  item.iter().map(|(k, v)| (k.clone(), convert_to(v.clone()))).collect()
}

#[cfg(test)]
mod tests {
  use std::collections::VecDeque;
  use std::sync::Mutex;

  use super::*;

  struct FakeProcessor {
    calls: Mutex<usize>,
    error: Option<UpdateReadModelError>,
  }

  impl FakeProcessor {
    fn ok() -> Self {
      Self {
        calls: Mutex::new(0),
        error: None,
      }
    }

    fn failing(error: UpdateReadModelError) -> Self {
      Self {
        calls: Mutex::new(0),
        error: Some(error),
      }
    }

    fn calls(&self) -> usize {
      *self.calls.lock().unwrap()
    }
  }

  #[async_trait]
  impl RecordProcessor for FakeProcessor {
    async fn process_record(&self, _: dynamodb::EventRecord) -> Result<RecordOutcome, UpdateReadModelError> {
      *self.calls.lock().unwrap() += 1;
      match &self.error {
        Some(error) => Err(clone_error(error)),
        None => Ok(RecordOutcome {
          event_id: "id".to_string(),
          aggregate_id: None,
          permanent: false,
        }),
      }
    }
  }

  fn clone_error(error: &UpdateReadModelError) -> UpdateReadModelError {
    match error {
      UpdateReadModelError::Payload(message) => UpdateReadModelError::Payload(message.clone()),
      UpdateReadModelError::RecoveryTaskStore(message) => UpdateReadModelError::RecoveryTaskStore(message.clone()),
      other => UpdateReadModelError::Recovery(other.to_string()),
    }
  }

  struct FakeClient {
    pages: Mutex<VecDeque<Result<StreamRecordPage, StreamReadError>>>,
    iterator_calls: Mutex<Vec<(String, Option<String>)>>,
    shard_iterator_error: Option<StreamReadError>,
  }

  impl FakeClient {
    fn new(pages: Vec<Result<StreamRecordPage, StreamReadError>>) -> Self {
      Self {
        pages: Mutex::new(pages.into()),
        iterator_calls: Mutex::new(Vec::new()),
        shard_iterator_error: None,
      }
    }

    fn with_iterator_error(error: StreamReadError) -> Self {
      Self {
        pages: Mutex::new(VecDeque::new()),
        iterator_calls: Mutex::new(Vec::new()),
        shard_iterator_error: Some(error),
      }
    }

    fn iterator_calls(&self) -> Vec<(String, Option<String>)> {
      self.iterator_calls.lock().unwrap().clone()
    }
  }

  #[async_trait]
  impl HeadStreamClient for FakeClient {
    async fn stream_arn(&self, _: &str) -> Result<String, StreamReadError> {
      Ok("arn".to_string())
    }

    async fn shards(&self, _: &str) -> Result<Vec<StreamShard>, StreamReadError> {
      Ok(vec![StreamShard {
        shard_id: "shard-1".to_string(),
      }])
    }

    async fn shard_iterator(
      &self,
      _: &str,
      shard_id: &str,
      after_sequence_number: Option<&str>,
    ) -> Result<String, StreamReadError> {
      self
        .iterator_calls
        .lock()
        .unwrap()
        .push((shard_id.to_string(), after_sequence_number.map(ToString::to_string)));
      if let Some(error) = &self.shard_iterator_error {
        return Err(error.clone());
      }
      Ok(format!("iterator-{}", self.iterator_calls().len()))
    }

    async fn get_records(&self, _: &str, _: i32) -> Result<StreamRecordPage, StreamReadError> {
      self.pages.lock().unwrap().pop_front().unwrap_or(Ok(StreamRecordPage {
        records: Vec::new(),
        next_iterator: None,
      }))
    }
  }

  fn record(aggregate_id: &str, sequence_number: &str) -> dynamodb::EventRecord {
    let image: serde_dynamo::Item = HashMap::from([
      (
        "aid".to_string(),
        serde_dynamo::AttributeValue::S(aggregate_id.to_string()),
      ),
      (
        "type_name".to_string(),
        serde_dynamo::AttributeValue::S("GroupChat".to_string()),
      ),
      ("seq_nr".to_string(), serde_dynamo::AttributeValue::N("1".to_string())),
      ("events".to_string(), serde_dynamo::AttributeValue::L(Vec::new())),
    ])
    .into();
    dynamodb::EventRecord {
      aws_region: "ap-northeast-1".to_string(),
      change: StreamRecord {
        approximate_creation_date_time: Utc::now(),
        keys: Default::default(),
        new_image: image,
        old_image: Default::default(),
        sequence_number: Some(sequence_number.to_string()),
        size_bytes: 1,
        stream_view_type: Some(StreamViewType::NewImage),
      },
      event_id: format!("event-{}", sequence_number),
      event_name: "MODIFY".to_string(),
      event_source: None,
      event_version: None,
      event_source_arn: None,
      user_identity: None,
      record_format: None,
      table_name: None,
    }
  }

  #[tokio::test]
  async fn should_resume_after_the_last_sequence_number_when_the_iterator_expires() {
    let client = FakeClient::new(vec![
      Ok(StreamRecordPage {
        records: vec![record("GroupChat-01H", "seq-1")],
        next_iterator: Some("iterator-2".to_string()),
      }),
      Err(StreamReadError::ExpiredIterator),
      Ok(StreamRecordPage {
        records: Vec::new(),
        next_iterator: None,
      }),
    ]);
    let processor = FakeProcessor::ok();
    let mut state = StreamState::default();

    let outcome = poll_shards(&client, &processor, "arn", &mut state, 10).await.unwrap();

    assert_eq!(1, outcome.processed);
    assert!(outcome.idle);
    assert_eq!(
      vec![
        ("shard-1".to_string(), None),
        ("shard-1".to_string(), Some("seq-1".to_string()))
      ],
      client.iterator_calls()
    );
  }

  #[tokio::test]
  async fn should_signal_a_full_resync_when_the_stream_is_trimmed() {
    let client = FakeClient::new(vec![Err(StreamReadError::TrimmedData)]);
    let processor = FakeProcessor::ok();
    let mut state = StreamState::default();

    let error = poll_shards(&client, &processor, "arn", &mut state, 10)
      .await
      .unwrap_err();

    assert!(matches!(error, StreamDriverError::Trimmed));
  }

  #[tokio::test]
  async fn should_block_an_aggregate_after_a_permanent_failure() {
    let client = FakeClient::new(vec![Ok(StreamRecordPage {
      records: vec![record("GroupChat-01H", "seq-1"), record("GroupChat-01H", "seq-2")],
      next_iterator: None,
    })]);
    let processor = FakeProcessor::failing(UpdateReadModelError::Payload("bad payload".to_string()));
    let mut state = StreamState::default();

    let outcome = poll_shards(&client, &processor, "arn", &mut state, 10).await.unwrap();

    assert_eq!(0, outcome.processed);
    assert_eq!(1, processor.calls());
    assert_eq!(Some(&1), state.blocked_aggregates.get("GroupChat-01H"));
    assert_eq!(Some(&"seq-2".to_string()), state.last_sequence_numbers.get("shard-1"));
  }

  #[tokio::test]
  async fn should_signal_a_full_resync_when_the_shard_iterator_is_trimmed() {
    let client = FakeClient::with_iterator_error(StreamReadError::TrimmedData);
    let processor = FakeProcessor::ok();
    let mut state = StreamState::default();

    let error = poll_shards(&client, &processor, "arn", &mut state, 10)
      .await
      .unwrap_err();

    assert!(matches!(error, StreamDriverError::Trimmed));
  }

  #[tokio::test]
  async fn should_return_a_transient_failure_to_the_caller() {
    let client = FakeClient::new(vec![Ok(StreamRecordPage {
      records: vec![record("GroupChat-01H", "seq-1")],
      next_iterator: None,
    })]);
    let processor = FakeProcessor::failing(UpdateReadModelError::RecoveryTaskStore("unavailable".to_string()));
    let mut state = StreamState::default();

    let error = poll_shards(&client, &processor, "arn", &mut state, 10)
      .await
      .unwrap_err();

    assert!(matches!(error, StreamDriverError::Other(_)));
    assert!(state.last_sequence_numbers.is_empty());
  }

  #[tokio::test]
  async fn should_mark_an_empty_page_idle_without_sleeping() {
    // The driver is polled by the caller; a zero-item page only marks the poll
    // idle. This test documents that contract without a timer.
    let client = FakeClient::new(vec![Ok(StreamRecordPage {
      records: Vec::new(),
      next_iterator: None,
    })]);
    let processor = FakeProcessor::ok();
    let mut state = StreamState::default();
    let outcome = poll_shards(&client, &processor, "arn", &mut state, 10).await.unwrap();
    assert!(outcome.idle);
    assert_eq!(0, outcome.processed);
  }
}

use std::collections::HashMap;
use std::fmt::Debug;

use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use chrono::{DateTime, Utc};
use thiserror::Error;

use command_domain::group_chat::{GroupChatEvent, GroupChatId};
use command_interface_adaptor_if::ReadModelEvent;

/// journal からイベントを読むためのエラー。
#[derive(Debug, Error)]
pub enum JournalReadError {
  #[error("DynamoDB query failed: {0}")]
  Query(String),
  #[error("The journal item is missing the {0} attribute")]
  MissingAttribute(&'static str),
  #[error("The journal attribute {0} has an unexpected type")]
  UnexpectedAttributeType(&'static str),
  #[error("The journal attribute {name} is not a valid number: {value}")]
  InvalidNumber { name: &'static str, value: String },
  #[error("The journal attribute occurred_at is out of range: {0}")]
  InvalidOccurredAt(String),
  #[error("Failed to deserialize the event payload: {0}")]
  Payload(String),
}

/// `seq_nr` の欠番を埋めるために journal を読む。
///
/// ヘッドの Streams で欠番を見つけたとき、足りないイベントをここから補う。
#[async_trait::async_trait]
pub trait JournalReader: Debug + Send + Sync {
  /// `from_seq_nr <= seq_nr <= to_seq_nr` のイベントを昇順で返す。
  async fn read_events(
    &self,
    aggregate_id: &GroupChatId,
    from_seq_nr: u64,
    to_seq_nr: u64,
  ) -> Result<Vec<ReadModelEvent>, JournalReadError>;
}

/// DynamoDB の journal テーブルを強整合で読む [JournalReader]。
#[derive(Debug, Clone)]
pub struct DynamoDbJournalReader {
  client: Client,
  table_name: String,
}

impl DynamoDbJournalReader {
  pub fn new(client: Client, table_name: impl Into<String>) -> Self {
    Self {
      client,
      table_name: table_name.into(),
    }
  }
}

#[async_trait::async_trait]
impl JournalReader for DynamoDbJournalReader {
  async fn read_events(
    &self,
    aggregate_id: &GroupChatId,
    from_seq_nr: u64,
    to_seq_nr: u64,
  ) -> Result<Vec<ReadModelEvent>, JournalReadError> {
    if from_seq_nr > to_seq_nr {
      return Ok(Vec::new());
    }
    let mut events = Vec::new();
    let mut exclusive_start_key: Option<HashMap<String, AttributeValue>> = None;
    loop {
      let mut request = self
        .client
        .query()
        .table_name(&self.table_name)
        .consistent_read(true)
        .scan_index_forward(true)
        .key_condition_expression("aid = :aid AND seq_nr BETWEEN :from_seq_nr AND :to_seq_nr")
        .expression_attribute_values(":aid", AttributeValue::S(aggregate_id.to_string()))
        .expression_attribute_values(":from_seq_nr", AttributeValue::N(from_seq_nr.to_string()))
        .expression_attribute_values(":to_seq_nr", AttributeValue::N(to_seq_nr.to_string()));
      if let Some(start_key) = exclusive_start_key.take() {
        request = request.set_exclusive_start_key(Some(start_key));
      }
      let output = request
        .send()
        .await
        .map_err(|e| JournalReadError::Query(e.to_string()))?;
      for item in output.items() {
        events.push(journal_event_from_item(item)?);
      }
      match output.last_evaluated_key() {
        Some(key) if !key.is_empty() => exclusive_start_key = Some(key.clone()),
        _ => break,
      }
    }
    Ok(events)
  }
}

fn journal_event_from_item(item: &HashMap<String, AttributeValue>) -> Result<ReadModelEvent, JournalReadError> {
  let seq_nr_value = number_attribute(item, "seq_nr")?;
  let seq_nr = u64::try_from(seq_nr_value).map_err(|_| JournalReadError::InvalidNumber {
    name: "seq_nr",
    value: seq_nr_value.to_string(),
  })?;
  let occurred_at = number_attribute(item, "occurred_at")?;
  let payload = bytes_attribute(item, "payload")?;
  let event =
    serde_json::from_slice::<GroupChatEvent>(&payload).map_err(|e| JournalReadError::Payload(e.to_string()))?;
  Ok(ReadModelEvent::new(seq_nr, nanos_to_datetime(occurred_at)?, event))
}

fn number_attribute(item: &HashMap<String, AttributeValue>, name: &'static str) -> Result<i64, JournalReadError> {
  match item.get(name) {
    None => Err(JournalReadError::MissingAttribute(name)),
    Some(AttributeValue::N(value)) => value.parse::<i64>().map_err(|_| JournalReadError::InvalidNumber {
      name,
      value: value.clone(),
    }),
    Some(_) => Err(JournalReadError::UnexpectedAttributeType(name)),
  }
}

fn bytes_attribute(item: &HashMap<String, AttributeValue>, name: &'static str) -> Result<Vec<u8>, JournalReadError> {
  match item.get(name) {
    None => Err(JournalReadError::MissingAttribute(name)),
    Some(AttributeValue::B(value)) => Ok(value.as_ref().to_vec()),
    Some(AttributeValue::S(value)) => Ok(value.as_bytes().to_vec()),
    Some(_) => Err(JournalReadError::UnexpectedAttributeType(name)),
  }
}

pub(crate) fn nanos_to_datetime(nanos: i64) -> Result<DateTime<Utc>, JournalReadError> {
  let seconds = nanos.div_euclid(1_000_000_000);
  let subsec_nanos = nanos.rem_euclid(1_000_000_000) as u32;
  DateTime::from_timestamp(seconds, subsec_nanos).ok_or_else(|| JournalReadError::InvalidOccurredAt(nanos.to_string()))
}

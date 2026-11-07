use std::collections::HashMap;
use std::str::FromStr;

use aws_lambda_events::dynamodb;
use chrono::{DateTime, Utc};
use lambda_runtime::LambdaEvent;
use serde_dynamo::AttributeValue;
use thiserror::Error;

use command_domain::group_chat::{GroupChatEvent, GroupChatId};
use command_interface_adaptor_if::{GroupChatReadModelUpdateDao, GroupChatReadModelUpdateDaoError, ReadModelEvent};

pub mod journal;

pub use journal::{DynamoDbJournalReader, JournalReadError, JournalReader};

/// 設定項目を保持する予約済みの集約 ID。ヘッドの Streams に一度だけ流れる。
const CONFIG_AGGREGATE_ID: &str = "__config__";

/// 購読対象の集約型名。
const GROUP_CHAT_TYPE_NAME: &str = "GroupChat";

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
}

/// ヘッドの Streams のイベントをリードモデルへ反映する。
///
/// INSERT と MODIFY の両方で `NewImage` を読み、`__config__` と他の型の集約は読み飛ばす。
/// 重複と古い `seq_nr` は捨て、欠番は journal から補う。
pub async fn update_read_model<D, J>(
  group_chat_read_model_dao: &D,
  journal_reader: &J,
  event: LambdaEvent<dynamodb::Event>,
) -> Result<(), UpdateReadModelError>
where
  D: GroupChatReadModelUpdateDao,
  J: JournalReader, {
  tracing::info!("Rust function invoked: event = {:?}", event);
  process_records(group_chat_read_model_dao, journal_reader, event.payload.records).await?;
  tracing::info!("Rust function responds to event");
  Ok(())
}

/// Streams のレコード列をリードモデルへ反映する。
pub async fn process_records<D, J>(
  group_chat_read_model_dao: &D,
  journal_reader: &J,
  records: Vec<dynamodb::EventRecord>,
) -> Result<(), UpdateReadModelError>
where
  D: GroupChatReadModelUpdateDao,
  J: JournalReader, {
  for record in records {
    let Some((aggregate_id, events)) = parse_head_record(&record)? else {
      continue;
    };
    for event in events {
      apply_head_event(group_chat_read_model_dao, journal_reader, &aggregate_id, event).await?;
    }
  }
  Ok(())
}

/// ヘッド遷移 1 件を適用する。現在の位置より後ろのイベントだけを、journal で欠番を
/// 埋めたうえで、1 つのトランザクションで適用する。
async fn apply_head_event<D, J>(
  group_chat_read_model_dao: &D,
  journal_reader: &J,
  aggregate_id: &GroupChatId,
  event: ReadModelEvent,
) -> Result<(), UpdateReadModelError>
where
  D: GroupChatReadModelUpdateDao,
  J: JournalReader, {
  let position = group_chat_read_model_dao.get_position(aggregate_id).await?;
  if event.seq_nr() <= position {
    tracing::debug!(
      "skip duplicate or stale head event: aggregate_id = {}, seq_nr = {}, position = {}",
      aggregate_id,
      event.seq_nr(),
      position
    );
    return Ok(());
  }
  let mut pending = Vec::new();
  if event.seq_nr() > position + 1 {
    let from_seq_nr = position + 1;
    let to_seq_nr = event.seq_nr() - 1;
    let journal_events = journal_reader.read_events(aggregate_id, from_seq_nr, to_seq_nr).await?;
    require_contiguous(aggregate_id, from_seq_nr, to_seq_nr, &journal_events)?;
    pending.extend(journal_events);
  }
  pending.push(event);
  group_chat_read_model_dao.apply_events(aggregate_id, &pending).await?;
  Ok(())
}

fn require_contiguous(
  aggregate_id: &GroupChatId,
  from_seq_nr: u64,
  to_seq_nr: u64,
  events: &[ReadModelEvent],
) -> Result<(), UpdateReadModelError> {
  let mut expected = from_seq_nr;
  for event in events {
    if event.seq_nr() != expected {
      return Err(UpdateReadModelError::JournalGap {
        aggregate_id: aggregate_id.to_string(),
        from_seq_nr,
        to_seq_nr,
      });
    }
    expected += 1;
  }
  if expected != to_seq_nr + 1 {
    return Err(UpdateReadModelError::JournalGap {
      aggregate_id: aggregate_id.to_string(),
      from_seq_nr,
      to_seq_nr,
    });
  }
  Ok(())
}

/// ヘッドの Streams レコードから集約 ID とイベントを組み立てる。
///
/// 設定項目、他の型の集約、REMOVE は `None` を返す。
fn parse_head_record(
  record: &dynamodb::EventRecord,
) -> Result<Option<(GroupChatId, Vec<ReadModelEvent>)>, UpdateReadModelError> {
  match record.event_name.as_str() {
    "INSERT" | "MODIFY" => {}
    "REMOVE" => return Ok(None),
    other => return Err(UpdateReadModelError::UnexpectedEventName(other.to_string())),
  }
  let image = &record.change.new_image;
  let aid = match image.get("aid") {
    Some(AttributeValue::S(value)) => value.clone(),
    Some(_) => return Err(UpdateReadModelError::UnexpectedAttributeType("aid")),
    None => return Err(UpdateReadModelError::MissingAttribute("aid")),
  };
  if aid == CONFIG_AGGREGATE_ID {
    return Ok(None);
  }
  let type_name = match image.get("type_name") {
    Some(AttributeValue::S(value)) => value.clone(),
    Some(_) => return Err(UpdateReadModelError::UnexpectedAttributeType("type_name")),
    None => aid
      .split_once('-')
      .map(|(name, _)| name.to_string())
      .unwrap_or_default(),
  };
  if type_name != GROUP_CHAT_TYPE_NAME {
    return Ok(None);
  }
  let aggregate_id = GroupChatId::from_str(&aid).map_err(|e| UpdateReadModelError::InvalidAggregateId {
    aggregate_id: aid.clone(),
    reason: e.to_string(),
  })?;
  let events = parse_head_events(image)?;
  Ok(Some((aggregate_id, events)))
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

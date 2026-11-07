use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use thiserror::Error;

use crate::CONFIG_AGGREGATE_ID;

/// One entry of the head table after the configuration item is removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Head {
  pub aggregate_id: String,
  pub type_name: String,
  pub seq_nr: u64,
}

#[derive(Debug, Error)]
pub enum HeadScanError {
  #[error("DynamoDB scan failed: {0}")]
  DynamoDb(String),
  #[error("The head item is missing the {0} attribute")]
  MissingAttribute(&'static str),
  #[error("The head attribute {0} has an unexpected type")]
  UnexpectedAttributeType(&'static str),
  #[error("The head attribute {0} is not a valid number: {1}")]
  InvalidNumber(&'static str, String),
  #[error("The head configuration item has no store_id")]
  MissingStoreId,
}

/// A DynamoDB failure is retryable; a malformed head item is not.
pub fn is_retryable_head_scan_error(error: &HeadScanError) -> bool {
  matches!(error, HeadScanError::DynamoDb(_))
}

/// Walks the whole head table. DY-15 requires a strongly consistent read so a
/// resync sees every aggregate that advanced while the reader was stopped.
#[async_trait]
pub trait HeadScanner: Send + Sync {
  async fn scan_heads(&self) -> Result<Vec<Head>, HeadScanError>;
}

/// Returns the store identifier shared by the three DynamoDB tables.
///
/// Recovery tasks record it so a task cannot be replayed against another
/// store's journal.
#[async_trait]
pub trait StoreIdProvider: Send + Sync {
  async fn store_id(&self) -> Result<String, HeadScanError>;
}

/// Scans the head table with `ConsistentRead = true` and skips the
/// `__config__` item. It keeps scanning pages until `LastEvaluatedKey` is
/// absent.
pub struct DynamoDbHeadScanner {
  client: Client,
  table_name: String,
  page_limit: Option<i32>,
  store_id: Mutex<Option<String>>,
}

impl DynamoDbHeadScanner {
  pub fn new(client: Client, table_name: impl Into<String>) -> Self {
    Self {
      client,
      table_name: table_name.into(),
      page_limit: None,
      store_id: Mutex::new(None),
    }
  }

  pub fn with_page_limit(mut self, page_limit: i32) -> Self {
    self.page_limit = Some(page_limit);
    self
  }
}

#[async_trait]
impl HeadScanner for DynamoDbHeadScanner {
  async fn scan_heads(&self) -> Result<Vec<Head>, HeadScanError> {
    let mut heads = Vec::new();
    let mut exclusive_start_key: Option<HashMap<String, AttributeValue>> = None;
    loop {
      let mut request = self
        .client
        .scan()
        .table_name(&self.table_name)
        .consistent_read(true)
        .projection_expression("aid, seq_nr, type_name")
        .filter_expression("aid <> :config")
        .expression_attribute_values(":config", AttributeValue::S(CONFIG_AGGREGATE_ID.to_string()));
      if let Some(limit) = self.page_limit {
        request = request.limit(limit);
      }
      if let Some(start_key) = exclusive_start_key.take() {
        request = request.set_exclusive_start_key(Some(start_key));
      }
      let output = request
        .send()
        .await
        .map_err(|error| HeadScanError::DynamoDb(error.to_string()))?;
      for item in output.items() {
        if let Some(head) = head_from_item(item)? {
          heads.push(head);
        }
      }
      match output.last_evaluated_key() {
        Some(key) if !key.is_empty() => exclusive_start_key = Some(key.clone()),
        _ => break,
      }
    }
    Ok(heads)
  }
}

#[async_trait]
impl StoreIdProvider for DynamoDbHeadScanner {
  async fn store_id(&self) -> Result<String, HeadScanError> {
    if let Some(store_id) = self.store_id.lock().expect("store id lock").as_ref() {
      return Ok(store_id.clone());
    }
    let output = self
      .client
      .get_item()
      .table_name(&self.table_name)
      .key("aid", AttributeValue::S(CONFIG_AGGREGATE_ID.to_string()))
      .consistent_read(true)
      .send()
      .await
      .map_err(|error| HeadScanError::DynamoDb(error.to_string()))?;
    let store_id = output
      .item()
      .and_then(|item| item.get("store_id"))
      .and_then(|value| match value {
        AttributeValue::S(value) => Some(value.clone()),
        _ => None,
      })
      .ok_or(HeadScanError::MissingStoreId)?;
    *self.store_id.lock().expect("store id lock") = Some(store_id.clone());
    Ok(store_id)
  }
}

fn head_from_item(item: &HashMap<String, AttributeValue>) -> Result<Option<Head>, HeadScanError> {
  let aggregate_id = match item.get("aid") {
    Some(AttributeValue::S(value)) => value.clone(),
    Some(_) => return Err(HeadScanError::UnexpectedAttributeType("aid")),
    None => return Err(HeadScanError::MissingAttribute("aid")),
  };
  if aggregate_id == CONFIG_AGGREGATE_ID {
    return Ok(None);
  }
  let type_name = match item.get("type_name") {
    Some(AttributeValue::S(value)) => value.clone(),
    Some(_) => return Err(HeadScanError::UnexpectedAttributeType("type_name")),
    None => aggregate_id
      .split_once('-')
      .map(|(name, _)| name.to_string())
      .unwrap_or_default(),
  };
  let seq_nr = match item.get("seq_nr") {
    Some(AttributeValue::N(value)) => value
      .parse::<u64>()
      .map_err(|_| HeadScanError::InvalidNumber("seq_nr", value.clone()))?,
    Some(_) => return Err(HeadScanError::UnexpectedAttributeType("seq_nr")),
    None => return Err(HeadScanError::MissingAttribute("seq_nr")),
  };
  Ok(Some(Head {
    aggregate_id,
    type_name,
    seq_nr,
  }))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn item(aggregate_id: &str, type_name: Option<&str>, seq_nr: Option<&str>) -> HashMap<String, AttributeValue> {
    let mut item = HashMap::from([("aid".to_string(), AttributeValue::S(aggregate_id.to_string()))]);
    if let Some(type_name) = type_name {
      item.insert("type_name".to_string(), AttributeValue::S(type_name.to_string()));
    }
    if let Some(seq_nr) = seq_nr {
      item.insert("seq_nr".to_string(), AttributeValue::N(seq_nr.to_string()));
    }
    item
  }

  #[test]
  fn should_fall_back_to_the_aggregate_id_prefix_when_type_name_is_missing() {
    let head = head_from_item(&item("GroupChat-01H", None, Some("3")))
      .unwrap()
      .unwrap();
    assert_eq!(
      Head {
        aggregate_id: "GroupChat-01H".to_string(),
        type_name: "GroupChat".to_string(),
        seq_nr: 3
      },
      head
    );
  }

  #[test]
  fn should_skip_the_configuration_item() {
    assert!(head_from_item(&item("__config__", Some("Config"), Some("0")))
      .unwrap()
      .is_none());
  }

  #[test]
  fn should_reject_a_missing_seq_nr() {
    assert!(matches!(
      head_from_item(&item("GroupChat-01H", Some("GroupChat"), None)),
      Err(HeadScanError::MissingAttribute("seq_nr"))
    ));
  }
}

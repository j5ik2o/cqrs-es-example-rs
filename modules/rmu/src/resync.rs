use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use command_domain::group_chat::GroupChatId;

use crate::applier::Applier;
use crate::failure::classify_update_error;
use crate::head_scanner::{is_retryable_head_scan_error, HeadScanner};
use crate::recovery::RecoveryService;
use crate::{UpdateReadModelError, CONFIG_AGGREGATE_ID, GROUP_CHAT_TYPE_NAME};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ResyncResult {
  pub heads: usize,
  pub skipped: usize,
  pub advanced: usize,
  pub failed: usize,
}

/// Separates permanent per-aggregate failures, which were recorded as recovery
/// tasks, from a storage failure. A retry must be driven by the storage failure:
/// permanent failures would otherwise hide a transient error and leave unscanned
/// heads behind.
#[derive(Debug)]
pub struct ResyncError {
  pub permanent: Vec<String>,
  pub failure: Option<Box<UpdateReadModelError>>,
  pub retryable: bool,
}

impl ResyncError {
  pub fn is_retryable(&self) -> bool {
    self.retryable
  }
}

impl fmt::Display for ResyncError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(failure) = &self.failure {
      if !self.permanent.is_empty() {
        return write!(
          f,
          "resync failed ({}) with {} permanent aggregate failures",
          failure,
          self.permanent.len()
        );
      }
      return write!(f, "resync failed: {}", failure);
    }
    write!(
      f,
      "resync recorded {} permanent aggregate failures",
      self.permanent.len()
    )
  }
}

impl std::error::Error for ResyncError {}

/// Implements DY-15's reconciliation: scan every head with strong consistency,
/// compare its seq_nr with the projection position, and fill the lag from the
/// journal with the same position guard as the stream path.
pub struct Resyncer {
  applier: Arc<Applier>,
  scanner: Arc<dyn HeadScanner>,
  recovery: Option<Arc<RecoveryService>>,
  type_name: String,
}

impl Resyncer {
  pub fn new(applier: Arc<Applier>, scanner: Arc<dyn HeadScanner>, recovery: Option<Arc<RecoveryService>>) -> Self {
    Self {
      applier,
      scanner,
      recovery,
      type_name: GROUP_CHAT_TYPE_NAME.to_string(),
    }
  }

  /// Reads the whole head table with strong consistency. A permanent failure is
  /// recorded as a recovery task and does not stop the remaining aggregates; a
  /// storage failure aborts the scan so the caller can retry it.
  pub async fn resync(&self) -> Result<ResyncResult, ResyncError> {
    let heads = match self.scanner.scan_heads().await {
      Ok(heads) => heads,
      Err(error) => {
        return Err(ResyncError {
          permanent: Vec::new(),
          retryable: is_retryable_head_scan_error(&error),
          failure: Some(Box::new(UpdateReadModelError::HeadScan(error.to_string()))),
        })
      }
    };
    let mut result = ResyncResult::default();
    let mut permanent = Vec::new();
    for head in heads {
      if head.aggregate_id == CONFIG_AGGREGATE_ID || head.type_name != self.type_name {
        result.skipped += 1;
        continue;
      }
      result.heads += 1;
      let aggregate_id = match GroupChatId::from_str(&head.aggregate_id) {
        Ok(aggregate_id) => aggregate_id,
        Err(error) => {
          result.failed += 1;
          permanent.push(format!(
            "{} seq_nr {}: invalid aggregate id: {}",
            head.aggregate_id, head.seq_nr, error
          ));
          continue;
        }
      };
      let position = match self.applier.position(&aggregate_id).await {
        Ok(position) => position,
        Err(error) => {
          return Err(ResyncError {
            permanent,
            retryable: crate::failure::is_retryable_update_error(&error),
            failure: Some(Box::new(error)),
          })
        }
      };
      if head.seq_nr <= position {
        continue;
      }
      result.advanced += 1;
      if let Err(error) = self.applier.apply_through(&aggregate_id, head.seq_nr).await {
        if classify_update_error(&error).is_some() {
          result.failed += 1;
          if let Some(recovery) = &self.recovery {
            if let Err(record_error) = recovery.record(&head.aggregate_id, head.seq_nr, &error).await {
              // Losing the task must abort the scan so the caller retries
              // instead of counting this aggregate as handled.
              return Err(ResyncError {
                permanent,
                retryable: true,
                failure: Some(Box::new(UpdateReadModelError::RecoveryRecord {
                  apply: error.to_string(),
                  record: record_error.to_string(),
                })),
              });
            }
          }
          tracing::error!(
            aggregate_id = %head.aggregate_id,
            seq_nr = head.seq_nr,
            failure = ?classify_update_error(&error),
            "resync could not apply aggregate; recorded a recovery task"
          );
          permanent.push(format!("{} seq_nr {}: {}", head.aggregate_id, head.seq_nr, error));
        } else {
          return Err(ResyncError {
            permanent,
            retryable: true,
            failure: Some(Box::new(error)),
          });
        }
      }
    }
    if permanent.is_empty() {
      Ok(result)
    } else {
      Err(ResyncError {
        permanent,
        retryable: false,
        failure: None,
      })
    }
  }
}

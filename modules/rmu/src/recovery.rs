use std::str::FromStr;
use std::sync::Arc;

use command_domain::group_chat::GroupChatId;
use command_interface_adaptor_if::{RecoveryTask, RecoveryTaskStore, RecoveryTaskStoreError};

use crate::applier::Applier;
use crate::failure::{classify_update_error, FailureKind};
use crate::head_scanner::StoreIdProvider;
use crate::UpdateReadModelError;

pub const PROJECTION_NAME: &str = "group-chat-read-model";
pub const PROJECTION_VERSION: &str = "1";
pub const RECOVERY_STATUS_PENDING: &str = "pending";
pub const RECOVERY_STATUS_COMPLETED: &str = "completed";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPendingResult {
  pub completed: usize,
  pub failures: Vec<String>,
}

/// Records failed events and replays them from the journal. It is the DLQ half
/// of the RMU contract: the task is durable, the replay is idempotent, and
/// completion is claimed only after the position reaches the task's seq_nr.
///
/// SQS is a delivery detail outside this type; a message only wakes a worker,
/// which re-reads the event body from the journal.
pub struct RecoveryService {
  store: Arc<dyn RecoveryTaskStore>,
  store_id: Arc<dyn StoreIdProvider>,
  applier: Arc<Applier>,
  projection_name: String,
  projection_version: String,
}

impl RecoveryService {
  pub fn new(store: Arc<dyn RecoveryTaskStore>, store_id: Arc<dyn StoreIdProvider>, applier: Arc<Applier>) -> Self {
    Self {
      store,
      store_id,
      applier,
      projection_name: PROJECTION_NAME.to_string(),
      projection_version: PROJECTION_VERSION.to_string(),
    }
  }

  /// Persists a recovery task. A failure here is returned to the caller on
  /// purpose: losing the task would hide a permanently failing event, so the
  /// record must fail and be retried by the delivery mechanism.
  pub async fn record(
    &self,
    aggregate_id: &str,
    seq_nr: u64,
    failure: &UpdateReadModelError,
  ) -> Result<(), RecoveryTaskStoreError> {
    let store_id = self
      .store_id
      .store_id()
      .await
      .map_err(|error| RecoveryTaskStoreError::Other(error.to_string()))?;
    let kind = classify_update_error(failure).unwrap_or(FailureKind::Unknown);
    let task = RecoveryTask {
      store_id,
      projection_name: self.projection_name.clone(),
      projection_version: self.projection_version.clone(),
      aggregate_id: aggregate_id.to_string(),
      seq_nr,
      failure_type: kind.as_str().to_string(),
      error_message: failure.to_string(),
      status: RECOVERY_STATUS_PENDING.to_string(),
    };
    self.store.record(&task).await
  }

  /// Replays one task. It re-reads the position first, so calling it twice is
  /// safe: the second call finds the position at or beyond the task and only
  /// marks the task complete.
  pub async fn recover(&self, task: &RecoveryTask) -> Result<(), UpdateReadModelError> {
    if task.projection_name != self.projection_name || task.projection_version != self.projection_version {
      return Err(UpdateReadModelError::Recovery(format!(
        "task targets projection {}/{}, not {}/{}",
        task.projection_name, task.projection_version, self.projection_name, self.projection_version
      )));
    }
    let store_id = self
      .store_id
      .store_id()
      .await
      .map_err(|error| UpdateReadModelError::Recovery(error.to_string()))?;
    if task.store_id != store_id {
      return Err(UpdateReadModelError::Recovery(format!(
        "task belongs to store {}, not {}",
        task.store_id, store_id
      )));
    }
    let aggregate_id =
      GroupChatId::from_str(&task.aggregate_id).map_err(|error| UpdateReadModelError::InvalidAggregateId {
        aggregate_id: task.aggregate_id.clone(),
        reason: error.to_string(),
      })?;
    let position = self.applier.position(&aggregate_id).await?;
    if position < task.seq_nr {
      self.applier.apply_through(&aggregate_id, task.seq_nr).await?;
    }
    let position = self.applier.position(&aggregate_id).await?;
    if position < task.seq_nr {
      return Err(UpdateReadModelError::Recovery(format!(
        "aggregate {} stayed at position {}, expected {}",
        task.aggregate_id, position, task.seq_nr
      )));
    }
    self
      .store
      .mark_completed(&task.store_id, &task.projection_name, &task.aggregate_id, task.seq_nr)
      .await
      .map_err(|error| UpdateReadModelError::Recovery(error.to_string()))
  }

  /// Replays every pending task and returns the tasks that still fail. One
  /// failing aggregate does not stop the others.
  pub async fn recover_pending(&self) -> Result<RecoveryPendingResult, UpdateReadModelError> {
    let tasks = self
      .store
      .list_pending()
      .await
      .map_err(|error| UpdateReadModelError::Recovery(error.to_string()))?;
    let store_id = self
      .store_id
      .store_id()
      .await
      .map_err(|error| UpdateReadModelError::Recovery(error.to_string()))?;
    let mut completed = 0;
    let mut failures = Vec::new();
    for task in tasks {
      // Tasks of another projection or store are not this worker's work.
      if task.projection_name != self.projection_name
        || task.projection_version != self.projection_version
        || task.store_id != store_id
      {
        continue;
      }
      match self.recover(&task).await {
        Ok(()) => completed += 1,
        Err(error) => failures.push(format!("{} seq_nr {}: {}", task.aggregate_id, task.seq_nr, error)),
      }
    }
    Ok(RecoveryPendingResult { completed, failures })
  }
}

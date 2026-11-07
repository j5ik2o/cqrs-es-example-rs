use crate::failure::is_retryable_update_error;
use crate::recovery::{RecoveryPendingResult, RecoveryService};
use crate::resync::{ResyncResult, Resyncer};

/// One recovery pass: reconcile the head table with the journal and replay the
/// pending recovery tasks. The SQS worker and the scheduled pass run the same
/// function, so an SQS notification is only a wake-up call.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RecoveryPassResult {
  pub resync: Option<ResyncResult>,
  pub resync_permanent_failures: Vec<String>,
  pub resync_error: Option<String>,
  pub resync_retryable: bool,
  pub recovery: Option<RecoveryPendingResult>,
  pub recovery_error: Option<String>,
  pub recovery_retryable: bool,
}

impl RecoveryPassResult {
  /// True when the worker should return the SQS messages for redelivery.
  pub fn retryable(&self) -> bool {
    self.resync_retryable || self.recovery_retryable
  }

  /// Tasks that were still pending when the pass finished.
  pub fn pending_tasks(&self) -> usize {
    self.recovery.as_ref().map_or(0, |result| result.pending)
  }

  pub fn failed_tasks(&self) -> usize {
    self.recovery.as_ref().map_or(0, |result| result.failures.len())
  }

  pub fn completed_tasks(&self) -> usize {
    self.recovery.as_ref().map_or(0, |result| result.completed)
  }

  pub fn resync_failed(&self) -> usize {
    self.resync_permanent_failures.len()
  }

  pub fn has_permanent_failures(&self) -> bool {
    !self.resync_permanent_failures.is_empty() || self.failed_tasks() > 0
  }
}

/// Runs a resync and then drains the pending recovery tasks. A transient failure
/// sets the retryable flag; permanent per-aggregate failures are already stored
/// as recovery tasks and are reported for metrics and logs.
pub async fn run_recovery_pass(resyncer: &Resyncer, recovery: &RecoveryService) -> RecoveryPassResult {
  let mut result = RecoveryPassResult::default();
  match resyncer.resync().await {
    Ok(summary) => result.resync = Some(summary),
    Err(error) => {
      result.resync_permanent_failures = error.permanent.clone();
      result.resync_retryable = error.is_retryable();
      result.resync_error = Some(error.to_string());
    }
  }
  match recovery.recover_pending().await {
    Ok(summary) => result.recovery = Some(summary),
    Err(error) => {
      result.recovery_retryable = is_retryable_update_error(&error);
      result.recovery_error = Some(error.to_string());
    }
  }
  result
}

use async_trait::async_trait;
use aws_sdk_cloudwatch::types::{Dimension, MetricDatum};
use aws_sdk_cloudwatch::Client as CloudWatchClient;

use rmu::{run_recovery_pass, RecoveryPassResult, RecoveryService, Resyncer};

pub const RMU_METRIC_NAMESPACE: &str = "CqrsEsExample/Rmu";
pub const RMU_METRIC_DIMENSION_NAME: &str = "Service";
pub const RMU_METRIC_DIMENSION_VALUE: &str = "read-model-updater";

/// Publishes the recovery pass counts. A failure to publish is logged but does
/// not fail the recovery; the metrics are observability, not the recovery
/// itself.
#[async_trait]
pub trait RecoveryMetrics: Send + Sync {
  async fn publish(&self, result: &RecoveryPassResult);
}

pub struct NoopRecoveryMetrics;

#[async_trait]
impl RecoveryMetrics for NoopRecoveryMetrics {
  async fn publish(&self, _: &RecoveryPassResult) {}
}

#[derive(Clone)]
pub struct CloudWatchRecoveryMetrics {
  client: CloudWatchClient,
  namespace: String,
}

impl CloudWatchRecoveryMetrics {
  pub fn new(client: CloudWatchClient, namespace: impl Into<String>) -> Self {
    Self {
      client,
      namespace: namespace.into(),
    }
  }
}

#[async_trait]
impl RecoveryMetrics for CloudWatchRecoveryMetrics {
  async fn publish(&self, result: &RecoveryPassResult) {
    let dimension = Dimension::builder()
      .name(RMU_METRIC_DIMENSION_NAME)
      .value(RMU_METRIC_DIMENSION_VALUE)
      .build();
    let metric_data = metric_values(result)
      .into_iter()
      .map(|(name, value)| {
        MetricDatum::builder()
          .metric_name(name)
          .dimensions(dimension.clone())
          .value(value)
          .build()
      })
      .collect::<Vec<_>>();
    if metric_data.is_empty() {
      return;
    }
    if let Err(error) = self
      .client
      .put_metric_data()
      .namespace(&self.namespace)
      .set_metric_data(Some(metric_data))
      .send()
      .await
    {
      tracing::warn!(error = %error, "failed to publish RMU recovery metrics");
    }
  }
}

/// Metrics for one pass. A metric is omitted when the pass could not produce
/// its result, so an alarm keeps its state instead of clearing on a zero that
/// only means "unknown".
pub fn metric_values(result: &RecoveryPassResult) -> Vec<(&'static str, f64)> {
  let mut values = Vec::new();
  if let Some(recovery) = &result.recovery {
    values.push(("RecoveryTasksPending", recovery.pending as f64));
    values.push(("RecoveryTasksFailed", recovery.failures.len() as f64));
    values.push(("RecoveryTasksCompleted", recovery.completed as f64));
  }
  if result.resync.is_some() || !result.resync_permanent_failures.is_empty() {
    values.push(("ResyncFailed", result.resync_failed() as f64));
  }
  values
}

/// SQS OnFailure handler and scheduled recovery pass.
///
/// The message body is ignored; a message only wakes the worker. The pass
/// reconciles the head table and replays the pending tasks, so a redelivered or
/// batched message is harmless. A retryable failure returns every message ID as
/// a batch item failure; permanent failures are acknowledged because they are
/// already stored as recovery tasks.
pub async fn handle_recovery_event(
  event: serde_json::Value,
  resyncer: &Resyncer,
  recovery: &RecoveryService,
  metrics: &dyn RecoveryMetrics,
) -> serde_json::Value {
  let message_ids = sqs_message_ids(&event);
  let result = run_recovery_pass(resyncer, recovery).await;
  metrics.publish(&result).await;
  if let Some(error) = &result.resync_error {
    tracing::error!(error = %error, "resync did not complete");
  }
  if let Some(error) = &result.recovery_error {
    tracing::error!(error = %error, "recovery run did not complete");
  }
  if let Some(summary) = &result.recovery {
    for failure in &summary.failures {
      tracing::error!(failure = %failure, "recovery task remains unresolved");
    }
  }
  tracing::info!(
    resync_heads = result.resync.as_ref().map_or(0, |summary| summary.heads),
    resync_advanced = result.resync.as_ref().map_or(0, |summary| summary.advanced),
    resync_failed = result.resync_failed(),
    recovery_completed = result.completed_tasks(),
    recovery_pending = result.pending_tasks(),
    recovery_failed = result.failed_tasks(),
    retryable = result.retryable(),
    "recovery pass finished"
  );
  batch_response(&message_ids, result.retryable())
}

fn sqs_message_ids(event: &serde_json::Value) -> Vec<String> {
  event
    .get("Records")
    .and_then(|records| records.as_array())
    .map(|records| {
      records
        .iter()
        .filter_map(|record| {
          record
            .get("messageId")
            .and_then(|message_id| message_id.as_str())
            .map(ToString::to_string)
        })
        .collect()
    })
    .unwrap_or_default()
}

fn batch_response(message_ids: &[String], retry: bool) -> serde_json::Value {
  let failures = if retry {
    message_ids
      .iter()
      .map(|message_id| serde_json::json!({ "itemIdentifier": message_id }))
      .collect::<Vec<_>>()
  } else {
    Vec::new()
  };
  serde_json::json!({ "batchItemFailures": failures })
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use super::*;
  use rmu::{RecoveryPendingResult, ResyncResult};

  fn result(retryable: bool, pending: usize, failed: usize) -> RecoveryPassResult {
    RecoveryPassResult {
      resync: Some(ResyncResult {
        heads: 2,
        skipped: 0,
        advanced: 1,
        failed: 0,
      }),
      resync_permanent_failures: Vec::new(),
      resync_error: None,
      resync_retryable: false,
      recovery: Some(RecoveryPendingResult {
        completed: 1,
        failures: vec!["task failed".to_string(); failed],
        pending,
      }),
      recovery_error: None,
      recovery_retryable: retryable,
    }
  }

  #[test]
  fn should_extract_sqs_message_ids_and_ignore_other_events() {
    let event = serde_json::json!({
      "Records": [
        {"messageId": "m-1", "body": "ignored"},
        {"messageId": "m-2", "body": "ignored"}
      ]
    });
    assert_eq!(vec!["m-1".to_string(), "m-2".to_string()], sqs_message_ids(&event));
    assert!(sqs_message_ids(&serde_json::json!({"source": "aws.events"})).is_empty());
  }

  #[test]
  fn should_return_batch_failures_only_for_a_retryable_pass() {
    let message_ids = vec!["m-1".to_string(), "m-2".to_string()];
    assert_eq!(
      serde_json::json!({"batchItemFailures": [{"itemIdentifier": "m-1"}, {"itemIdentifier": "m-2"}]}),
      batch_response(&message_ids, true)
    );
    assert_eq!(
      serde_json::json!({"batchItemFailures": []}),
      batch_response(&message_ids, false)
    );
    assert_eq!(serde_json::json!({"batchItemFailures": []}), batch_response(&[], true));
  }

  #[test]
  fn should_report_the_recovery_counts_as_metrics() {
    let values = metric_values(&result(false, 3, 2));
    let map = values.into_iter().collect::<HashMap<_, _>>();
    assert_eq!(Some(&3.0), map.get("RecoveryTasksPending"));
    assert_eq!(Some(&2.0), map.get("RecoveryTasksFailed"));
    assert_eq!(Some(&1.0), map.get("RecoveryTasksCompleted"));
    assert_eq!(Some(&0.0), map.get("ResyncFailed"));
  }
}

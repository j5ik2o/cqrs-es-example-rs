use std::fs;
use std::path::Path;

use serde_json::Value;

fn load_ops_config() -> Value {
  let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/deploy/rmu/ops.json");
  serde_json::from_slice(&fs::read(&path).expect("read tools/deploy/rmu/ops.json")).expect("parse the ops config")
}

fn u64_at(config: &Value, pointer: &str) -> u64 {
  config
    .pointer(pointer)
    .and_then(Value::as_u64)
    .unwrap_or_else(|| panic!("missing numeric setting {pointer}"))
}

#[test]
fn should_keep_the_recovery_worker_timeout_and_visibility_consistent() {
  let config = load_ops_config();
  let timeout = u64_at(&config, "/recoveryWorker/timeoutSeconds");
  let visibility = u64_at(&config, "/recoveryQueue/visibilityTimeoutSeconds");

  assert!(timeout >= 60, "a recovery pass needs at least a minute");
  assert_eq!(
    1,
    u64_at(&config, "/recoveryWorker/reservedConcurrentExecutions"),
    "one worker avoids duplicate recovery passes"
  );
  assert_eq!(1, u64_at(&config, "/recoveryQueue/batchSize"));
  assert!(
    visibility >= timeout.saturating_mul(6),
    "the queue visibility timeout should cover six function timeouts"
  );
  assert!(u64_at(&config, "/recoveryQueue/maxReceiveCount") >= 1);
  assert!(u64_at(&config, "/recoveryQueue/maximumRetryAttempts") >= 1);
}

#[test]
fn should_define_the_recovery_schedule_and_alarm_thresholds() {
  let config = load_ops_config();
  let schedule = config
    .pointer("/schedule/recoveryPass")
    .and_then(Value::as_str)
    .expect("missing schedule/recoveryPass");
  assert!(
    schedule.starts_with("rate(") || schedule.starts_with("cron("),
    "the recovery pass must run on an EventBridge schedule"
  );

  for key in [
    "queueDepthThreshold",
    "pendingTasksThreshold",
    "failedTasksThreshold",
    "resyncFailuresThreshold",
  ] {
    let pointer = format!("/alarms/{key}");
    assert!(
      config.pointer(&pointer).and_then(Value::as_u64).is_some(),
      "missing alarm threshold {pointer}"
    );
  }
  assert!(u64_at(&config, "/alarms/periodSeconds") >= 60);
  assert!(u64_at(&config, "/alarms/evaluationPeriods") >= 1);
  assert!(u64_at(&config, "/alarms/customEvaluationPeriods") >= 1);
  assert!(u64_at(&config, "/alarms/queueAgeSeconds") >= 60);
}

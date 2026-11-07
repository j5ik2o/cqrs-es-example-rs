# RMU recovery operations

This document covers the production assets and procedure for recovering the Read Model Updater (RMU). The code was implemented in PR #603. This document defines the SQS OnFailure worker, the scheduled recovery pass, monitoring, and the procedure for failures. It does not deploy anything.

## Components

| Resource | Responsibility |
|:--|:--|
| RMU Lambda (`<prefix>-lambda-read-model-updater`) | Subscribes to the head Streams and updates the read model and `read_model_positions`. When an event cannot be applied, it writes a recovery task to `read_model_recovery_tasks`. |
| OnFailure queue (`<prefix>-read-model-updater-dlq`) | Receives batches that exhausted the Streams retries. It is also the recovery worker's input. |
| Recovery worker Lambda (`<prefix>-lambda-read-model-updater-recovery`) | Runs the same image with `APP__RECOVERY_WORKER=true`. It is triggered by SQS or EventBridge and runs one recovery pass: a full head reconciliation and a replay of pending tasks. |
| Recovery DLQ (`<prefix>-read-model-updater-recovery-dlq`) | Holds messages that the recovery worker could not process after its retries. |
| EventBridge rule (`<prefix>-rmu-recovery-pass`) | Invokes the recovery worker every 15 minutes so pending tasks are drained even when no notification arrives. |
| SNS topic (`<prefix>-rmu-ops-alerts`) | Destination for CloudWatch alarms. Setting `rmu_ops_alert_email` also adds an email subscription. |

The durable source of truth for recovery is the `read_model_recovery_tasks` table. SQS only wakes the worker; the worker ignores the message body and re-reads the event from the journal.

## Settings

`tools/deploy/rmu/ops.json` is the source of truth for the operational values. Terraform reads it through `local.rmu_ops`, and `applications/read-model-updater/tests/ops_config.rs` checks its consistency.

| Setting | Value | Reason |
|:--|:--|:--|
| Recovery worker timeout | 300 seconds | Enough for one full reconciliation and replay. |
| Recovery queue visibility timeout | 1800 seconds | Six times the worker timeout, so a message in flight is not handed to another worker. |
| Recovery worker concurrency | 1 | Two workers do not process the same task. Row locks also prevent conflicts, but one worker avoids wasted work. |
| SQS batch size | 1 | One message runs one recovery pass. |
| SQS max receive count | 5 | Retry transient failures a few times, then park the message in the recovery DLQ. |
| Scheduled recovery pass | every 15 minutes | Drains pending tasks without an SQS notification. |
| RMU Lambda timeout | 60 seconds | Enough to return a partial batch response for one stream batch. |
| RMU retry | 5 attempts / 30 seconds | The code's default bound for transient failures. |

With concurrency 1, an SQS invocation and a scheduled invocation that overlap throttle one of them. If the throttling persists, the message moves to the recovery DLQ, but the task remains in MySQL and the scheduled pass keeps processing it. The recovery worker publishes CloudWatch metrics, so the Lambda security group allows TCP 443 through the NAT gateway.

## Procedure

### A message arrived on the OnFailure queue

The OnFailure queue receives the metadata of a batch that exhausted the Streams retries; the message body is not the original event. Check MySQL for recovery tasks.

1. Check the queue depth and the age of the oldest message.

```sh
aws sqs get-queue-attributes \
  --queue-url "$(terraform -chdir=tools/deploy/terraform output -raw read_model_updater_on_failure_queue_url)" \
  --attribute-names ApproximateNumberOfMessages ApproximateAgeOfOldestMessage
```

2. List the pending recovery tasks.

```sql
SELECT store_id, projection_name, projection_version, aggregate_id, seq_nr,
       failure_type, error_message, status
FROM read_model_recovery_tasks
WHERE status = 'pending'
ORDER BY created_at;
```

`failure_type` is one of `event_payload`, `unsupported_event`, `journal_gap`, `read_model_constraint`, or `unknown`.

3. Remove the cause. Check that the journal has the event and that the read model constraint is resolved.

4. Run one recovery pass by hand. The worker ignores the message body, so an empty event replays the pending tasks.

```sh
aws lambda invoke \
  --function-name "$(terraform -chdir=tools/deploy/terraform output -raw read_model_updater_recovery_function_name)" \
  --payload '{"Records":[]}' \
  --cli-binary-format raw-in-base64-out \
  /tmp/recovery-worker-response.json
jq . /tmp/recovery-worker-response.json
```

5. Check that the task's `status` is `completed` and that `read_model_positions` reached the task's `seq_nr`.

```sql
SELECT status FROM read_model_recovery_tasks
WHERE aggregate_id = ? AND projection_name = 'group-chat-read-model' AND seq_nr = ?;

SELECT seq_nr FROM read_model_positions WHERE aggregate_id = ?;
```

### A message moved to the recovery DLQ

A recovery DLQ message means the recovery worker could not process it after its retries. The message body only wakes the worker and the task may already be complete, so do not use the body as the source of truth.

1. Check `read_model_recovery_tasks` for `status = 'pending'` and read `failure_type` and `error_message`.
2. Remove the cause.
3. Run the recovery worker again. If tasks remain pending, the worker log contains `recovery task remains unresolved`.
4. Check that the task's `status` is `completed` and the position advanced.

Deleting the message without fixing the task leaves the same task pending for the next scheduled pass.

### The resync failed

The `<prefix>-rmu-resync-failures` alarm or the `resync did not complete` log reports this. Either the full head Scan failed or the journal has a gap.

- Transient failures (DynamoDB throttling, network errors) are retried by the next scheduled pass.
- A persistent `journal_gap` means the journal is missing events. Check `aid` and `seq_nr` in the journal and restore the missing events. The worker does not advance the position.
- After a stop longer than 24 hours, the full head Scan and journal replay catch up. For a large head table, adjust the schedule interval.

### Run recovery locally

To run the same logic outside Lambda, use the `recover` binary with the same `APP__DATABASE__URL`, `APP__AWS__*`, and `APP__STREAM__*` settings as the local RMU.

```sh
cargo run -p read-model-updater --bin recover
```

The recovery pass is idempotent. Running the same task twice does not duplicate read model rows.

## Monitoring

Terraform defines the following CloudWatch alarms. They notify the SNS topic `<prefix>-rmu-ops-alerts`.

| Alarm | Metric | Threshold | Meaning |
|:--|:--|:--|:--|
| `<prefix>-rmu-stream-errors` | `AWS/Lambda Errors` (RMU) | 1 or more in 5 minutes | Stream updates are failing. |
| `<prefix>-rmu-stream-throttles` | `AWS/Lambda Throttles` (RMU) | 1 or more in 5 minutes | The concurrency limit is reached. |
| `<prefix>-rmu-recovery-worker-errors` | `AWS/Lambda Errors` (recovery) | 1 or more in 5 minutes | The recovery pass is failing. |
| `<prefix>-rmu-on-failure-queue-depth` | `AWS/SQS ApproximateNumberOfMessagesVisible` (OnFailure) | 1 or more for 2 periods | Failed batches are accumulating. |
| `<prefix>-rmu-on-failure-queue-age` | `AWS/SQS ApproximateAgeOfOldestMessage` (OnFailure) | over 3600 seconds for 2 periods | A failed batch has waited too long. |
| `<prefix>-rmu-recovery-dlq-depth` | `AWS/SQS ApproximateNumberOfMessagesVisible` (recovery DLQ) | 1 or more | The retry limit was exceeded. Follow the recovery DLQ procedure. |
| `<prefix>-rmu-pending-recovery-tasks` | `CqrsEsExample/Rmu RecoveryTasksPending` | over 0 in one recovery pass | Recovery tasks are pending. |
| `<prefix>-rmu-failed-recovery-tasks` | `CqrsEsExample/Rmu RecoveryTasksFailed` | over 0 in one recovery pass | The recovery pass could not apply tasks. |
| `<prefix>-rmu-resync-failures` | `CqrsEsExample/Rmu ResyncFailed` | over 0 in one recovery pass | One or more aggregates could not be reconciled. |
| `<prefix>-rmu-recovery-schedule-failures` | `AWS/Events FailedInvocations` (schedule) | 1 or more | The schedule could not invoke the recovery worker. |

The recovery worker publishes `RecoveryTasksPending`, `RecoveryTasksFailed`, `RecoveryTasksCompleted`, and `ResyncFailed` in the `CqrsEsExample/Rmu` namespace with the `Service=read-model-updater` dimension. The metrics are emitted only when a pass runs, so the custom-metric alarms keep their state on missing data (`ignore`).

Setting `rmu_ops_alert_email` creates an SNS email subscription. Add a different subscription to the SNS topic for another notification channel.

## Remaining work

- Deploy to the actual AWS account and decide the SNS notification target. This change provides the Terraform definitions and the procedure.
- The MySQL password is passed from the Terraform output. Use Secrets Manager or SSM Parameter Store in production.
- The recovery pass runs every 15 minutes. For a large head table, watch the Scan cost and lengthen the interval.
- The recovery worker concurrency is fixed at 1. If one pass cannot drain the tasks, add a per-pass task limit before increasing concurrency.
- `terraform validate` needs the provider and module downloads and is not run in this repository's CI. The `ops.json` values are covered by a test.

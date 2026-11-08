use chrono::Utc;
use command_interface_adaptor_if::{RecoveryTask, RecoveryTaskStore, RecoveryTaskStoreError};
use sqlx::MySqlPool;

/// Persists recovery tasks in the MySQL read model.
///
/// The primary key `(store_id, projection_name, aggregate_id, seq_nr)` makes
/// recording idempotent, so a redelivered record does not create duplicates.
#[derive(Debug)]
pub struct MySqlRecoveryTaskStore {
  pool: MySqlPool,
}

impl MySqlRecoveryTaskStore {
  pub fn new(pool: MySqlPool) -> Self {
    Self { pool }
  }
}

#[async_trait::async_trait]
impl RecoveryTaskStore for MySqlRecoveryTaskStore {
  async fn record(&self, task: &RecoveryTask) -> Result<(), RecoveryTaskStoreError> {
    let now = Utc::now().naive_utc();
    sqlx::query!(
      r#"INSERT INTO read_model_recovery_tasks
           (store_id, projection_name, projection_version, aggregate_id, seq_nr,
            failure_type, error_message, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE
           projection_version = VALUES(projection_version),
           failure_type = VALUES(failure_type),
           error_message = VALUES(error_message),
           status = VALUES(status),
           updated_at = VALUES(updated_at)"#,
      task.store_id,
      task.projection_name,
      task.projection_version,
      task.aggregate_id,
      task.seq_nr as i64,
      task.failure_type,
      task.error_message,
      task.status,
      now,
      now
    )
    .execute(&self.pool)
    .await
    .map(|_| ())
    .map_err(|error| RecoveryTaskStoreError::Other(error.to_string()))
  }

  async fn list_pending(&self) -> Result<Vec<RecoveryTask>, RecoveryTaskStoreError> {
    let rows = sqlx::query!(
      r#"SELECT store_id, projection_name, projection_version, aggregate_id, seq_nr,
                failure_type, error_message, status
         FROM read_model_recovery_tasks
         WHERE status = 'pending'
         ORDER BY created_at, aggregate_id, seq_nr"#
    )
    .fetch_all(&self.pool)
    .await
    .map_err(|error| RecoveryTaskStoreError::Other(error.to_string()))?;
    Ok(
      rows
        .into_iter()
        .map(|row| RecoveryTask {
          store_id: row.store_id,
          projection_name: row.projection_name,
          projection_version: row.projection_version,
          aggregate_id: row.aggregate_id,
          seq_nr: row.seq_nr as u64,
          failure_type: row.failure_type,
          error_message: row.error_message,
          status: row.status,
        })
        .collect(),
    )
  }

  async fn mark_completed(
    &self,
    store_id: &str,
    projection_name: &str,
    aggregate_id: &str,
    seq_nr: u64,
  ) -> Result<(), RecoveryTaskStoreError> {
    let now = Utc::now().naive_utc();
    sqlx::query!(
      r#"UPDATE read_model_recovery_tasks
         SET status = 'completed', updated_at = ?
         WHERE store_id = ? AND projection_name = ? AND aggregate_id = ? AND seq_nr = ?"#,
      now,
      store_id,
      projection_name,
      aggregate_id,
      seq_nr as i64
    )
    .execute(&self.pool)
    .await
    .map(|_| ())
    .map_err(|error| RecoveryTaskStoreError::Other(error.to_string()))
  }
}

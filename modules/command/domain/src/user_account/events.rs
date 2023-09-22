use crate::user_account::UserAccountId;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 現在未使用
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UserAccountEvent {
  /// ユーザアカウントが作成された
  UserAccountCreated(UserAccountEventCreatedBody),
  /// ユーザアカウントが削除された
  UserAccountDeleted(UserAccountEventDeletedBody),
}

/// 現在未使用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccountEventCreatedBody {
  pub aggregate_id: UserAccountId,
  pub user_name: String,
  pub password: String,
  pub occurred_at: DateTime<Utc>,
}

/// 現在未使用
impl UserAccountEventCreatedBody {
  pub fn new(
    aggregate_id: UserAccountId,
    user_name: String,
    password: String,
    occurred_at: DateTime<Utc>,
  ) -> UserAccountEventCreatedBody {
    Self {
      aggregate_id,
      user_name,
      password,
      occurred_at,
    }
  }
}

/// 現在未使用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccountEventDeletedBody {
  pub aggregate_id: UserAccountId,
  pub occurred_at: DateTime<Utc>,
}

impl UserAccountEventDeletedBody {
  pub fn new(aggregate_id: UserAccountId, occurred_at: DateTime<Utc>) -> UserAccountEventDeletedBody {
    Self {
      aggregate_id,
      occurred_at,
    }
  }
}

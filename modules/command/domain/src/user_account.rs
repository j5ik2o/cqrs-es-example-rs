pub mod events;

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use anyhow::{anyhow, Result};

use crate::aggregate::AggregateId;
use crate::id_generate;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid_generator_rs::ULID;

use crate::user_account::events::{UserAccountEvent, UserAccountEventCreatedBody, UserAccountEventDeletedBody};

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct UserAccountId {
  value: ULID,
}

impl UserAccountId {
  pub fn new() -> Self {
    let value = id_generate();
    Self { value }
  }

  pub fn from_ulid(value: ULID) -> Self {
    Self { value }
  }
}

impl AggregateId for UserAccountId {
  fn type_name(&self) -> String {
    "user-account".to_string()
  }

  fn value(&self) -> String {
    self.value.to_string()
  }
}

impl FromStr for UserAccountId {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match ULID::from_str(s) {
      Ok(value) => Ok(Self { value }),
      Err(err) => Err(anyhow!(err)),
    }
  }
}

impl Display for UserAccountId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.value)
  }
}

/// ユーザアカウント。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserAccount {
  id: UserAccountId,
  deleted: bool,
  user_name: String,
  password: String,
  seq_nr_counter: usize,
  version: usize,
}

impl UserAccount {
  pub fn breach_encapsulation_of_seq_nr_counter(&self) -> usize {
    self.seq_nr_counter
  }

  /// コンストラクタ
  ///
  /// # 引数
  /// - `user_name` - ユーザー名
  /// - `password` - パスワード
  ///
  /// # 戻り値
  /// - `UserAccount` - ユーザーアカウント
  /// - `UserAccountEvent` - ユーザーアカウントイベント
  pub fn new(user_name: String, password: String) -> (Self, UserAccountEvent) {
    let id = UserAccountId::new();
    Self::from(id, false, user_name, password, 0, 1)
  }

  pub fn delete(&mut self) -> Result<UserAccountEvent> {
    if self.deleted {
      return Err(anyhow!("The user account is deleted"));
    }
    self.deleted = true;
    self.seq_nr_counter += 1;
    Ok(UserAccountEvent::UserAccountDeleted(UserAccountEventDeletedBody::new(
      self.id.clone(),
      Utc::now(),
    )))
  }

  pub fn from(
    id: UserAccountId,
    deleted: bool,
    user_name: String,
    password: String,
    seq_nr_counter: usize,
    version: usize,
  ) -> (Self, UserAccountEvent) {
    (
      Self {
        id: id.clone(),
        deleted,
        user_name: user_name.clone(),
        password: password.clone(),
        seq_nr_counter,
        version,
      },
      UserAccountEvent::UserAccountCreated(UserAccountEventCreatedBody::new(id, user_name, password, Utc::now())),
    )
  }
}

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::id_generate;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use ulid_generator_rs::ULID;

/// [Message]のIDを表す値オブジェクト。
#[derive(Debug, Clone, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct MessageId {
  value: ULID,
}

impl MessageId {
  /// コンストラクタ。
  pub fn new() -> Self {
    let value = id_generate();
    Self { value }
  }
}

impl Display for MessageId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.value)
  }
}

impl From<ULID> for MessageId {
  fn from(value: ULID) -> Self {
    Self { value }
  }
}

impl FromStr for MessageId {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match ULID::from_str(s) {
      Ok(value) => Ok(Self { value }),
      Err(err) => Err(anyhow!(err)),
    }
  }
}

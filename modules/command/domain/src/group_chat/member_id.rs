use crate::id_generate;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use ulid_generator_rs::ULID;

/// メンバーID。
#[derive(Debug, Clone, Eq, Hash, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct MemberId(ULID);

impl MemberId {
  /// コンストラクタ。
  pub fn new() -> Self {
    let value = id_generate();
    Self(value)
  }
}

impl Display for MemberId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<ULID> for MemberId {
  fn from(value: ULID) -> Self {
    Self(value)
  }
}

impl FromStr for MemberId {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    match ULID::from_str(s) {
      Ok(value) => Ok(Self(value)),
      Err(err) => Err(anyhow!(err)),
    }
  }
}

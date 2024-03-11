use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// グループチャット名を表す値オブジェクト。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupChatName(String);

#[derive(Error, Debug, Clone)]
pub enum GroupChatNameError {
  #[error("the group chat name is empty")]
  Empty,
  #[error("the group chat name is too long")]
  TooLong,
}

impl GroupChatName {
  pub fn new(name: &str) -> Result<Self, GroupChatNameError> {
    if name.is_empty() {
      Err(GroupChatNameError::Empty)
    } else if name.len() > 100 {
      Err(GroupChatNameError::TooLong)
    } else {
      Ok(Self(name.to_string()))
    }
  }
}

impl FromStr for GroupChatName {
  type Err = GroupChatNameError;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    Self::new(s)
  }
}

impl Display for GroupChatName {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

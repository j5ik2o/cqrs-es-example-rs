use crate::group_chat::ParseError;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// [Member]のロール
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemberRole {
  /// 管理者
  Admin,
  /// メンバー
  Member,
}

impl Display for MemberRole {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Admin => write!(f, "Admin"),
      Self::Member => write!(f, "Member"),
    }
  }
}

impl FromStr for MemberRole {
  type Err = ParseError;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "admin" => Ok(Self::Admin),
      "member" => Ok(Self::Member),
      _ => Err(ParseError::InvalidRole(s.to_string())),
    }
  }
}

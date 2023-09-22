use crate::id_generate;
use anyhow::anyhow;
use event_store_adapter_rs::types::AggregateId;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use ulid_generator_rs::ULID;

/// [GroupChat]のIDを表す値オブジェクト。
///
/// NOTE: IDはすべてULIDでよいのでは？わざわざ個別に定義しなくてもよいのでは？という見方もあるが、
/// idを意図しない引数に渡してしまうというミスを防ぐために、型を分けている。
///
/// この手の要求はHaskellではnewtypeやScalaではopaque type aliasで実現するが、Rustでは新たに型を定義するしかない。
/// e.g. https://keens.github.io/blog/2018/12/15/rustdetsuyomenikatawotsukerupart_1__new_type_pattern/
#[derive(Debug, Clone, Eq, Hash, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct GroupChatId {
  value: ULID,
}

impl AggregateId for GroupChatId {
  fn type_name(&self) -> String {
    "group-chat".to_string()
  }

  fn value(&self) -> String {
    self.value.to_string()
  }
}

impl GroupChatId {
  pub fn new() -> Self {
    let value = id_generate();
    Self { value }
  }
}

impl Display for GroupChatId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.value)
  }
}

impl From<ULID> for GroupChatId {
  fn from(value: ULID) -> Self {
    Self { value }
  }
}

impl FromStr for GroupChatId {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    match ULID::from_str(s) {
      Ok(value) => Ok(Self { value }),
      Err(err) => Err(anyhow!(err)),
    }
  }
}

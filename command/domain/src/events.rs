use crate::group_chat::{GroupChatID, GroupChatName, MemberID, Message, MessageID};
use crate::ID_GENERATOR;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid_generator_rs::{ULIDGenerator, ULID};

pub trait Event {
  type ID: std::fmt::Display;
  type AggregateID: std::fmt::Display;
  fn id(&self) -> &Self::ID;
  fn aggregate_id(&self) -> &Self::AggregateID;
  fn seq_nr(&self) -> usize;
  fn occurred_at(&self) -> &DateTime<Utc>;
  fn is_created(&self) -> bool;
}

pub type GroupChatEventID = ULID;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GroupChatEvent {
  GroupChatCreated(GroupChatCreated),
  GroupChatDestroyed(GroupChatDestroyed),
  GroupChatMessagePosted(GroupChatMessagePosted),
  GroupChatMessageDeleted(GroupChatMessageDeleted),
  GroupChatMemberAdd(GroupChatMemberAdd),
  GroupChatMemberRemoved(GroupChatMemberRemoved),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupChatCreated {
  id: GroupChatEventID,
  pub(crate) group_chat_id: GroupChatID,
  pub(crate) seq_nr: usize,
  pub(crate) name: GroupChatName,
  pub(crate) member_ids: Vec<MemberID>,
  occurred_at: DateTime<Utc>,
}

impl GroupChatCreated {
  pub fn new(group_chat_id: GroupChatID, seq_nr: usize, name: GroupChatName, member_ids: Vec<MemberID>) -> Self {
    let id = ID_GENERATOR.lock().unwrap().generate().unwrap();
    let occurred_at = Utc::now();
    Self {
      id,
      group_chat_id,
      seq_nr,
      name,
      member_ids,
      occurred_at,
    }
  }
}

impl Event for GroupChatEvent {
  type AggregateID = GroupChatID;
  type ID = GroupChatEventID;

  fn id(&self) -> &GroupChatEventID {
    match self {
      GroupChatEvent::GroupChatCreated(event) => &event.id,
      GroupChatEvent::GroupChatDestroyed(event) => &event.id,
      GroupChatEvent::GroupChatMessagePosted(event) => &event.id,
      GroupChatEvent::GroupChatMessageDeleted(event) => &event.id,
      GroupChatEvent::GroupChatMemberAdd(event) => &event.id,
      GroupChatEvent::GroupChatMemberRemoved(event) => &event.id,
    }
  }

  fn seq_nr(&self) -> usize {
    match self {
      GroupChatEvent::GroupChatCreated(event) => event.seq_nr,
      GroupChatEvent::GroupChatDestroyed(event) => event.seq_nr,
      GroupChatEvent::GroupChatMessagePosted(event) => event.seq_nr,
      GroupChatEvent::GroupChatMessageDeleted(event) => event.seq_nr,
      GroupChatEvent::GroupChatMemberAdd(event) => event.seq_nr,
      GroupChatEvent::GroupChatMemberRemoved(event) => event.seq_nr,
    }
  }

  fn aggregate_id(&self) -> &GroupChatID {
    match self {
      GroupChatEvent::GroupChatCreated(event) => &event.group_chat_id,
      GroupChatEvent::GroupChatDestroyed(event) => &event.group_chat_id,
      GroupChatEvent::GroupChatMessagePosted(event) => &event.group_chat_id,
      GroupChatEvent::GroupChatMessageDeleted(event) => &event.group_chat_id,
      GroupChatEvent::GroupChatMemberAdd(event) => &event.group_chat_id,
      GroupChatEvent::GroupChatMemberRemoved(event) => &event.group_chat_id,
    }
  }

  fn occurred_at(&self) -> &DateTime<Utc> {
    match self {
      GroupChatEvent::GroupChatCreated(event) => &event.occurred_at,
      GroupChatEvent::GroupChatDestroyed(event) => &event.occurred_at,
      GroupChatEvent::GroupChatMessagePosted(event) => &event.occurred_at,
      GroupChatEvent::GroupChatMessageDeleted(event) => &event.occurred_at,
      GroupChatEvent::GroupChatMemberAdd(event) => &event.occurred_at,
      GroupChatEvent::GroupChatMemberRemoved(event) => &event.occurred_at,
    }
  }

  fn is_created(&self) -> bool {
    match self {
      GroupChatEvent::GroupChatCreated(_) => true,
      _ => false,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupChatDestroyed {
  id: GroupChatEventID,
  group_chat_id: GroupChatID,
  seq_nr: usize,
  occurred_at: DateTime<Utc>,
}

impl GroupChatDestroyed {
  pub fn new(group_chat_id: GroupChatID, seq_nr: usize) -> Self {
    let id = ID_GENERATOR.lock().unwrap().generate().unwrap();
    let occurred_at = Utc::now();
    Self {
      id,
      group_chat_id,
      seq_nr,
      occurred_at,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupChatMessagePosted {
  id: GroupChatEventID,
  group_chat_id: GroupChatID,
  seq_nr: usize,
  message: Message,
  occurred_at: DateTime<Utc>,
}

impl GroupChatMessagePosted {
  pub fn new(group_chat_id: GroupChatID, seq_nr: usize, message: Message) -> Self {
    let id = ID_GENERATOR.lock().unwrap().generate().unwrap();
    let occurred_at = Utc::now();
    Self {
      id,
      group_chat_id,
      seq_nr,
      message,
      occurred_at,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupChatMessageDeleted {
  id: GroupChatEventID,
  group_chat_id: GroupChatID,
  seq_nr: usize,
  message_id: MessageID,
  occurred_at: DateTime<Utc>,
}

impl GroupChatMessageDeleted {
  pub fn new(group_chat_id: GroupChatID, seq_nr: usize, message_id: MessageID) -> Self {
    let mut idgen = ULIDGenerator::new();
    let id = idgen.generate().unwrap();
    let occurred_at = Utc::now();
    Self {
      id,
      group_chat_id,
      seq_nr,
      message_id,
      occurred_at,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupChatMemberAdd {
  id: GroupChatEventID,
  group_chat_id: GroupChatID,
  seq_nr: usize,
  pub(crate) member_id: MemberID,
  occurred_at: DateTime<Utc>,
}

impl GroupChatMemberAdd {
  pub fn new(group_chat_id: GroupChatID, seq_nr: usize, member_id: MemberID) -> Self {
    let mut idgen = ULIDGenerator::new();
    let id = idgen.generate().unwrap();
    let occurred_at = Utc::now();
    Self {
      id,
      group_chat_id,
      seq_nr,
      member_id,
      occurred_at,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupChatMemberRemoved {
  id: GroupChatEventID,
  group_chat_id: GroupChatID,
  seq_nr: usize,
  pub(crate) member_id: MemberID,
  occurred_at: DateTime<Utc>,
}

impl GroupChatMemberRemoved {
  pub fn new(group_chat_id: GroupChatID, seq_nr: usize, member_id: MemberID) -> Self {
    let mut idgen = ULIDGenerator::new();
    let id = idgen.generate().unwrap();
    let occurred_at = Utc::now();
    Self {
      id,
      group_chat_id,
      seq_nr,
      member_id,
      occurred_at,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::events::{GroupChatCreated, GroupChatEvent};
  use crate::group_chat::{GroupChatID, GroupChatName};

  #[test]
  fn test() {
    let group_chat_id = GroupChatID::new();
    let group_chat_name = GroupChatName::new("test".to_string());
    let event = GroupChatEvent::GroupChatCreated(GroupChatCreated::new(group_chat_id, 1usize, group_chat_name, vec![]));
    let json = serde_json::to_string(&event);
    println!("{}", json.unwrap());
  }
}

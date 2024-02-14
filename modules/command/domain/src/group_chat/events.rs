use chrono::{DateTime, Utc};
use event_store_adapter_rs::types::Event;
use serde::{Deserialize, Serialize};
use ulid_generator_rs::{ULIDGenerator, ULID};

use crate::group_chat::member::Member;
use crate::group_chat::{GroupChatId, GroupChatName, Members, Message, MessageId};
use crate::id_generate;
use crate::user_account::UserAccountId;

pub type GroupChatEventId = ULID;

/// グループチャットに関するイベント。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GroupChatEvent {
  /// グループチャットが作成された
  GroupChatCreated(GroupChatEventCreatedBody),
  /// グループチャットが削除された
  GroupChatDeleted(GroupChatEventDeletedBody),
  /// グループチャットがリネームされた
  GroupChatRenamed(GroupChatEventRenamedBody),
  /// グループチャットにメッセージが投稿された
  GroupChatMessagePosted(GroupChatEventMessagePostedBody),
  /// グループチャットのメッセージが削除された
  GroupChatMessageDeleted(GroupChatventMessageDeletedBody),
  /// グループチャットにメンバーが追加された
  GroupChatMemberAdded(GroupChatEventMemberAddedBody),
  /// グループチャットのメンバーが削除された
  GroupChatMemberRemoved(GroupChatEventMemberRemovedBody),
}

impl Event for GroupChatEvent {
  type AggregateID = GroupChatId;
  type ID = GroupChatEventId;

  fn id(&self) -> &GroupChatEventId {
    match self {
      GroupChatEvent::GroupChatCreated(event) => &event.id,
      GroupChatEvent::GroupChatDeleted(event) => &event.id,
      GroupChatEvent::GroupChatRenamed(event) => &event.id,
      GroupChatEvent::GroupChatMessagePosted(event) => &event.id,
      GroupChatEvent::GroupChatMessageDeleted(event) => &event.id,
      GroupChatEvent::GroupChatMemberAdded(event) => &event.id,
      GroupChatEvent::GroupChatMemberRemoved(event) => &event.id,
    }
  }

  fn seq_nr(&self) -> usize {
    match self {
      GroupChatEvent::GroupChatCreated(event) => event.seq_nr,
      GroupChatEvent::GroupChatDeleted(event) => event.seq_nr,
      GroupChatEvent::GroupChatRenamed(event) => event.seq_nr,
      GroupChatEvent::GroupChatMessagePosted(event) => event.seq_nr,
      GroupChatEvent::GroupChatMessageDeleted(event) => event.seq_nr,
      GroupChatEvent::GroupChatMemberAdded(event) => event.seq_nr,
      GroupChatEvent::GroupChatMemberRemoved(event) => event.seq_nr,
    }
  }

  fn aggregate_id(&self) -> &GroupChatId {
    match self {
      GroupChatEvent::GroupChatCreated(event) => &event.aggregate_id,
      GroupChatEvent::GroupChatDeleted(event) => &event.aggregate_id,
      GroupChatEvent::GroupChatRenamed(event) => &event.aggregate_id,
      GroupChatEvent::GroupChatMessagePosted(event) => &event.aggregate_id,
      GroupChatEvent::GroupChatMessageDeleted(event) => &event.aggregate_id,
      GroupChatEvent::GroupChatMemberAdded(event) => &event.aggregate_id,
      GroupChatEvent::GroupChatMemberRemoved(event) => &event.aggregate_id,
    }
  }

  fn occurred_at(&self) -> &DateTime<Utc> {
    match self {
      GroupChatEvent::GroupChatCreated(event) => &event.occurred_at,
      GroupChatEvent::GroupChatDeleted(event) => &event.occurred_at,
      GroupChatEvent::GroupChatRenamed(event) => &event.occurred_at,
      GroupChatEvent::GroupChatMessagePosted(event) => &event.occurred_at,
      GroupChatEvent::GroupChatMessageDeleted(event) => &event.occurred_at,
      GroupChatEvent::GroupChatMemberAdded(event) => &event.occurred_at,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatEventCreatedBody {
  pub id: GroupChatEventId,
  pub aggregate_id: GroupChatId,
  pub seq_nr: usize,
  pub name: GroupChatName,
  pub members: Members,
  pub occurred_at: DateTime<Utc>,
}

impl GroupChatEventCreatedBody {
  pub fn new(aggregate_id: GroupChatId, seq_nr: usize, name: GroupChatName, members: Members) -> Self {
    let id = id_generate();
    let occurred_at = Utc::now();
    Self {
      id,
      aggregate_id,
      seq_nr,
      name,
      members,
      occurred_at,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatEventDeletedBody {
  pub id: GroupChatEventId,
  pub aggregate_id: GroupChatId,
  pub seq_nr: usize,
  pub executor_id: UserAccountId,
  pub occurred_at: DateTime<Utc>,
}

impl GroupChatEventDeletedBody {
  pub fn new(aggregate_id: GroupChatId, seq_nr: usize, executor_id: UserAccountId) -> Self {
    let id = id_generate();
    let occurred_at = Utc::now();
    Self {
      id,
      aggregate_id,
      seq_nr,
      executor_id,
      occurred_at,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatEventRenamedBody {
  pub id: GroupChatEventId,
  pub aggregate_id: GroupChatId,
  pub seq_nr: usize,
  pub name: GroupChatName,
  pub executor_id: UserAccountId,
  pub occurred_at: DateTime<Utc>,
}

impl GroupChatEventRenamedBody {
  pub fn new(aggregate_id: GroupChatId, seq_nr: usize, name: GroupChatName, executor_id: UserAccountId) -> Self {
    let id = id_generate();
    let occurred_at = Utc::now();
    Self {
      id,
      aggregate_id,
      seq_nr,
      name,
      executor_id,
      occurred_at,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatEventMessagePostedBody {
  pub(crate) id: GroupChatEventId,
  pub aggregate_id: GroupChatId,
  pub(crate) seq_nr: usize,
  pub message: Message,
  pub(crate) executor_id: UserAccountId,
  pub occurred_at: DateTime<Utc>,
}

impl GroupChatEventMessagePostedBody {
  pub fn new(aggregate_id: GroupChatId, seq_nr: usize, message: Message, executor_id: UserAccountId) -> Self {
    let id = id_generate();
    let occurred_at = Utc::now();
    Self {
      id,
      aggregate_id,
      seq_nr,
      message,
      executor_id,
      occurred_at,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatventMessageDeletedBody {
  pub(crate) id: GroupChatEventId,
  pub aggregate_id: GroupChatId,
  pub(crate) seq_nr: usize,
  pub message_id: MessageId,
  pub(crate) executor_id: UserAccountId,
  pub occurred_at: DateTime<Utc>,
}

impl GroupChatventMessageDeletedBody {
  pub fn new(aggregate_id: GroupChatId, seq_nr: usize, message_id: MessageId, executor_id: UserAccountId) -> Self {
    let mut idgen = ULIDGenerator::new();
    let id = idgen.generate().unwrap();
    let occurred_at = Utc::now();
    Self {
      id,
      aggregate_id,
      seq_nr,
      message_id,
      executor_id,
      occurred_at,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatEventMemberAddedBody {
  pub id: GroupChatEventId,
  pub aggregate_id: GroupChatId,
  pub seq_nr: usize,
  pub member: Member,
  pub executor_id: UserAccountId,
  pub occurred_at: DateTime<Utc>,
}

impl GroupChatEventMemberAddedBody {
  pub fn new(aggregate_id: GroupChatId, seq_nr: usize, member: Member, executor_id: UserAccountId) -> Self {
    let id = id_generate();
    let occurred_at = Utc::now();
    Self {
      id,
      aggregate_id,
      seq_nr,
      member,
      executor_id,
      occurred_at,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatEventMemberRemovedBody {
  pub(crate) id: GroupChatEventId,
  pub aggregate_id: GroupChatId,
  pub(crate) seq_nr: usize,
  pub user_account_id: UserAccountId,
  pub(crate) executor_id: UserAccountId,
  pub(crate) occurred_at: DateTime<Utc>,
}

impl GroupChatEventMemberRemovedBody {
  pub fn new(
    aggregate_id: GroupChatId,
    seq_nr: usize,
    user_account_id: UserAccountId,
    executor_id: UserAccountId,
  ) -> Self {
    let id = id_generate();
    let occurred_at = Utc::now();
    Self {
      id,
      aggregate_id,
      seq_nr,
      user_account_id,
      executor_id,
      occurred_at,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::group_chat::events::{GroupChatEvent, GroupChatEventCreatedBody};
  use crate::group_chat::{GroupChatId, GroupChatName, Members};
  use crate::user_account::UserAccountId;
  use event_store_adapter_rs::types::Event;

  #[test]
  fn test_to_json() {
    let group_chat_id = GroupChatId::new();
    let group_chat = GroupChatName::new("test").unwrap();
    let admin_user_account_id = UserAccountId::new();
    let event = GroupChatEvent::GroupChatCreated(GroupChatEventCreatedBody::new(
      group_chat_id,
      1usize,
      group_chat,
      Members::new(admin_user_account_id),
    ));
    let json = serde_json::to_string(&event);
    let _occurred_at = event.occurred_at().timestamp_millis();
    println!("{}", json.unwrap());
  }
}

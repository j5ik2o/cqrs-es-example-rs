use serde::{Deserialize, Serialize};
use ulid_generator_rs::{ULIDGenerator, ULID};

use crate::group_chat::member::Member;
use crate::group_chat::{GroupChatName, Members, Message, MessageId};
use crate::id_generate;
use crate::user_account::UserAccountId;

pub type GroupChatEventId = ULID;

/// payload のスキーマ版。envelope の manifest に載せる。
pub const SCHEMA_VERSION: u32 = 1;

/// グループチャットに関するイベント。
///
/// ESA v5 では集約 ID、`seq_nr`、`occurred_at` は envelope が持つため payload には含めない。
/// イベント型の判別子（`type`）だけは payload に残す。ESA のシリアライザは `manifest` を
/// デシリアライザに渡さないため、payload 単体で型を復元できる必要がある。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GroupChatEvent {
  /// グループチャットが作成された
  GroupChatCreated(GroupChatEventCreatedBody),
  /// グループチャットが削除された
  GroupChatDeleted(GroupChatEventDeletedBody),
  /// グループチャットがリネームされた
  GroupChatRenamed(GroupChatEventRenamedBody),
  /// グループチャットにメンバーが追加された
  GroupChatMemberAdded(GroupChatEventMemberAddedBody),
  /// グループチャットのメンバーが削除された
  GroupChatMemberRemoved(GroupChatEventMemberRemovedBody),
  /// グループチャットにメッセージが投稿された
  GroupChatMessagePosted(GroupChatEventMessagePostedBody),
  /// グループチャットのメッセージが編集された
  GroupChatMessageEdited(GroupChatEventMessageEditedBody),
  /// グループチャットのメッセージが削除された
  GroupChatMessageDeleted(GroupChatEventMessageDeletedBody),
}

impl GroupChatEvent {
  /// イベントIDを返す。
  pub fn id(&self) -> &GroupChatEventId {
    match self {
      GroupChatEvent::GroupChatCreated(event) => &event.id,
      GroupChatEvent::GroupChatDeleted(event) => &event.id,
      GroupChatEvent::GroupChatRenamed(event) => &event.id,
      GroupChatEvent::GroupChatMemberAdded(event) => &event.id,
      GroupChatEvent::GroupChatMemberRemoved(event) => &event.id,
      GroupChatEvent::GroupChatMessagePosted(event) => &event.id,
      GroupChatEvent::GroupChatMessageEdited(event) => &event.id,
      GroupChatEvent::GroupChatMessageDeleted(event) => &event.id,
    }
  }

  /// イベント型名を返す。payload の `type` と同じ値になる。
  pub fn event_type_name(&self) -> &'static str {
    match self {
      GroupChatEvent::GroupChatCreated(_) => "GroupChatCreated",
      GroupChatEvent::GroupChatDeleted(_) => "GroupChatDeleted",
      GroupChatEvent::GroupChatRenamed(_) => "GroupChatRenamed",
      GroupChatEvent::GroupChatMemberAdded(_) => "GroupChatMemberAdded",
      GroupChatEvent::GroupChatMemberRemoved(_) => "GroupChatMemberRemoved",
      GroupChatEvent::GroupChatMessagePosted(_) => "GroupChatMessagePosted",
      GroupChatEvent::GroupChatMessageEdited(_) => "GroupChatMessageEdited",
      GroupChatEvent::GroupChatMessageDeleted(_) => "GroupChatMessageDeleted",
    }
  }

  /// envelope に載せる manifest を `<イベント型名>@<スキーマ版>` の形で返す。
  pub fn manifest(&self) -> String {
    format!("{}@{}", self.event_type_name(), SCHEMA_VERSION)
  }

  /// 集約の作成イベントかどうかを返す。
  pub fn is_created(&self) -> bool {
    matches!(self, GroupChatEvent::GroupChatCreated(_))
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatEventCreatedBody {
  pub id: GroupChatEventId,
  pub name: GroupChatName,
  pub members: Members,
}

impl GroupChatEventCreatedBody {
  pub fn new(name: GroupChatName, members: Members) -> Self {
    let id = id_generate();
    Self { id, name, members }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatEventDeletedBody {
  pub id: GroupChatEventId,
  pub executor_id: UserAccountId,
}

impl GroupChatEventDeletedBody {
  pub fn new(executor_id: UserAccountId) -> Self {
    let id = id_generate();
    Self { id, executor_id }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatEventRenamedBody {
  pub id: GroupChatEventId,
  pub name: GroupChatName,
  pub executor_id: UserAccountId,
}

impl GroupChatEventRenamedBody {
  pub fn new(name: GroupChatName, executor_id: UserAccountId) -> Self {
    let id = id_generate();
    Self { id, name, executor_id }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatEventMessagePostedBody {
  pub(crate) id: GroupChatEventId,
  pub message: Message,
  pub(crate) executor_id: UserAccountId,
}

impl GroupChatEventMessagePostedBody {
  pub fn new(message: Message, executor_id: UserAccountId) -> Self {
    let id = id_generate();
    Self {
      id,
      message,
      executor_id,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatEventMessageEditedBody {
  pub(crate) id: GroupChatEventId,
  pub message: Message,
  pub(crate) executor_id: UserAccountId,
}

impl GroupChatEventMessageEditedBody {
  pub fn new(message: Message, executor_id: UserAccountId) -> Self {
    let id = id_generate();
    Self {
      id,
      message,
      executor_id,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatEventMessageDeletedBody {
  pub(crate) id: GroupChatEventId,
  pub message_id: MessageId,
  pub(crate) executor_id: UserAccountId,
}

impl GroupChatEventMessageDeletedBody {
  pub fn new(message_id: MessageId, executor_id: UserAccountId) -> Self {
    let mut idgen = ULIDGenerator::new();
    let id = idgen.generate().unwrap();
    Self {
      id,
      message_id,
      executor_id,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatEventMemberAddedBody {
  pub id: GroupChatEventId,
  pub member: Member,
  pub executor_id: UserAccountId,
}

impl GroupChatEventMemberAddedBody {
  pub fn new(member: Member, executor_id: UserAccountId) -> Self {
    let id = id_generate();
    Self {
      id,
      member,
      executor_id,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatEventMemberRemovedBody {
  pub(crate) id: GroupChatEventId,
  pub user_account_id: UserAccountId,
  pub(crate) executor_id: UserAccountId,
}

impl GroupChatEventMemberRemovedBody {
  pub fn new(user_account_id: UserAccountId, executor_id: UserAccountId) -> Self {
    let id = id_generate();
    Self {
      id,
      user_account_id,
      executor_id,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::group_chat::events::{GroupChatEvent, GroupChatEventCreatedBody};
  use crate::group_chat::{GroupChatName, Members};
  use crate::user_account::UserAccountId;

  #[test]
  fn should_serialize_payload_without_envelope_metadata() {
    let group_chat_name = GroupChatName::new("test").unwrap();
    let admin_user_account_id = UserAccountId::new();
    let event = GroupChatEvent::GroupChatCreated(GroupChatEventCreatedBody::new(
      group_chat_name,
      Members::new(admin_user_account_id),
    ));

    let json = serde_json::to_value(&event).unwrap();

    assert_eq!(json["type"], "GroupChatCreated");
    assert!(json.get("aggregate_id").is_none());
    assert!(json.get("seq_nr").is_none());
    assert!(json.get("occurred_at").is_none());
    assert!(json.get("manifest").is_none());
    assert!(json.get("name").is_some());
    assert!(json.get("members").is_some());

    let restored = serde_json::from_value::<GroupChatEvent>(json).unwrap();
    assert!(matches!(restored, GroupChatEvent::GroupChatCreated(_)));
    assert_eq!(restored.event_type_name(), "GroupChatCreated");
    assert_eq!(restored.manifest(), "GroupChatCreated@1");
  }
}

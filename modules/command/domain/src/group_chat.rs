use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use event_store_adapter_rs::types::Aggregate;
use serde::{Deserialize, Serialize};

pub use crate::group_chat::events::{
  GroupChatEvent, GroupChatEventCreatedBody, GroupChatEventDeletedBody, GroupChatEventMemberAddedBody,
  GroupChatEventMemberRemovedBody, GroupChatEventMessagePostedBody, GroupChatEventRenamedBody,
  GroupChatventMessageDeletedBody,
};
pub use crate::group_chat::group_chat_id::GroupChatId;
pub use crate::group_chat::group_chat_name::GroupChatName;
pub use crate::group_chat::member::Member;
pub use crate::group_chat::member_id::MemberId;
pub use crate::group_chat::member_role::MemberRole;
pub use crate::group_chat::members::Members;
pub use crate::group_chat::message::Message;
pub use crate::group_chat::message_id::MessageId;
pub use crate::group_chat::messages::Messages;
use crate::user_account::UserAccountId;

mod events;
mod group_chat_id;
mod group_chat_name;
mod member;
mod member_id;
mod member_role;
mod members;
mod message;
mod message_id;
mod messages;

/// [Message]をやりとりする場であるグループチャットを表すモデル。
///
/// NOTE: Serialize, Deserializeをドメインモデルに適用することはレイヤーの責務違反になるので
/// 実際はドメインモデルには適用しない。インターフェイスアダプタ層のモデルに適用することが望ましい。
/// 今回は時間がなかったこともあり、ドメインモデルに適用しています。
///
/// CAUTION: ドメインモデルの構造を変えた場合は、直接的にAPIのレスポンスの構造が変わってしまう可能性が高いの
/// 注意してください。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChat {
  id: GroupChatId,
  deleted: bool,
  name: GroupChatName,
  members: Members,
  messages: Messages,
  seq_nr_counter: usize,
  version: usize,
    last_updated_at: DateTime<Utc>,
}

impl PartialEq for GroupChat {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id && self.name == other.name && self.members == other.members && self.messages == other.messages
  }
}

impl Aggregate for GroupChat {
  type ID = GroupChatId;

  fn id(&self) -> &Self::ID {
    &self.id
  }

  fn seq_nr(&self) -> usize {
    self.seq_nr_counter
  }

  fn version(&self) -> usize {
    self.version
  }

  fn set_version(&mut self, version: usize) {
    self.version = version;
  }

    fn last_updated_at(&self) -> &DateTime<Utc> {
        &self.last_updated_at
    }
}

impl GroupChat {
  // pub fn breach_encapsulation_of_seq_nr_counter(&self) -> usize {
  //  self.seq_nr_counter
  //}

  /// コンストラクタ(ID自動生成)
  ///
  /// NOTE: 厳密にはコンストラクタとは言えませんが、Self::newを便宜上コンストラクタと呼ぶことにする。
  ///
  /// # 引数
  /// - `name`: [GroupChatName]
  /// - `members`: [Members]
  ///
  /// # 戻り値
  /// - [GroupChat]
  /// - [GroupChatEvent]
  pub fn new(name: GroupChatName, members: Members) -> (Self, GroupChatEvent) {
    let id = GroupChatId::new();
    Self::from(id, false, name, members, 0, 1)
  }

  /// コンストラクタ(ID指定)
  ///
  /// # 引数
  /// - `id`: グループチャットID
  /// - `deleted`: 削除フラグ
  /// - `name`: [GroupChatName]
  /// - `members`: [Members]
  /// - `seq_nr_counter`: シーケンス番号
  /// - `version`: バージョン
  ///
  /// # 戻り値
  /// - [GroupChat]
  /// - [GroupChatEvent]
  pub fn from(
    id: GroupChatId,
    deleted: bool,
    name: GroupChatName,
    members: Members,
    seq_nr_counter: usize,
    version: usize,
  ) -> (Self, GroupChatEvent) {
    let mut my_self = Self {
      id: id.clone(),
      deleted,
      name: name.clone(),
      members: members.clone(),
      messages: Messages::new([]),
      seq_nr_counter,
      version,
        last_updated_at: Utc::now(),
    };
    my_self.seq_nr_counter += 1;
    let event = GroupChatEvent::GroupChatCreated(GroupChatEventCreatedBody::new(
      id,
      my_self.seq_nr_counter,
      name,
      members,
    ));
    (my_self, event)
  }

  /// 既存のインスタンスにイベントを適用する。
  ///
  /// # 引数
  /// - `event`: 適用するイベント
  fn apply_event(&mut self, event: &GroupChatEvent) {
    match event {
      GroupChatEvent::GroupChatDeleted(body) => {
        self.delete(body.executor_id.clone()).unwrap();
      }
      GroupChatEvent::GroupChatRenamed(body) => {
        self.rename(body.name.clone(), body.executor_id.clone()).unwrap();
      }
      GroupChatEvent::GroupChatMemberAdded(body) => {
        self
          .add_member(
            body.member.breach_encapsulation_of_id().clone(),
            body.member.breach_encapsulation_of_user_account_id().clone(),
            body.member.breach_encapsulation_of_role().clone(),
            body.executor_id.clone(),
          )
          .unwrap();
      }
      GroupChatEvent::GroupChatMemberRemoved(body) => {
        self
          .remove_member(body.user_account_id.clone(), body.executor_id.clone())
          .unwrap();
      }
      GroupChatEvent::GroupChatMessagePosted(body) => {
        self
          .post_message(body.message.clone(), body.executor_id.clone())
          .unwrap();
      }
      GroupChatEvent::GroupChatMessageDeleted(body) => {
        self
          .delete_message(body.message_id.clone(), body.executor_id.clone())
          .unwrap();
      }
      _ => {}
    }
  }

  /// イベント及びスナップショットを利用して、グループチャットを再生する
  ///
  /// # 引数
  /// - `events`: イベントの集合
  /// - `snapshot`: スナップショット(任意)
  /// - `version`: スナップショットのバージョン
  ///
  /// # 戻り値
  /// - [GroupChat]
  pub fn replay(events: Vec<GroupChatEvent>, snapshot_opt: Option<GroupChat>, version: usize) -> Self {
    log::debug!("event.size = {}", events.len());
    let mut result = events
      .iter()
      .fold(snapshot_opt, |result, event| match (result, event) {
        (Some(mut this), event) => {
          log::debug!("Replaying snapshot: {:?}", this);
          log::debug!("Replaying event: {:?}", event);
          this.apply_event(event);
          Some(this)
        }
        (..) => None,
      })
      .unwrap();
    result.set_version(version);
    result
  }

  /// [GroupChatName]の参照を返す。
  pub fn name(&self) -> &GroupChatName {
    &self.name
  }

  /// [Members]の参照を返す
  pub fn members(&self) -> &Members {
    &self.members
  }

  /// [Messages]の参照を返す
  pub fn messages(&self) -> &Messages {
    &self.messages
  }

  /// グループチャットをリネームする
  ///
  /// # 引数
  /// - name: 新しいグループチャット名
  /// - executor_id: 実行者のユーザアカウントID
  ///
  /// # 戻り値
  /// - グループチャットが削除されている場合はエラーを返す。
  /// - 実行者がメンバーでない場合はエラーを返す。
  /// - 新しい名前が既に設定されている場合はエラーを返す。
  /// - 成功した場合は、GroupChatRenamedイベントを返す。
  pub fn rename(&mut self, name: GroupChatName, executor_id: UserAccountId) -> Result<GroupChatEvent> {
    if self.deleted {
      return Err(anyhow!("The group chat is deleted"));
    }
    if !self.members.is_member(&executor_id) {
      return Err(anyhow!("executor_id is not a member of the group chat"));
    }
    if self.name == name {
      return Err(anyhow!("Name already set"));
    }
    self.name = name;
    self.seq_nr_counter += 1;
    Ok(GroupChatEvent::GroupChatRenamed(GroupChatEventRenamedBody::new(
      self.id.clone(),
      self.seq_nr_counter,
      self.name.clone(),
      executor_id,
    )))
  }

  /// グループチャットにメンバーを追加する
  ///
  /// # 引数
  /// - member_id: メンバーID
  /// - user_account_id: ユーザアカウントID
  /// - role: メンバーの役割
  /// - executor_id: 実行者のユーザアカウントID
  ///
  /// # 戻り値
  /// - グループチャットが削除されている場合はエラーを返す。
  /// - 実行者が管理者でない場合はエラーを返す。
  /// - ユーザアカウントIDが既にメンバーに設定されている場合はエラーを返す。
  /// - 成功した場合は、GroupChatMemberAddedイベントを返す。
  pub fn add_member(
    &mut self,
    member_id: MemberId,
    user_account_id: UserAccountId,
    role: MemberRole,
    executor_id: UserAccountId,
  ) -> Result<GroupChatEvent> {
    if self.deleted {
      return Err(anyhow!("The group chat is deleted"));
    }
    if !self.members.is_administrator(&executor_id) {
      return Err(anyhow!("executor_id is not a member of the group chat"));
    }
    if self.members.is_member(&user_account_id) {
      return Err(anyhow!("user_account_id is already a member of the group chat"));
    }
    let member = Member::new(member_id, user_account_id, role);
    self.members.add_member(member.clone());
    self.seq_nr_counter += 1;
    Ok(GroupChatEvent::GroupChatMemberAdded(
      GroupChatEventMemberAddedBody::new(self.id.clone(), self.seq_nr_counter, member, executor_id),
    ))
  }

  /// グループチャットからメンバーを削除する
  ///
  /// # 引数
  /// - user_account_id: ユーザアカウントID
  /// - executor_id: 実行者のユーザアカウントID
  ///
  /// # 戻り値
  /// - グループチャットが削除されている場合はエラーを返す。
  /// - 実行者が管理者でない場合はエラーを返す。
  /// - ユーザアカウントIDがメンバーに設定されていない場合はエラーを返す。
  /// - 成功した場合は、GroupChatMemberRemovedイベントを返す。
  pub fn remove_member(
    &mut self,
    user_account_id: UserAccountId,
    executor_id: UserAccountId,
  ) -> Result<GroupChatEvent> {
    if self.deleted {
      return Err(anyhow!("The group chat is deleted"));
    }
    if !self.members.is_administrator(&executor_id) {
      return Err(anyhow!("User is not a member of the group chat"));
    }
    if !self.members.is_member(&user_account_id) {
      return Err(anyhow!("user_account_id is not a member of the group chat"));
    }
    self.members.remove_member_by_user_account_id(&user_account_id);
    self.seq_nr_counter += 1;
    Ok(GroupChatEvent::GroupChatMemberRemoved(
      GroupChatEventMemberRemovedBody::new(self.id.clone(), self.seq_nr_counter, user_account_id, executor_id),
    ))
  }

  /// グループチャットにメッセージを投稿する
  ///
  /// # 引数
  /// - message: メッセージ
  /// - executor_id: 実行者のユーザアカウントID
  ///
  /// # 戻り値
  /// - グループチャットが削除されている場合はエラーを返す。
  /// - 実行者がメッセージの送信者でない場合はエラーを返す。
  /// - 実行者がメンバーでない場合はエラーを返す。
  /// - メッセージIDが既に存在する場合はエラーを返す。
  /// - 成功した場合は、GroupChatMessagePostedイベントを返す。
  pub fn post_message(&mut self, _message: Message, _executor_id: UserAccountId) -> Result<GroupChatEvent> {
    todo!() // 必須課題 難易度:高
  }

  /// メッセージを削除する
  ///
  /// # 引数
  /// - message_id: メッセージID
  /// - executor_id: 実行者のユーザアカウントID
  ///
  /// # 戻り値
  /// - グループチャットが削除されている場合はエラーを返す。
  /// - 実行者がメッセージの送信者でない場合はエラーを返す。
  /// - 実行者がメンバーでない場合はエラーを返す。
  /// - メッセージIDが存在しない場合はエラーを返す。
  /// - 成功した場合は、GroupChatMessageDeletedイベントを返す。
  pub fn delete_message(&mut self, message_id: MessageId, executor_id: UserAccountId) -> Result<GroupChatEvent> {
    if self.deleted {
      return Err(anyhow!("The group chat is deleted"));
    }
    if !self.members.is_member(&executor_id) {
      return Err(anyhow!("User is not a member of the group chat"));
    }
    let result = self.messages.get(&message_id);
    match result {
      None => Err(anyhow!("Message not found")),
      Some(message) => {
        let member = self
          .members
          .find_by_user_account_id(message.breach_encapsulation_of_sender_id())
          .unwrap();
        if *member.breach_encapsulation_of_user_account_id() != executor_id {
          return Err(anyhow!("User is not the sender of the message"));
        }
        self.messages.remove(&message_id).unwrap();
        self.seq_nr_counter += 1;
        Ok(GroupChatEvent::GroupChatMessageDeleted(
          GroupChatventMessageDeletedBody::new(self.id.clone(), self.seq_nr_counter, message_id, executor_id),
        ))
      }
    }
  }

  /// グループチャットを削除する
  ///
  /// # 引数
  /// - executor_id: 実行者のユーザアカウントID
  ///
  /// # 戻り値
  /// - グループチャットが削除されている場合はエラーを返す。
  /// - 実行者が管理者でない場合はエラーを返す。
  /// - 成功した場合は、GroupChatDeletedイベントを返す。
  pub fn delete(&mut self, executor_id: UserAccountId) -> Result<GroupChatEvent> {
    if self.deleted {
      return Err(anyhow!("The group chat is deleted"));
    }
    if !self.members.is_administrator(&executor_id) {
      return Err(anyhow!("User is not a administrator of the group chat"));
    }
    self.deleted = true;
    self.seq_nr_counter += 1;
    Ok(GroupChatEvent::GroupChatDeleted(GroupChatEventDeletedBody::new(
      self.id.clone(),
      self.seq_nr_counter,
      executor_id,
    )))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_delete_group_chat() {
    let group_chat_name = GroupChatName::new("test").unwrap();
    let admin_user_account_id = UserAccountId::new();
    let members = Members::new(admin_user_account_id.clone());
    let (mut group_chat, _) = GroupChat::new(group_chat_name.clone(), members);
    let user_account_id = UserAccountId::new();
    let member_id = MemberId::new();

    let _ = group_chat
      .add_member(
        member_id,
        user_account_id.clone(),
        MemberRole::Member,
        admin_user_account_id.clone(),
      )
      .unwrap();
    assert!(group_chat.members().is_member(&user_account_id));

    let result = group_chat.delete(user_account_id.clone());
    assert!(result.is_err());

    let result = group_chat.delete(admin_user_account_id.clone());
    assert!(result.is_ok());
  }

  #[test]
  fn test_rename() {
    let group_chat_name = GroupChatName::new("test").unwrap();
    let admin_user_account_id = UserAccountId::new();
    let members = Members::new(admin_user_account_id.clone());
    let (mut group_chat, _) = GroupChat::new(group_chat_name.clone(), members);
    assert_eq!(group_chat.name, group_chat_name);

    let _ = group_chat
      .rename(GroupChatName::new("test2").unwrap(), admin_user_account_id.clone())
      .unwrap();

    assert!(group_chat.name().to_string() == "test2");
  }

  #[test]
  fn test_add_member() {
    let group_chat_name = GroupChatName::new("test").unwrap();
    let admin_user_account_id = UserAccountId::new();
    let members = Members::new(admin_user_account_id.clone());
    let (mut group_chat, _) = GroupChat::new(group_chat_name.clone(), members);
    assert_eq!(group_chat.name, group_chat_name);

    let user_account_id = UserAccountId::new();
    let member_id = MemberId::new();

    let _ = group_chat
      .add_member(
        member_id,
        user_account_id.clone(),
        MemberRole::Member,
        admin_user_account_id.clone(),
      )
      .unwrap();

    assert!(group_chat.members().is_member(&user_account_id));
  }

  #[test]
  fn test_remove_member() {
    let group_chat_name = GroupChatName::new("test").unwrap();
    let admin_user_account_id = UserAccountId::new();
    let members = Members::new(admin_user_account_id.clone());
    let (mut group_chat, _) = GroupChat::new(group_chat_name.clone(), members);
    assert_eq!(group_chat.name, group_chat_name);

    let user_account_id = UserAccountId::new();
    let member_id = MemberId::new();

    let _ = group_chat
      .add_member(
        member_id,
        user_account_id.clone(),
        MemberRole::Member,
        admin_user_account_id.clone(),
      )
      .unwrap();

    assert!(group_chat.members().is_member(&user_account_id));

    let _ = group_chat
      .remove_member(user_account_id.clone(), admin_user_account_id.clone())
      .unwrap();

    assert!(!group_chat.members().is_member(&user_account_id));
  }

  #[test]
  #[ignore] // post_messageの実装完了後に#[ignore]を削除してください。
  fn test_post_message() {
    let group_chat_name = GroupChatName::new("test").unwrap();
    let admin_user_account_id = UserAccountId::new();
    let members = Members::new(admin_user_account_id.clone());
    let (mut group_chat, _) = GroupChat::new(group_chat_name.clone(), members);
    assert_eq!(group_chat.name, group_chat_name);

    let user_account_id = UserAccountId::new();
    let member_id = MemberId::new();
    let _ = group_chat
      .add_member(
        member_id,
        user_account_id.clone(),
        MemberRole::Member,
        admin_user_account_id.clone(),
      )
      .unwrap();

    let message = Message::new("test".to_string(), user_account_id.clone());
    let _ = group_chat
      .post_message(message.clone(), user_account_id.clone())
      .unwrap();

    assert!(group_chat.messages().contains(message.breach_encapsulation_of_id()));
  }

  #[test]
  #[ignore] // post_messageの実装完了後に#[ignore]を削除してください。
  fn test_delete_message() {
    let group_chat_name = GroupChatName::new("test").unwrap();
    let admin_user_account_id = UserAccountId::new();
    let members = Members::new(admin_user_account_id.clone());
    let (mut group_chat, _) = GroupChat::new(group_chat_name.clone(), members);

    let user_account_id = UserAccountId::new();
    let member_id = MemberId::new();
    let _ = group_chat
      .add_member(
        member_id,
        user_account_id.clone(),
        MemberRole::Member,
        admin_user_account_id.clone(),
      )
      .unwrap();

    let message = Message::new("test".to_string(), user_account_id.clone());
    let _ = group_chat
      .post_message(message.clone(), user_account_id.clone())
      .unwrap();

    assert!(group_chat.messages().contains(message.breach_encapsulation_of_id()));

    let _ = group_chat
      .delete_message(message.breach_encapsulation_of_id().clone(), user_account_id.clone())
      .unwrap();

    assert!(!group_chat.messages().contains(message.breach_encapsulation_of_id()));
  }

  #[test]
  #[ignore] // post_messageの実装完了後に#[ignore]を削除してください。
  fn test_to_json() {
    let group_chat_name = GroupChatName::new("test").unwrap();
    let admin_user_account_id = UserAccountId::new();
    let members = Members::new(admin_user_account_id.clone());
    let (mut group_chat, _) = GroupChat::new(group_chat_name.clone(), members);
    assert_eq!(group_chat.name, group_chat_name);

    let message = Message::new("test".to_string(), admin_user_account_id.clone());
    let _ = group_chat
      .post_message(message.clone(), admin_user_account_id.clone())
      .unwrap();

    let json = serde_json::to_string(&group_chat);
    println!("{}", json.unwrap());
  }
}

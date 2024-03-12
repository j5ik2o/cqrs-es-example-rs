use std::fmt::Debug;

use chrono::{DateTime, Utc};
use event_store_adapter_rs::types::{EventStoreReadError, EventStoreWriteError};
use thiserror::Error;

use command_domain::group_chat::*;
use command_domain::user_account::UserAccountId;

#[derive(Debug, Error)]
pub enum GroupChatRepositoryError {
  #[error("Failed to store the group chat: {0:?}")]
  StoreError(GroupChat, EventStoreWriteError),
  #[error("Failed to find the group chat by id: {0:?}")]
  FindByIdError(GroupChatId, EventStoreReadError),
}

/// グループチャットのリポジトリ。
#[async_trait::async_trait]
pub trait GroupChatRepository: Debug + Clone + Sync + Send + 'static {
  /// グループチャットのイベント及びスナップを保存する。
  ///
  /// # 引数
  /// - `event` - グループチャットのイベント
  /// - `version` - グループチャットのバージョン
  /// - `snapshot` - グループチャットのスナップショット
  ///
  /// # 戻り値
  /// - 成功した場合はOk, 失敗した場合はErrを返す。
  async fn store(&mut self, event: &GroupChatEvent, snapshot: &GroupChat) -> Result<(), GroupChatRepositoryError>;

  /// 指定したグループチャットIDに該当するグループチャットを取得する。
  ///
  /// # 引数
  /// - `id` - グループチャットID
  ///
  /// # 戻り値
  /// - 取得できた場合はOk(GroupChat), 取得できなかった場合はErrを返す。
  async fn find_by_id(&self, id: &GroupChatId) -> Result<Option<GroupChat>, GroupChatRepositoryError>;
}

#[derive(Debug, Clone, Error)]
pub enum GroupChatReadModelUpdateDaoError {
  #[error("Failed to insert group chat")]
  InsertGroupChatError,
  #[error("Failed to delete group chat")]
  DeleteGroupChatError,
  #[error("Failed to rename group chat")]
  RenameGroupChatError,
  #[error("Failed to insert member")]
  InsertMemberError,
  #[error("Failed to delete member")]
  DeleteMemberError,
  #[error("Failed to insert message")]
  InsertMessageError,
  #[error("Failed to update message")]
  UpdateMessageError,
  #[error("Failed to delete message")]
  DeleteMessageError,
}

/// グループチャットリードモデル更新用のデータアクセスオブジェクト。
///
/// NOTE: このデータアクセスオブジェクトはあくまで書き込み用です。読み込み用のデータアクセスオブジェクトはクエリ側に別途定義します。
#[async_trait::async_trait]
pub trait GroupChatReadModelUpdateDao {
  /// グループチャットリードモデルを作成します。
  async fn insert_group_chat(
    &self,
    aggregate_id: GroupChatId,
    name: GroupChatName,
    administrator_id: UserAccountId,
    created_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError>;
  /// グループチャットリードモデルを削除します。
  async fn delete_group_chat(
    &self,
    aggregate_id: GroupChatId,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError>;
  /// グループチャットリードモデルの名前を変更します。
  async fn rename_group_chat(
    &self,
    aggregate_id: GroupChatId,
    name: GroupChatName,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError>;
  /// メンバーリードモデルを追加します。
  async fn insert_member(
    &self,
    aggregate_id: GroupChatId,
    member_id: MemberId,
    account_id: UserAccountId,
    role: MemberRole,
    created_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError>;
  /// メンバーリードモデルを削除します。
  async fn delete_member(
    &self,
    aggregate_id: GroupChatId,
    account_id: UserAccountId,
  ) -> Result<(), GroupChatReadModelUpdateDaoError>;
  /// メッセージリードモデル追加します。
  async fn insert_message(
    &self,
    aggregate_id: GroupChatId,
    message: Message,
    created_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError>;

  async fn update_message(
    &self,
    aggregate_id: GroupChatId,
    message: Message,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError>;

  /// メッセージリードモデルを削除します。
  async fn delete_message(
    &self,
    message_id: MessageId,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError>;
}

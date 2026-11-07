use std::fmt::Debug;

use chrono::{DateTime, Utc};
use event_store_adapter_rs::EventStoreError;
use thiserror::Error;

use command_domain::group_chat::*;
use command_domain::user_account::UserAccountId;

#[derive(Debug, Error)]
pub enum GroupChatRepositoryError {
  #[error("Failed to store the group chat: {0:?}")]
  StoreError(GroupChat, EventStoreError),
  #[error("Failed to find the group chat by id: {0:?}")]
  FindByIdError(GroupChatId, EventStoreError),
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
  #[error("Failed to acquire a database connection")]
  AcquireConnectionError,
  #[error("Failed to begin a database transaction")]
  BeginTransactionError,
  #[error("Failed to commit a database transaction")]
  CommitTransactionError,
  #[error("Failed to read the read model position")]
  ReadPositionError,
  #[error("Failed to update the read model position")]
  UpdatePositionError,
  #[error("Expected seq_nr {expected} but received {actual}")]
  UnexpectedSeqNr { expected: u64, actual: u64 },
}

/// リードモデルに適用するイベント。
///
/// `seq_nr` と `occurred_at` はヘッドまたは journal の封筒が持つ値を使う。payload に
/// 埋め込まれた値は使わない。
#[derive(Debug, Clone)]
pub struct ReadModelEvent {
  seq_nr: u64,
  occurred_at: DateTime<Utc>,
  event: GroupChatEvent,
}

impl ReadModelEvent {
  pub fn new(seq_nr: u64, occurred_at: DateTime<Utc>, event: GroupChatEvent) -> Self {
    Self {
      seq_nr,
      occurred_at,
      event,
    }
  }

  /// イベントの `seq_nr` を返す。
  pub fn seq_nr(&self) -> u64 {
    self.seq_nr
  }

  /// イベントの発生時刻を返す。
  pub fn occurred_at(&self) -> &DateTime<Utc> {
    &self.occurred_at
  }

  /// イベント本体を返す。
  pub fn event(&self) -> &GroupChatEvent {
    &self.event
  }
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

  /// 集約の処理済み `seq_nr` を返します。まだ 1 件も適用していない場合は 0 を返します。
  async fn get_position(&self, aggregate_id: &GroupChatId) -> Result<u64, GroupChatReadModelUpdateDaoError>;

  /// イベントを適用し、処理位置を進めます。
  ///
  /// リードモデルの更新と `read_model_positions` の更新は 1 つのトランザクションで行います。
  /// トランザクション内で位置を読み直すため、位置以下の `seq_nr` のイベントは重複・古い
  /// イベントとして捨てます。`events` は `seq_nr` の昇順で、現在の位置から連続している
  /// 必要があります。
  async fn apply_events(
    &self,
    aggregate_id: &GroupChatId,
    events: &[ReadModelEvent],
  ) -> Result<(), GroupChatReadModelUpdateDaoError>;
}

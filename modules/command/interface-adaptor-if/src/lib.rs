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
  /// The storage reported a failure that may succeed when retried, such as a
  /// connection loss, a deadlock, or a lock wait timeout.
  #[error("Transient read model update failure: {0}")]
  Transient(String),
  /// The storage reported a failure that the same event cannot fix, such as a
  /// constraint violation or an oversized value.
  #[error("Permanent read model update failure: {0}")]
  Permanent(String),
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
pub trait GroupChatReadModelUpdateDao: Send + Sync {
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

/// A durable record of one event that the read model updater could not apply.
///
/// Only identifiers and the failure classification are stored. The event body
/// is always re-read from the journal, so repairing the payload does not leave
/// a stale copy in the task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryTask {
  pub store_id: String,
  pub projection_name: String,
  pub projection_version: String,
  pub aggregate_id: String,
  pub seq_nr: u64,
  pub failure_type: String,
  pub error_message: String,
  pub status: String,
}

#[derive(Debug, Clone, Error)]
pub enum RecoveryTaskStoreError {
  #[error("Recovery task store failed: {0}")]
  Other(String),
}

/// Persists recovery tasks. Recording is idempotent for the same
/// (store, projection, aggregate, seq_nr) key.
#[async_trait::async_trait]
pub trait RecoveryTaskStore: Send + Sync {
  async fn record(&self, task: &RecoveryTask) -> Result<(), RecoveryTaskStoreError>;
  async fn list_pending(&self) -> Result<Vec<RecoveryTask>, RecoveryTaskStoreError>;
  async fn mark_completed(
    &self,
    store_id: &str,
    projection_name: &str,
    aggregate_id: &str,
    seq_nr: u64,
  ) -> Result<(), RecoveryTaskStoreError>;
}

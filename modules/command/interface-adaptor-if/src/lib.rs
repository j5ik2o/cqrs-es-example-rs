use std::fmt::Debug;

use anyhow::Result;
use chrono::{DateTime, Utc};

use command_domain::group_chat::*;
use command_domain::user_account::UserAccountId;

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
  async fn store(&mut self, event: &GroupChatEvent, version: usize, snapshot: Option<&GroupChat>) -> Result<()>;

  /// 指定したグループチャットIDに該当するグループチャットを取得する。
  ///
  /// # 引数
  /// - `id` - グループチャットID
  ///
  /// # 戻り値
  /// - 取得できた場合はOk(GroupChat), 取得できなかった場合はErrを返す。
  async fn find_by_id(&self, id: &GroupChatId) -> Result<GroupChat>;
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
  ) -> Result<()>;
  /// グループチャットリードモデルを削除します。
  async fn delete_group_chat(&self, aggregate_id: GroupChatId) -> Result<()>;
  /// グループチャットリードモデルの名前を変更します。
  async fn rename_group_chat(&self, aggregate_id: GroupChatId, name: GroupChatName) -> Result<()>;
  /// メンバーリードモデルを追加します。
  ///
  /// TODO: 任意課題 このメソッドはMemberReadModelUpdateDaoを新設して移動する。
  async fn insert_member(
    &self,
    aggregate_id: GroupChatId,
    member_id: MemberId,
    account_id: UserAccountId,
    role: MemberRole,
    created_at: DateTime<Utc>,
  ) -> Result<()>;
  /// メンバーリードモデルを削除します。
  ///
  /// TODO: 任意課題 このメソッドはMemberReadModelUpdateDaoを新設して移動する。
  async fn delete_member(&self, aggregate_id: GroupChatId, account_id: UserAccountId) -> Result<()>;
  /// メッセージリードモデル追加します。
  ///
  /// TODO: 任意課題 このメソッドはMessageReadModelUpdateDaoを新設して移動する。
  async fn insert_message(&self, aggregate_id: GroupChatId, message: Message, created_at: DateTime<Utc>) -> Result<()>;
  /// メッセージリードモデルを削除します。
  ///
  /// TODO: 任意課題 このメソッドはMessageReadModelUpdateDaoを新設して移動する。
  async fn delete_message(&self, aggregate_id: GroupChatId, message_id: MessageId) -> Result<()>;
}

pub trait GroupChatPresenter {
  fn present(&mut self, group_chat_event: GroupChatEvent);
}

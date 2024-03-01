use std::fmt::Debug;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;

use command_domain::group_chat::MemberId;
use command_domain::group_chat::{GroupChatId, GroupChatName, MemberRole, Message, MessageId};
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::GroupChatReadModelUpdateDao;

#[derive(Debug)]
pub struct GroupChatReadModelUpdateDaoImpl {
  pool: MySqlPool,
}

impl GroupChatReadModelUpdateDaoImpl {
  pub fn new(pool: MySqlPool) -> Self {
    Self { pool }
  }
}

#[async_trait::async_trait]
impl GroupChatReadModelUpdateDao for GroupChatReadModelUpdateDaoImpl {
  async fn insert_group_chat(
    &self,
    aggregate_id: GroupChatId,
    name: GroupChatName,
    administrator_id: UserAccountId,
    created_at: DateTime<Utc>,
  ) -> Result<()> {
    // NOTE: 今回の実装ではseq_nrが照合は行っていません。興味があれば実装してみてください。
    // イベントのseq_nrをリードモデルに保存しておくと、後に発生するUPDATE, DELETE時に不整合を検知できる
    // イベントが発生するたびに、group_chats#seq_nrを更新しておき
    // GroupChatDeletedが発生したときに、当該イベントのseq_nrを取得し、group_chats#seq_nrと比較する
    // DELETE FROM group_chats WHERE id = ? AND seq_nr = (group_chat_deleted.seq_nr - 1)
    // のようなクエリを実行して更新件数が0件だった場合は、発生したイベントもしくはリードモデルの状態に不整合が発生した判断できる。お
    // 不整合が発生した場合はシステムは続行できないので、データが破壊される前にプログラムを即時終了し、障害扱いとする
    sqlx::query!(
      "INSERT INTO group_chats (id, disabled, name, owner_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
      aggregate_id.to_string(),
      false,
      name.to_string(),
      administrator_id.to_string(),
      created_at.clone(),
      created_at.clone(),
    )
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn delete_group_chat(&self, aggregate_id: GroupChatId, updated_at: DateTime<Utc>) -> Result<()> {
    // NOTE: 現状は物理削除になっている。論理削除変えたい場合はstatusフラグを導入しUPDATEに変更する。
    // もう一つの方法は履歴テーブルを作り、そちらに移動させる方法もある。
    sqlx::query!(
      "UPDATE group_chats SET disabled = ?, updated_at = ? WHERE id = ?",
      true,
      updated_at.clone(),
      aggregate_id.to_string()
    )
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn rename_group_chat(
    &self,
    aggregate_id: GroupChatId,
    name: GroupChatName,
    updated_at: DateTime<Utc>,
  ) -> Result<()> {
    sqlx::query!(
      "UPDATE group_chats SET name = ?, updated_at = ? WHERE id = ?",
      name.to_string(),
      updated_at.clone(),
      aggregate_id.to_string()
    )
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn insert_member(
    &self,
    aggregate_id: GroupChatId,
    member_id: MemberId,
    account_id: UserAccountId,
    role: MemberRole,
    created_at: DateTime<Utc>,
  ) -> Result<()> {
    sqlx::query!(
      "INSERT INTO members (id, group_chat_id, user_account_id, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
      member_id.to_string(),
      aggregate_id.to_string(),
      account_id.to_string(),
      role.to_string().to_lowercase(),
      created_at.clone(),
      created_at.clone()
    )
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn delete_member(&self, aggregate_id: GroupChatId, account_id: UserAccountId) -> Result<()> {
    // NOTE: 現状は物理削除になっている。論理削除変えたい場合はstatusフラグを導入しUPDATEに変更する。
    // もう一つの方法は履歴テーブルを作り、そちらに移動させる方法もある。
    sqlx::query!(
      "DELETE FROM members WHERE id = ? AND group_chat_id = ?",
      account_id.to_string(),
      aggregate_id.to_string()
    )
    .execute(&self.pool)
    .await?;
    Ok(())
  }

  async fn insert_message(&self, aggregate_id: GroupChatId, message: Message, created_at: DateTime<Utc>) -> Result<()> {
    sqlx::query!(
      "INSERT INTO messages (id, disabled, group_chat_id, user_account_id, text, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
      message.breach_encapsulation_of_id().to_string(),
      false,
      aggregate_id.to_string(),
      message.breach_encapsulation_of_sender_id().to_string(),
      message.breach_encapsulation_of_text(),
      created_at.clone(),
      created_at.clone()
    )
    .execute(&self.pool)
    .await?;
    Ok(())
  }

  async fn delete_message(&self, message_id: MessageId, updated_at: DateTime<Utc>) -> Result<()> {
    // NOTE: 現状は物理削除になっている。論理削除変えたい場合はstatusフラグを導入しUPDATEに変更する。
    // もう一つの方法は履歴テーブルを作り、そちらに移動させる方法もある。
    sqlx::query!(
      "UPDATE messages SET disabled = ?, updated_at = ? WHERE id = ?",
      true,
      updated_at.clone(),
      message_id.to_string()
    )
    .execute(&self.pool)
    .await?;
    Ok(())
  }
}

#[derive(Debug)]
pub struct MockGroupChatReadModelUpdateDao;

#[async_trait::async_trait]
impl GroupChatReadModelUpdateDao for MockGroupChatReadModelUpdateDao {
  async fn insert_group_chat(
    &self,
    _aggregate_id: GroupChatId,
    _name: GroupChatName,
    _administrator_id: UserAccountId,
    _created_at: DateTime<Utc>,
  ) -> Result<()> {
    Ok(())
  }

  async fn delete_group_chat(&self, _aggregate_id: GroupChatId, _: DateTime<Utc>) -> Result<()> {
    Ok(())
  }

  async fn rename_group_chat(&self, _aggregate_id: GroupChatId, _name: GroupChatName, _: DateTime<Utc>) -> Result<()> {
    Ok(())
  }

  async fn insert_member(
    &self,
    _aggregate_id: GroupChatId,
    _member_id: MemberId,
    _account_id: UserAccountId,
    _role: MemberRole,
    _created_at: DateTime<Utc>,
  ) -> Result<()> {
    Ok(())
  }

  async fn delete_member(&self, _aggregate_id: GroupChatId, _account_id: UserAccountId) -> Result<()> {
    Ok(())
  }

  async fn insert_message(
    &self,
    _aggregate_id: GroupChatId,
    _message: Message,
    _created_at: DateTime<Utc>,
  ) -> Result<()> {
    Ok(())
  }

  async fn delete_message(&self, _message_id: MessageId, _: DateTime<Utc>) -> Result<()> {
    Ok(())
  }
}

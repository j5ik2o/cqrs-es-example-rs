use std::fmt::Debug;

use chrono::{DateTime, Utc};
use sqlx::MySqlPool;

use command_domain::group_chat::MemberId;
use command_domain::group_chat::{GroupChatId, GroupChatName, MemberRole, Message, MessageId};
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::{GroupChatReadModelUpdateDao, GroupChatReadModelUpdateDaoError};

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
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    // NOTE: 今回の実装ではseq_nrが照合は行っていません。興味があれば実装してみてください。
    // イベントのseq_nrをリードモデルに保存しておくと、後に発生するUPDATE, DELETE時に不整合を検知できる
    // イベントが発生するたびに、group_chats#seq_nrを更新しておき
    // GroupChatDeletedが発生したときに、当該イベントのseq_nrを取得し、group_chats#seq_nrと比較する
    // DELETE FROM group_chats WHERE id = ? AND seq_nr = (group_chat_deleted.seq_nr - 1)
    // のようなクエリを実行して更新件数が0件だった場合は、発生したイベントもしくはリードモデルの状態に不整合が発生した判断できる。お
    // 不整合が発生した場合はシステムは続行できないので、データが破壊される前にプログラムを即時終了し、障害扱いとする
    let result = sqlx::query!(
      "INSERT INTO group_chats (id, disabled, name, owner_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
      aggregate_id.to_string(),
      false,
      name.to_string(),
      administrator_id.to_string(),
      created_at.clone(),
      created_at.clone(),
    )
    .execute(&self.pool)
    .await;

    match result {
      Ok(_) => Ok(()),
      Err(e) => {
        log::error!("Failed to insert group chat: {:?}", e);
        Err(GroupChatReadModelUpdateDaoError::InsertGroupChatError)
      }
    }
  }

  async fn delete_group_chat(
    &self,
    aggregate_id: GroupChatId,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    // NOTE: 現状は物理削除になっている。論理削除変えたい場合はstatusフラグを導入しUPDATEに変更する。
    // もう一つの方法は履歴テーブルを作り、そちらに移動させる方法もある。
    let result = sqlx::query!(
      "UPDATE group_chats SET disabled = ?, updated_at = ? WHERE id = ?",
      true,
      updated_at.clone(),
      aggregate_id.to_string()
    )
    .execute(&self.pool)
    .await;

    match result {
      Ok(_) => Ok(()),
      Err(e) => {
        log::error!("Failed to delete group chat: {:?}", e);
        Err(GroupChatReadModelUpdateDaoError::DeleteGroupChatError)
      }
    }
  }

  async fn rename_group_chat(
    &self,
    aggregate_id: GroupChatId,
    name: GroupChatName,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    let result = sqlx::query!(
      "UPDATE group_chats SET name = ?, updated_at = ? WHERE id = ?",
      name.to_string(),
      updated_at.clone(),
      aggregate_id.to_string()
    )
    .execute(&self.pool)
    .await;

    match result {
      Ok(_) => Ok(()),
      Err(e) => {
        log::error!("Failed to rename group chat: {:?}", e);
        Err(GroupChatReadModelUpdateDaoError::RenameGroupChatError)
      }
    }
  }

  async fn insert_member(
    &self,
    aggregate_id: GroupChatId,
    member_id: MemberId,
    account_id: UserAccountId,
    role: MemberRole,
    created_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    let result = sqlx::query!(
      "INSERT INTO members (id, group_chat_id, user_account_id, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
      member_id.to_string(),
      aggregate_id.to_string(),
      account_id.to_string(),
      role.to_string().to_lowercase(),
      created_at.clone(),
      created_at.clone()
    )
    .execute(&self.pool)
    .await;

    match result {
      Ok(_) => Ok(()),
      Err(e) => {
        log::error!("Failed to insert member: {:?}", e);
        Err(GroupChatReadModelUpdateDaoError::InsertMemberError)
      }
    }
  }

  async fn delete_member(
    &self,
    aggregate_id: GroupChatId,
    account_id: UserAccountId,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    // NOTE: 現状は物理削除になっている。論理削除変えたい場合はstatusフラグを導入しUPDATEに変更する。
    // もう一つの方法は履歴テーブルを作り、そちらに移動させる方法もある。
    let result = sqlx::query!(
      "DELETE FROM members WHERE id = ? AND group_chat_id = ?",
      account_id.to_string(),
      aggregate_id.to_string()
    )
    .execute(&self.pool)
    .await;

    match result {
      Ok(_) => Ok(()),
      Err(e) => {
        log::error!("Failed to delete member: {:?}", e);
        Err(GroupChatReadModelUpdateDaoError::DeleteMemberError)
      }
    }
  }

  async fn insert_message(
    &self,
    aggregate_id: GroupChatId,
    message: Message,
    created_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    let result = sqlx::query!(
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
    .await;

    match result {
      Ok(_) => Ok(()),
      Err(e) => {
        log::error!("Failed to insert message: {:?}", e);
        Err(GroupChatReadModelUpdateDaoError::InsertMessageError)
      }
    }
  }

  async fn update_message(
    &self,
    aggregate_id: GroupChatId,
    message: Message,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    let result = sqlx::query!(
      "UPDATE messages SET text = ?, updated_at = ? WHERE id = ?",
      message.breach_encapsulation_of_text(),
      updated_at.clone(),
      message.breach_encapsulation_of_id().to_string()
    )
    .execute(&self.pool)
    .await;
    match result {
      Ok(_) => Ok(()),
      Err(e) => {
        log::error!("Failed to update message: {:?}", e);
        Err(GroupChatReadModelUpdateDaoError::UpdateMessageError)
      }
    }
  }

  async fn delete_message(
    &self,
    message_id: MessageId,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    // NOTE: 現状は物理削除になっている。論理削除変えたい場合はstatusフラグを導入しUPDATEに変更する。
    // もう一つの方法は履歴テーブルを作り、そちらに移動させる方法もある。
    let result = sqlx::query!(
      "UPDATE messages SET disabled = ?, updated_at = ? WHERE id = ?",
      true,
      updated_at.clone(),
      message_id.to_string()
    )
    .execute(&self.pool)
    .await;
    match result {
      Ok(_) => Ok(()),
      Err(e) => {
        log::error!("Failed to delete message: {:?}", e);
        Err(GroupChatReadModelUpdateDaoError::DeleteMessageError)
      }
    }
  }
}

#[derive(Debug)]
pub struct MockGroupChatReadModelUpdateDao;

#[async_trait::async_trait]
impl GroupChatReadModelUpdateDao for MockGroupChatReadModelUpdateDao {
  async fn insert_group_chat(
    &self,
    _: GroupChatId,
    _: GroupChatName,
    _: UserAccountId,
    _: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn delete_group_chat(&self, _: GroupChatId, _: DateTime<Utc>) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn rename_group_chat(
    &self,
    _: GroupChatId,
    _: GroupChatName,
    _: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn insert_member(
    &self,
    _: GroupChatId,
    _: MemberId,
    _: UserAccountId,
    _: MemberRole,
    _: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn delete_member(&self, _: GroupChatId, _: UserAccountId) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn insert_message(
    &self,
    _: GroupChatId,
    _: Message,
    _created_a: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn update_message(
    &self,
    _: GroupChatId,
    _: Message,
    _: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn delete_message(&self, _: MessageId, _: DateTime<Utc>) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }
}

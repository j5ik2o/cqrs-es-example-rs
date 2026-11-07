use std::fmt::Debug;

use chrono::{DateTime, Utc};
use sqlx::{MySqlConnection, MySqlPool};

use command_domain::group_chat::MemberId;
use command_domain::group_chat::{GroupChatEvent, GroupChatId, GroupChatName, MemberRole, Message, MessageId};
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::{GroupChatReadModelUpdateDao, GroupChatReadModelUpdateDaoError, ReadModelEvent};

#[derive(Debug)]
pub struct GroupChatReadModelUpdateDaoImpl {
  pool: MySqlPool,
}

impl GroupChatReadModelUpdateDaoImpl {
  pub fn new(pool: MySqlPool) -> Self {
    Self { pool }
  }

  async fn insert_group_chat_sql(
    conn: &mut MySqlConnection,
    aggregate_id: GroupChatId,
    name: GroupChatName,
    administrator_id: UserAccountId,
    created_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    sqlx::query!(
      "INSERT INTO group_chats (id, disabled, name, owner_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
      aggregate_id.to_string(),
      false,
      name.to_string(),
      administrator_id.to_string(),
      created_at.clone(),
      created_at.clone(),
    )
    .execute(&mut *conn)
    .await
    .map(|_| ())
    .map_err(|e| {
      log::error!("Failed to insert group chat: {:?}", e);
      map_sqlx_error(e, GroupChatReadModelUpdateDaoError::InsertGroupChatError)
    })
  }

  async fn delete_group_chat_sql(
    conn: &mut MySqlConnection,
    aggregate_id: GroupChatId,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    sqlx::query!(
      "UPDATE group_chats SET disabled = ?, updated_at = ? WHERE id = ?",
      true,
      updated_at.clone(),
      aggregate_id.to_string()
    )
    .execute(&mut *conn)
    .await
    .map(|_| ())
    .map_err(|e| {
      log::error!("Failed to delete group chat: {:?}", e);
      map_sqlx_error(e, GroupChatReadModelUpdateDaoError::DeleteGroupChatError)
    })
  }

  async fn rename_group_chat_sql(
    conn: &mut MySqlConnection,
    aggregate_id: GroupChatId,
    name: GroupChatName,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    sqlx::query!(
      "UPDATE group_chats SET name = ?, updated_at = ? WHERE id = ?",
      name.to_string(),
      updated_at.clone(),
      aggregate_id.to_string()
    )
    .execute(&mut *conn)
    .await
    .map(|_| ())
    .map_err(|e| {
      log::error!("Failed to rename group chat: {:?}", e);
      map_sqlx_error(e, GroupChatReadModelUpdateDaoError::RenameGroupChatError)
    })
  }

  async fn insert_member_sql(
    conn: &mut MySqlConnection,
    aggregate_id: GroupChatId,
    member_id: MemberId,
    account_id: UserAccountId,
    role: MemberRole,
    created_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    sqlx::query!(
      "INSERT INTO members (id, group_chat_id, user_account_id, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
      member_id.to_string(),
      aggregate_id.to_string(),
      account_id.to_string(),
      role.to_string().to_lowercase(),
      created_at.clone(),
      created_at.clone()
    )
    .execute(&mut *conn)
    .await
    .map(|_| ())
    .map_err(|e| {
      log::error!("Failed to insert member: {:?}", e);
      map_sqlx_error(e, GroupChatReadModelUpdateDaoError::InsertMemberError)
    })
  }

  async fn delete_member_sql(
    conn: &mut MySqlConnection,
    aggregate_id: GroupChatId,
    account_id: UserAccountId,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    sqlx::query!(
      "DELETE FROM members WHERE group_chat_id = ? AND user_account_id = ?",
      aggregate_id.to_string(),
      account_id.to_string()
    )
    .execute(&mut *conn)
    .await
    .map(|_| ())
    .map_err(|e| {
      log::error!("Failed to delete member: {:?}", e);
      map_sqlx_error(e, GroupChatReadModelUpdateDaoError::DeleteMemberError)
    })
  }

  async fn insert_message_sql(
    conn: &mut MySqlConnection,
    aggregate_id: GroupChatId,
    message: Message,
    created_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
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
    .execute(&mut *conn)
    .await
    .map(|_| ())
    .map_err(|e| {
      log::error!("Failed to insert message: {:?}", e);
      map_sqlx_error(e, GroupChatReadModelUpdateDaoError::InsertMessageError)
    })
  }

  async fn update_message_sql(
    conn: &mut MySqlConnection,
    message: Message,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    sqlx::query!(
      "UPDATE messages SET text = ?, updated_at = ? WHERE id = ?",
      message.breach_encapsulation_of_text(),
      updated_at.clone(),
      message.breach_encapsulation_of_id().to_string()
    )
    .execute(&mut *conn)
    .await
    .map(|_| ())
    .map_err(|e| {
      log::error!("Failed to update message: {:?}", e);
      map_sqlx_error(e, GroupChatReadModelUpdateDaoError::UpdateMessageError)
    })
  }

  async fn delete_message_sql(
    conn: &mut MySqlConnection,
    message_id: MessageId,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    sqlx::query!(
      "UPDATE messages SET disabled = ?, updated_at = ? WHERE id = ?",
      true,
      updated_at.clone(),
      message_id.to_string()
    )
    .execute(&mut *conn)
    .await
    .map(|_| ())
    .map_err(|e| {
      log::error!("Failed to delete message: {:?}", e);
      map_sqlx_error(e, GroupChatReadModelUpdateDaoError::DeleteMessageError)
    })
  }

  async fn apply_event_sql(
    conn: &mut MySqlConnection,
    aggregate_id: &GroupChatId,
    event: &ReadModelEvent,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    let occurred_at = *event.occurred_at();
    match event.event() {
      GroupChatEvent::GroupChatCreated(body) => {
        let administrator_id = body.members.administrator_id();
        Self::insert_group_chat_sql(
          conn,
          aggregate_id.clone(),
          body.name.clone(),
          administrator_id.breach_encapsulation_of_user_account_id().clone(),
          occurred_at,
        )
        .await?;
        Self::insert_member_sql(
          conn,
          aggregate_id.clone(),
          administrator_id.breach_encapsulation_of_id().clone(),
          administrator_id.breach_encapsulation_of_user_account_id().clone(),
          MemberRole::Admin,
          occurred_at,
        )
        .await
      }
      GroupChatEvent::GroupChatDeleted(_) => Self::delete_group_chat_sql(conn, aggregate_id.clone(), occurred_at).await,
      GroupChatEvent::GroupChatRenamed(body) => {
        Self::rename_group_chat_sql(conn, aggregate_id.clone(), body.name.clone(), occurred_at).await
      }
      GroupChatEvent::GroupChatMemberAdded(body) => {
        Self::insert_member_sql(
          conn,
          aggregate_id.clone(),
          body.member.breach_encapsulation_of_id().clone(),
          body.member.breach_encapsulation_of_user_account_id().clone(),
          body.member.breach_encapsulation_of_role().clone(),
          occurred_at,
        )
        .await
      }
      GroupChatEvent::GroupChatMemberRemoved(body) => {
        Self::delete_member_sql(conn, aggregate_id.clone(), body.user_account_id.clone()).await
      }
      GroupChatEvent::GroupChatMessagePosted(body) => {
        Self::insert_message_sql(conn, aggregate_id.clone(), body.message.clone(), occurred_at).await
      }
      GroupChatEvent::GroupChatMessageEdited(body) => {
        Self::update_message_sql(conn, body.message.clone(), occurred_at).await
      }
      GroupChatEvent::GroupChatMessageDeleted(body) => {
        Self::delete_message_sql(conn, body.message_id.clone(), occurred_at).await
      }
    }
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
    let mut conn = self
      .pool
      .acquire()
      .await
      .map_err(|_| GroupChatReadModelUpdateDaoError::AcquireConnectionError)?;
    Self::insert_group_chat_sql(&mut conn, aggregate_id, name, administrator_id, created_at).await
  }

  async fn delete_group_chat(
    &self,
    aggregate_id: GroupChatId,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    let mut conn = self
      .pool
      .acquire()
      .await
      .map_err(|_| GroupChatReadModelUpdateDaoError::AcquireConnectionError)?;
    Self::delete_group_chat_sql(&mut conn, aggregate_id, updated_at).await
  }

  async fn rename_group_chat(
    &self,
    aggregate_id: GroupChatId,
    name: GroupChatName,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    let mut conn = self
      .pool
      .acquire()
      .await
      .map_err(|_| GroupChatReadModelUpdateDaoError::AcquireConnectionError)?;
    Self::rename_group_chat_sql(&mut conn, aggregate_id, name, updated_at).await
  }

  async fn insert_member(
    &self,
    aggregate_id: GroupChatId,
    member_id: MemberId,
    account_id: UserAccountId,
    role: MemberRole,
    created_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    let mut conn = self
      .pool
      .acquire()
      .await
      .map_err(|_| GroupChatReadModelUpdateDaoError::AcquireConnectionError)?;
    Self::insert_member_sql(&mut conn, aggregate_id, member_id, account_id, role, created_at).await
  }

  async fn delete_member(
    &self,
    aggregate_id: GroupChatId,
    account_id: UserAccountId,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    let mut conn = self
      .pool
      .acquire()
      .await
      .map_err(|_| GroupChatReadModelUpdateDaoError::AcquireConnectionError)?;
    Self::delete_member_sql(&mut conn, aggregate_id, account_id).await
  }

  async fn insert_message(
    &self,
    aggregate_id: GroupChatId,
    message: Message,
    created_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    let mut conn = self
      .pool
      .acquire()
      .await
      .map_err(|_| GroupChatReadModelUpdateDaoError::AcquireConnectionError)?;
    Self::insert_message_sql(&mut conn, aggregate_id, message, created_at).await
  }

  async fn update_message(
    &self,
    _aggregate_id: GroupChatId,
    message: Message,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    let mut conn = self
      .pool
      .acquire()
      .await
      .map_err(|_| GroupChatReadModelUpdateDaoError::AcquireConnectionError)?;
    Self::update_message_sql(&mut conn, message, updated_at).await
  }

  async fn delete_message(
    &self,
    message_id: MessageId,
    updated_at: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    let mut conn = self
      .pool
      .acquire()
      .await
      .map_err(|_| GroupChatReadModelUpdateDaoError::AcquireConnectionError)?;
    Self::delete_message_sql(&mut conn, message_id, updated_at).await
  }

  async fn get_position(&self, aggregate_id: &GroupChatId) -> Result<u64, GroupChatReadModelUpdateDaoError> {
    let position = sqlx::query_scalar!(
      "SELECT seq_nr FROM read_model_positions WHERE aggregate_id = ?",
      aggregate_id.to_string()
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| {
      log::error!("Failed to read the read model position: {:?}", e);
      GroupChatReadModelUpdateDaoError::ReadPositionError
    })?;
    Ok(position.map(|value| value as u64).unwrap_or(0))
  }

  async fn apply_events(
    &self,
    aggregate_id: &GroupChatId,
    events: &[ReadModelEvent],
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    if events.is_empty() {
      return Ok(());
    }
    let mut tx = self.pool.begin().await.map_err(|e| {
      log::error!("Failed to begin a transaction: {:?}", e);
      GroupChatReadModelUpdateDaoError::BeginTransactionError
    })?;
    let updated_at = Utc::now();
    // 位置の行が無い状態で SELECT ... FOR UPDATE するとギャップロックになり、
    // 同じギャップに入る新しい集約どうしがデッドロックしうる。先に行を作る。
    sqlx::query!(
      "INSERT IGNORE INTO read_model_positions (aggregate_id, seq_nr, updated_at) VALUES (?, ?, ?)",
      aggregate_id.to_string(),
      0_i64,
      updated_at
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
      log::error!("Failed to create the read model position: {:?}", e);
      GroupChatReadModelUpdateDaoError::ReadPositionError
    })?;
    let current: i64 = sqlx::query_scalar!(
      "SELECT seq_nr FROM read_model_positions WHERE aggregate_id = ? FOR UPDATE",
      aggregate_id.to_string()
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
      log::error!("Failed to lock the read model position: {:?}", e);
      GroupChatReadModelUpdateDaoError::ReadPositionError
    })?;
    let mut position = current as u64;
    for event in events {
      if event.seq_nr() <= position {
        log::debug!(
          "skip duplicate or stale event: aggregate_id = {}, seq_nr = {}, position = {}",
          aggregate_id,
          event.seq_nr(),
          position
        );
        continue;
      }
      if event.seq_nr() != position + 1 {
        return Err(GroupChatReadModelUpdateDaoError::UnexpectedSeqNr {
          expected: position + 1,
          actual: event.seq_nr(),
        });
      }
      Self::apply_event_sql(&mut tx, aggregate_id, event).await?;
      position = event.seq_nr();
    }
    sqlx::query!(
      "UPDATE read_model_positions SET seq_nr = ?, updated_at = ? WHERE aggregate_id = ?",
      position as i64,
      updated_at,
      aggregate_id.to_string()
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
      log::error!("Failed to update the read model position: {:?}", e);
      GroupChatReadModelUpdateDaoError::UpdatePositionError
    })?;
    tx.commit().await.map_err(|e| {
      log::error!("Failed to commit a transaction: {:?}", e);
      GroupChatReadModelUpdateDaoError::CommitTransactionError
    })
  }
}

/// Keeps the operation-specific variant for permanent data errors, and marks
/// connection, locking, and throttling failures as retryable.
fn map_sqlx_error(error: sqlx::Error, permanent: GroupChatReadModelUpdateDaoError) -> GroupChatReadModelUpdateDaoError {
  if is_transient_sqlx_error(&error) {
    GroupChatReadModelUpdateDaoError::Transient(error.to_string())
  } else {
    permanent
  }
}

fn is_transient_sqlx_error(error: &sqlx::Error) -> bool {
  match error {
    sqlx::Error::Database(database_error) => {
      // `code()` is the SQLSTATE (for example 23000). The MySQL error number
      // tells a retryable connection/locking failure apart from a constraint or
      // schema problem, which the same event cannot fix.
      match database_error
        .as_error()
        .downcast_ref::<sqlx::mysql::MySqlDatabaseError>()
      {
        Some(mysql_error) => matches!(
          mysql_error.number(),
          1040 | 1205 | 1213 | 1317 | 1927 | 2002 | 2003 | 2006 | 2013
        ),
        None => false,
      }
    }
    sqlx::Error::RowNotFound
    | sqlx::Error::ColumnNotFound(_)
    | sqlx::Error::ColumnIndexOutOfBounds { .. }
    | sqlx::Error::ColumnDecode { .. }
    | sqlx::Error::TypeNotFound { .. }
    | sqlx::Error::Encode(_)
    | sqlx::Error::Decode(_) => false,
    _ => true,
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

  async fn get_position(&self, _: &GroupChatId) -> Result<u64, GroupChatReadModelUpdateDaoError> {
    Ok(0)
  }

  async fn apply_events(&self, _: &GroupChatId, _: &[ReadModelEvent]) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }
}

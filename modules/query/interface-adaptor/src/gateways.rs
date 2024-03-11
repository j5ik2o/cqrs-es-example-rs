use async_graphql::async_trait::async_trait;
use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use sqlx::MySqlPool;
use thiserror::Error;

/// グループチャットリードモデル
///
/// NOTE: リードモデルはDTOとして利用されるものです。
/// 特段振る舞いのようなものはありません。
#[derive(SimpleObject)]
pub struct GroupChat {
  /// グループチャットID
  id: String,
  /// グループチャット名
  name: String,
  /// 管理者ID
  owner_id: String,
  /// 作成日時
  created_at: NaiveDateTime,
  /// 更新日時
  updated_at: NaiveDateTime,
}

#[derive(Debug, Error)]
pub enum GroupChatDaoError {
  #[error("GroupChat not found.: {0}")]
  NotFoundError(String),
  #[error("OtherError: {0}")]
  OtherError(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum MemberDaoError {
  #[error("Member not found.: {0}")]
  NotFoundError(String),
  #[error("OtherError: {0}")]
  OtherError(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum MessageDaoError {
  #[error("Message not found.: {0}")]
  NotFoundError(String),
  #[error("OtherError: {0}")]
  OtherError(#[from] sqlx::Error),
}

impl GroupChat {
  /// コンストラクタ
  pub fn new(id: String, name: String, owner_id: String, created_at: NaiveDateTime, updated_at: NaiveDateTime) -> Self {
    Self {
      id,
      name,
      owner_id,
      created_at,
      updated_at,
    }
  }
}

/// グループチャット用データアクセスオブジェクト。
///
/// グループチャットを取得するためのインターフェース
#[async_trait]
pub trait GroupChatDao: Send + Sync {
  async fn get_group_chat(
    &self,
    group_chat_id: String,
    user_account_id: String,
  ) -> Result<Option<GroupChat>, GroupChatDaoError>;
  async fn get_group_chats(&self, user_account_id: String) -> Result<Vec<GroupChat>, GroupChatDaoError>;
}

/// [GroupChatDao]の実装
pub struct GroupChatDaoImpl {
  my_sql_pool: MySqlPool,
}

impl GroupChatDaoImpl {
  pub fn new(my_sql_pool: MySqlPool) -> Self {
    Self { my_sql_pool }
  }
}

#[async_trait]
impl GroupChatDao for GroupChatDaoImpl {
  async fn get_group_chat(
    &self,
    group_chat_id: String,
    user_account_id: String,
  ) -> Result<Option<GroupChat>, GroupChatDaoError> {
    sqlx::query_as!(
      GroupChat,
      r#"SELECT gc.id, gc.name, gc.owner_id, gc.created_at, gc.updated_at
		 FROM group_chats AS gc JOIN members AS m ON gc.id = m.group_chat_id
		 WHERE gc.disabled = 'false' AND m.group_chat_id = ? AND m.user_account_id = ?"#,
      group_chat_id.clone(),
      user_account_id.clone()
    )
    .fetch_optional(&self.my_sql_pool)
    .await
    .map_err(|e| GroupChatDaoError::OtherError(e))
  }

  async fn get_group_chats(&self, user_account_id: String) -> Result<Vec<GroupChat>, GroupChatDaoError> {
    sqlx::query_as!(
      GroupChat,
      r#"SELECT gc.id, gc.name, gc.owner_id, gc.created_at, gc.updated_at
		 FROM group_chats AS gc JOIN members AS m ON gc.id = m.group_chat_id
         WHERE gc.disabled = 'false' AND m.user_account_id = ?"#,
      user_account_id.clone()
    )
    .fetch_all(&self.my_sql_pool)
    .await
    .map_err(|e| GroupChatDaoError::OtherError(e))
  }
}

// ---

/// メンバーリードモデル
#[derive(SimpleObject)]
pub struct Member {
  /// メンバーID
  id: String,
  /// グループチャットID
  group_chat_id: String,
  /// アカウントID
  user_account_id: String,
  /// ロール
  role: String,
  /// 作成日時
  created_at: NaiveDateTime,
  /// 更新日時
  updated_at: NaiveDateTime,
}

impl Member {
  pub fn new(
    id: String,
    group_chat_id: String,
    user_account_id: String,
    role: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
  ) -> Self {
    Self {
      id,
      group_chat_id,
      user_account_id,
      role,
      created_at,
      updated_at,
    }
  }
}

/// メンバー用データアクセスオブジェクト。
///
/// メンバーを取得するためのインターフェース
#[async_trait]
pub trait MemberDao: Send + Sync {
  async fn get_member(&self, group_chat_id: String, user_account_id: String) -> Result<Option<Member>, MemberDaoError>;
  async fn get_members(&self, group_chat_id: String, user_account_id: String) -> Result<Vec<Member>, MemberDaoError>;
}

/// [MemberDao]の実装
pub struct MemberDaoImpl {
  my_sql_pool: MySqlPool,
}

impl MemberDaoImpl {
  pub fn new(my_sql_pool: MySqlPool) -> Self {
    Self { my_sql_pool }
  }
}

#[async_trait]
impl MemberDao for MemberDaoImpl {
  async fn get_member(&self, group_chat_id: String, user_account_id: String) -> Result<Option<Member>, MemberDaoError> {
    sqlx::query_as!(
      Member,
      r#"SELECT m.id, m.group_chat_id, m.user_account_id, m.role, m.created_at, m.updated_at
		 FROM group_chats AS gc JOIN members AS m ON gc.id = m.group_chat_id
		 WHERE gc.disabled = 'false' AND m.group_chat_id = ? AND m.user_account_id = ?"#,
      group_chat_id.clone(),
      user_account_id.clone()
    )
    .fetch_optional(&self.my_sql_pool)
    .await
    .map_err(|e| MemberDaoError::OtherError(e))
  }

  async fn get_members(&self, group_chat_id: String, user_account_id: String) -> Result<Vec<Member>, MemberDaoError> {
    sqlx::query_as!(
      Member,
      r#"SELECT m.id, m.group_chat_id, m.user_account_id, m.role, m.created_at, m.updated_at
         FROM group_chats AS gc JOIN members AS m ON gc.id = m.group_chat_id
         WHERE gc.disabled = 'false' AND m.group_chat_id = ?
			AND EXISTS (SELECT 1 FROM members AS m2 WHERE m2.group_chat_id = m.group_chat_id AND m2.user_account_id = ?)"#,
      group_chat_id.clone(),
      user_account_id.clone()
    )
    .fetch_all(&self.my_sql_pool)
    .await
    .map_err(|e| MemberDaoError::OtherError(e))
  }
}

// ---

/// メッセージリードモデル
#[derive(SimpleObject, Clone)]
pub struct Message {
  /// メッセージID
  id: String,
  /// グループチャットID
  group_chat_id: String,
  /// アカウントID
  user_account_id: String,
  /// メッセージ本文
  text: String,
  /// 作成日時
  created_at: NaiveDateTime,
  /// 更新日時
  updated_at: NaiveDateTime,
}

impl Message {
  pub fn new(
    id: String,
    group_chat_id: String,
    user_account_id: String,
    text: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
  ) -> Self {
    Self {
      id,
      group_chat_id,
      user_account_id,
      text,
      created_at,
      updated_at,
    }
  }
}

/// メッセージ用データアクセスオブジェクト。
///
/// メッセージを取得するためのインターフェース
#[async_trait]
pub trait MessageDao: Send + Sync {
  async fn get_message(&self, message_id: String, user_account_id: String) -> Result<Option<Message>, MessageDaoError>;
  async fn get_messages(&self, group_chat_id: String, user_account_id: String)
    -> Result<Vec<Message>, MessageDaoError>;
}

/// [MessageDao]の実装
pub struct MessageDaoImpl {
  my_sql_pool: MySqlPool,
}

impl MessageDaoImpl {
  pub fn new(my_sql_pool: MySqlPool) -> Self {
    Self { my_sql_pool }
  }
}

#[async_trait]
impl MessageDao for MessageDaoImpl {
  async fn get_message(&self, message_id: String, user_account_id: String) -> Result<Option<Message>, MessageDaoError> {
    sqlx::query_as!(
      Message,
      r#"SELECT m.id, m.group_chat_id, m.user_account_id, m.text, m.created_at, m.updated_at
		 FROM group_chats AS gc JOIN messages AS m ON gc.id = m.group_chat_id
         WHERE gc.disabled = 'false' AND m.disabled = 'false' AND m.id = ?
           AND EXISTS (SELECT 1 FROM members AS mem WHERE mem.group_chat_id = m.group_chat_id AND mem.user_account_id = ?)"#,
      message_id.clone(),
      user_account_id.clone()
    )
        .fetch_optional(&self.my_sql_pool)
        .await
        .map_err(|e| MessageDaoError::OtherError(e))
  }

  async fn get_messages(
    &self,
    group_chat_id: String,
    user_account_id: String,
  ) -> Result<Vec<Message>, MessageDaoError> {
    sqlx::query_as!(
      Message,
      r#"SELECT m.id, m.group_chat_id, m.user_account_id, m.text, m.created_at, m.updated_at
		 FROM group_chats AS gc JOIN messages AS m ON gc.id = m.group_chat_id
         WHERE gc.disabled = 'false' AND m.disabled = 'false' AND m.group_chat_id = ?
          AND EXISTS (SELECT 1 FROM members AS mem WHERE mem.group_chat_id = m.group_chat_id AND mem.user_account_id = ?)"#,
      group_chat_id.clone(),
      user_account_id.clone()
    )
        .fetch_all(&self.my_sql_pool)
        .await
        .map_err(|e| MessageDaoError::OtherError(e))
  }
}

#[cfg(test)]
mod tests {
  use crate::gateways::{GroupChatDao, GroupChatDaoImpl, MemberDao, MemberDaoImpl, MessageDao, MessageDaoImpl};
  use chrono::{DateTime, Utc};
  use command_domain::group_chat::{GroupChatId, GroupChatName, MemberId, MemberRole, Message};
  use command_domain::user_account::UserAccountId;
  use command_interface_adaptor_if::GroupChatReadModelUpdateDao;
  use command_interface_adaptor_impl::gateways::group_chat_read_model_dao_impl::GroupChatReadModelUpdateDaoImpl;
  use serial_test::serial;
  use sqlx::MySqlPool;
  use std::sync::OnceLock;
  use testcontainers::clients::Cli;
  use testcontainers::core::WaitFor;
  use testcontainers::{clients, Container, GenericImage};

  fn mysql_image() -> GenericImage {
    GenericImage::new("mysql", "8.0")
      .with_exposed_port(3306)
      .with_wait_for(WaitFor::message_on_stdout("Ready for start up"))
      .with_env_var("MYSQL_ROOT_PASSWORD", "password")
      .with_env_var("MYSQL_DATABASE", "ceer")
      .with_env_var("MYSQL_USER", "ceer")
      .with_env_var("MYSQL_PASSWORD", "ceer")
  }

  mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("../../../tools/refinery/migrations");
  }

  fn make_database_url_for_migration(port: u16) -> String {
    format!("mysql://root:password@localhost:{}/ceer", port)
  }

  fn make_database_url_for_application(port: u16) -> String {
    format!("mysql://ceer:ceer@localhost:{}/ceer", port)
  }

  fn refinery_migrate(port: u16) {
    let url = make_database_url_for_migration(port);
    log::debug!("url: {:?}", url);
    let opts = refinery_core::mysql::Opts::from_url(&url).unwrap();
    let mut pool_result;
    while {
      pool_result = refinery_core::mysql::Pool::new(opts.clone());
      pool_result.is_err()
    } {
      log::debug!("wait for mysql...");
      std::thread::sleep(std::time::Duration::from_secs(1));
    }
    let pool = pool_result.unwrap();
    let mut conn = pool.get_conn().unwrap();
    let _report = embedded::migrations::runner().run(&mut conn).unwrap();
  }

  fn init_logger() {
    std::env::set_var("RUST_LOG", "debug");
    let _ = env_logger::builder().is_test(true).try_init();
  }

  async fn insert_group_chat_and_member(
    update_dao: &GroupChatReadModelUpdateDaoImpl,
    group_chat_name: GroupChatName,
    admin_id: UserAccountId,
    created_at: DateTime<Utc>,
  ) -> GroupChatId {
    let group_chat_id = GroupChatId::new();
    update_dao
      .insert_group_chat(group_chat_id.clone(), group_chat_name, admin_id.clone(), created_at)
      .await
      .unwrap();
    let member_id = MemberId::new();
    let member_role = MemberRole::Admin;
    update_dao
      .insert_member(
        group_chat_id.clone(),
        member_id,
        admin_id.clone(),
        member_role,
        created_at,
      )
      .await
      .unwrap();
    group_chat_id
  }

  async fn insert_member_read_model(
    update_dao: GroupChatReadModelUpdateDaoImpl,
    group_chat_id: GroupChatId,
    user_account_id: UserAccountId,
    created_at: DateTime<Utc>,
  ) {
    let member_id = MemberId::new();
    let member_role = MemberRole::Admin;
    update_dao
      .insert_member(group_chat_id, member_id, user_account_id, member_role, created_at)
      .await
      .unwrap();
  }

  pub static DOCKER: OnceLock<clients::Cli> = OnceLock::new();

  #[tokio::test]
  #[serial]
  async fn test_get_group_chat() {
    init_logger();
    let docker = DOCKER.get_or_init(Cli::default);
    let mysql_node: Container<GenericImage> = docker.run(mysql_image());
    let mysql_port = mysql_node.get_host_port_ipv4(3306);

    refinery_migrate(mysql_port);

    let url = make_database_url_for_application(mysql_port);
    let pool = MySqlPool::connect(&url).await.unwrap();
    let update_dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());

    let admin_id = UserAccountId::new();
    let group_chat_name = GroupChatName::new("test").unwrap();
    let created_at = Utc::now();

    let group_chat_id =
      insert_group_chat_and_member(&update_dao, group_chat_name.clone(), admin_id.clone(), created_at).await;

    let dao = GroupChatDaoImpl::new(pool);
    let group_chat_read_model = dao
      .get_group_chat(group_chat_id.to_string(), admin_id.to_string())
      .await
      .unwrap()
      .unwrap();
    assert_eq!(group_chat_read_model.id, group_chat_id.to_string());
    assert_eq!(group_chat_read_model.name, group_chat_name.to_string());
    assert_eq!(group_chat_read_model.owner_id, admin_id.to_string());
  }

  #[tokio::test]
  #[serial]
  async fn test_get_group_chats() {
    init_logger();
    let docker = DOCKER.get_or_init(Cli::default);
    let mysql_node: Container<GenericImage> = docker.run(mysql_image());
    let mysql_port = mysql_node.get_host_port_ipv4(3306);

    refinery_migrate(mysql_port);

    let url = make_database_url_for_application(mysql_port);
    let pool = MySqlPool::connect(&url).await.unwrap();
    let update_dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());

    let admin_id = UserAccountId::new();
    let group_chat_name1 = GroupChatName::new("test1").unwrap();
    let group_chat_name2 = GroupChatName::new("test2").unwrap();
    let created_at = Utc::now();

    let group_chat_id1 =
      insert_group_chat_and_member(&update_dao, group_chat_name1.clone(), admin_id.clone(), created_at).await;
    let group_chat_id2 =
      insert_group_chat_and_member(&update_dao, group_chat_name2.clone(), admin_id.clone(), created_at).await;

    let dao = GroupChatDaoImpl::new(pool);
    let group_chat_read_models = dao.get_group_chats(admin_id.to_string()).await.unwrap();

    assert_eq!(group_chat_read_models.len(), 2);

    assert_eq!(group_chat_read_models[0].id, group_chat_id1.to_string());
    assert_eq!(group_chat_read_models[0].name, group_chat_name1.to_string());
    assert_eq!(group_chat_read_models[0].owner_id, admin_id.to_string());

    assert_eq!(group_chat_read_models[1].id, group_chat_id2.to_string());
    assert_eq!(group_chat_read_models[1].name, group_chat_name2.to_string());
    assert_eq!(group_chat_read_models[1].owner_id, admin_id.to_string());
  }

  #[tokio::test]
  #[serial]
  async fn test_get_members() {
    init_logger();
    let docker = DOCKER.get_or_init(Cli::default);
    let mysql_node: Container<GenericImage> = docker.run(mysql_image());
    let mysql_port = mysql_node.get_host_port_ipv4(3306);

    refinery_migrate(mysql_port);

    let url = make_database_url_for_application(mysql_port);
    let pool = MySqlPool::connect(&url).await.unwrap();
    let update_dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());

    let admin_id = UserAccountId::new();
    let user_account_id = UserAccountId::new();
    let group_chat_name = GroupChatName::new("test").unwrap();
    let created_at = Utc::now();

    let group_chat_id =
      insert_group_chat_and_member(&update_dao, group_chat_name.clone(), admin_id.clone(), created_at).await;
    insert_member_read_model(update_dao, group_chat_id.clone(), user_account_id.clone(), created_at).await;

    let dao = MemberDaoImpl::new(pool);
    let members = dao
      .get_members(group_chat_id.to_string(), admin_id.to_string())
      .await
      .unwrap();

    assert_eq!(members.len(), 2);
    assert!(members.iter().any(|e| e.user_account_id == admin_id.to_string()));
    assert!(members.iter().any(|e| e.user_account_id == user_account_id.to_string()));
  }

  #[tokio::test]
  async fn test_get_message() {
    init_logger();
    let docker = DOCKER.get_or_init(Cli::default);
    let mysql_node: Container<GenericImage> = docker.run(mysql_image());
    let mysql_port = mysql_node.get_host_port_ipv4(3306);

    refinery_migrate(mysql_port);

    let url = make_database_url_for_application(mysql_port);
    let pool = MySqlPool::connect(&url).await.unwrap();
    let update_dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());

    let admin_id = UserAccountId::new();
    let group_chat_name = GroupChatName::new("test").unwrap();
    let created_at = Utc::now();

    let group_chat_id =
      insert_group_chat_and_member(&update_dao, group_chat_name.clone(), admin_id.clone(), created_at).await;

    let dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());
    let message_text = "test".to_string();
    let message = Message::new(message_text, admin_id.clone());

    dao
      .insert_message(group_chat_id, message.clone(), created_at)
      .await
      .unwrap();
    let dao = MessageDaoImpl::new(pool.clone());

    let message = dao
      .get_message(
        message.breach_encapsulation_of_id().to_string(),
        admin_id.clone().to_string(),
      )
      .await
      .unwrap()
      .unwrap();

    assert_eq!(message.text, "test");
    assert_eq!(message.user_account_id, admin_id.to_string());
  }

  #[tokio::test]
  async fn test_get_messages() {
    init_logger();
    let docker = DOCKER.get_or_init(Cli::default);
    let mysql_node: Container<GenericImage> = docker.run(mysql_image());
    let mysql_port = mysql_node.get_host_port_ipv4(3306);

    refinery_migrate(mysql_port);

    let url = make_database_url_for_application(mysql_port);
    let pool = MySqlPool::connect(&url).await.unwrap();
    let update_dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());

    let admin_id = UserAccountId::new();
    let group_chat_name = GroupChatName::new("test").unwrap();
    let created_at = Utc::now();

    let group_chat_id =
      insert_group_chat_and_member(&update_dao, group_chat_name.clone(), admin_id.clone(), created_at).await;

    let dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());
    let message_text1 = "test1".to_string();
    let message_text2 = "test2".to_string();
    let message1 = Message::new(message_text1, admin_id.clone());
    let message2 = Message::new(message_text2, admin_id.clone());

    dao
      .insert_message(group_chat_id.clone(), message1.clone(), created_at)
      .await
      .unwrap();
    dao
      .insert_message(group_chat_id.clone(), message2.clone(), created_at)
      .await
      .unwrap();

    let dao = MessageDaoImpl::new(pool.clone());
    let messages = dao
      .get_messages(group_chat_id.clone().to_string(), admin_id.to_string())
      .await
      .unwrap();

    println!(
      "{:?}",
      messages
        .to_vec()
        .iter()
        .map(|e| (e.id.clone(), e.text.clone()))
        .collect::<Vec<_>>()
    );
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].text, "test1");
    assert_eq!(messages[1].text, "test2");
    assert_eq!(messages[0].user_account_id, admin_id.to_string());
    assert_eq!(messages[1].user_account_id, admin_id.to_string());
  }
}

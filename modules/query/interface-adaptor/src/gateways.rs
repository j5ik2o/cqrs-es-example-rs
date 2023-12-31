use anyhow::Result;
use async_graphql::async_trait::async_trait;
use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use sqlx::MySqlPool;

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
}

impl GroupChat {
  /// コンストラクタ
  pub fn new(id: String, name: String, owner_id: String, created_at: NaiveDateTime) -> Self {
    Self {
      id,
      name,
      owner_id,
      created_at,
    }
  }
}

/// グループチャット用データアクセスオブジェクト。
///
/// グループチャットを取得するためのインターフェース
#[async_trait]
pub trait GroupChatDao: Send + Sync {
  async fn get_group_chat(&self, group_chat_id: String, account_id: String) -> Result<GroupChat>;
  async fn get_group_chats(&self, account_id: String) -> Result<Vec<GroupChat>>;
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
  async fn get_group_chat(&self, group_chat_id: String, account_id: String) -> Result<GroupChat> {
    let group_chat = sqlx::query_as!(
      GroupChat,
      r#"SELECT gc.id, gc.name, gc.owner_id, gc.created_at
        FROM group_chats AS gc JOIN members AS m ON gc.id = m.group_chat_id
        WHERE m.group_chat_id = ? AND m.account_id = ?"#,
      group_chat_id,
      account_id
    )
    .fetch_one(&self.my_sql_pool)
    .await?;
    Ok(group_chat)
  }

  async fn get_group_chats(&self, account_id: String) -> Result<Vec<GroupChat>> {
    let group_chats = sqlx::query_as!(
      GroupChat,
      r#"SELECT gc.id, gc.name, gc.owner_id, gc.created_at
        FROM group_chats AS gc JOIN members AS m ON gc.id = m.group_chat_id
        WHERE m.account_id = ?"#,
      account_id
    )
    .fetch_all(&self.my_sql_pool)
    .await?;
    Ok(group_chats)
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
  account_id: String,
  /// ロール
  role: String,
  /// 作成日時
  created_at: NaiveDateTime,
}

impl Member {
  pub fn new(id: String, group_chat_id: String, account_id: String, role: String, created_at: NaiveDateTime) -> Self {
    Self {
      id,
      group_chat_id,
      account_id,
      role,
      created_at,
    }
  }
}

/// メンバー用データアクセスオブジェクト。
///
/// メンバーを取得するためのインターフェース
#[async_trait]
pub trait MemberDao: Send + Sync {
  async fn get_member(&self, group_chat_id: String, account_id: String) -> Result<Member>;
  async fn get_members(&self, group_chat_id: String, account_id: String) -> Result<Vec<Member>>;
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
  async fn get_member(&self, group_chat_id: String, account_id: String) -> Result<Member> {
    let member = sqlx::query_as!(
      Member,
      "SELECT id, group_chat_id, account_id, role, created_at FROM members WHERE group_chat_id = ? AND account_id = ?",
      group_chat_id,
      account_id
    )
    .fetch_one(&self.my_sql_pool)
    .await?;
    Ok(member)
  }

  async fn get_members(&self, group_chat_id: String, account_id: String) -> Result<Vec<Member>> {
    let members = sqlx::query_as!(
      Member,
      r#"SELECT id, group_chat_id, account_id, role, created_at
        FROM members
        WHERE group_chat_id = ?
            AND EXISTS (SELECT 1 FROM members AS m2 WHERE m2.group_chat_id = members.group_chat_id AND m2.account_id = ?)"#,
      group_chat_id,
      account_id
    )
            .fetch_all(&self.my_sql_pool)
            .await?;
    Ok(members)
  }
}

// ---

/// メッセージリードモデル
#[derive(SimpleObject)]
pub struct Message {
  /// メッセージID
  id: String,
  /// グループチャットID
  group_chat_id: String,
  /// アカウントID
  account_id: String,
  /// メッセージ本文
  text: String,
  /// 作成日時
  created_at: NaiveDateTime,
}

impl Message {
  pub fn new(id: String, group_chat_id: String, account_id: String, text: String, created_at: NaiveDateTime) -> Self {
    Self {
      id,
      group_chat_id,
      account_id,
      text,
      created_at,
    }
  }
}

/// メッセージ用データアクセスオブジェクト。
///
/// メッセージを取得するためのインターフェース
#[async_trait]
pub trait MessageDao: Send + Sync {
  async fn get_message(&self, message_id: String, account_id: String) -> Result<Message>;
  async fn get_messages(&self, group_chat_id: String, account_id: String) -> Result<Vec<Message>>;
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
  async fn get_message(&self, _message_id: String, _account_id: String) -> Result<Message> {
    todo!() // 必須課題 難易度:中
  }

  async fn get_messages(&self, _group_chat_id: String, _account_id: String) -> Result<Vec<Message>> {
    todo!() // 必須課題 難易度:中
  }
}

#[cfg(test)]
mod tests {
  use crate::gateways::{GroupChatDao, GroupChatDaoImpl, MemberDao, MemberDaoImpl};
  use chrono::{DateTime, Utc};
  use command_domain::group_chat::{GroupChatId, GroupChatName, MemberId, MemberRole};
  use command_domain::user_account::UserAccountId;
  use command_interface_adaptor_if::GroupChatReadModelUpdateDao;
  use command_interface_adaptor_impl::gateways::group_chat_read_model_dao_impl::GroupChatReadModelUpdateDaoImpl;
  use sqlx::MySqlPool;
  use testcontainers::clients::Cli;
  use testcontainers::core::WaitFor;
  use testcontainers::{Container, GenericImage};

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

  fn init() {
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
    account_id: UserAccountId,
    created_at: DateTime<Utc>,
  ) {
    let member_id = MemberId::new();
    let member_role = MemberRole::Admin;
    update_dao
      .insert_member(group_chat_id, member_id, account_id, member_role, created_at)
      .await
      .unwrap();
  }

  #[tokio::test]
  async fn test_get_group_chat() {
    init();
    let docker = Cli::default();
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
      .unwrap();
    assert_eq!(group_chat_read_model.id, group_chat_id.to_string());
    assert_eq!(group_chat_read_model.name, group_chat_name.to_string());
    assert_eq!(group_chat_read_model.owner_id, admin_id.to_string());
  }

  #[tokio::test]
  async fn test_get_group_chats() {
    init();
    let docker = Cli::default();
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
  async fn test_get_members() {
    init();
    let docker = Cli::default();
    let mysql_node: Container<GenericImage> = docker.run(mysql_image());
    let mysql_port = mysql_node.get_host_port_ipv4(3306);

    refinery_migrate(mysql_port);

    let url = make_database_url_for_application(mysql_port);
    let pool = MySqlPool::connect(&url).await.unwrap();
    let update_dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());

    let admin_id = UserAccountId::new();
    let account_id = UserAccountId::new();
    let group_chat_name = GroupChatName::new("test").unwrap();
    let created_at = Utc::now();

    let group_chat_id =
      insert_group_chat_and_member(&update_dao, group_chat_name.clone(), admin_id.clone(), created_at).await;
    insert_member_read_model(update_dao, group_chat_id.clone(), account_id.clone(), created_at).await;

    let dao = MemberDaoImpl::new(pool);
    let members = dao
      .get_members(group_chat_id.to_string(), admin_id.to_string())
      .await
      .unwrap();

    assert_eq!(members.len(), 2);
    assert!(members.iter().any(|e| e.account_id == admin_id.to_string()));
    assert!(members.iter().any(|e| e.account_id == account_id.to_string()));
  }

  #[test]
  #[ignore] // post_messageの実装完了後に#[ignore]を削除してください。
  fn test_get_message() {
    todo!() // 必須課題 難易度:低
  }

  #[test]
  #[ignore] // post_messageの実装完了後に#[ignore]を削除してください。
  fn test_get_messages() {
    todo!() // 必須課題 難易度:低
  }
}

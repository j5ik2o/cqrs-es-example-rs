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
      "INSERT INTO group_chats (id, name, owner_id, created_at) VALUES (?, ?, ?, ?)",
      aggregate_id.to_string(),
      name.to_string(),
      administrator_id.to_string(),
      created_at
    )
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn delete_group_chat(&self, aggregate_id: GroupChatId) -> Result<()> {
    // NOTE: 現状は物理削除になっている。論理削除変えたい場合はstatusフラグを導入しUPDATEに変更する。
    // もう一つの方法は履歴テーブルを作り、そちらに移動させる方法もある。
    sqlx::query!("DELETE FROM group_chats WHERE id = ?", aggregate_id.to_string())
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  async fn rename_group_chat(&self, aggregate_id: GroupChatId, name: GroupChatName) -> Result<()> {
    sqlx::query!(
      "UPDATE group_chats SET name = ? WHERE id = ?",
      name.to_string(),
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
      "INSERT INTO members (id, group_chat_id, account_id, role, created_at) VALUES (?, ?, ?, ?, ?)",
      member_id.to_string(),
      aggregate_id.to_string(),
      account_id.to_string(),
      role.to_string().to_lowercase(),
      created_at
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
      "INSERT INTO messages (id, group_chat_id, account_id, text, created_at) VALUES (?, ?, ?, ?, ?)",
      message.breach_encapsulation_of_id().to_string(),
      aggregate_id.to_string(),
      message.breach_encapsulation_of_sender_id().to_string(),
      message.breach_encapsulation_of_text(),
      created_at
    )
    .execute(&self.pool)
    .await?;
    Ok(())
  }

  async fn delete_message(&self, aggregate_id: GroupChatId, message_id: MessageId) -> Result<()> {
    // NOTE: 現状は物理削除になっている。論理削除変えたい場合はstatusフラグを導入しUPDATEに変更する。
    // もう一つの方法は履歴テーブルを作り、そちらに移動させる方法もある。
    sqlx::query!(
      "DELETE FROM messages WHERE id = ? AND group_chat_id = ?",
      message_id.to_string(),
      aggregate_id.to_string()
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

  async fn delete_group_chat(&self, _aggregate_id: GroupChatId) -> Result<()> {
    Ok(())
  }

  async fn rename_group_chat(&self, _aggregate_id: GroupChatId, _name: GroupChatName) -> Result<()> {
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

  async fn delete_message(&self, _aggregate_id: GroupChatId, _message_id: MessageId) -> Result<()> {
    Ok(())
  }
}

#[cfg(test)]
pub mod tests {
  use std::{env, thread};

  use chrono::Utc;
  use serial_test::serial;
  use sqlx::MySqlPool;
  use testcontainers::clients::Cli;
  use testcontainers::core::WaitFor;
  use testcontainers::images::generic::GenericImage;
  use testcontainers::Container;

  use command_domain::group_chat::MemberId;
  use command_domain::group_chat::{GroupChatId, GroupChatName, MemberRole, Message};
  use command_domain::user_account::UserAccountId;
  use command_interface_adaptor_if::GroupChatReadModelUpdateDao;

  use crate::gateways::group_chat_read_model_dao_impl::GroupChatReadModelUpdateDaoImpl;

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
      thread::sleep(std::time::Duration::from_secs(1));
    }
    let pool = pool_result.unwrap();
    let mut conn = pool.get_conn().unwrap();
    let _report = embedded::migrations::runner().run(&mut conn).unwrap();
  }

  fn init() {
    env::set_var("RUST_LOG", "debug");
    let _ = env_logger::builder().is_test(true).try_init();
  }

  #[tokio::test]
  #[serial]
  async fn test_insert_group_chat() {
    init();
    let docker = Cli::docker();
    let mysql_node: Container<GenericImage> = docker.run(mysql_image());
    let mysql_port = mysql_node.get_host_port_ipv4(3306);

    refinery_migrate(mysql_port);

    let url = make_database_url_for_application(mysql_port);
    let pool = MySqlPool::connect(&url).await.unwrap();
    let dao = GroupChatReadModelUpdateDaoImpl::new(pool);

    let aggregate_id = GroupChatId::new();
    let name = GroupChatName::new("test").unwrap();
    let admin_id = UserAccountId::new();

    dao
      .insert_group_chat(aggregate_id, name, admin_id, Utc::now())
      .await
      .unwrap();
  }

  #[tokio::test]
  #[serial]
  async fn test_delete_group_chat() {
    init();
    let docker = Cli::docker();
    let mysql_node: Container<GenericImage> = docker.run(mysql_image());
    let mysql_port = mysql_node.get_host_port_ipv4(3306);

    refinery_migrate(mysql_port);

    let url = make_database_url_for_application(mysql_port);
    let pool = MySqlPool::connect(&url).await.unwrap();
    let dao = GroupChatReadModelUpdateDaoImpl::new(pool);

    let aggregate_id = GroupChatId::new();
    let name = GroupChatName::new("test").unwrap();
    let admin_id = UserAccountId::new();

    dao
      .insert_group_chat(aggregate_id.clone(), name, admin_id, Utc::now())
      .await
      .unwrap();
    dao.delete_group_chat(aggregate_id).await.unwrap();
  }

  #[tokio::test]
  #[serial]
  async fn test_rename_group_chat() {
    init();
    let docker = Cli::docker();
    let mysql_node: Container<GenericImage> = docker.run(mysql_image());
    let mysql_port = mysql_node.get_host_port_ipv4(3306);

    refinery_migrate(mysql_port);

    let url = make_database_url_for_application(mysql_port);
    let pool = MySqlPool::connect(&url).await.unwrap();
    let dao = GroupChatReadModelUpdateDaoImpl::new(pool);

    let aggregate_id = GroupChatId::new();
    let name = GroupChatName::new("test").unwrap();
    let admin_id = UserAccountId::new();

    dao
      .insert_group_chat(aggregate_id.clone(), name, admin_id.clone(), Utc::now())
      .await
      .unwrap();

    let name = GroupChatName::new("test-2").unwrap();
    dao.rename_group_chat(aggregate_id, name).await.unwrap();
  }

  #[tokio::test]
  #[serial]
  async fn test_insert_member() {
    init();
    let docker = Cli::docker();
    let mysql_node: Container<GenericImage> = docker.run(mysql_image());
    let mysql_port = mysql_node.get_host_port_ipv4(3306);

    refinery_migrate(mysql_port);

    let url = make_database_url_for_application(mysql_port);
    let pool = MySqlPool::connect(&url).await.unwrap();
    let dao = GroupChatReadModelUpdateDaoImpl::new(pool);

    let aggregate_id = GroupChatId::new();
    let name = GroupChatName::new("test").unwrap();
    let admin_id = UserAccountId::new();

    dao
      .insert_group_chat(aggregate_id.clone(), name, admin_id, Utc::now())
      .await
      .unwrap();

    let member_id = MemberId::new();
    let user_account_id = UserAccountId::new();
    let role = MemberRole::Member;

    dao
      .insert_member(aggregate_id, member_id, user_account_id, role, Utc::now())
      .await
      .unwrap();
  }

  #[tokio::test]
  #[serial]
  async fn test_delete_member() {
    init();
    let docker = Cli::docker();
    let mysql_node: Container<GenericImage> = docker.run(mysql_image());
    let mysql_port = mysql_node.get_host_port_ipv4(3306);

    refinery_migrate(mysql_port);

    let url = make_database_url_for_application(mysql_port);
    let pool = MySqlPool::connect(&url).await.unwrap();
    let dao = GroupChatReadModelUpdateDaoImpl::new(pool);

    let aggregate_id = GroupChatId::new();
    let _seq_nr = 1;
    let name = GroupChatName::new("test").unwrap();
    let admin_id = UserAccountId::new();

    dao
      .insert_group_chat(aggregate_id.clone(), name, admin_id, Utc::now())
      .await
      .unwrap();

    let member_id = MemberId::new();
    let user_account_id = UserAccountId::new();
    let role = MemberRole::Member;

    dao
      .insert_member(
        aggregate_id.clone(),
        member_id,
        user_account_id.clone(),
        role,
        Utc::now(),
      )
      .await
      .unwrap();

    dao.delete_member(aggregate_id, user_account_id).await.unwrap();
  }

  #[tokio::test]
  #[serial]
  async fn test_post_message() {
    init();
    let docker = Cli::docker();
    let mysql_node: Container<GenericImage> = docker.run(mysql_image());
    let mysql_port = mysql_node.get_host_port_ipv4(3306);

    refinery_migrate(mysql_port);

    let url = make_database_url_for_application(mysql_port);
    let pool = MySqlPool::connect(&url).await.unwrap();
    let dao = GroupChatReadModelUpdateDaoImpl::new(pool);

    let aggregate_id = GroupChatId::new();
    let _seq_nr = 1;
    let name = GroupChatName::new("test").unwrap();
    let admin_id = UserAccountId::new();

    dao
      .insert_group_chat(aggregate_id.clone(), name, admin_id, Utc::now())
      .await
      .unwrap();

    let member_id = MemberId::new();
    let user_account_id = UserAccountId::new();
    let role = MemberRole::Member;

    dao
      .insert_member(
        aggregate_id.clone(),
        member_id,
        user_account_id.clone(),
        role,
        Utc::now(),
      )
      .await
      .unwrap();

    let message = Message::new("test".to_string(), user_account_id.clone());

    dao.insert_message(aggregate_id, message, Utc::now()).await.unwrap();
  }

  #[tokio::test]
  #[serial]
  #[ignore] // post_messageの実装完了後に#[ignore]を削除してください。
  async fn test_delete_message() {
    todo!() // 必須課題 難易度:中
  }
}

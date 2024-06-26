use std::{env, thread};

use chrono::Utc;
use serial_test::serial;
use sqlx::MySqlPool;
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerRequest, GenericImage, ImageExt};

use crate::common::init_logger;
use command_domain::group_chat::{GroupChatId, GroupChatName, MemberRole, Message};
use command_domain::group_chat::{MemberId, MessageId};
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::GroupChatReadModelUpdateDao;
use command_interface_adaptor_impl::gateways::group_chat_read_model_dao_impl::GroupChatReadModelUpdateDaoImpl;

fn mysql_image() -> ContainerRequest<GenericImage> {
  let port = ContainerPort::from(3306);
  GenericImage::new("mysql", "8.0")
    .with_wait_for(WaitFor::message_on_stdout("Ready for start up"))
    .with_exposed_port(port)
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

  let mysql_node = mysql_image().start().await.unwrap();
  let mysql_port = mysql_node.get_host_port_ipv4(3306).await.unwrap();

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
  init_logger();

  let mysql_node = mysql_image().start().await.unwrap();
  let mysql_port = mysql_node.get_host_port_ipv4(3306).await.unwrap();

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
  dao.delete_group_chat(aggregate_id, Utc::now()).await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_rename_group_chat() {
  init_logger();

  let mysql_node = mysql_image().start().await.unwrap();
  let mysql_port = mysql_node.get_host_port_ipv4(3306).await.unwrap();

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
  dao.rename_group_chat(aggregate_id, name, Utc::now()).await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_insert_member() {
  init_logger();

  let mysql_node = mysql_image().start().await.unwrap();
  let mysql_port = mysql_node.get_host_port_ipv4(3306).await.unwrap();

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
  init_logger();

  let mysql_node = mysql_image().start().await.unwrap();
  let mysql_port = mysql_node.get_host_port_ipv4(3306).await.unwrap();

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
  init_logger();

  let mysql_node = mysql_image().start().await.unwrap();
  let mysql_port = mysql_node.get_host_port_ipv4(3306).await.unwrap();

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

  let message_id = MessageId::new();
  let message = Message::new(message_id, "test".to_string(), user_account_id.clone());

  dao.insert_message(aggregate_id, message, Utc::now()).await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_delete_message() {
  init_logger();

  let mysql_node = mysql_image().start().await.unwrap();
  let mysql_port = mysql_node.get_host_port_ipv4(3306).await.unwrap();

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

  let message_id = MessageId::new();
  let message = Message::new(message_id, "test".to_string(), user_account_id.clone());

  dao
    .insert_message(aggregate_id.clone(), message.clone(), Utc::now())
    .await
    .unwrap();

  dao
    .delete_message(message.breach_encapsulation_of_id().clone(), Utc::now())
    .await
    .unwrap();
}

use anyhow::Result;
use sqlx::MySqlPool;

use cqrs_es_example_domain::thread::events::{
  ThreadCreated, ThreadDeleted, ThreadMemberAdded, ThreadMemberRemoved, ThreadMessageDeleted, ThreadMessagePosted,
  ThreadRenamed,
};

#[async_trait::async_trait]
pub trait ThreadReadModelDao {
  async fn insert_thread(&self, thread_created: &ThreadCreated) -> Result<()>;
  async fn delete_thread(&self, thread_deleted: &ThreadDeleted) -> Result<()>;
  async fn update_thread_name(&self, thread_renamed: &ThreadRenamed) -> Result<()>;
  async fn insert_member(&self, thread_member_added: &ThreadMemberAdded) -> Result<()>;
  async fn delete_member(&self, thread_member_removed: &ThreadMemberRemoved) -> Result<()>;
  async fn post_message(&self, thread_message_posted: &ThreadMessagePosted) -> Result<()>;
  async fn delete_message(&self, thread_message_deleted: &ThreadMessageDeleted) -> Result<()>;
}

pub struct MockThreadReadModelDao;

#[async_trait::async_trait]
impl ThreadReadModelDao for MockThreadReadModelDao {
  async fn insert_thread(&self, _thread_created: &ThreadCreated) -> Result<()> {
    Ok(())
  }

  async fn delete_thread(&self, _thread_deleted: &ThreadDeleted) -> Result<()> {
    Ok(())
  }

  async fn update_thread_name(&self, _thread_renamed: &ThreadRenamed) -> Result<()> {
    Ok(())
  }

  async fn insert_member(&self, _thread_member_added: &ThreadMemberAdded) -> Result<()> {
    Ok(())
  }

  async fn delete_member(&self, _thread_member_removed: &ThreadMemberRemoved) -> Result<()> {
    Ok(())
  }

  async fn post_message(&self, _thread_message_posted: &ThreadMessagePosted) -> Result<()> {
    Ok(())
  }

  async fn delete_message(&self, _thread_message_deleted: &ThreadMessageDeleted) -> Result<()> {
    Ok(())
  }
}

pub struct ThreadReadModelDaoImpl {
  pool: MySqlPool,
}

impl ThreadReadModelDaoImpl {
  pub fn new(pool: MySqlPool) -> Self {
    Self { pool }
  }
}

#[async_trait::async_trait]
impl ThreadReadModelDao for ThreadReadModelDaoImpl {
  async fn insert_thread(&self, thread_created: &ThreadCreated) -> Result<()> {
    let aggregate_id = thread_created.aggregate_id.to_string();
    let name = thread_created.name.to_string();
    let administrator_id = thread_created.members.administrator_id().user_account_id.to_string();
    let created_at = thread_created.occurred_at;

    sqlx::query!(
      "INSERT INTO threads (id, name, owner_id, created_at) VALUES (?, ?, ?, ?)",
      aggregate_id,
      name,
      administrator_id,
      created_at
    )
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn delete_thread(&self, thread_deleted: &ThreadDeleted) -> Result<()> {
    let aggregate_id = thread_deleted.aggregate_id.to_string();

    sqlx::query!(r"DELETE FROM threads WHERE id = ?", aggregate_id)
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  async fn update_thread_name(&self, thread_renamed: &ThreadRenamed) -> Result<()> {
    let aggregate_id = thread_renamed.aggregate_id.to_string();
    let name = thread_renamed.name.to_string();

    sqlx::query!("UPDATE threads SET name = ? WHERE id = ?", name, aggregate_id)
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  async fn insert_member(&self, _thread_member_added: &ThreadMemberAdded) -> Result<()> {
    Ok(())
  }

  async fn delete_member(&self, _thread_member_removed: &ThreadMemberRemoved) -> Result<()> {
    Ok(())
  }

  async fn post_message(&self, _thread_message_posted: &ThreadMessagePosted) -> Result<()> {
    Ok(())
  }

  async fn delete_message(&self, _thread_message_deleted: &ThreadMessageDeleted) -> Result<()> {
    Ok(())
  }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
  
  use std::{env, thread};

  use once_cell::sync::Lazy;
  use refinery_core::mysql;
  use sqlx::MySqlPool;
  use testcontainers::clients::Cli;
  use testcontainers::core::WaitFor;
  use testcontainers::images::generic::GenericImage;
  use testcontainers::Container;

  use cqrs_es_example_domain::thread::events::{ThreadCreated, ThreadDeleted};
  use cqrs_es_example_domain::thread::member::Members;
  use cqrs_es_example_domain::thread::{ThreadId, ThreadName};
  use cqrs_es_example_domain::user_account::UserAccountId;

  use crate::thread_read_model_dao::{ThreadReadModelDao, ThreadReadModelDaoImpl};

  static DOCKER: Lazy<Cli> = Lazy::new(|| Cli::default());

  static MYSQL_IMAGE: Lazy<GenericImage> = Lazy::new(|| {
    GenericImage::new("mysql", "8.0")
      .with_exposed_port(3306)
      .with_wait_for(WaitFor::message_on_stdout("Ready for start up"))
      .with_env_var("MYSQL_ROOT_PASSWORD", "password")
      .with_env_var("MYSQL_DATABASE", "ceer")
      .with_env_var("MYSQL_USER", "ceer")
      .with_env_var("MYSQL_PASSWORD", "ceer")
  });

  mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("../tools/rdb-migration/migrations");
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
    let opts = mysql::Opts::from_url(&url).unwrap();
    let mut pool_result;
    while {
      pool_result = mysql::Pool::new(opts.clone());
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
  async fn test_insert_thread() {
    init();
    let mysql_node: Container<GenericImage> = DOCKER.run(MYSQL_IMAGE.clone());
    let mysql_port = mysql_node.get_host_port_ipv4(3306);

    refinery_migrate(mysql_port);

    let url = make_database_url_for_application(mysql_port);
    let pool = MySqlPool::connect(&url).await.unwrap();
    let dao = ThreadReadModelDaoImpl::new(pool);

    let aggregate_id = ThreadId::new();
    let seq_nr = 1;
    let name = ThreadName::new("test".to_string());
    let admin_id = UserAccountId::new();
    let members = Members::new(admin_id);
    let body = ThreadCreated::new(aggregate_id, seq_nr, name, members);

    let _ = dao.insert_thread(&body).await;
  }

  #[tokio::test]
  async fn test_delete_thread() {
    init();
    let mysql_node: Container<GenericImage> = DOCKER.run(MYSQL_IMAGE.clone());
    let mysql_port = mysql_node.get_host_port_ipv4(3306);

    refinery_migrate(mysql_port);

    let url = make_database_url_for_application(mysql_port);
    let pool = MySqlPool::connect(&url).await.unwrap();
    let dao = ThreadReadModelDaoImpl::new(pool);

    let aggregate_id = ThreadId::new();
    let seq_nr = 1;
    let name = ThreadName::new("test".to_string());
    let admin_id = UserAccountId::new();
    let members = Members::new(admin_id.clone());
    let body = ThreadCreated::new(aggregate_id.clone(), seq_nr, name, members);

    let _ = dao.insert_thread(&body).await;

    let body = ThreadDeleted::new(aggregate_id, seq_nr + 1, admin_id);
    let _ = dao.delete_thread(&body).await;
  }
}

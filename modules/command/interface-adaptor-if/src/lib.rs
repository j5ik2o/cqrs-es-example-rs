use anyhow::Result;
use cqrs_es_example_domain::thread::events::*;
use cqrs_es_example_domain::thread::*;
use std::fmt::Debug;

#[async_trait::async_trait]
pub trait ThreadRepository: Clone + Send + Sync + 'static {
  async fn store(&mut self, event: &ThreadEvent, version: usize, snapshot: Option<&Thread>) -> Result<()>;
  async fn find_by_id(&self, id: &ThreadId) -> Result<Thread>;
}

#[async_trait::async_trait]
pub trait ThreadReadModelDao: Debug {
  async fn insert_thread(&self, thread_created: &ThreadCreated) -> Result<()>;
  async fn delete_thread(&self, thread_deleted: &ThreadDeleted) -> Result<()>;
  async fn rename_thread(&self, thread_renamed: &ThreadRenamed) -> Result<()>;
  async fn insert_member(&self, thread_member_added: &ThreadMemberAdded) -> Result<()>;
  async fn delete_member(&self, thread_member_removed: &ThreadMemberRemoved) -> Result<()>;
  async fn post_message(&self, thread_message_posted: &ThreadMessagePosted) -> Result<()>;
  async fn delete_message(&self, thread_message_deleted: &ThreadMessageDeleted) -> Result<()>;
}

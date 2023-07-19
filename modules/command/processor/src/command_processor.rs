use anyhow::Result;

use cqrs_es_example_command_interface_adaptor_if::ThreadRepository;
use cqrs_es_example_domain::aggregate::Aggregate;
use cqrs_es_example_domain::thread::member::{MemberId, Members};
use cqrs_es_example_domain::thread::*;
use cqrs_es_example_domain::user_account::UserAccountId;

pub struct ThreadCommandProcessor<'a, TR: ThreadRepository> {
  thread_repository: &'a mut TR,
}

impl<'a, TR: ThreadRepository> ThreadCommandProcessor<'a, TR> {
  pub fn new(thread_repository: &'a mut TR) -> Self {
    Self { thread_repository }
  }

  pub async fn create_thread(&mut self, name: ThreadName, executor_id: UserAccountId) -> Result<ThreadId> {
    let members = Members::new(executor_id);
    let (t, te) = Thread::new(name, members);
    self.thread_repository.store(&te, 1, Some(&t)).await?;
    Ok(t.id().clone())
  }

  fn resolve_snapshot(&self, thread: Thread) -> Option<Thread> {
    if thread.seq_nr() % 10 == 0 {
      Some(thread)
    } else {
      None
    }
  }

  pub async fn rename_thread(
    &mut self,
    id: ThreadId,
    name: ThreadName,
    executor_id: UserAccountId,
  ) -> Result<ThreadId> {
    let mut thread = self.thread_repository.find_by_id(&id).await?;
    let event = thread.rename(name, executor_id)?;
    let snapshot_opt = self.resolve_snapshot(thread.clone());
    self
      .thread_repository
      .store(&event, thread.version(), snapshot_opt.as_ref())
      .await?;
    Ok(id)
  }

  pub async fn add_member(
    &mut self,
    id: ThreadId,
    user_account_id: UserAccountId,
    role: MemberRole,
    executor_id: UserAccountId,
  ) -> Result<ThreadId> {
    let mut thread = self.thread_repository.find_by_id(&id).await?;
    log::debug!("thread.seq_nr_counter: {:?}", thread.seq_nr_counter);
    let member_id = MemberId::new();
    let event = thread.add_member(member_id, user_account_id, role, executor_id)?;
    let snapshot_opt = self.resolve_snapshot(thread.clone());
    self
      .thread_repository
      .store(&event, thread.version(), snapshot_opt.as_ref())
      .await?;
    Ok(id)
  }

  pub async fn remove_member(
    &mut self,
    id: ThreadId,
    user_account_id: UserAccountId,
    executor_id: UserAccountId,
  ) -> Result<ThreadId> {
    let mut thread = self.thread_repository.find_by_id(&id).await?;
    let event = thread.remove_member(user_account_id, executor_id)?;
    let snapshot_opt = self.resolve_snapshot(thread.clone());
    self
      .thread_repository
      .store(&event, thread.version(), snapshot_opt.as_ref())
      .await?;
    Ok(id)
  }

  pub async fn delete_thread(&mut self, id: ThreadId, executor_id: UserAccountId) -> Result<ThreadId> {
    let mut thread = self.thread_repository.find_by_id(&id).await?;
    let event = thread.delete(executor_id)?;
    let snapshot_opt = self.resolve_snapshot(thread.clone());
    self
      .thread_repository
      .store(&event, thread.version(), snapshot_opt.as_ref())
      .await?;
    Ok(id)
  }

  pub async fn post_message(&mut self, id: ThreadId, message: Message, executor_id: UserAccountId) -> Result<ThreadId> {
    let mut thread = self.thread_repository.find_by_id(&id).await?;
    let event = thread.post_message(message, executor_id)?;
    let snapshot_opt = self.resolve_snapshot(thread.clone());
    self
      .thread_repository
      .store(&event, thread.version(), snapshot_opt.as_ref())
      .await?;
    Ok(id)
  }

  pub async fn delete_message(
    &mut self,
    id: ThreadId,
    message_id: MessageId,
    executor_id: UserAccountId,
  ) -> Result<ThreadId> {
    let mut thread = self.thread_repository.find_by_id(&id).await?;
    let event = thread.delete_message(message_id, executor_id)?;
    let snapshot_opt = self.resolve_snapshot(thread.clone());
    self
      .thread_repository
      .store(&event, thread.version(), snapshot_opt.as_ref())
      .await?;
    Ok(id)
  }
}

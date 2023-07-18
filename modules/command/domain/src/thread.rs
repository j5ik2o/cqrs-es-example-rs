use std::fmt::{Display, Formatter};
use std::str::FromStr;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use ulid_generator_rs::{ULIDError, ULID};

use crate::aggregate::{Aggregate, AggregateId};
use crate::thread::events::{
  ThreadCreated, ThreadDeleted, ThreadEvent, ThreadMemberAdded, ThreadMemberRemoved, ThreadMessageDeleted,
  ThreadMessagePosted, ThreadRenamed,
};
use crate::thread::member::{Member, MemberId, Members};
use crate::user_account::UserAccountId;
use crate::ID_GENERATOR;

pub mod events;
pub mod member;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreadId {
  value: ULID,
}

impl AggregateId for ThreadId {
  fn type_name(&self) -> String {
    "thread".to_string()
  }

  fn value(&self) -> String {
    self.value.to_string()
  }
}

impl FromStr for ThreadId {
  type Err = ULIDError;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    match ULID::from_str(s) {
      Ok(value) => Ok(Self { value }),
      Err(error) => Err(error),
    }
  }
}

impl ThreadId {
  pub fn new() -> Self {
    let value = ID_GENERATOR.lock().unwrap().generate().unwrap();
    Self { value }
  }
}

impl Display for ThreadId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.value)
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreadName(String);

impl ThreadName {
  pub fn new(name: String) -> Self {
    Self(name)
  }
}

impl Display for ThreadName {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageId {
  value: ULID,
}

impl MessageId {
  pub fn new() -> Self {
    let value = ID_GENERATOR.lock().unwrap().generate().unwrap();
    Self { value }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
  pub id: MessageId,
  pub text: String,
  pub sender_id: UserAccountId,
}

impl Message {
  pub fn new(text: String, sender_id: UserAccountId) -> Self {
    let id = MessageId::new();
    Self { id, text, sender_id }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemberRole {
  Admin,
  Member,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
  id: ThreadId,
  name: ThreadName,
  members: Members,
  messages: Vec<Message>,
  pub seq_nr_counter: usize,
  version: usize,
}

impl PartialEq for Thread {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id && self.name == other.name && self.members == other.members && self.messages == other.messages
  }
}

impl Aggregate for Thread {
  type ID = ThreadId;

  fn id(&self) -> &Self::ID {
    &self.id
  }

  fn seq_nr(&self) -> usize {
    self.seq_nr_counter
  }

  fn version(&self) -> usize {
    self.version
  }

  fn set_version(&mut self, version: usize) {
    self.version = version;
  }
}

impl Thread {
  pub fn new(name: ThreadName, members: Members) -> (Self, ThreadEvent) {
    let id = ThreadId::new();
    Self::new_with_id(id, name, members, 0, 1)
  }

  pub fn new_with_id(
    id: ThreadId,
    name: ThreadName,
    members: Members,
    seq_nr_counter: usize,
    version: usize,
  ) -> (Self, ThreadEvent) {
    let mut my_self = Self {
      id: id.clone(),
      name: name.clone(),
      members: members.clone(),
      messages: vec![],
      seq_nr_counter,
      version,
    };
    my_self.seq_nr_counter += 1;
    let event = ThreadEvent::ThreadCreated(ThreadCreated::new(id, my_self.seq_nr_counter, name, members));
    (my_self, event)
  }

  fn apply_event(&mut self, event: &ThreadEvent) {
    match event {
      ThreadEvent::ThreadDeleted(body) => {
        self.delete(body.executor_id.clone()).unwrap();
      }
      ThreadEvent::ThreadRenamed(body) => {
        self.rename(body.name.clone(), body.executor_id.clone()).unwrap();
      }
      ThreadEvent::ThreadMemberAdd(body) => {
        self
          .add_member(
            body.member.id.clone(),
            body.member.user_account_id.clone(),
            body.member.role.clone(),
            body.executor_id.clone(),
          )
          .unwrap();
      }
      ThreadEvent::ThreadMemberRemoved(body) => {
        self
          .remove_member(body.user_account_id.clone(), body.executor_id.clone())
          .unwrap();
      }
      ThreadEvent::ThreadMessagePosted(body) => {
        self
          .post_message(body.message.clone(), body.executor_id.clone())
          .unwrap();
      }
      ThreadEvent::ThreadMessageDeleted(body) => {
        self
          .delete_message(body.message_id.clone(), body.executor_id.clone())
          .unwrap();
      }
      _ => {}
    }
  }

  pub fn replay(events: Vec<ThreadEvent>, snapshot: Option<Thread>, version: usize) -> Self {
    log::debug!("event.size = {}", events.len());
    let mut result = events
      .iter()
      .fold(snapshot, |result, event| match (result, event) {
        (Some(mut this), event) => {
          log::debug!("Replaying snapshot: {:?}", this);
          log::debug!("Replaying event: {:?}", event);
          this.apply_event(event);
          Some(this)
        }
        (..) => None,
      })
      .unwrap();
    result.set_version(version);
    result
  }

  pub fn name(&self) -> &ThreadName {
    &self.name
  }

  pub fn members_vec(&self) -> Vec<&Member> {
    self.members.values()
  }

  pub fn members(&self) -> &Members {
    &self.members
  }

  pub fn messages(&self) -> &Vec<Message> {
    &self.messages
  }

  pub fn rename(&mut self, name: ThreadName, executor_id: UserAccountId) -> Result<ThreadEvent> {
    if !self.members.is_member(&executor_id) {
      return Err(anyhow!("executor_id is not a member of the thread"));
    }
    if self.name == name {
      return Err(anyhow!("Name already set"));
    }
    self.name = name;
    self.seq_nr_counter += 1;
    Ok(ThreadEvent::ThreadRenamed(ThreadRenamed::new(
      self.id.clone(),
      self.seq_nr_counter,
      self.name.clone(),
      executor_id,
    )))
  }

  pub fn add_member(
    &mut self,
    member_id: MemberId,
    user_account_id: UserAccountId,
    role: MemberRole,
    executor_id: UserAccountId,
  ) -> Result<ThreadEvent> {
    if !self.members.is_member(&executor_id) {
      return Err(anyhow!("executor_id is not a member of the thread"));
    }
    if self.members.is_member(&user_account_id) {
      return Err(anyhow!("user_account_id is already a member of the thread"));
    }
    let member = Member::new(member_id, user_account_id, role);
    self.members.add_member(member.clone());
    self.seq_nr_counter += 1;
    Ok(ThreadEvent::ThreadMemberAdd(ThreadMemberAdded::new(
      self.id.clone(),
      self.seq_nr_counter,
      member,
      executor_id,
    )))
  }

  pub fn remove_member(&mut self, user_account_id: UserAccountId, executor_id: UserAccountId) -> Result<ThreadEvent> {
    if !self.members.is_member(&executor_id) {
      return Err(anyhow!("User is not a member of the thread"));
    }
    if !self.members.is_member(&user_account_id) {
      return Err(anyhow!("user_account_id is not a member of the thread"));
    }
    self.members.remove_member_by_user_account_id(&user_account_id);
    self.seq_nr_counter += 1;
    Ok(ThreadEvent::ThreadMemberRemoved(ThreadMemberRemoved::new(
      self.id.clone(),
      self.seq_nr_counter,
      user_account_id,
      executor_id,
    )))
  }

  pub fn post_message(&mut self, message: Message, executor_id: UserAccountId) -> Result<ThreadEvent> {
    if message.sender_id != executor_id {
      return Err(anyhow!("User is not the sender of the message"));
    }
    if !self.members.is_member(&executor_id) {
      return Err(anyhow!("User is not a member of the thread"));
    }
    if self.messages.contains(&message) {
      return Err(anyhow!("Message already exists"));
    }
    self.messages.push(message.clone());
    self.seq_nr_counter += 1;
    Ok(ThreadEvent::ThreadMessagePosted(ThreadMessagePosted::new(
      self.id.clone(),
      self.seq_nr_counter,
      message,
      executor_id,
    )))
  }

  pub fn delete_message(&mut self, message_id: MessageId, executor_id: UserAccountId) -> Result<ThreadEvent> {
    if !self.members.is_member(&executor_id) {
      return Err(anyhow!("User is not a member of the thread"));
    }
    let result = self.messages.iter().position(|message| message.id == message_id);
    match result {
      None => return Err(anyhow!("Message not found")),
      Some(index) => {
        let message = &self.messages[index];
        let member = self.members.find_by_user_account_id(&message.sender_id).unwrap();
        if member.user_account_id != executor_id {
          return Err(anyhow!("User is not the sender of the message"));
        }
        self.messages.remove(index);
        self.seq_nr_counter += 1;
        Ok(ThreadEvent::ThreadMessageDeleted(ThreadMessageDeleted::new(
          self.id.clone(),
          self.seq_nr_counter,
          message_id,
          executor_id,
        )))
      }
    }
  }

  pub fn delete(&mut self, executor_id: UserAccountId) -> Result<ThreadEvent> {
    if !self.members.is_member(&executor_id) {
      return Err(anyhow!("User is not a member of the thread"));
    }
    self.seq_nr_counter += 1;
    Ok(ThreadEvent::ThreadDeleted(ThreadDeleted::new(
      self.id.clone(),
      self.seq_nr_counter,
      executor_id,
    )))
  }
}

#[async_trait::async_trait]
pub trait ThreadRepository: Clone + Send + Sync + 'static {
  async fn store(&mut self, event: &ThreadEvent, version: usize, snapshot: Option<&Thread>) -> Result<()>;

  async fn find_by_id(&self, id: &ThreadId) -> Result<Thread>;
}

#[test]
fn test() {
  let thread_name = ThreadName::new("test".to_string());
  let admin_user_account_id = UserAccountId::new();
  let members = Members::new(admin_user_account_id.clone());
  let (thread, _) = Thread::new(thread_name.clone(), members);
  assert_eq!(thread.name, thread_name);

  let json = serde_json::to_string(&thread);
  println!("{}", json.unwrap());
}

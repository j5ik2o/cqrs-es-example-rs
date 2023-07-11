use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid_generator_rs::{ULID, ULIDGenerator};

use crate::{Event, ID_GENERATOR};
use crate::thread::{Message, MessageId, ThreadId, ThreadName};
use crate::thread::member::{Member, Members};
use crate::user_account::UserAccountId;

pub type ThreadEventId = ULID;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ThreadEvent {
  ThreadCreated(ThreadCreated),
  ThreadDeleted(ThreadDeleted),
  ThreadRenamed(ThreadRenamed),
  ThreadMessagePosted(ThreadMessagePosted),
  ThreadMessageDeleted(ThreadMessageDeleted),
  ThreadMemberAdd(ThreadMemberAdded),
  ThreadMemberRemoved(ThreadMemberRemoved),
}

impl Event for ThreadEvent {
  type AggregateID = ThreadId;
  type ID = ThreadEventId;

  fn id(&self) -> &ThreadEventId {
    match self {
      ThreadEvent::ThreadCreated(event) => &event.id,
      ThreadEvent::ThreadDeleted(event) => &event.id,
      ThreadEvent::ThreadRenamed(event) => &event.id,
      ThreadEvent::ThreadMessagePosted(event) => &event.id,
      ThreadEvent::ThreadMessageDeleted(event) => &event.id,
      ThreadEvent::ThreadMemberAdd(event) => &event.id,
      ThreadEvent::ThreadMemberRemoved(event) => &event.id,
    }
  }

  fn seq_nr(&self) -> usize {
    match self {
      ThreadEvent::ThreadCreated(event) => event.seq_nr,
      ThreadEvent::ThreadDeleted(event) => event.seq_nr,
      ThreadEvent::ThreadRenamed(event) => event.seq_nr,
      ThreadEvent::ThreadMessagePosted(event) => event.seq_nr,
      ThreadEvent::ThreadMessageDeleted(event) => event.seq_nr,
      ThreadEvent::ThreadMemberAdd(event) => event.seq_nr,
      ThreadEvent::ThreadMemberRemoved(event) => event.seq_nr,
    }
  }

  fn aggregate_id(&self) -> &ThreadId {
    match self {
      ThreadEvent::ThreadCreated(event) => &event.aggregate_id,
      ThreadEvent::ThreadDeleted(event) => &event.aggregate_id,
      ThreadEvent::ThreadRenamed(event) => &event.aggregate_id,
      ThreadEvent::ThreadMessagePosted(event) => &event.aggregate_id,
      ThreadEvent::ThreadMessageDeleted(event) => &event.aggregate_id,
      ThreadEvent::ThreadMemberAdd(event) => &event.aggregate_id,
      ThreadEvent::ThreadMemberRemoved(event) => &event.aggregate_id,
    }
  }

  fn occurred_at(&self) -> &DateTime<Utc> {
    match self {
      ThreadEvent::ThreadCreated(event) => &event.occurred_at,
      ThreadEvent::ThreadDeleted(event) => &event.occurred_at,
      ThreadEvent::ThreadRenamed(event) => &event.occurred_at,
      ThreadEvent::ThreadMessagePosted(event) => &event.occurred_at,
      ThreadEvent::ThreadMessageDeleted(event) => &event.occurred_at,
      ThreadEvent::ThreadMemberAdd(event) => &event.occurred_at,
      ThreadEvent::ThreadMemberRemoved(event) => &event.occurred_at,
    }
  }

  fn is_created(&self) -> bool {
    match self {
      ThreadEvent::ThreadCreated(_) => true,
      _ => false,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadCreated {
  pub id: ThreadEventId,
  pub aggregate_id: ThreadId,
  pub seq_nr: usize,
  pub name: ThreadName,
  pub members: Members,
  pub occurred_at: DateTime<Utc>,
}

impl ThreadCreated {
  pub fn new(aggregate_id: ThreadId, seq_nr: usize, name: ThreadName, members: Members) -> Self {
    let id = ID_GENERATOR.lock().unwrap().generate().unwrap();
    let occurred_at = Utc::now();
    Self {
      id,
      aggregate_id,
      seq_nr,
      name,
      members,
      occurred_at,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadDeleted {
    pub id: ThreadEventId,
    pub aggregate_id: ThreadId,
    pub seq_nr: usize,
    pub executor_id: UserAccountId,
    pub occurred_at: DateTime<Utc>,
}

impl ThreadDeleted {
  pub fn new(aggregate_id: ThreadId, seq_nr: usize, executor_id: UserAccountId) -> Self {
    let id = ID_GENERATOR.lock().unwrap().generate().unwrap();
    let occurred_at = Utc::now();
    Self {
      id,
      aggregate_id,
      seq_nr,
      executor_id,
      occurred_at,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadRenamed {
    pub id: ThreadEventId,
    pub aggregate_id: ThreadId,
    pub seq_nr: usize,
    pub name: ThreadName,
    pub executor_id: UserAccountId,
    pub occurred_at: DateTime<Utc>,
}

impl ThreadRenamed {
  pub fn new(aggregate_id: ThreadId, seq_nr: usize, name: ThreadName, executor_id: UserAccountId) -> Self {
    let id = ID_GENERATOR.lock().unwrap().generate().unwrap();
    let occurred_at = Utc::now();
    Self {
      id,
      aggregate_id,
      seq_nr,
      name,
      executor_id,
      occurred_at,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMessagePosted {
  pub(crate) id: ThreadEventId,
  pub(crate) aggregate_id: ThreadId,
  pub(crate) seq_nr: usize,
  pub(crate) message: Message,
  pub(crate) executor_id: UserAccountId,
  pub(crate) occurred_at: DateTime<Utc>,
}

impl ThreadMessagePosted {
  pub fn new(aggregate_id: ThreadId, seq_nr: usize, message: Message, executor_id: UserAccountId) -> Self {
    let id = ID_GENERATOR.lock().unwrap().generate().unwrap();
    let occurred_at = Utc::now();
    Self {
      id,
      aggregate_id,
      seq_nr,
      message,
      executor_id,
      occurred_at,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMessageDeleted {
  pub(crate) id: ThreadEventId,
  pub(crate) aggregate_id: ThreadId,
  pub(crate) seq_nr: usize,
  pub(crate) message_id: MessageId,
  pub(crate) executor_id: UserAccountId,
  pub(crate) occurred_at: DateTime<Utc>,
}

impl ThreadMessageDeleted {
  pub fn new(aggregate_id: ThreadId, seq_nr: usize, message_id: MessageId, executor_id: UserAccountId) -> Self {
    let mut idgen = ULIDGenerator::new();
    let id = idgen.generate().unwrap();
    let occurred_at = Utc::now();
    Self {
      id,
      aggregate_id,
      seq_nr,
      message_id,
      executor_id,
      occurred_at,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMemberAdded {
  pub(crate) id: ThreadEventId,
  pub(crate) aggregate_id: ThreadId,
  pub(crate) seq_nr: usize,
  pub(crate) member: Member,
  pub(crate) executor_id: UserAccountId,
  pub(crate) occurred_at: DateTime<Utc>,
}

impl ThreadMemberAdded {
  pub fn new(aggregate_id: ThreadId, seq_nr: usize, member: Member, executor_id: UserAccountId) -> Self {
    let id = ID_GENERATOR.lock().unwrap().generate().unwrap();
    let occurred_at = Utc::now();
    Self {
      id,
      aggregate_id,
      seq_nr,
      member,
      executor_id,
      occurred_at,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMemberRemoved {
  pub(crate) id: ThreadEventId,
  pub(crate) aggregate_id: ThreadId,
  pub(crate) seq_nr: usize,
  pub(crate) user_account_id: UserAccountId,
  pub(crate) executor_id: UserAccountId,
  pub(crate) occurred_at: DateTime<Utc>,
}

impl ThreadMemberRemoved {
  pub fn new(
    aggregate_id: ThreadId,
    seq_nr: usize,
    user_account_id: UserAccountId,
    executor_id: UserAccountId,
  ) -> Self {
    let id = ID_GENERATOR.lock().unwrap().generate().unwrap();
    let occurred_at = Utc::now();
    Self {
      id,
      aggregate_id,
      seq_nr,
      user_account_id,
      executor_id,
      occurred_at,
    }
  }
}

#[cfg(test)]
mod tests {
    use crate::Event;
    use crate::thread::{ThreadId, ThreadName};
    use crate::thread::events::{ThreadCreated, ThreadEvent};
    use crate::thread::member::Members;
    use crate::user_account::UserAccountId;

    #[test]
  fn test_to_json() {
    let thread_id = ThreadId::new();
    let thread_name = ThreadName::new("test".to_string());
    let admin_user_account_id = UserAccountId::new();
    let event = ThreadEvent::ThreadCreated(ThreadCreated::new(
      thread_id,
      1usize,
      thread_name,
      Members::new(admin_user_account_id),
    ));
    let json = serde_json::to_string(&event);
    let occurred_at = event.occurred_at().timestamp_millis();
    println!("{}", json.unwrap());
  }
}

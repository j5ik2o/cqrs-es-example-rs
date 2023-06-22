use crate::aggregate::Aggregate;
use crate::events::{
  Event, GroupChatCreated, GroupChatDestroyed, GroupChatEvent, GroupChatMemberAdd, GroupChatMemberRemoved,
  GroupChatMessageDeleted, GroupChatMessagePosted,
};
use crate::ID_GENERATOR;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use ulid_generator_rs::ULID;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupChatID {
  value: ULID,
}

impl GroupChatID {
  pub fn new() -> Self {
    let value = ID_GENERATOR.lock().unwrap().generate().unwrap();
    Self { value }
  }

  pub fn from_string(id: String) -> Result<Self> {
    let value = ULID::from_str(&id).map_err(|_| anyhow!("Invalid ULID"))?;
    Ok(Self { value })
  }
}

impl Display for GroupChatID {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.value)
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupChatName(String);

impl GroupChatName {
  pub fn new(name: String) -> Self {
    Self(name)
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberID(ULID);

impl MemberID {
  pub fn new() -> Self {
    let value = ID_GENERATOR.lock().unwrap().generate().unwrap();
    Self(value)
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageID(ULID);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
  id: MessageID,
  text: String,
  sender_id: MemberID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChat {
  id: GroupChatID,
  name: GroupChatName,
  member_ids: Vec<MemberID>,
  messages: Vec<Message>,
  seq_nr_counter: usize,
  version: usize,
}

impl PartialEq for GroupChat {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
      && self.name == other.name
      && self.member_ids == other.member_ids
      && self.messages == other.messages
  }
}

impl Aggregate for GroupChat {
  type ID = GroupChatID;

  fn id(&self) -> &Self::ID {
    &self.id
  }

  fn version(&self) -> usize {
    self.version
  }
}

impl GroupChat {
  pub fn new(name: GroupChatName, member_ids: Vec<MemberID>) -> (Self, GroupChatEvent) {
    let id = ID_GENERATOR.lock().unwrap().generate().unwrap();
    let id = GroupChatID { value: id };
    Self::new_with_id(id, name, member_ids, 0, 1)
  }

  pub fn new_with_id(
    id: GroupChatID,
    name: GroupChatName,
    member_ids: Vec<MemberID>,
    seq_nr_counter: usize,
    version: usize,
  ) -> (Self, GroupChatEvent) {
    let mut my_self = Self {
      id: id.clone(),
      name: name.clone(),
      member_ids: member_ids.clone(),
      messages: vec![],
      seq_nr_counter,
      version,
    };
    my_self.seq_nr_counter += 1;
    let event = GroupChatEvent::GroupChatCreated(GroupChatCreated::new(id, my_self.seq_nr_counter, name, member_ids));
    (my_self, event)
  }

  pub fn apply_event(&mut self, event: &GroupChatEvent) {
    match event {
      GroupChatEvent::GroupChatMemberAdd(body) => {
        self.add_member_id(body.member_id.clone()).unwrap();
      }
      GroupChatEvent::GroupChatMemberRemoved(body) => {
        self.remove_member_id(body.member_id.clone()).unwrap();
      }
      _ => panic!("Unsupported event type"),
    }
  }

  pub fn replay(events: Vec<GroupChatEvent>, snapshot: Option<GroupChat>) -> Self {
    events.iter().fold(snapshot, |result, event| match (result, event) {
      (None, GroupChatEvent::GroupChatCreated(body)) => Some(
        Self::new_with_id(
          body.group_chat_id.clone(),
          body.name.clone(),
          body.member_ids.clone(),
          event.seq_nr() - 1,
          event.seq_nr(),
        )
        .0,
      ),
      (Some(mut this), event) => {
        this.apply_event(event);
        Some(this)
      }
      (..) => None,
    }).unwrap()
  }

  pub fn name(&self) -> &GroupChatName {
    &self.name
  }

  pub fn member_ids(&self) -> &Vec<MemberID> {
    &self.member_ids
  }

  pub fn messages(&self) -> &Vec<Message> {
    &self.messages
  }

  pub fn add_member_id(&mut self, member_id: MemberID) -> Result<GroupChatEvent> {
    if self.member_ids.contains(&member_id) {
      return Err(anyhow!("Member already exists"));
    }
    self.member_ids.push(member_id.clone());
    self.seq_nr_counter += 1;
    Ok(GroupChatEvent::GroupChatMemberAdd(GroupChatMemberAdd::new(
      self.id.clone(),
      self.seq_nr_counter,
      member_id,
    )))
  }

  pub fn remove_member_id(&mut self, member_id: MemberID) -> Result<GroupChatEvent> {
    let result = self.member_ids.iter().position(|id| id == &member_id);
    match result {
      None => return Err(anyhow!("Member not found")),
      Some(index) => {
        self.member_ids.remove(index);
        self.seq_nr_counter += 1;
        Ok(GroupChatEvent::GroupChatMemberRemoved(GroupChatMemberRemoved::new(
          self.id.clone(),
          self.seq_nr_counter,
          member_id,
        )))
      }
    }
  }

  pub fn post_message(&mut self, message: Message) -> Result<GroupChatEvent> {
    if self.messages.contains(&message) {
      return Err(anyhow!("Message already exists"));
    }
    self.messages.push(message.clone());
    self.seq_nr_counter += 1;
    Ok(GroupChatEvent::GroupChatMessagePosted(GroupChatMessagePosted::new(
      self.id.clone(),
      self.seq_nr_counter,
      message,
    )))
  }

  pub fn delete_message(&mut self, message_id: MessageID) -> Result<GroupChatEvent> {
    let result = self.messages.iter().position(|message| message.id == message_id);
    match result {
      None => return Err(anyhow!("Message not found")),
      Some(index) => {
        self.messages.remove(index);
        self.seq_nr_counter += 1;
        Ok(GroupChatEvent::GroupChatMessageDeleted(GroupChatMessageDeleted::new(
          self.id.clone(),
          self.seq_nr_counter,
          message_id,
        )))
      }
    }
  }

  pub fn destroy(&mut self) -> GroupChatEvent {
    self.seq_nr_counter += 1;
    GroupChatEvent::GroupChatDestroyed(GroupChatDestroyed::new(self.id.clone(), self.seq_nr_counter))
  }
}

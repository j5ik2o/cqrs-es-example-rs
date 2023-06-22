use crate::gateway::EventPersistenceGateway;
use anyhow::Result;
use palupunte_domain::events::GroupChatEvent;
use palupunte_domain::group_chat::{GroupChat, GroupChatID};

pub struct GroupChatRepository<'a> {
  event_persistence_gateway: EventPersistenceGateway<'a>,
}

impl<'a> GroupChatRepository<'a> {
  pub fn new(gateway: EventPersistenceGateway<'a>) -> Self {
    Self {
      event_persistence_gateway: gateway,
    }
  }

  pub async fn store(&mut self, event: &GroupChatEvent, version: usize, snapshot: Option<&GroupChat>) -> Result<()> {
    self
      .event_persistence_gateway
      .store_event_with_snapshot_opt(event, version, snapshot)
      .await
  }

  pub async fn find_by_id(&self, id: &GroupChatID) -> Result<GroupChat> {
    let (mut snapshot, seq_nr) = self
      .event_persistence_gateway
      .get_snapshot_by_id::<GroupChat>(id.to_string())
      .await?;
    let events = self
      .event_persistence_gateway
      .get_events_by_id_and_seq_nr::<GroupChatEvent>(id.to_string(), seq_nr)
      .await?;
    let result = GroupChat::replay(events, Some(snapshot));
    Ok(result)
  }
}

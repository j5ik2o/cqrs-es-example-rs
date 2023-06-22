use anyhow::Result;

use cqrs_es_example_domain::thread::events::ThreadEvent;
use cqrs_es_example_domain::thread::{Thread, ThreadId, ThreadRepository};

use crate::gateways::event_persistence_gateway::EventPersistenceGateway;

#[derive(Debug, Clone)]
pub struct ThreadRepositoryImpl {
  event_persistence_gateway: EventPersistenceGateway,
}

impl ThreadRepositoryImpl {
  pub fn new(event_persistence_gateway: EventPersistenceGateway) -> Self {
    Self {
      event_persistence_gateway,
    }
  }
}

#[async_trait::async_trait]
impl ThreadRepository for ThreadRepositoryImpl {
  async fn store(&mut self, event: &ThreadEvent, version: usize, snapshot: Option<&Thread>) -> Result<()> {
    self
      .event_persistence_gateway
      .store_event_with_snapshot_opt(event, version, snapshot)
      .await
  }

  async fn find_by_id(&self, id: &ThreadId) -> Result<Thread> {
    let (snapshot, seq_nr, version) = self
      .event_persistence_gateway
      .get_snapshot_by_id::<Thread, ThreadId>(id)
      .await?;
    log::debug!(">>> seq_nr: {:?}", seq_nr);
    let events = self
      .event_persistence_gateway
      .get_events_by_id_and_seq_nr::<ThreadEvent, ThreadId>(id, seq_nr)
      .await?;
    let result = Thread::replay(events, Some(snapshot), version);
    Ok(result)
  }
}

use cqrs_es_example_domain::aggregate::{Aggregate, AggregateId};
use std::fmt::Debug;

pub mod event_persistence_gateway_with_transaction;
pub mod event_persistence_gateway_without_transaction;
pub mod thread_read_model_dao_impl;
pub mod thread_repository;

use anyhow::Result;
use cqrs_es_example_domain::Event;
use serde::{de, Serialize};

#[async_trait::async_trait]
pub trait EventPersistenceGateway {
  async fn get_snapshot_by_id<E, T, AID: AggregateId>(&self, aid: &AID) -> Result<(T, usize, usize)>
  where
    E: ?Sized + Serialize + Event + for<'de> de::Deserialize<'de>,
    T: ?Sized + Serialize + Aggregate + for<'de> de::Deserialize<'de>;

  async fn get_events_by_id_and_seq_nr<T, AID: AggregateId>(&self, aid: &AID, seq_nr: usize) -> Result<Vec<T>>
  where
    T: Debug + for<'de> de::Deserialize<'de>;

  async fn store_event_with_snapshot_opt<A, E>(
    &mut self,
    event: &E,
    version: usize,
    aggregate: Option<&A>,
  ) -> Result<()>
  where
    A: ?Sized + Serialize + Aggregate,
    E: ?Sized + Serialize + Event;
}

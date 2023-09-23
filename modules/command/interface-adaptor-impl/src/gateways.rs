use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod group_chat_read_model_dao_impl;
pub mod group_chat_repository;

use anyhow::Result;
use event_store_adapter_rs::types::{Aggregate, AggregateId, Event};
use serde::{de, Serialize};

pub trait KeyResolver: Debug + Send + 'static {
  fn resolve_pkey(&self, id_type_name: &str, value: &str, shard_count: u64) -> String;
  fn resolve_skey(&self, id_type_name: &str, value: &str, seq_nr: usize) -> String;
}

#[derive(Debug, Clone)]
pub struct DefaultPartitionKeyResolver;

impl KeyResolver for DefaultPartitionKeyResolver {
  fn resolve_pkey(&self, id_type_name: &str, value: &str, shard_count: u64) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    let hash_value = hasher.finish();
    let remainder = hash_value % shard_count;
    format!("{}-{}", id_type_name, remainder)
  }

  fn resolve_skey(&self, id_type_name: &str, value: &str, seq_nr: usize) -> String {
    format!("{}-{}-{}", id_type_name, value, seq_nr)
  }
}

#[async_trait::async_trait]
pub trait EventPersistenceGateway: Debug + Clone + Sync + Send + 'static {
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

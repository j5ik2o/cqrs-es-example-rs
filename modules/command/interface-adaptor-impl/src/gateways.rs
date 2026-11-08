use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod group_chat_read_model_dao_impl;
pub mod group_chat_repository;
pub mod recovery_task_store_impl;

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

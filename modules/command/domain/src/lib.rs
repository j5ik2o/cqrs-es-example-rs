use std::sync::Mutex;

use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use ulid_generator_rs::ULIDGenerator;

use crate::aggregate::AggregateId;

pub mod aggregate;
pub mod thread;
pub mod user_account;

static ID_GENERATOR: Lazy<Mutex<ULIDGenerator>> = Lazy::new(|| Mutex::new(ULIDGenerator::new()));

pub trait Event: std::fmt::Debug + Send + Sync {
    type ID: std::fmt::Display;
    type AggregateID: AggregateId;
    fn id(&self) -> &Self::ID;
    fn aggregate_id(&self) -> &Self::AggregateID;
    fn seq_nr(&self) -> usize;
    fn occurred_at(&self) -> &DateTime<Utc>;
    fn is_created(&self) -> bool;
}

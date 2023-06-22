use once_cell::sync::Lazy;
use std::sync::Mutex;
use ulid_generator_rs::ULIDGenerator;

pub mod aggregate;
pub mod events;
pub mod group_chat;

static ID_GENERATOR: Lazy<Mutex<ULIDGenerator>> = Lazy::new(|| Mutex::new(ULIDGenerator::new()));

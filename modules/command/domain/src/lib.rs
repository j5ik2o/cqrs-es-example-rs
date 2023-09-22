use std::sync::Mutex;

use once_cell::sync::Lazy;
use ulid_generator_rs::{ULIDGenerator, ULID};

pub mod aggregate;
pub mod group_chat;
pub mod user_account;

struct ULIDGeneratorState {
  generator: ULIDGenerator,
  last_id: Option<ULID>,
}

impl ULIDGeneratorState {
  fn new() -> Self {
    Self {
      generator: ULIDGenerator::new(),
      last_id: None,
    }
  }
}

static ID_GENERATOR_STATE: Lazy<Mutex<ULIDGeneratorState>> = Lazy::new(|| Mutex::new(ULIDGeneratorState::new()));

/// 初回以降の採番が衝突しない単調増加するIDを生成する。
pub fn id_generate() -> ULID {
  let mut state = ID_GENERATOR_STATE.lock().unwrap();
  match state.last_id {
    None => {
      let id = state.generator.generate().unwrap();
      state.last_id = Some(id);
      id
    }
    Some(last_id) => {
      let id = state.generator.generate_monotonic(&last_id).unwrap();
      state.last_id = Some(id);
      id
    }
  }
}


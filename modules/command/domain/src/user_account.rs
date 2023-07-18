use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use ulid_generator_rs::ULID;

use crate::ID_GENERATOR;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct UserAccountId {
  value: ULID,
}

impl UserAccountId {
  pub fn new() -> Self {
    let value = ID_GENERATOR.lock().unwrap().generate().unwrap();
    Self { value }
  }

  pub fn from_ulid(value: ULID) -> Self {
    Self { value }
  }
}

impl Display for UserAccountId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.value)
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserAccount {
  id: UserAccountId,
  user_name: String,
  password: String,
}

impl UserAccount {
  pub fn new(user_name: String, password: String) -> Self {
    let id = UserAccountId::new();
    Self {
      id,
      user_name,
      password,
    }
  }

  pub fn from(id: UserAccountId, user_name: String, password: String) -> Self {
    Self {
      id,
      user_name,
      password,
    }
  }
}

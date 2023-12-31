use crate::group_chat::MessageId;
use crate::user_account::UserAccountId;
use serde::{Deserialize, Serialize};

/// メッセージを表すローカルエンティティ。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
  id: MessageId,
  text: String,
  sender_id: UserAccountId,
}

impl Message {
  pub fn breach_encapsulation_of_id(&self) -> &MessageId {
    &self.id
  }

  pub fn breach_encapsulation_of_text(&self) -> &str {
    &self.text
  }

  pub fn breach_encapsulation_of_sender_id(&self) -> &UserAccountId {
    &self.sender_id
  }

  pub fn new(text: String, sender_id: UserAccountId) -> Self {
    let id = MessageId::new();
    Self { id, text, sender_id }
  }
}

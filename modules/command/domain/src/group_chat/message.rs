use crate::group_chat::MessageId;
use crate::user_account::UserAccountId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

  pub fn with_text(mut self, text: String) -> Self {
    self.text = text;
    self
  }

  pub fn breach_encapsulation_of_sender_id(&self) -> &UserAccountId {
    &self.sender_id
  }

  pub fn new(id: MessageId, text: String, sender_id: UserAccountId) -> Self {
    Self { id, text, sender_id }
  }

  pub fn validate(text: &str, message_id: MessageId, sender_id: UserAccountId) -> Result<Self, MessageError> {
    if text.is_empty() {
      return Err(MessageError::Empty);
    }
    if text.len() > 1000 {
      return Err(MessageError::TooLong);
    }
    Ok(Message::new(message_id, text.to_string(), sender_id))
  }
}

#[derive(Debug, Clone, Error)]
pub enum MessageError {
  #[error("the message is empty")]
  Empty,
  #[error("the message is too long")]
  TooLong,
}

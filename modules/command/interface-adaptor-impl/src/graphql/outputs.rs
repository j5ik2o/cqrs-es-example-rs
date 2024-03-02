use async_graphql::SimpleObject;

#[derive(Debug, Clone, SimpleObject)]
pub struct GroupChatOut {
  group_chat_id: String,
}

impl GroupChatOut {
  pub fn new(group_chat_id: String) -> Self {
    Self { group_chat_id }
  }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct MessageOut {
  group_chat_id: String,
  message_id: String,
}

impl MessageOut {
  pub fn new(group_chat_id: String, message_id: String) -> Self {
    Self {
      group_chat_id,
      message_id,
    }
  }
}

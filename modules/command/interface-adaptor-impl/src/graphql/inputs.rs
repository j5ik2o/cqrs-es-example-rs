use async_graphql::InputObject;

#[derive(Debug, Clone, InputObject)]
pub struct CreateGroupChatInput {
  pub name: String,
  pub executor_id: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct DeleteGroupChatInput {
  pub group_chat_id: String,
  pub executor_id: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct RenameGroupChatInput {
  pub group_chat_id: String,
  pub name: String,
  pub executor_id: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct AddMemberInput {
  pub group_chat_id: String,
  pub user_account_id: String,
  pub role: String,
  pub executor_id: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct RemoveMemberInput {
  pub group_chat_id: String,
  pub user_account_id: String,
  pub executor_id: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct PostMessageInput {
  pub group_chat_id: String,
  pub content: String,
  pub executor_id: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct EditMessageInput {
  pub group_chat_id: String,
  pub content: String,
  pub executor_id: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct DeleteMessageInput {
  pub group_chat_id: String,
  pub message_id: String,
  pub executor_id: String,
}

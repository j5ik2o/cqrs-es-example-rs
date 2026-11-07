use async_graphql::{Enum, InputObject};

/// 共通スキーマが定めるメンバーの役割。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum MemberRole {
  #[graphql(name = "MEMBER")]
  Member,
  #[graphql(name = "ADMIN")]
  Admin,
}

impl MemberRole {
  pub fn into_domain(self) -> command_domain::group_chat::MemberRole {
    match self {
      MemberRole::Member => command_domain::group_chat::MemberRole::Member,
      MemberRole::Admin => command_domain::group_chat::MemberRole::Admin,
    }
  }
}

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
  pub role: MemberRole,
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
  pub message_id: String,
  pub content: String,
  pub executor_id: String,
}

#[derive(Debug, Clone, InputObject)]
pub struct DeleteMessageInput {
  pub group_chat_id: String,
  pub message_id: String,
  pub executor_id: String,
}

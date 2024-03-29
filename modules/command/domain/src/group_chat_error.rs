use crate::group_chat::{GroupChatId, GroupChatName, MessageId};
use crate::user_account::UserAccountId;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GroupChatError {
  #[error("The group chat is deleted: {0:?}")]
  AlreadyDeletedError(GroupChatId),
  #[error("The {0} is not an administrator of the group chat: {1:?}")]
  NotAdministratorError(String, UserAccountId),
  #[error("The {0} is not a member of the group chat: {1:?}")]
  NotMemberError(String, UserAccountId),
  #[error("The {0} is already a member of the group chat: {1:?}")]
  AlreadyMemberError(String, UserAccountId),
  #[error("Both {0} and {1} are not mismatched")]
  MismatchedUserAccountError(String, String),
  #[error("The message is already exist: {0:?}")]
  AlreadyExistsMessageError(MessageId),
  #[error("The message is not found: {0:?}")]
  NotFoundMessageError(MessageId),
  #[error("This {0} is not the sender of the message: {1:?}")]
  NotSenderError(String, UserAccountId),
  #[error("The group chat name is already exists: {0:?}, {1:?}")]
  AlreadyExistsNameError(GroupChatId, GroupChatName),
}

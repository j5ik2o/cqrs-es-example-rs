use crate::user_account::UserAccountId;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum UserAccountError {
  #[error("The user account is deleted: {0:?}")]
  AlreadyDeletedError(UserAccountId),
}

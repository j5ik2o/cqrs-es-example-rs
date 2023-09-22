use serde::{Deserialize, Serialize};

use crate::group_chat::member_id::MemberId;
use crate::group_chat::member_role::MemberRole;
use crate::user_account::UserAccountId;

/// メンバー。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Member {
  id: MemberId,
  user_account_id: UserAccountId,
  role: MemberRole,
}

impl Member {
  pub fn breach_encapsulation_of_id(&self) -> &MemberId {
    &self.id
  }

  pub fn breach_encapsulation_of_user_account_id(&self) -> &UserAccountId {
    &self.user_account_id
  }

  pub fn breach_encapsulation_of_role(&self) -> &MemberRole {
    &self.role
  }

  /// コンストラクタ。
  ///
  /// # 引数
  /// - `id` - [MemberId]
  /// - `user_account_id` - [UserAccountId]
  /// - `role` - [MemberRole]
  ///
  /// # 戻り値
  /// - [Member]
  pub fn new(id: MemberId, user_account_id: UserAccountId, role: MemberRole) -> Self {
    Self {
      id,
      user_account_id,
      role,
    }
  }
}

impl PartialOrd for Member {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.id.partial_cmp(&other.id)
  }
}

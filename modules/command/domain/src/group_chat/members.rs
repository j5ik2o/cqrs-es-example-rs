use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::group_chat::{Member, MemberId, MemberRole};
use crate::user_account::UserAccountId;

/// メンバー集合。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Members {
  members_ids_by_user_account_id: BTreeMap<String, MemberId>,
  members: BTreeMap<String, Member>,
}

impl Members {
  /// コンストラクタ。
  ///
  /// # 引数
  /// - `administrator_id` - 管理者のユーザアカウントID
  ///
  /// # 戻り値
  /// - [Members]
  pub fn new(administrator_id: UserAccountId) -> Self {
    let mut my_self = Self {
      members_ids_by_user_account_id: BTreeMap::new(),
      members: BTreeMap::new(),
    };
    my_self.add_member(Member::new(MemberId::new(), administrator_id, MemberRole::Admin));
    my_self
  }

  /// 最初の管理者を取得する。
  pub fn administrator_id(&self) -> &Member {
    self
      .members
      .iter()
      .find(|(_, member)| *member.breach_encapsulation_of_role() == MemberRole::Admin)
      .unwrap()
      .1
  }

  /// 管理者かどうかを判定する。
  pub fn is_administrator(&self, user_account_id: &UserAccountId) -> bool {
    self.is_role(user_account_id, &[MemberRole::Admin])
  }

  /// メンバーかどうかを判定する。
  pub fn is_member(&self, user_account_id: &UserAccountId) -> bool {
    self.is_role(user_account_id, &[MemberRole::Member, MemberRole::Admin])
  }

  /// ロールを判定する。
  pub fn is_role(&self, user_account_id: &UserAccountId, roles: &[MemberRole]) -> bool {
    if let Some(member_id) = self.members_ids_by_user_account_id.get(&user_account_id.to_string()) {
      if let Some(member) = self.members.get(&member_id.to_string()) {
        return roles.contains(member.breach_encapsulation_of_role());
      }
    }
    false
  }

  /// メンバーを追加する。
  pub fn add_member(&mut self, member: Member) {
    self
      .members
      .insert(member.breach_encapsulation_of_id().to_string(), member.clone());
    self.members_ids_by_user_account_id.insert(
      member.breach_encapsulation_of_user_account_id().to_string(),
      member.breach_encapsulation_of_id().clone(),
    );
  }

  /// 指定したメンバーIDのメンバーを取得する。
  pub fn find_by_id(&self, member_id: &MemberId) -> Option<&Member> {
    self.members.get(&member_id.to_string())
  }

  /// 指定したユーザアカウントIDのメンバーを取得する。
  pub fn find_by_user_account_id(&self, user_account_id: &UserAccountId) -> Option<&Member> {
    if let Some(member_id) = self.members_ids_by_user_account_id.get(&user_account_id.to_string()) {
      self.find_by_id(member_id)
    } else {
      None
    }
  }

  /// 指定したメンバーIDのメンバーを削除する。
  pub fn remove_member(&mut self, member_id: &MemberId) {
    if let Some(member) = self.members.remove(&member_id.to_string()) {
      self
        .members_ids_by_user_account_id
        .remove(&member.breach_encapsulation_of_user_account_id().to_string());
    }
  }

  /// 指定したユーザアカウントのメンバーを削除する。
  pub fn remove_member_by_user_account_id(&mut self, user_account_id: &UserAccountId) {
    if let Some(member_id) = self.members_ids_by_user_account_id.remove(&user_account_id.to_string()) {
      self.members.remove(&member_id.to_string());
    }
  }

  /// メンバーの一覧を取得する。
  pub fn to_vec(&self) -> Vec<&Member> {
    self.members.values().collect()
  }
}

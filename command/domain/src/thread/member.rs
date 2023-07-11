use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use ulid_generator_rs::ULID;

use crate::thread::MemberRole;
use crate::user_account::UserAccountId;
use crate::ID_GENERATOR;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct MemberId(ULID);

impl MemberId {
  pub fn new() -> Self {
    let value = ID_GENERATOR.lock().unwrap().generate().unwrap();
    Self(value)
  }
}

impl Display for MemberId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Member {
  pub id: MemberId,
  pub user_account_id: UserAccountId,
  pub role: MemberRole,
}

impl Member {
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
    self.id.0.partial_cmp(&other.id.0)
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Members {
  members_ids_by_user_account_id: BTreeMap<String, MemberId>,
  members: BTreeMap<String, Member>,
}

impl Members {
  pub fn new(administrator_id: UserAccountId) -> Self {
    let mut my_self = Self {
      members_ids_by_user_account_id: BTreeMap::new(),
      members: BTreeMap::new(),
    };
    my_self.add_member(Member::new(MemberId::new(), administrator_id, MemberRole::Admin));
    my_self
  }

  pub fn administrator_id(&self) -> &Member {
    self
      .members
      .iter()
      .find(|(_, member)| member.role == MemberRole::Admin)
      .unwrap()
      .1
  }

  pub fn is_administrator(&self, user_account_id: &UserAccountId) -> bool {
    self.is_role(user_account_id, &[MemberRole::Admin])
  }

  pub fn is_member(&self, user_account_id: &UserAccountId) -> bool {
    self.is_role(user_account_id, &[MemberRole::Member, MemberRole::Admin])
  }

  pub fn is_role(&self, user_account_id: &UserAccountId, roles: &[MemberRole]) -> bool {
    if let Some(member_id) = self.members_ids_by_user_account_id.get(&user_account_id.to_string()) {
      if let Some(member) = self.members.get(&member_id.to_string()) {
        return roles.contains(&member.role);
      }
    }
    false
  }

  pub fn add_member(&mut self, member: Member) {
    self.members.insert(member.id.to_string(), member.clone());
    self
      .members_ids_by_user_account_id
      .insert(member.user_account_id.to_string(), member.id);
  }

  pub fn find_by_id(&self, member_id: &MemberId) -> Option<&Member> {
    self.members.get(&member_id.to_string())
  }

  pub fn find_by_user_account_id(&self, user_account_id: &UserAccountId) -> Option<&Member> {
    if let Some(member_id) = self.members_ids_by_user_account_id.get(&user_account_id.to_string()) {
      self.find_by_id(member_id)
    } else {
      None
    }
  }

  pub fn remove_member(&mut self, member_id: &MemberId) {
    if let Some(member) = self.members.remove(&member_id.to_string()) {
      self
        .members_ids_by_user_account_id
        .remove(&member.user_account_id.to_string());
    }
  }

  pub fn remove_member_by_user_account_id(&mut self, user_account_id: &UserAccountId) {
    if let Some(member_id) = self.members_ids_by_user_account_id.remove(&user_account_id.to_string()) {
      self.members.remove(&member_id.to_string());
    }
  }

  pub fn values(&self) -> Vec<&Member> {
    self.members.values().collect()
  }
}

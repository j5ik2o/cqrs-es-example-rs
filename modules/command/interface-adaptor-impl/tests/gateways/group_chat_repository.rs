/// [GroupChatRepository]のテスト
#[cfg(test)]
mod tests {
  use super::super::super::common::*;
  use command_domain::group_chat::{GroupChat, GroupChatName, MemberRole};
  use command_domain::group_chat::{MemberId, Members};
  use command_domain::user_account::UserAccountId;
  use command_interface_adaptor_if::GroupChatRepository;
  use event_store_adapter_rs::types::Aggregate;
  use testcontainers::clients::Cli;

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  async fn test() {
    test_group_chat_create().await;
    test_group_chat_add_member().await;
  }

  async fn test_group_chat_create() {
    let docker = Cli::default();
    let (mut repository, container, client) = get_repository(&docker).await;
    // Given
    let name = GroupChatName::new("ABC").unwrap();
    let admin_id = UserAccountId::new();
    let members = Members::new(admin_id.clone());

    // When
    let (group_chat, create_event) = GroupChat::new(name, members);
    let result = repository.store(&create_event, 1, Some(&group_chat)).await;
    assert!(result.is_ok());

    let actual = repository.find_by_id(group_chat.id()).await.unwrap().unwrap();
    assert_eq!(actual.id(), group_chat.id());
    assert_eq!(actual.name(), group_chat.name());
    assert!(actual.members().is_member(&admin_id));

    drop(client);
    container.stop();
    drop(container);
  }

  async fn test_group_chat_add_member() {
    let docker = Cli::default();
    let (mut repository, container, client) = get_repository(&docker).await;
    let name = GroupChatName::new("ABC").unwrap();
    let admin_user_account_id = UserAccountId::new();
    let user_account_id = UserAccountId::new();
    let members = Members::new(admin_user_account_id.clone());

    let (actual, create_event) = GroupChat::new(name, members);
    let result = repository.store(&create_event, 1, Some(&actual)).await;
    assert!(result.is_ok());

    let mut actual = repository.find_by_id(actual.id()).await.unwrap().unwrap();
    let member_id = MemberId::new();
    let add_member_event = actual
      .add_member(
        member_id,
        user_account_id.clone(),
        MemberRole::Member,
        admin_user_account_id.clone(),
      )
      .unwrap();
    let result = repository.store(&add_member_event, actual.version(), None).await;
    assert!(result.is_ok());

    let mut actual = repository.find_by_id(actual.id()).await.unwrap().unwrap();
    let member_id = MemberId::new();
    let user_account_id2 = UserAccountId::new();
    let add_member_event = actual
      .add_member(
        member_id,
        user_account_id2.clone(),
        MemberRole::Member,
        admin_user_account_id.clone(),
      )
      .unwrap();

    let result = repository.store(&add_member_event, actual.version(), None).await;
    assert!(result.is_ok());

    let actual = repository.find_by_id(actual.id()).await.unwrap().unwrap();
    assert!(actual.members().is_administrator(&admin_user_account_id));
    assert!(actual.members().is_member(&user_account_id));

    drop(client);
    container.stop();
    drop(container);
  }
}

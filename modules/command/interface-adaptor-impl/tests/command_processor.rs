use serial_test::serial;
use testcontainers::clients;

use command_domain::group_chat::Members;
use command_domain::group_chat::{GroupChatName, MemberRole, Message};
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::*;
use command_processor::group_chat_command_processor::GroupChatCommandProcessor;
use common::*;

mod common;

#[tokio::test]
#[serial]
async fn test_group_chat_create() {
  let docker = DOCKER.get_or_init(clients::Cli::default);

  let (mut repository, container, client) = get_repository(&docker).await;
  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let _members = Members::new(admin_id.clone());
  let mut command_processor = GroupChatCommandProcessor::new(repository);
  // When
  let result = command_processor.create_group_chat(name, admin_id).await;

  // Then
  assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_group_chat_rename() {
  let docker = DOCKER.get_or_init(clients::Cli::default);

  let (repository, container, client) = get_repository(docker).await;
  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let _members = Members::new(admin_id.clone());
  let mut command_processor = GroupChatCommandProcessor::new(repository.clone());

  let id = command_processor
    .create_group_chat(name.clone(), admin_id.clone())
    .await
    .unwrap();

  // When
  let name = GroupChatName::new("DEF").unwrap();
  let result = command_processor
    .rename_group_chat(id.clone(), name.clone(), admin_id)
    .await;

  // Then
  assert!(result.is_ok());
  let group_chat = repository.find_by_id(&id).await.unwrap().unwrap();
  assert_eq!(*group_chat.name(), name);
}

#[tokio::test]
#[serial]
async fn test_group_chat_add_member() {
  let docker = DOCKER.get_or_init(clients::Cli::default);

  let (repository, container, client) = get_repository(docker).await;
  // with_repository(|mut repository| async move {
  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let _members = Members::new(admin_id.clone());
  let mut command_processor = GroupChatCommandProcessor::new(repository.clone());
  let id = command_processor
    .create_group_chat(name.clone(), admin_id.clone())
    .await
    .unwrap();
  let user_account_id = UserAccountId::new();
  let role = MemberRole::Member;

  // When
  let result = command_processor
    .add_member(id.clone(), user_account_id.clone(), role, admin_id.clone())
    .await;

  // Then
  assert!(result.is_ok());
  let group_chat = repository.find_by_id(&id).await.unwrap().unwrap();
  assert!(group_chat.members().is_administrator(&admin_id));
  assert!(group_chat.members().is_member(&user_account_id));
}

#[tokio::test]
#[serial]
async fn test_group_chat_remove_member() {
  let docker = DOCKER.get_or_init(clients::Cli::default);

  let (repository, container, client) = get_repository(docker).await;
  let user_account_id = UserAccountId::new();
  let admin_id = UserAccountId::new();

  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let _members = Members::new(admin_id.clone());
  let mut command_processor = GroupChatCommandProcessor::new(repository.clone());
  let id = command_processor
    .create_group_chat(name.clone(), admin_id.clone())
    .await
    .unwrap();
  command_processor
    .add_member(
      id.clone(),
      user_account_id.clone(),
      MemberRole::Member,
      admin_id.clone(),
    )
    .await
    .unwrap();

  // When
  let result = command_processor
    .remove_member(id.clone(), user_account_id.clone(), admin_id.clone())
    .await;

  // Then
  assert!(result.is_ok());
  let group_chat = repository.find_by_id(&id).await.unwrap().unwrap();
  assert!(group_chat.members().is_administrator(&admin_id));
  assert!(!group_chat.members().is_member(&user_account_id));
}

#[tokio::test]
#[serial]
async fn test_group_chat_post_message() {
  let docker = DOCKER.get_or_init(clients::Cli::default);

  let (repository, container, client) = get_repository(docker).await;
  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let _members = Members::new(admin_id.clone());
  let mut command_processor = GroupChatCommandProcessor::new(repository.clone());
  let id = command_processor
    .create_group_chat(name.clone(), admin_id.clone())
    .await
    .unwrap();
  let user_account_id = UserAccountId::new();
  let role = MemberRole::Member;
  command_processor
    .add_member(id.clone(), user_account_id.clone(), role, admin_id.clone())
    .await
    .unwrap();
  let text = "ABC".to_string();
  let message = Message::new(text.clone(), user_account_id.clone());

  // When
  let result = command_processor
    .post_message(id.clone(), message, user_account_id.clone())
    .await;

  // Then
  assert!(result.is_ok());
  let group_chat = repository.find_by_id(&id).await.unwrap().unwrap();
  assert_eq!(group_chat.messages().len(), 1);
  assert_eq!(
    group_chat.messages().get_at(0).unwrap().breach_encapsulation_of_text(),
    text
  );
}

#[tokio::test]
#[serial]
async fn test_group_chat_delete_message() {
  let docker = DOCKER.get_or_init(clients::Cli::default);

  let (repository, container, client) = get_repository(docker).await;
  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let _members = Members::new(admin_id.clone());
  let mut command_processor = GroupChatCommandProcessor::new(repository.clone());
  let id = command_processor
    .create_group_chat(name.clone(), admin_id.clone())
    .await
    .unwrap();
  let user_account_id = UserAccountId::new();
  let role = MemberRole::Member;
  command_processor
    .add_member(id.clone(), user_account_id.clone(), role, admin_id.clone())
    .await
    .unwrap();
  let text = "ABC".to_string();
  let message = Message::new(text.clone(), user_account_id.clone());
  command_processor
    .post_message(id.clone(), message.clone(), user_account_id.clone())
    .await
    .unwrap();

  // When
  let result = command_processor
    .delete_message(
      id.clone(),
      message.breach_encapsulation_of_id().clone(),
      user_account_id.clone(),
    )
    .await;

  // Then
  assert!(result.is_ok());
  let group_chat = repository.find_by_id(&id).await.unwrap().unwrap();
  assert_eq!(group_chat.messages().len(), 0);
}

#[tokio::test]
#[serial]
async fn test_group_chat_destroy() {
  init_logger();
  let docker = DOCKER.get_or_init(clients::Cli::default);

  let (repository, container, client) = get_repository(docker).await;
  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let _members = Members::new(admin_id.clone());
  let mut command_processor = GroupChatCommandProcessor::new(repository.clone());
  let id = command_processor
    .create_group_chat(name, admin_id.clone())
    .await
    .unwrap();
  // When
  let result = command_processor.delete_group_chat(id, admin_id).await;

  // Then
  assert!(result.is_ok());
}

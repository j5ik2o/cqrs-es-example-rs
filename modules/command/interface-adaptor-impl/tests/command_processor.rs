use command_domain::group_chat::Members;
use command_domain::group_chat::{GroupChatName, MemberRole, Message};
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::*;
use command_interface_adaptor_impl::controllers::GroupChatIdPresenter;
use command_processor::command_processor::GroupChatCommandProcessor;
use common::*;
use testcontainers::clients::Cli;

mod common;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test() {
    let docker = Cli::default();
  test_group_chat_create(&docker).await;
  test_group_chat_rename(&docker).await;
  test_group_chat_add_member(&docker).await;
  test_group_chat_remove_member(&docker).await;
  // test_group_chat_post_message(&docker).await;
  // test_group_chat_delete_message(&docker).await;
  test_group_chat_destroy(&docker).await;
}

async fn test_group_chat_create(docker: &Cli) {
  let (mut repository, container, client) = get_repository(&docker).await;
  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let _members = Members::new(admin_id.clone());
  let mut command_processor = GroupChatCommandProcessor::new(&mut repository);
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();
  // When
  let result = command_processor
    .create_group_chat(&mut group_chat_id_presenter, name, admin_id)
    .await;

  // Then
  assert!(result.is_ok());
}

async fn test_group_chat_rename(docker: &Cli) {
  let (mut repository, container, client) = get_repository(docker).await;
  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let _members = Members::new(admin_id.clone());
  let mut command_processor = GroupChatCommandProcessor::new(&mut repository);
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();

  command_processor
    .create_group_chat(&mut group_chat_id_presenter, name.clone(), admin_id.clone())
    .await
    .unwrap();
  let id = group_chat_id_presenter.group_chat_id();

  // When
  let name = GroupChatName::new("DEF").unwrap();
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();
  let result = command_processor
    .rename_group_chat(&mut group_chat_id_presenter, id.clone(), name.clone(), admin_id)
    .await;

  // Then
  assert!(result.is_ok());
  let group_chat = repository.find_by_id(id).await.unwrap().unwrap();
  assert_eq!(*group_chat.name(), name);

}

async fn test_group_chat_add_member(docker: &Cli) {
  let (mut repository, container, client) = get_repository(docker).await;
  // with_repository(|mut repository| async move {
  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let _members = Members::new(admin_id.clone());
  let mut command_processor = GroupChatCommandProcessor::new(&mut repository);
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();
  command_processor
    .create_group_chat(&mut group_chat_id_presenter, name.clone(), admin_id.clone())
    .await
    .unwrap();
  let id = group_chat_id_presenter.group_chat_id();
  let user_account_id = UserAccountId::new();
  let role = MemberRole::Member;
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();

  // When
  let result = command_processor
    .add_member(
      &mut group_chat_id_presenter,
      id.clone(),
      user_account_id.clone(),
      role,
      admin_id.clone(),
    )
    .await;

  // Then
  assert!(result.is_ok());
  let group_chat = repository.find_by_id(id).await.unwrap().unwrap();
  assert!(group_chat.members().is_administrator(&admin_id));
  assert!(group_chat.members().is_member(&user_account_id));

}

async fn test_group_chat_remove_member(docker: &Cli) {
  let (mut repository, container, client) = get_repository(docker).await;
  let user_account_id = UserAccountId::new();
  let admin_id = UserAccountId::new();

  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let _members = Members::new(admin_id.clone());
  let mut command_processor = GroupChatCommandProcessor::new(&mut repository);
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();
  command_processor
    .create_group_chat(&mut group_chat_id_presenter, name.clone(), admin_id.clone())
    .await
    .unwrap();
  let id = group_chat_id_presenter.group_chat_id();
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();
  command_processor
    .add_member(
      &mut group_chat_id_presenter,
      id.clone(),
      user_account_id.clone(),
      MemberRole::Member,
      admin_id.clone(),
    )
    .await
    .unwrap();
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();

  // When
  let result = command_processor
    .remove_member(
      &mut group_chat_id_presenter,
      id.clone(),
      user_account_id.clone(),
      admin_id.clone(),
    )
    .await;

  // Then
  assert!(result.is_ok());
  let group_chat = repository.find_by_id(id).await.unwrap().unwrap();
  assert!(group_chat.members().is_administrator(&admin_id));
  assert!(!group_chat.members().is_member(&user_account_id));

}

async fn test_group_chat_post_message(docker: &Cli) {
  let (mut repository, container, client) = get_repository(docker).await;
  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let _members = Members::new(admin_id.clone());
  let mut command_processor = GroupChatCommandProcessor::new(&mut repository);
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();
  command_processor
    .create_group_chat(&mut group_chat_id_presenter, name.clone(), admin_id.clone())
    .await
    .unwrap();
  let id = group_chat_id_presenter.group_chat_id();
  let user_account_id = UserAccountId::new();
  let role = MemberRole::Member;
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();
  command_processor
    .add_member(
      &mut group_chat_id_presenter,
      id.clone(),
      user_account_id.clone(),
      role,
      admin_id.clone(),
    )
    .await
    .unwrap();
  let text = "ABC".to_string();
  let message = Message::new(text.clone(), user_account_id.clone());
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();

  // When
  let result = command_processor
    .post_message(
      &mut group_chat_id_presenter,
      id.clone(),
      message,
      user_account_id.clone(),
    )
    .await;

  // Then
  assert!(result.is_ok());
  let group_chat = repository.find_by_id(id).await.unwrap().unwrap();
  assert_eq!(group_chat.messages().len(), 1);
  assert_eq!(
    group_chat.messages().get_at(0).unwrap().breach_encapsulation_of_text(),
    text
  );

}

async fn test_group_chat_delete_message(docker: &Cli) {
  let (mut repository, container, client) = get_repository(docker).await;
  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let _members = Members::new(admin_id.clone());
  let mut command_processor = GroupChatCommandProcessor::new(&mut repository);
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();
  command_processor
    .create_group_chat(&mut group_chat_id_presenter, name.clone(), admin_id.clone())
    .await
    .unwrap();
  let id = group_chat_id_presenter.group_chat_id();
  let user_account_id = UserAccountId::new();
  let role = MemberRole::Member;
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();
  command_processor
    .add_member(
      &mut group_chat_id_presenter,
      id.clone(),
      user_account_id.clone(),
      role,
      admin_id.clone(),
    )
    .await
    .unwrap();
  let text = "ABC".to_string();
  let message = Message::new(text.clone(), user_account_id.clone());
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();
  command_processor
    .post_message(
      &mut group_chat_id_presenter,
      id.clone(),
      message.clone(),
      user_account_id.clone(),
    )
    .await
    .unwrap();
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();

  // When
  let result = command_processor
    .delete_message(
      &mut group_chat_id_presenter,
      id.clone(),
      message.breach_encapsulation_of_id().clone(),
      user_account_id.clone(),
    )
    .await;

  // Then
  assert!(result.is_ok());
  let group_chat = repository.find_by_id(id).await.unwrap().unwrap();
  assert_eq!(group_chat.messages().len(), 0);

}

async fn test_group_chat_destroy(docker: &Cli) {
  let (mut repository, container, client) = get_repository(docker).await;
  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let _members = Members::new(admin_id.clone());
  let mut command_processor = GroupChatCommandProcessor::new(&mut repository);
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();
  command_processor
    .create_group_chat(&mut group_chat_id_presenter, name, admin_id.clone())
    .await
    .unwrap();
  let id = group_chat_id_presenter.group_chat_id().clone();
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();
  // When
  let result = command_processor
    .delete_group_chat(&mut group_chat_id_presenter, id, admin_id)
    .await;

  // Then
  assert!(result.is_ok());

}

use common::*;
use cqrs_es_example_command_interface_adaptor_if::ThreadRepository;
use cqrs_es_example_command_processor::command_processor::ThreadCommandProcessor;
use cqrs_es_example_domain::thread::member::Members;
use cqrs_es_example_domain::thread::{MemberRole, Message, ThreadName};
use cqrs_es_example_domain::user_account::UserAccountId;

mod common;

#[tokio::test]
async fn test_thread_create() {
  with_repository(|mut repository| async move {
    // Given
    let name = ThreadName::new("ABC".to_string());
    let admin_id = UserAccountId::new();
    let _members = Members::new(admin_id.clone());
    let mut command_processor = ThreadCommandProcessor::new(&mut repository);

    // When
    let result = command_processor.create_thread(name, admin_id).await;

    // Then
    assert!(result.is_ok());
  })
  .await;
}

#[tokio::test]
async fn test_thread_rename() {
  with_repository(|mut repository| async move {
    // Given
    let name = ThreadName::new("ABC".to_string());
    let admin_id = UserAccountId::new();
    let _members = Members::new(admin_id.clone());
    let mut command_processor = ThreadCommandProcessor::new(&mut repository);
    let id = command_processor
      .create_thread(name.clone(), admin_id.clone())
      .await
      .unwrap();

    // When
    let name = ThreadName::new("DEF".to_string());
    let result = command_processor
      .rename_thread(id.clone(), name.clone(), admin_id)
      .await;

    // Then
    assert!(result.is_ok());
    let thread = repository.find_by_id(&id).await.unwrap();
    assert_eq!(*thread.name(), name);
  })
  .await;
}

#[tokio::test]
async fn test_thread_add_member() {
  with_repository(|mut repository| async move {
    // Given
    let name = ThreadName::new("ABC".to_string());
    let admin_id = UserAccountId::new();
    let _members = Members::new(admin_id.clone());
    let mut command_processor = ThreadCommandProcessor::new(&mut repository);
    let id = command_processor
      .create_thread(name.clone(), admin_id.clone())
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
    let thread = repository.find_by_id(&id).await.unwrap();
    assert!(thread.members().is_administrator(&admin_id));
    assert!(thread.members().is_member(&user_account_id));
  })
  .await;
}

#[tokio::test]
async fn test_thread_remove_member() {
  with_repository(|mut repository| async move {
    let user_account_id = UserAccountId::new();
    let admin_id = UserAccountId::new();

    // Given
    let name = ThreadName::new("ABC".to_string());
    let _members = Members::new(admin_id.clone());
    let mut command_processor = ThreadCommandProcessor::new(&mut repository);
    let id = command_processor
      .create_thread(name.clone(), admin_id.clone())
      .await
      .unwrap();
    let _ = command_processor
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
    let thread = repository.find_by_id(&id).await.unwrap();
    assert!(thread.members().is_administrator(&admin_id));
    assert!(!thread.members().is_member(&user_account_id));
  })
  .await;
}

#[tokio::test]
async fn test_thread_post_message() {
  with_repository(|mut repository| async move {
    // Given
    let name = ThreadName::new("ABC".to_string());
    let admin_id = UserAccountId::new();
    let _members = Members::new(admin_id.clone());
    let mut command_processor = ThreadCommandProcessor::new(&mut repository);
    let id = command_processor
      .create_thread(name.clone(), admin_id.clone())
      .await
      .unwrap();
    let user_account_id = UserAccountId::new();
    let role = MemberRole::Member;
    let _ = command_processor
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
    let thread = repository.find_by_id(&id).await.unwrap();
    assert_eq!(thread.messages().len(), 1);
    assert_eq!(thread.messages().get_at(0).unwrap().text, text);
  })
  .await;
}

#[tokio::test]
async fn test_thread_delete_message() {
  with_repository(|mut repository| async move {
    // Given
    let name = ThreadName::new("ABC".to_string());
    let admin_id = UserAccountId::new();
    let _members = Members::new(admin_id.clone());
    let mut command_processor = ThreadCommandProcessor::new(&mut repository);
    let id = command_processor
      .create_thread(name.clone(), admin_id.clone())
      .await
      .unwrap();
    let user_account_id = UserAccountId::new();
    let role = MemberRole::Member;
    let _ = command_processor
      .add_member(id.clone(), user_account_id.clone(), role, admin_id.clone())
      .await
      .unwrap();
    let text = "ABC".to_string();
    let message = Message::new(text.clone(), user_account_id.clone());
    let _ = command_processor
      .post_message(id.clone(), message.clone(), user_account_id.clone())
      .await
      .unwrap();

    // When
    let result = command_processor
      .delete_message(id.clone(), message.id, user_account_id.clone())
      .await;

    // Then
    assert!(result.is_ok());
    let thread = repository.find_by_id(&id).await.unwrap();
    assert_eq!(thread.messages().len(), 0);
  })
  .await;
}

#[tokio::test]
async fn test_thread_destroy() {
  with_repository(|mut repository| async move {
    // Given
    let name = ThreadName::new("ABC".to_string());
    let admin_id = UserAccountId::new();
    let _members = Members::new(admin_id.clone());
    let mut command_processor = ThreadCommandProcessor::new(&mut repository);
    let id = command_processor.create_thread(name, admin_id.clone()).await.unwrap();

    // When
    let result = command_processor.delete_thread(id, admin_id).await;

    // Then
    assert!(result.is_ok());
  })
  .await;
}

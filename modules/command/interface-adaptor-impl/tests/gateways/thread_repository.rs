use cqrs_es_example_command_interface_adaptor_if::ThreadRepository;
use cqrs_es_example_domain::aggregate::Aggregate;
use cqrs_es_example_domain::thread::member::{MemberId, Members};
use cqrs_es_example_domain::thread::{MemberRole, Thread, ThreadName};
use cqrs_es_example_domain::user_account::UserAccountId;

use super::super::common::*;

#[tokio::test]
async fn test_thread_create() {
  with_repository(|mut repository| async move {
    // Given
    let name = ThreadName::new("ABC".to_string());
    let admin_id = UserAccountId::new();
    let members = Members::new(admin_id.clone());

    // When
    let (thread, create_event) = Thread::new(name, members);
    let result = repository.store(&create_event, 1, Some(&thread)).await;
    assert!(result.is_ok());

    let actual = repository.find_by_id(thread.id()).await.unwrap();
    assert_eq!(actual.id(), thread.id());
    assert_eq!(actual.name(), thread.name());
    assert!(actual.members().is_member(&admin_id));
  })
  .await;
}

#[tokio::test]
async fn test_thread_add_member() {
  with_repository(|mut repository| async move {
    let name = ThreadName::new("ABC".to_string());
    let admin_user_account_id = UserAccountId::new();
    let user_account_id = UserAccountId::new();
    let members = Members::new(admin_user_account_id.clone());

    let (actual, create_event) = Thread::new(name, members);
    let result = repository.store(&create_event, 1, Some(&actual)).await;
    assert!(result.is_ok());

    let mut actual = repository.find_by_id(actual.id()).await.unwrap();
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

    let mut actual = repository.find_by_id(actual.id()).await.unwrap();
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

    let actual = repository.find_by_id(actual.id()).await.unwrap();
    assert!(actual.members().is_administrator(&admin_user_account_id));
    assert!(actual.members().is_member(&user_account_id));
  })
  .await;
}

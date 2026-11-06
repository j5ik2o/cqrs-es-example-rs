use crate::common::{get_repository, get_repository_and_store, init_logger};
use command_domain::group_chat::{GroupChat, GroupChatEvent, GroupChatId, GroupChatName, MemberRole};
use command_domain::group_chat::{MemberId, Members};
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::GroupChatRepository;
use event_store_adapter_rs::{EventEnvelope, SeqNr};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_group_chat_create() {
  init_logger();
  let (mut repository, container, client) = get_repository().await;
  // Given
  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let members = Members::new(admin_id.clone());

  // When
  let (group_chat, create_event) = GroupChat::new(name, members);
  let result = repository.store(&create_event, &group_chat).await;
  assert!(result.is_ok());

  let actual = repository.find_by_id(group_chat.id()).await.unwrap().unwrap();
  assert_eq!(actual.id(), group_chat.id());
  assert_eq!(actual.name(), group_chat.name());
  assert!(actual.members().is_member(&admin_id));

  drop(client);
  let _ = container.stop().await;
  drop(container);
}

#[tokio::test]
#[serial]
async fn test_group_chat_add_member() {
  init_logger();
  let (mut repository, container, client) = get_repository().await;
  let name = GroupChatName::new("ABC").unwrap();
  let admin_user_account_id = UserAccountId::new();
  let user_account_id = UserAccountId::new();
  let members = Members::new(admin_user_account_id.clone());

  let (actual, create_event) = GroupChat::new(name, members);
  let result = repository.store(&create_event, &actual).await;
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
  let result = repository.store(&add_member_event, &actual).await;
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

  let result = repository.store(&add_member_event, &actual).await;
  assert!(result.is_ok());

  let actual = repository.find_by_id(actual.id()).await.unwrap().unwrap();
  assert!(actual.members().is_administrator(&admin_user_account_id));
  assert!(actual.members().is_member(&user_account_id));

  drop(client);
  let _ = container.stop().await.unwrap();
  drop(container);
}

#[tokio::test]
#[serial]
async fn test_group_chat_find_by_id_replays_events_without_snapshot() {
  init_logger();
  let (repository, event_store, container, client) = get_repository_and_store().await;
  // Given: 作成イベントと後続イベントを persist_event で書き、スナップショットは書かない。
  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let members = Members::new(admin_id.clone());
  let (group_chat, create_event) = GroupChat::new(name, members);
  let group_chat_id = group_chat.id().clone();
  event_store.persist_event(event_envelope(&create_event)).await.unwrap();

  let renamed_name = GroupChatName::new("DEF").unwrap();
  let rename_event = repository
    .find_by_id(&group_chat_id)
    .await
    .unwrap()
    .unwrap()
    .rename(renamed_name.clone(), admin_id.clone())
    .unwrap();
  event_store.persist_event(event_envelope(&rename_event)).await.unwrap();

  // When
  let actual = repository.find_by_id(&group_chat_id).await.unwrap().unwrap();

  // Then
  assert_eq!(actual.id(), &group_chat_id);
  assert_eq!(actual.name(), &renamed_name);
  assert_eq!(actual.seq_nr(), 2);

  drop(client);
  let _ = container.stop().await.unwrap();
  drop(container);
}

fn event_envelope(event: &GroupChatEvent) -> EventEnvelope<GroupChatId, GroupChatEvent> {
  EventEnvelope::new(
    event.aggregate_id().clone(),
    event.seq_nr() as SeqNr,
    event.occurred_at().clone(),
    event.clone(),
  )
}

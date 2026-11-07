use aws_sdk_dynamodb::types::AttributeValue;
use chrono::Utc;

use crate::common::{get_repository, get_repository_and_store, init_logger};
use command_domain::group_chat::{GroupChat, GroupChatEvent, GroupChatId, GroupChatName, MemberRole};
use command_domain::group_chat::{MemberId, Members};
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::{GroupChatRepository, GroupChatRepositoryError};
use event_store_adapter_rs::{EventEnvelope, EventStoreError, SeqNr};
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
  event_store
    .persist_event(event_envelope(&create_event, &group_chat))
    .await
    .unwrap();

  let renamed_name = GroupChatName::new("DEF").unwrap();
  let mut renamed_group_chat = repository.find_by_id(&group_chat_id).await.unwrap().unwrap();
  let rename_event = renamed_group_chat
    .rename(renamed_name.clone(), admin_id.clone())
    .unwrap();
  event_store
    .persist_event(event_envelope(&rename_event, &renamed_group_chat))
    .await
    .unwrap();

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

fn event_envelope(event: &GroupChatEvent, snapshot: &GroupChat) -> EventEnvelope<GroupChatId, GroupChatEvent> {
  EventEnvelope::new(
    snapshot.id().clone(),
    snapshot.seq_nr() as SeqNr,
    Utc::now(),
    event.clone(),
  )
  .with_manifest(event.manifest())
}

fn payload_json(item: &std::collections::HashMap<String, AttributeValue>) -> serde_json::Value {
  let payload = match item.get("payload").unwrap() {
    AttributeValue::B(blob) => blob.as_ref().to_vec(),
    AttributeValue::S(value) => value.as_bytes().to_vec(),
    other => panic!("unexpected payload type: {:?}", other),
  };
  serde_json::from_slice(&payload).unwrap()
}

#[tokio::test]
#[serial]
async fn should_write_v5_envelopes_to_journal_snapshot_and_head() {
  init_logger();
  let (mut repository, _event_store, container, client) = get_repository_and_store().await;

  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let members = Members::new(admin_id.clone());
  let (group_chat, create_event) = GroupChat::new(name, members);
  let group_chat_id = group_chat.id().clone();

  repository.store(&create_event, &group_chat).await.unwrap();

  let journal = client
    .get_item()
    .table_name("journal")
    .key("aid", AttributeValue::S(group_chat_id.to_string()))
    .key("seq_nr", AttributeValue::N("1".to_string()))
    .consistent_read(true)
    .send()
    .await
    .unwrap()
    .item
    .unwrap();
  assert_eq!(
    journal.get("manifest"),
    Some(&AttributeValue::S("GroupChatCreated@1".to_string()))
  );
  let journal_payload = payload_json(&journal);
  assert_eq!(journal_payload["type"], "GroupChatCreated");
  assert!(journal_payload.get("aggregate_id").is_none());
  assert!(journal_payload.get("seq_nr").is_none());
  assert!(journal_payload.get("occurred_at").is_none());

  let head = client
    .get_item()
    .table_name("head")
    .key("aid", AttributeValue::S(group_chat_id.to_string()))
    .consistent_read(true)
    .send()
    .await
    .unwrap()
    .item
    .unwrap();
  assert_eq!(head.get("seq_nr"), Some(&AttributeValue::N("1".to_string())));
  let head_events = match head.get("events").unwrap() {
    AttributeValue::L(events) => events,
    other => panic!("unexpected events type: {:?}", other),
  };
  assert_eq!(head_events.len(), 1);
  let head_event = match &head_events[0] {
    AttributeValue::M(event) => event,
    other => panic!("unexpected head event type: {:?}", other),
  };
  assert_eq!(
    head_event.get("manifest"),
    Some(&AttributeValue::S("GroupChatCreated@1".to_string()))
  );

  let snapshot = client
    .get_item()
    .table_name("snapshot")
    .key("aid", AttributeValue::S(group_chat_id.to_string()))
    .key("skey", AttributeValue::N("0".to_string()))
    .consistent_read(true)
    .send()
    .await
    .unwrap()
    .item
    .unwrap();
  assert_eq!(
    snapshot.get("manifest"),
    Some(&AttributeValue::S("GroupChat@1".to_string()))
  );
  assert_eq!(snapshot.get("seq_nr"), Some(&AttributeValue::N("1".to_string())));
  let snapshot_payload = payload_json(&snapshot);
  assert_eq!(snapshot_payload["seq_nr_counter"], 1);
  assert!(snapshot_payload.get("version").is_none());

  drop(client);
  container.stop().await.unwrap();
  drop(container);
}

#[tokio::test]
#[serial]
async fn should_reject_stale_write_with_optimistic_lock() {
  init_logger();
  let (mut repository, _event_store, container, client) = get_repository_and_store().await;

  let name = GroupChatName::new("ABC").unwrap();
  let admin_id = UserAccountId::new();
  let members = Members::new(admin_id.clone());
  let (group_chat, create_event) = GroupChat::new(name, members);
  let group_chat_id = group_chat.id().clone();
  repository.store(&create_event, &group_chat).await.unwrap();

  let mut first = repository.find_by_id(&group_chat_id).await.unwrap().unwrap();
  let mut second = repository.find_by_id(&group_chat_id).await.unwrap().unwrap();

  let first_event = first
    .rename(GroupChatName::new("FIRST").unwrap(), admin_id.clone())
    .unwrap();
  repository.store(&first_event, &first).await.unwrap();

  let second_event = second
    .rename(GroupChatName::new("SECOND").unwrap(), admin_id.clone())
    .unwrap();
  let error = repository.store(&second_event, &second).await.unwrap_err();
  match error {
    GroupChatRepositoryError::StoreError(_, EventStoreError::OptimisticLock { .. }) => {}
    other => panic!("expected an optimistic lock error, got {:?}", other),
  }

  let head = client
    .get_item()
    .table_name("head")
    .key("aid", AttributeValue::S(group_chat_id.to_string()))
    .consistent_read(true)
    .send()
    .await
    .unwrap()
    .item
    .unwrap();
  assert_eq!(head.get("seq_nr"), Some(&AttributeValue::N("2".to_string())));

  drop(client);
  container.stop().await.unwrap();
  drop(container);
}

use std::collections::HashMap;
use std::sync::Mutex;

use aws_lambda_events::dynamodb;
use aws_lambda_events::dynamodb::{StreamRecord, StreamViewType};
use chrono::{DateTime, TimeZone, Utc};
use serde_dynamo::{AttributeValue, Item};

use command_domain::group_chat::{
  GroupChatEvent, GroupChatEventCreatedBody, GroupChatEventMemberAddedBody, GroupChatEventRenamedBody, GroupChatId,
  GroupChatName, Member, MemberId, MemberRole, Members,
};
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::{GroupChatReadModelUpdateDao, GroupChatReadModelUpdateDaoError, ReadModelEvent};

use crate::journal::{JournalReadError, JournalReader};
use crate::{process_records, UpdateReadModelError, CONFIG_AGGREGATE_ID, GROUP_CHAT_TYPE_NAME};

const TEST_TIMESTAMP_MILLIS: i64 = 1_767_225_600_000;

fn test_time() -> DateTime<Utc> {
  Utc.timestamp_millis_opt(TEST_TIMESTAMP_MILLIS).unwrap()
}

fn created_event(admin_id: &UserAccountId) -> GroupChatEvent {
  GroupChatEvent::GroupChatCreated(GroupChatEventCreatedBody::new(
    GroupChatName::new("group-chat").unwrap(),
    Members::new(admin_id.clone()),
  ))
}

fn renamed_event(admin_id: &UserAccountId) -> GroupChatEvent {
  GroupChatEvent::GroupChatRenamed(GroupChatEventRenamedBody::new(
    GroupChatName::new("renamed").unwrap(),
    admin_id.clone(),
  ))
}

fn member_added_event(admin_id: &UserAccountId, user_id: &UserAccountId) -> GroupChatEvent {
  GroupChatEvent::GroupChatMemberAdded(GroupChatEventMemberAddedBody::new(
    Member::new(MemberId::new(), user_id.clone(), MemberRole::Member),
    admin_id.clone(),
  ))
}

fn read_model_event(seq_nr: u64, event: GroupChatEvent) -> ReadModelEvent {
  ReadModelEvent::new(seq_nr, test_time(), event)
}

fn head_event_value(seq_nr: u64, event: &GroupChatEvent) -> AttributeValue {
  let payload = serde_json::to_vec(event).unwrap();
  AttributeValue::M(HashMap::from([
    ("seq_nr".into(), AttributeValue::N(seq_nr.to_string())),
    (
      "occurred_at".into(),
      AttributeValue::N(test_time().timestamp_nanos_opt().unwrap().to_string()),
    ),
    ("manifest".into(), AttributeValue::S(String::new())),
    ("payload".into(), AttributeValue::B(payload)),
  ]))
}

fn head_record(event_name: &str, aid: &str, events: Vec<AttributeValue>) -> dynamodb::EventRecord {
  let seq_nr = events.len() as u64;
  let image: Item = HashMap::from([
    ("aid".into(), AttributeValue::S(aid.to_string())),
    (
      "type_name".into(),
      AttributeValue::S(
        aid
          .split_once('-')
          .map(|(name, _)| name)
          .unwrap_or_default()
          .to_string(),
      ),
    ),
    ("seq_nr".into(), AttributeValue::N(seq_nr.to_string())),
    ("events".into(), AttributeValue::L(events)),
  ])
  .into();
  dynamodb::EventRecord {
    aws_region: "ap-northeast-1".to_string(),
    change: StreamRecord {
      approximate_creation_date_time: test_time(),
      keys: HashMap::from([("aid".into(), AttributeValue::S(aid.to_string()))]).into(),
      new_image: image,
      old_image: Item::default(),
      sequence_number: Some("1".to_string()),
      size_bytes: 1,
      stream_view_type: Some(StreamViewType::NewImage),
    },
    event_id: format!("{}:{}", aid, event_name),
    event_name: event_name.to_string(),
    event_source: Some("aws:dynamodb".to_string()),
    event_version: Some("1.1".to_string()),
    event_source_arn: None,
    user_identity: None,
    record_format: None,
    table_name: None,
  }
}

#[derive(Default)]
struct FakeDao {
  position: Mutex<u64>,
  applied: Mutex<Vec<u64>>,
  fail_seq_nr: Option<u64>,
}

impl FakeDao {
  fn position(&self) -> u64 {
    *self.position.lock().unwrap()
  }

  fn applied(&self) -> Vec<u64> {
    self.applied.lock().unwrap().clone()
  }
}

#[async_trait::async_trait]
impl GroupChatReadModelUpdateDao for FakeDao {
  async fn insert_group_chat(
    &self,
    _: GroupChatId,
    _: GroupChatName,
    _: UserAccountId,
    _: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn delete_group_chat(&self, _: GroupChatId, _: DateTime<Utc>) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn rename_group_chat(
    &self,
    _: GroupChatId,
    _: GroupChatName,
    _: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn insert_member(
    &self,
    _: GroupChatId,
    _: MemberId,
    _: UserAccountId,
    _: MemberRole,
    _: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn delete_member(&self, _: GroupChatId, _: UserAccountId) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn insert_message(
    &self,
    _: GroupChatId,
    _: command_domain::group_chat::Message,
    _: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn update_message(
    &self,
    _: GroupChatId,
    _: command_domain::group_chat::Message,
    _: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn delete_message(
    &self,
    _: command_domain::group_chat::MessageId,
    _: DateTime<Utc>,
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    Ok(())
  }

  async fn get_position(&self, _: &GroupChatId) -> Result<u64, GroupChatReadModelUpdateDaoError> {
    Ok(self.position())
  }

  async fn apply_events(
    &self,
    _: &GroupChatId,
    events: &[ReadModelEvent],
  ) -> Result<(), GroupChatReadModelUpdateDaoError> {
    let mut position = self.position.lock().unwrap();
    let start = *position;
    let mut next = start;
    for event in events {
      if event.seq_nr() <= next {
        continue;
      }
      if event.seq_nr() != next + 1 {
        return Err(GroupChatReadModelUpdateDaoError::UnexpectedSeqNr {
          expected: next + 1,
          actual: event.seq_nr(),
        });
      }
      if self.fail_seq_nr == Some(event.seq_nr()) {
        return Err(GroupChatReadModelUpdateDaoError::InsertMessageError);
      }
      next = event.seq_nr();
    }
    let applied = events
      .iter()
      .filter(|event| event.seq_nr() > start && event.seq_nr() <= next)
      .map(ReadModelEvent::seq_nr)
      .collect::<Vec<_>>();
    *position = next;
    self.applied.lock().unwrap().extend(applied);
    Ok(())
  }
}

#[derive(Debug, Default)]
struct FakeJournal {
  events: Mutex<HashMap<String, Vec<ReadModelEvent>>>,
  calls: Mutex<Vec<(String, u64, u64)>>,
}

impl FakeJournal {
  fn with_events(aggregate_id: &GroupChatId, events: Vec<ReadModelEvent>) -> Self {
    Self {
      events: Mutex::new(HashMap::from([(aggregate_id.to_string(), events)])),
      calls: Mutex::new(Vec::new()),
    }
  }

  fn calls(&self) -> Vec<(String, u64, u64)> {
    self.calls.lock().unwrap().clone()
  }
}

#[async_trait::async_trait]
impl JournalReader for FakeJournal {
  async fn read_events(
    &self,
    aggregate_id: &GroupChatId,
    from_seq_nr: u64,
    to_seq_nr: u64,
  ) -> Result<Vec<ReadModelEvent>, JournalReadError> {
    self
      .calls
      .lock()
      .unwrap()
      .push((aggregate_id.to_string(), from_seq_nr, to_seq_nr));
    Ok(
      self
        .events
        .lock()
        .unwrap()
        .get(&aggregate_id.to_string())
        .map(|events| {
          events
            .iter()
            .filter(|event| from_seq_nr <= event.seq_nr() && event.seq_nr() <= to_seq_nr)
            .cloned()
            .collect()
        })
        .unwrap_or_default(),
    )
  }
}

fn fixture() -> (
  GroupChatId,
  UserAccountId,
  UserAccountId,
  GroupChatEvent,
  GroupChatEvent,
  GroupChatEvent,
) {
  let aggregate_id = GroupChatId::new();
  let admin_id = UserAccountId::new();
  let user_id = UserAccountId::new();
  let created = created_event(&admin_id);
  let member_added = member_added_event(&admin_id, &user_id);
  let renamed = renamed_event(&admin_id);
  (aggregate_id, admin_id, user_id, created, member_added, renamed)
}

fn journal_events(
  created: &GroupChatEvent,
  member_added: &GroupChatEvent,
  renamed: &GroupChatEvent,
) -> Vec<ReadModelEvent> {
  vec![
    read_model_event(1, created.clone()),
    read_model_event(2, member_added.clone()),
    read_model_event(3, renamed.clone()),
  ]
}

#[tokio::test]
async fn should_apply_insert_and_modify_head_records() {
  let (aggregate_id, _, _, created, member_added, renamed) = fixture();
  let dao = FakeDao::default();
  let journal = FakeJournal::with_events(&aggregate_id, journal_events(&created, &member_added, &renamed));
  let aid = aggregate_id.to_string();

  process_records(
    &dao,
    &journal,
    vec![
      head_record("INSERT", &aid, vec![head_event_value(1, &created)]),
      head_record("MODIFY", &aid, vec![head_event_value(2, &member_added)]),
      head_record("MODIFY", &aid, vec![head_event_value(3, &renamed)]),
    ],
  )
  .await
  .unwrap();

  assert_eq!(3, dao.position());
  assert_eq!(vec![1, 2, 3], dao.applied());
}

#[tokio::test]
async fn should_drop_duplicate_and_stale_head_records() {
  let (aggregate_id, _, _, created, member_added, renamed) = fixture();
  let dao = FakeDao::default();
  let journal = FakeJournal::with_events(&aggregate_id, journal_events(&created, &member_added, &renamed));
  let aid = aggregate_id.to_string();
  let batch = vec![
    head_record("INSERT", &aid, vec![head_event_value(1, &created)]),
    head_record("MODIFY", &aid, vec![head_event_value(2, &member_added)]),
    head_record("MODIFY", &aid, vec![head_event_value(3, &renamed)]),
  ];

  process_records(&dao, &journal, batch.clone()).await.unwrap();
  process_records(&dao, &journal, batch).await.unwrap();
  process_records(
    &dao,
    &journal,
    vec![head_record("MODIFY", &aid, vec![head_event_value(1, &created)])],
  )
  .await
  .unwrap();

  assert_eq!(3, dao.position());
  assert_eq!(vec![1, 2, 3], dao.applied());
}

#[tokio::test]
async fn should_fill_a_missing_seq_nr_from_the_journal() {
  let (aggregate_id, _, _, created, member_added, renamed) = fixture();
  let dao = FakeDao::default();
  let journal = FakeJournal::with_events(&aggregate_id, journal_events(&created, &member_added, &renamed));
  let aid = aggregate_id.to_string();

  process_records(
    &dao,
    &journal,
    vec![head_record("MODIFY", &aid, vec![head_event_value(3, &renamed)])],
  )
  .await
  .unwrap();

  assert_eq!(3, dao.position());
  assert_eq!(vec![1, 2, 3], dao.applied());
  assert_eq!(vec![(aid, 1, 2)], journal.calls());
}

#[tokio::test]
async fn should_apply_head_records_that_arrive_out_of_order() {
  let (aggregate_id, _, _, created, member_added, renamed) = fixture();
  let dao = FakeDao::default();
  let journal = FakeJournal::with_events(&aggregate_id, journal_events(&created, &member_added, &renamed));
  let aid = aggregate_id.to_string();

  process_records(
    &dao,
    &journal,
    vec![head_record("MODIFY", &aid, vec![head_event_value(3, &renamed)])],
  )
  .await
  .unwrap();
  process_records(
    &dao,
    &journal,
    vec![head_record("INSERT", &aid, vec![head_event_value(1, &created)])],
  )
  .await
  .unwrap();

  assert_eq!(3, dao.position());
  assert_eq!(vec![1, 2, 3], dao.applied());
}

#[tokio::test]
async fn should_fail_when_the_journal_has_a_gap() {
  let (aggregate_id, _, _, created, _, renamed) = fixture();
  let dao = FakeDao::default();
  let journal = FakeJournal::with_events(&aggregate_id, vec![read_model_event(1, created)]);
  let aid = aggregate_id.to_string();

  let error = process_records(
    &dao,
    &journal,
    vec![head_record("MODIFY", &aid, vec![head_event_value(3, &renamed)])],
  )
  .await
  .unwrap_err();

  assert!(matches!(error, UpdateReadModelError::JournalGap { .. }));
  assert_eq!(0, dao.position());
  assert!(dao.applied().is_empty());
}

#[tokio::test]
async fn should_skip_config_foreign_type_and_remove_records() {
  let (aggregate_id, _, _, created, _, _) = fixture();
  let dao = FakeDao::default();
  let journal = FakeJournal::default();

  process_records(
    &dao,
    &journal,
    vec![
      head_record("INSERT", CONFIG_AGGREGATE_ID, vec![head_event_value(1, &created)]),
      head_record(
        "INSERT",
        "UserAccount-01H42K4ABWQ5V2XQEP3A48VE0Z",
        vec![head_event_value(1, &created)],
      ),
      head_record("REMOVE", &aggregate_id.to_string(), vec![]),
    ],
  )
  .await
  .unwrap();

  assert_eq!(0, dao.position());
  assert!(dao.applied().is_empty());
}

#[tokio::test]
async fn should_reject_an_unexpected_stream_event_name() {
  let (aggregate_id, _, _, created, _, _) = fixture();
  let dao = FakeDao::default();
  let journal = FakeJournal::default();

  let error = process_records(
    &dao,
    &journal,
    vec![head_record(
      "UPDATE",
      &aggregate_id.to_string(),
      vec![head_event_value(1, &created)],
    )],
  )
  .await
  .unwrap_err();

  assert!(matches!(error, UpdateReadModelError::UnexpectedEventName(name) if name == "UPDATE"));
}

#[tokio::test]
async fn should_apply_multiple_events_in_one_head_record_in_seq_nr_order() {
  let (aggregate_id, _, _, created, member_added, renamed) = fixture();
  let dao = FakeDao::default();
  let journal = FakeJournal::with_events(&aggregate_id, journal_events(&created, &member_added, &renamed));
  let aid = aggregate_id.to_string();

  process_records(
    &dao,
    &journal,
    vec![head_record(
      "MODIFY",
      &aid,
      vec![head_event_value(3, &renamed), head_event_value(2, &member_added)],
    )],
  )
  .await
  .unwrap();

  assert_eq!(3, dao.position());
  assert_eq!(vec![1, 2, 3], dao.applied());
}

#[tokio::test]
async fn should_round_trip_through_the_lambda_event_json() {
  let (aggregate_id, _, _, created, _, _) = fixture();
  let dao = FakeDao::default();
  let journal = FakeJournal::default();
  let event = dynamodb::Event {
    records: vec![head_record(
      "INSERT",
      &aggregate_id.to_string(),
      vec![head_event_value(1, &created)],
    )],
  };

  let json = serde_json::to_string(&event).unwrap();
  let parsed: dynamodb::Event = serde_json::from_str(&json).unwrap();
  process_records(&dao, &journal, parsed.records).await.unwrap();

  assert_eq!(1, dao.position());
  assert_eq!(vec![1], dao.applied());
}

#[test]
fn should_report_the_group_chat_type_name() {
  assert_eq!("GroupChat", GROUP_CHAT_TYPE_NAME);
}

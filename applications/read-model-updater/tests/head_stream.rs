use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use aws_lambda_events::dynamodb;
use aws_lambda_events::dynamodb::{StreamRecord, StreamViewType};
use chrono::{DateTime, TimeZone, Utc};
use serde_dynamo::{AttributeValue, Item};
use serial_test::serial;
use sqlx::MySqlPool;
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ContainerRequest, GenericImage, ImageExt};

use command_domain::group_chat::{
  GroupChatEvent, GroupChatEventCreatedBody, GroupChatEventMemberAddedBody, GroupChatEventMemberRemovedBody,
  GroupChatEventRenamedBody, GroupChatId, GroupChatName, Member, MemberId, MemberRole, Members,
};
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::{GroupChatReadModelUpdateDao, ReadModelEvent};
use command_interface_adaptor_impl::gateways::group_chat_read_model_dao_impl::GroupChatReadModelUpdateDaoImpl;

use command_interface_adaptor_impl::gateways::recovery_task_store_impl::MySqlRecoveryTaskStore;
use rmu::journal::{DynamoDbJournalReader, JournalReadError, JournalReader};
use rmu::{
  process_records, Applier, DynamoDbHeadScanner, Head, HeadScanError, HeadScanner, ReadModelUpdater, RecoveryService,
  Resyncer, RetryPolicy, StoreIdProvider,
};

const GROUP_CHAT_AID_PREFIX: &str = "GroupChat";

fn mysql_image() -> ContainerRequest<GenericImage> {
  GenericImage::new("mysql", "8.0")
    .with_wait_for(WaitFor::message_on_stdout("Ready for start up"))
    .with_exposed_port(ContainerPort::from(3306))
    .with_env_var("MYSQL_ROOT_PASSWORD", "password")
    .with_env_var("MYSQL_DATABASE", "ceer")
    .with_env_var("MYSQL_USER", "ceer")
    .with_env_var("MYSQL_PASSWORD", "ceer")
    .with_startup_timeout(Duration::from_secs(180))
}

mod embedded {
  use refinery::embed_migrations;

  embed_migrations!("../../tools/refinery/migrations");
}

fn refinery_migrate(port: u16) {
  let url = format!("mysql://root:password@127.0.0.1:{}/ceer", port);
  let opts = refinery_core::mysql::Opts::from_url(&url).unwrap();
  let mut pool_result;
  while {
    pool_result = refinery_core::mysql::Pool::new(opts.clone());
    pool_result.is_err()
  } {
    thread::sleep(Duration::from_secs(1));
  }
  let pool = pool_result.unwrap();
  let mut attempts = 0;
  loop {
    let result = pool
      .get_conn()
      .map_err(|error| format!("{:?}", error))
      .and_then(|mut conn| {
        embedded::migrations::runner()
          .run(&mut conn)
          .map_err(|error| format!("{:?}", error))
      });
    match result {
      Ok(_) => break,
      Err(error) if attempts < 5 => {
        eprintln!("retry migration after error: {}", error);
        attempts += 1;
        thread::sleep(Duration::from_secs(2));
      }
      Err(error) => panic!("failed to migrate the read model: {}", error),
    }
  }
}

async fn new_database() -> (ContainerAsync<GenericImage>, MySqlPool) {
  let node = mysql_image().start().await.unwrap();
  let port = node.get_host_port_ipv4(3306).await.unwrap();
  refinery_migrate(port);
  let pool = MySqlPool::connect(&format!("mysql://ceer:ceer@127.0.0.1:{}/ceer", port))
    .await
    .unwrap();
  (node, pool)
}

fn test_time() -> DateTime<Utc> {
  Utc.timestamp_millis_opt(1_767_225_600_000).unwrap()
}

fn created_event(admin_id: &UserAccountId) -> GroupChatEvent {
  GroupChatEvent::GroupChatCreated(GroupChatEventCreatedBody::new(
    GroupChatName::new("group-chat").unwrap(),
    Members::new(admin_id.clone()),
  ))
}

fn member_added_event(admin_id: &UserAccountId, user_id: &UserAccountId) -> GroupChatEvent {
  GroupChatEvent::GroupChatMemberAdded(GroupChatEventMemberAddedBody::new(
    Member::new(MemberId::new(), user_id.clone(), MemberRole::Member),
    admin_id.clone(),
  ))
}

fn duplicate_member_event(admin_id: &UserAccountId) -> GroupChatEvent {
  GroupChatEvent::GroupChatMemberAdded(GroupChatEventMemberAddedBody::new(
    Member::new(MemberId::new(), admin_id.clone(), MemberRole::Member),
    admin_id.clone(),
  ))
}

fn member_removed_event(admin_id: &UserAccountId, user_id: &UserAccountId) -> GroupChatEvent {
  GroupChatEvent::GroupChatMemberRemoved(GroupChatEventMemberRemovedBody::new(user_id.clone(), admin_id.clone()))
}

fn renamed_event(admin_id: &UserAccountId) -> GroupChatEvent {
  GroupChatEvent::GroupChatRenamed(GroupChatEventRenamedBody::new(
    GroupChatName::new("renamed").unwrap(),
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
  let mut change = StreamRecord::default();
  change.approximate_creation_date_time = test_time();
  change.keys = HashMap::from([("aid".into(), AttributeValue::S(aid.to_string()))]).into();
  change.new_image = image;
  change.old_image = Item::default();
  change.sequence_number = Some("1".to_string());
  change.size_bytes = 1;
  change.stream_view_type = Some(StreamViewType::NewImage);

  let mut event = dynamodb::EventRecord::default();
  event.aws_region = "ap-northeast-1".to_string();
  event.change = change;
  event.event_id = format!("{}:{}", aid, event_name);
  event.event_name = event_name.to_string();
  event.event_source = Some("aws:dynamodb".to_string());
  event.event_version = Some("1.1".to_string());
  event
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

async fn position(pool: &MySqlPool, aggregate_id: &GroupChatId) -> i64 {
  sqlx::query_scalar::<_, i64>("SELECT seq_nr FROM read_model_positions WHERE aggregate_id = ?")
    .bind(aggregate_id.to_string())
    .fetch_optional(pool)
    .await
    .unwrap()
    .unwrap_or(0)
}

async fn count(pool: &MySqlPool, table: &str) -> i64 {
  sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {}", table))
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn group_chat_name(pool: &MySqlPool, aggregate_id: &GroupChatId) -> String {
  sqlx::query_scalar::<_, String>("SELECT name FROM group_chats WHERE id = ?")
    .bind(aggregate_id.to_string())
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
#[serial]
async fn should_apply_insert_and_modify_head_records_and_record_the_position() {
  let (_node, pool) = new_database().await;
  let (aggregate_id, _, _, created, member_added, renamed) = fixture();
  let journal = FakeJournal::with_events(
    &aggregate_id,
    vec![
      read_model_event(1, created.clone()),
      read_model_event(2, member_added.clone()),
      read_model_event(3, renamed.clone()),
    ],
  );
  let dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());
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

  assert_eq!(3, position(&pool, &aggregate_id).await);
  assert_eq!(1, count(&pool, "group_chats").await);
  assert_eq!(2, count(&pool, "members").await);
  assert_eq!("renamed", group_chat_name(&pool, &aggregate_id).await);
}

#[tokio::test]
#[serial]
async fn should_drop_duplicate_and_stale_head_records() {
  let (_node, pool) = new_database().await;
  let (aggregate_id, _, _, created, member_added, renamed) = fixture();
  let journal = FakeJournal::with_events(
    &aggregate_id,
    vec![
      read_model_event(1, created.clone()),
      read_model_event(2, member_added.clone()),
      read_model_event(3, renamed.clone()),
    ],
  );
  let dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());
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

  assert_eq!(3, position(&pool, &aggregate_id).await);
  assert_eq!(1, count(&pool, "group_chats").await);
  assert_eq!(2, count(&pool, "members").await);
}

#[tokio::test]
#[serial]
async fn should_fill_a_missing_seq_nr_from_the_journal() {
  let (_node, pool) = new_database().await;
  let (aggregate_id, _, _, created, member_added, renamed) = fixture();
  let journal = FakeJournal::with_events(
    &aggregate_id,
    vec![
      read_model_event(1, created.clone()),
      read_model_event(2, member_added.clone()),
      read_model_event(3, renamed.clone()),
    ],
  );
  let dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());
  let aid = aggregate_id.to_string();

  process_records(
    &dao,
    &journal,
    vec![head_record("MODIFY", &aid, vec![head_event_value(3, &renamed)])],
  )
  .await
  .unwrap();

  assert_eq!(3, position(&pool, &aggregate_id).await);
  assert_eq!(1, count(&pool, "group_chats").await);
  assert_eq!(2, count(&pool, "members").await);
  assert_eq!(vec![(aid, 1, 2)], journal.calls());
}

#[tokio::test]
#[serial]
async fn should_apply_head_records_that_arrive_out_of_order() {
  let (_node, pool) = new_database().await;
  let (aggregate_id, _, _, created, member_added, renamed) = fixture();
  let journal = FakeJournal::with_events(
    &aggregate_id,
    vec![
      read_model_event(1, created.clone()),
      read_model_event(2, member_added.clone()),
      read_model_event(3, renamed.clone()),
    ],
  );
  let dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());
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

  assert_eq!(3, position(&pool, &aggregate_id).await);
  assert_eq!(1, count(&pool, "group_chats").await);
  assert_eq!(2, count(&pool, "members").await);
}

#[tokio::test]
#[serial]
async fn should_roll_back_the_read_model_and_position_together() {
  let (_node, pool) = new_database().await;
  let (aggregate_id, admin_id, _, created, _, _) = fixture();
  let dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());

  // seq_nr 2 re-adds the administrator, which violates the unique key on
  // (group_chat_id, user_account_id). seq_nr 1 and 2 share one transaction.
  let error = dao
    .apply_events(
      &aggregate_id,
      &[
        read_model_event(1, created),
        read_model_event(2, duplicate_member_event(&admin_id)),
      ],
    )
    .await
    .unwrap_err();

  assert!(matches!(
    error,
    command_interface_adaptor_if::GroupChatReadModelUpdateDaoError::InsertMemberError
  ));
  assert_eq!(0, position(&pool, &aggregate_id).await);
  assert_eq!(0, count(&pool, "group_chats").await);
  assert_eq!(0, count(&pool, "members").await);
}

#[tokio::test]
#[serial]
async fn should_keep_the_last_committed_record_when_a_batch_fails() {
  let (_node, pool) = new_database().await;
  let (aggregate_id, admin_id, user_id, created, member_added, _) = fixture();
  let dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());
  let aid = aggregate_id.to_string();
  let journal = FakeJournal::with_events(
    &aggregate_id,
    vec![
      read_model_event(1, created.clone()),
      read_model_event(2, member_added.clone()),
    ],
  );

  let error = process_records(
    &dao,
    &journal,
    vec![
      head_record("INSERT", &aid, vec![head_event_value(1, &created)]),
      head_record(
        "MODIFY",
        &aid,
        vec![head_event_value(2, &duplicate_member_event(&admin_id))],
      ),
    ],
  )
  .await
  .unwrap_err();

  assert!(matches!(
    error,
    rmu::UpdateReadModelError::ReadModelUpdate(
      command_interface_adaptor_if::GroupChatReadModelUpdateDaoError::InsertMemberError
    )
  ));
  assert_eq!(1, position(&pool, &aggregate_id).await);
  assert_eq!(1, count(&pool, "group_chats").await);
  assert_eq!(1, count(&pool, "members").await);

  // Retrying the failed position with a valid event succeeds.
  process_records(
    &dao,
    &journal,
    vec![head_record("MODIFY", &aid, vec![head_event_value(2, &member_added)])],
  )
  .await
  .unwrap();

  assert_eq!(2, position(&pool, &aggregate_id).await);
  assert_eq!(2, count(&pool, "members").await);
  let member = sqlx::query_scalar::<_, String>(
    "SELECT user_account_id FROM members WHERE group_chat_id = ? AND user_account_id = ?",
  )
  .bind(aggregate_id.to_string())
  .bind(user_id.to_string())
  .fetch_one(&pool)
  .await
  .unwrap();
  assert_eq!(user_id.to_string(), member);
}

#[tokio::test]
#[serial]
async fn should_resume_from_the_committed_position_after_a_restart() {
  let (_node, pool) = new_database().await;
  let (aggregate_id, _, _, created, member_added, _) = fixture();
  let journal = FakeJournal::with_events(
    &aggregate_id,
    vec![
      read_model_event(1, created.clone()),
      read_model_event(2, member_added.clone()),
    ],
  );
  let aid = aggregate_id.to_string();

  let first = GroupChatReadModelUpdateDaoImpl::new(pool.clone());
  process_records(
    &first,
    &journal,
    vec![head_record("INSERT", &aid, vec![head_event_value(1, &created)])],
  )
  .await
  .unwrap();

  // A new process replays the retained stream from the beginning.
  let restarted = GroupChatReadModelUpdateDaoImpl::new(pool.clone());
  process_records(
    &restarted,
    &journal,
    vec![
      head_record("INSERT", &aid, vec![head_event_value(1, &created)]),
      head_record("MODIFY", &aid, vec![head_event_value(2, &member_added)]),
    ],
  )
  .await
  .unwrap();

  assert_eq!(2, position(&pool, &aggregate_id).await);
  assert_eq!(1, count(&pool, "group_chats").await);
  assert_eq!(2, count(&pool, "members").await);
}

#[tokio::test]
#[serial]
async fn should_read_a_contiguous_seq_nr_range_from_the_journal() {
  let localstack = GenericImage::new("localstack/localstack", "2.1.0")
    .with_wait_for(WaitFor::message_on_stdout("Ready."))
    .with_env_var("SERVICES", "dynamodb")
    .with_env_var("DEFAULT_REGION", "ap-northeast-1")
    .with_env_var("EAGER_SERVICE_LOADING", "1")
    .with_env_var("DYNAMODB_SHARED_DB", "1")
    .with_env_var("DYNAMODB_IN_MEMORY", "1")
    .start()
    .await
    .unwrap();
  let port = localstack.get_host_port_ipv4(4566).await.unwrap();
  let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
    .region(aws_sdk_dynamodb::config::Region::new("ap-northeast-1"))
    .endpoint_url(format!("http://127.0.0.1:{}", port))
    .credentials_provider(aws_sdk_dynamodb::config::Credentials::new(
      "x", "x", None, None, "default",
    ))
    .load()
    .await;
  let client = aws_sdk_dynamodb::Client::new(&config);
  client
    .create_table()
    .table_name("journal")
    .attribute_definitions(
      aws_sdk_dynamodb::types::AttributeDefinition::builder()
        .attribute_name("aid")
        .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
        .build()
        .unwrap(),
    )
    .attribute_definitions(
      aws_sdk_dynamodb::types::AttributeDefinition::builder()
        .attribute_name("seq_nr")
        .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::N)
        .build()
        .unwrap(),
    )
    .key_schema(
      aws_sdk_dynamodb::types::KeySchemaElement::builder()
        .attribute_name("aid")
        .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
        .build()
        .unwrap(),
    )
    .key_schema(
      aws_sdk_dynamodb::types::KeySchemaElement::builder()
        .attribute_name("seq_nr")
        .key_type(aws_sdk_dynamodb::types::KeyType::Range)
        .build()
        .unwrap(),
    )
    .billing_mode(aws_sdk_dynamodb::types::BillingMode::PayPerRequest)
    .send()
    .await
    .unwrap();

  for _ in 0..60 {
    let active = client
      .describe_table()
      .table_name("journal")
      .send()
      .await
      .ok()
      .and_then(|output| {
        output
          .table()
          .and_then(|table| table.table_status().map(|status| status.as_str() == "ACTIVE"))
      })
      .unwrap_or(false);
    if active {
      break;
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
  }

  let aggregate_id = GroupChatId::from_str(&format!("{}-01H42K4ABWQ5V2XQEP3A48VE0Z", GROUP_CHAT_AID_PREFIX)).unwrap();
  let admin_id = UserAccountId::new();
  let events = [
    created_event(&admin_id),
    member_added_event(&admin_id, &UserAccountId::new()),
    renamed_event(&admin_id),
  ];
  for (index, event) in events.iter().enumerate() {
    let seq_nr = (index + 1) as u64;
    client
      .put_item()
      .table_name("journal")
      .item(
        "aid",
        aws_sdk_dynamodb::types::AttributeValue::S(aggregate_id.to_string()),
      )
      .item("seq_nr", aws_sdk_dynamodb::types::AttributeValue::N(seq_nr.to_string()))
      .item(
        "occurred_at",
        aws_sdk_dynamodb::types::AttributeValue::N(test_time().timestamp_nanos_opt().unwrap().to_string()),
      )
      .item("manifest", aws_sdk_dynamodb::types::AttributeValue::S(String::new()))
      .item(
        "payload",
        aws_sdk_dynamodb::types::AttributeValue::B(aws_sdk_dynamodb::primitives::Blob::new(
          serde_json::to_vec(event).unwrap(),
        )),
      )
      .send()
      .await
      .unwrap();
  }

  let reader = DynamoDbJournalReader::new(client, "journal");
  let read = reader.read_events(&aggregate_id, 2, 3).await.unwrap();

  assert_eq!(vec![2, 3], read.iter().map(ReadModelEvent::seq_nr).collect::<Vec<_>>());
}

#[tokio::test]
#[serial]
async fn should_remove_a_member_from_the_read_model() {
  let (_node, pool) = new_database().await;
  let aggregate_id = GroupChatId::new();
  let admin_id = UserAccountId::new();
  let user_id = UserAccountId::new();
  let created = created_event(&admin_id);
  let member_added = member_added_event(&admin_id, &user_id);
  let member_removed = member_removed_event(&admin_id, &user_id);
  let journal = FakeJournal::with_events(
    &aggregate_id,
    vec![
      read_model_event(1, created.clone()),
      read_model_event(2, member_added.clone()),
      read_model_event(3, member_removed.clone()),
    ],
  );
  let dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());
  let aid = aggregate_id.to_string();

  process_records(
    &dao,
    &journal,
    vec![
      head_record("INSERT", &aid, vec![head_event_value(1, &created)]),
      head_record("MODIFY", &aid, vec![head_event_value(2, &member_added)]),
      head_record("MODIFY", &aid, vec![head_event_value(3, &member_removed)]),
    ],
  )
  .await
  .unwrap();

  assert_eq!(3, position(&pool, &aggregate_id).await);
  assert_eq!(1, count(&pool, "members").await);
  let remaining = sqlx::query_scalar::<_, String>("SELECT user_account_id FROM members WHERE group_chat_id = ?")
    .bind(aggregate_id.to_string())
    .fetch_one(&pool)
    .await
    .unwrap();
  assert_eq!(admin_id.to_string(), remaining);
}

struct FakeHeadScanner(Vec<Head>);

#[async_trait::async_trait]
impl HeadScanner for FakeHeadScanner {
  async fn scan_heads(&self) -> Result<Vec<Head>, HeadScanError> {
    Ok(self.0.clone())
  }
}

struct FakeStoreIdProvider(String);

#[async_trait::async_trait]
impl StoreIdProvider for FakeStoreIdProvider {
  async fn store_id(&self) -> Result<String, HeadScanError> {
    Ok(self.0.clone())
  }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn should_apply_the_same_head_record_concurrently_without_duplicating_rows() {
  let (_node, pool) = new_database().await;
  let (aggregate_id, _, _, created, _, _) = fixture();
  let journal = Arc::new(FakeJournal::with_events(
    &aggregate_id,
    vec![read_model_event(1, created.clone())],
  ));
  let dao = Arc::new(GroupChatReadModelUpdateDaoImpl::new(pool.clone()));
  let applier = Arc::new(Applier::new(dao, journal, 1));
  let updater = Arc::new(ReadModelUpdater::new(applier, None, RetryPolicy::no_retry()));
  let record = head_record("INSERT", &aggregate_id.to_string(), vec![head_event_value(1, &created)]);

  let mut handles = Vec::new();
  for _ in 0..2 {
    let updater = updater.clone();
    let record = record.clone();
    handles.push(tokio::spawn(async move { updater.process_record(record).await }));
  }
  for handle in handles {
    handle.await.unwrap().unwrap();
  }

  assert_eq!(1, position(&pool, &aggregate_id).await);
  assert_eq!(1, count(&pool, "group_chats").await);
  assert_eq!(1, count(&pool, "members").await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn should_create_position_rows_concurrently_for_different_aggregates() {
  let (_node, pool) = new_database().await;
  let dao = Arc::new(GroupChatReadModelUpdateDaoImpl::new(pool.clone()));
  let mut handles = Vec::new();
  for _ in 0..2 {
    let dao = dao.clone();
    handles.push(tokio::spawn(async move {
      let aggregate_id = GroupChatId::new();
      let admin_id = UserAccountId::new();
      let event = read_model_event(1, created_event(&admin_id));
      dao.apply_events(&aggregate_id, &[event]).await.unwrap();
      aggregate_id
    }));
  }

  for handle in handles {
    let aggregate_id = handle.await.unwrap();
    assert_eq!(1, position(&pool, &aggregate_id).await);
  }
}

#[tokio::test]
#[serial]
async fn should_resync_a_head_that_fell_behind_after_a_long_stop() {
  let (_node, pool) = new_database().await;
  let (aggregate_id, _, _, created, member_added, renamed) = fixture();
  let journal = Arc::new(FakeJournal::with_events(
    &aggregate_id,
    vec![
      read_model_event(1, created),
      read_model_event(2, member_added),
      read_model_event(3, renamed),
    ],
  ));
  let dao = Arc::new(GroupChatReadModelUpdateDaoImpl::new(pool.clone()));
  let applier = Arc::new(Applier::new(dao, journal, 1));
  let scanner = Arc::new(FakeHeadScanner(vec![
    Head {
      aggregate_id: aggregate_id.to_string(),
      type_name: "GroupChat".to_string(),
      seq_nr: 3,
    },
    Head {
      aggregate_id: "UserAccount-01H42K4ABWQ5V2XQEP3A48VE0Z".to_string(),
      type_name: "UserAccount".to_string(),
      seq_nr: 1,
    },
    Head {
      aggregate_id: "__config__".to_string(),
      type_name: "Config".to_string(),
      seq_nr: 0,
    },
  ]));
  let resyncer = Resyncer::new(applier, scanner, None);

  let result = resyncer.resync().await.unwrap();

  assert_eq!(1, result.heads);
  assert_eq!(2, result.skipped);
  assert_eq!(1, result.advanced);
  assert_eq!(3, position(&pool, &aggregate_id).await);
  assert_eq!(1, count(&pool, "group_chats").await);
  assert_eq!(2, count(&pool, "members").await);
}

#[tokio::test]
#[serial]
async fn should_record_and_complete_a_recovery_task() {
  let (_node, pool) = new_database().await;
  let (aggregate_id, _, _, created, member_added, _) = fixture();
  let journal = Arc::new(FakeJournal::with_events(
    &aggregate_id,
    vec![read_model_event(1, created), read_model_event(2, member_added)],
  ));
  let dao = Arc::new(GroupChatReadModelUpdateDaoImpl::new(pool.clone()));
  let applier = Arc::new(Applier::new(dao, journal, 1));
  let store = Arc::new(MySqlRecoveryTaskStore::new(pool.clone()));
  let recovery = Arc::new(RecoveryService::new(
    store,
    Arc::new(FakeStoreIdProvider("store-1".to_string())),
    applier,
  ));

  recovery
    .record(
      &aggregate_id.to_string(),
      2,
      &rmu::UpdateReadModelError::Payload("bad payload".to_string()),
    )
    .await
    .unwrap();

  let result = recovery.recover_pending().await.unwrap();
  assert_eq!(1, result.completed);
  assert!(result.failures.is_empty());
  assert_eq!(2, position(&pool, &aggregate_id).await);
  let status = sqlx::query_scalar::<_, String>(
    "SELECT status FROM read_model_recovery_tasks WHERE aggregate_id = ? AND seq_nr = 2",
  )
  .bind(aggregate_id.to_string())
  .fetch_one(&pool)
  .await
  .unwrap();
  assert_eq!("completed", status);

  let second = recovery.recover_pending().await.unwrap();
  assert_eq!(0, second.completed);
  assert_eq!(2, position(&pool, &aggregate_id).await);
}

#[tokio::test]
#[serial]
async fn should_record_a_permanent_resync_failure_and_continue_with_other_aggregates() {
  let (_node, pool) = new_database().await;
  let broken_aggregate = GroupChatId::new();
  let healthy_aggregate = GroupChatId::new();
  let admin_id = UserAccountId::new();
  let created = created_event(&admin_id);
  let journal = Arc::new(FakeJournal::default());
  journal
    .events
    .lock()
    .unwrap()
    .insert(broken_aggregate.to_string(), vec![read_model_event(1, created.clone())]);
  journal
    .events
    .lock()
    .unwrap()
    .insert(healthy_aggregate.to_string(), vec![read_model_event(1, created)]);
  let dao = Arc::new(GroupChatReadModelUpdateDaoImpl::new(pool.clone()));
  let applier = Arc::new(Applier::new(dao, journal, 1));
  let store = Arc::new(MySqlRecoveryTaskStore::new(pool.clone()));
  let recovery = Arc::new(RecoveryService::new(
    store,
    Arc::new(FakeStoreIdProvider("store-1".to_string())),
    applier.clone(),
  ));
  let scanner = Arc::new(FakeHeadScanner(vec![
    Head {
      aggregate_id: broken_aggregate.to_string(),
      type_name: "GroupChat".to_string(),
      seq_nr: 3,
    },
    Head {
      aggregate_id: healthy_aggregate.to_string(),
      type_name: "GroupChat".to_string(),
      seq_nr: 1,
    },
  ]));
  let resyncer = Resyncer::new(applier, scanner, Some(recovery));

  let error = resyncer.resync().await.unwrap_err();

  assert!(!error.is_retryable());
  assert_eq!(1, error.permanent.len());
  assert_eq!(0, position(&pool, &broken_aggregate).await);
  assert_eq!(1, position(&pool, &healthy_aggregate).await);
  let pending = sqlx::query_scalar::<_, i64>(
    "SELECT COUNT(*) FROM read_model_recovery_tasks WHERE aggregate_id = ? AND status = 'pending'",
  )
  .bind(broken_aggregate.to_string())
  .fetch_one(&pool)
  .await
  .unwrap();
  assert_eq!(1, pending);
}

#[tokio::test]
#[serial]
async fn should_scan_all_pages_of_the_head_table() {
  let localstack = GenericImage::new("localstack/localstack", "2.1.0")
    .with_wait_for(WaitFor::message_on_stdout("Ready."))
    .with_env_var("SERVICES", "dynamodb")
    .with_env_var("DEFAULT_REGION", "ap-northeast-1")
    .with_env_var("EAGER_SERVICE_LOADING", "1")
    .with_env_var("DYNAMODB_SHARED_DB", "1")
    .with_env_var("DYNAMODB_IN_MEMORY", "1")
    .with_startup_timeout(Duration::from_secs(180))
    .start()
    .await
    .unwrap();
  let port = localstack.get_host_port_ipv4(4566).await.unwrap();
  let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
    .region(aws_sdk_dynamodb::config::Region::new("ap-northeast-1"))
    .endpoint_url(format!("http://127.0.0.1:{}", port))
    .credentials_provider(aws_sdk_dynamodb::config::Credentials::new(
      "x", "x", None, None, "default",
    ))
    .load()
    .await;
  let client = aws_sdk_dynamodb::Client::new(&config);
  client
    .create_table()
    .table_name("head")
    .attribute_definitions(
      aws_sdk_dynamodb::types::AttributeDefinition::builder()
        .attribute_name("aid")
        .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
        .build()
        .unwrap(),
    )
    .key_schema(
      aws_sdk_dynamodb::types::KeySchemaElement::builder()
        .attribute_name("aid")
        .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
        .build()
        .unwrap(),
    )
    .billing_mode(aws_sdk_dynamodb::types::BillingMode::PayPerRequest)
    .send()
    .await
    .unwrap();
  for _ in 0..60 {
    let active = client
      .describe_table()
      .table_name("head")
      .send()
      .await
      .ok()
      .and_then(|output| {
        output
          .table()
          .and_then(|table| table.table_status().map(|status| status.as_str() == "ACTIVE"))
      })
      .unwrap_or(false);
    if active {
      break;
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
  }
  for index in 0..3 {
    client
      .put_item()
      .table_name("head")
      .item(
        "aid",
        aws_sdk_dynamodb::types::AttributeValue::S(format!("GroupChat-01H42K4ABWQ5V2XQEP3A48VE0{}", index)),
      )
      .item(
        "seq_nr",
        aws_sdk_dynamodb::types::AttributeValue::N((index + 1).to_string()),
      )
      .item(
        "type_name",
        aws_sdk_dynamodb::types::AttributeValue::S("GroupChat".to_string()),
      )
      .send()
      .await
      .unwrap();
  }
  client
    .put_item()
    .table_name("head")
    .item(
      "aid",
      aws_sdk_dynamodb::types::AttributeValue::S("__config__".to_string()),
    )
    .item("seq_nr", aws_sdk_dynamodb::types::AttributeValue::N("0".to_string()))
    .item(
      "type_name",
      aws_sdk_dynamodb::types::AttributeValue::S("Config".to_string()),
    )
    .item(
      "store_id",
      aws_sdk_dynamodb::types::AttributeValue::S("store-1".to_string()),
    )
    .send()
    .await
    .unwrap();

  let scanner = DynamoDbHeadScanner::new(client, "head").with_page_limit(1);
  let heads = scanner.scan_heads().await.unwrap();

  assert_eq!(3, heads.len());
  assert!(heads.iter().all(|head| head.type_name == "GroupChat"));
}

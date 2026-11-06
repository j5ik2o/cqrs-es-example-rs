use std::env;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use aws_sdk_dynamodb::config::{Credentials, Region};
use aws_sdk_dynamodb::operation::create_table::CreateTableOutput;
use aws_sdk_dynamodb::types::{
  AttributeDefinition, GlobalSecondaryIndex, KeySchemaElement, KeyType, Projection, ProjectionType,
  ProvisionedThroughput, ScalarAttributeType, StreamSpecification, StreamViewType,
};
use aws_sdk_dynamodb::Client;
use command_domain::group_chat::{GroupChat, GroupChatEvent, GroupChatId};
use event_store_adapter_rs::{DynamoDbOptions, DynamoDbTables, EventStoreForDynamoDB};
use testcontainers::core::WaitFor;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

use command_interface_adaptor_impl::gateways::group_chat_repository::GroupChatRepositoryImpl;

pub fn init_logger() {
  env::set_var("RUST_LOG", "debug");
  let _ = env_logger::builder().is_test(true).try_init();
}

pub async fn create_journal_table(client: &Client, table_name: &str) -> Result<CreateTableOutput> {
  let aid_attribute_definition = AttributeDefinition::builder()
    .attribute_name("aid")
    .attribute_type(ScalarAttributeType::S)
    .build()?;

  let seq_nr_attribute_definition = AttributeDefinition::builder()
    .attribute_name("seq_nr")
    .attribute_type(ScalarAttributeType::N)
    .build()?;

  let aid_schema = KeySchemaElement::builder()
    .attribute_name("aid")
    .key_type(KeyType::Hash)
    .build()?;

  let seq_nr_schema = KeySchemaElement::builder()
    .attribute_name("seq_nr")
    .key_type(KeyType::Range)
    .build()?;

  let provisioned_throughput = ProvisionedThroughput::builder()
    .read_capacity_units(10)
    .write_capacity_units(5)
    .build()?;

  let result = client
    .create_table()
    .table_name(table_name)
    .attribute_definitions(aid_attribute_definition)
    .attribute_definitions(seq_nr_attribute_definition)
    .key_schema(aid_schema)
    .key_schema(seq_nr_schema)
    .provisioned_throughput(provisioned_throughput)
    .send()
    .await?;

  Ok(result)
}

pub async fn create_snapshot_table(client: &Client, table_name: &str, gsi_name: &str) -> Result<CreateTableOutput> {
  let aid_attribute_definition = AttributeDefinition::builder()
    .attribute_name("aid")
    .attribute_type(ScalarAttributeType::S)
    .build()?;

  let skey_attribute_definition = AttributeDefinition::builder()
    .attribute_name("skey")
    .attribute_type(ScalarAttributeType::N)
    .build()?;

  let active_history_seq_nr_attribute_definition = AttributeDefinition::builder()
    .attribute_name("active_history_seq_nr")
    .attribute_type(ScalarAttributeType::N)
    .build()?;

  let aid_schema = KeySchemaElement::builder()
    .attribute_name("aid")
    .key_type(KeyType::Hash)
    .build()?;

  let skey_schema = KeySchemaElement::builder()
    .attribute_name("skey")
    .key_type(KeyType::Range)
    .build()?;

  let provisioned_throughput = ProvisionedThroughput::builder()
    .read_capacity_units(10)
    .write_capacity_units(5)
    .build()?;

  let gsi = GlobalSecondaryIndex::builder()
    .index_name(gsi_name)
    .key_schema(
      KeySchemaElement::builder()
        .attribute_name("aid")
        .key_type(KeyType::Hash)
        .build()?,
    )
    .key_schema(
      KeySchemaElement::builder()
        .attribute_name("active_history_seq_nr")
        .key_type(KeyType::Range)
        .build()?,
    )
    .projection(Projection::builder().projection_type(ProjectionType::KeysOnly).build())
    .provisioned_throughput(provisioned_throughput.clone())
    .build()?;

  let result = client
    .create_table()
    .table_name(table_name)
    .attribute_definitions(aid_attribute_definition)
    .attribute_definitions(skey_attribute_definition)
    .attribute_definitions(active_history_seq_nr_attribute_definition)
    .key_schema(aid_schema)
    .key_schema(skey_schema)
    .global_secondary_indexes(gsi)
    .provisioned_throughput(provisioned_throughput)
    .send()
    .await?;

  Ok(result)
}

pub async fn create_head_table(client: &Client, table_name: &str) -> Result<CreateTableOutput> {
  let aid_attribute_definition = AttributeDefinition::builder()
    .attribute_name("aid")
    .attribute_type(ScalarAttributeType::S)
    .build()?;

  let aid_schema = KeySchemaElement::builder()
    .attribute_name("aid")
    .key_type(KeyType::Hash)
    .build()?;

  let provisioned_throughput = ProvisionedThroughput::builder()
    .read_capacity_units(10)
    .write_capacity_units(5)
    .build()?;

  let stream_specification = StreamSpecification::builder()
    .stream_enabled(true)
    .stream_view_type(StreamViewType::NewImage)
    .build()?;

  let result = client
    .create_table()
    .table_name(table_name)
    .attribute_definitions(aid_attribute_definition)
    .key_schema(aid_schema)
    .provisioned_throughput(provisioned_throughput)
    .stream_specification(stream_specification)
    .send()
    .await?;

  Ok(result)
}

pub async fn create_client(dynamodb_port: u16) -> Client {
  let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
    .region(Region::new("us-west-1"))
    .endpoint_url(format!("http://localhost:{}", dynamodb_port))
    .credentials_provider(Credentials::new("x", "x", None, None, "default"))
    .load()
    .await;

  Client::new(&config)
}

async fn wait_table(client: &Client, target_table_name: &str) -> bool {
  let lto = client.list_tables().send().await;
  match lto {
    Ok(lto) => {
      log::info!("table_names: {:?}", lto.table_names());
      lto.table_names().iter().any(|tn| tn == target_table_name)
    }
    Err(e) => {
      println!("Error: {}", e);
      false
    }
  }
}

pub async fn get_repository() -> (
  GroupChatRepositoryImpl<EventStoreForDynamoDB<GroupChatId, GroupChat, GroupChatEvent>>,
  ContainerAsync<GenericImage>,
  Client,
) {
  let (repository, _event_store, container, client) = get_repository_and_store().await;
  (repository, container, client)
}

pub async fn get_repository_and_store() -> (
  GroupChatRepositoryImpl<EventStoreForDynamoDB<GroupChatId, GroupChat, GroupChatEvent>>,
  EventStoreForDynamoDB<GroupChatId, GroupChat, GroupChatEvent>,
  ContainerAsync<GenericImage>,
  Client,
) {
  init_logger();
  let wait_for = WaitFor::message_on_stdout("Ready.");
  let image = GenericImage::new("localstack/localstack", "2.1.0")
    .with_wait_for(wait_for)
    .with_env_var("SERVICES", "dynamodb")
    .with_env_var("DEFAULT_REGION", "us-west-1")
    .with_env_var("EAGER_SERVICE_LOADING", "1")
    .with_env_var("DYNAMODB_SHARED_DB", "1")
    .with_env_var("DYNAMODB_IN_MEMORY", "1");
  let dynamodb_node = image.start().await.unwrap();
  let port = dynamodb_node.get_host_port_ipv4(4566).await.unwrap();
  log::debug!("DynamoDB port: {}", port);

  let test_time_factor = env::var("TEST_TIME_FACTOR")
    .unwrap_or("1".to_string())
    .parse::<u64>()
    .unwrap();

  sleep(Duration::from_millis(1000 * test_time_factor));

  let client = create_client(port).await;

  let journal_table_name = "journal";
  let snapshot_table_name = "snapshot";
  let head_table_name = "head";
  let snapshot_history_index_name = "snapshot-active-history-index";
  let _journal_table_output = create_journal_table(&client, journal_table_name).await;
  let _snapshot_table_output = create_snapshot_table(&client, snapshot_table_name, snapshot_history_index_name).await;
  let _head_table_output = create_head_table(&client, head_table_name).await;

  for table_name in [journal_table_name, snapshot_table_name, head_table_name] {
    while !(wait_table(&client, table_name).await) {
      log::info!("Waiting for {} table to be created", table_name);
      sleep(Duration::from_millis(1000 * test_time_factor));
    }
  }

  let epg = EventStoreForDynamoDB::open(
    client.clone(),
    DynamoDbTables {
      journal_table_name: journal_table_name.to_string(),
      snapshot_table_name: snapshot_table_name.to_string(),
      head_table_name: head_table_name.to_string(),
      snapshot_history_index_name: snapshot_history_index_name.to_string(),
    },
    DynamoDbOptions::default(),
  )
  .await
  .unwrap();
  let repository = GroupChatRepositoryImpl::new(epg.clone(), 10);
  (repository, epg, dynamodb_node, client)
}

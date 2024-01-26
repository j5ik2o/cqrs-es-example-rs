use std::env;
use std::sync::OnceLock;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use aws_sdk_dynamodb::config::{Credentials, Region};
use aws_sdk_dynamodb::operation::create_table::CreateTableOutput;
use aws_sdk_dynamodb::types::{
  AttributeDefinition, GlobalSecondaryIndex, KeySchemaElement, KeyType, Projection, ProjectionType,
  ProvisionedThroughput, ScalarAttributeType,
};
use aws_sdk_dynamodb::Client;
use command_domain::group_chat::{GroupChat, GroupChatEvent, GroupChatId};
use event_store_adapter_rs::EventStoreForDynamoDB;
use testcontainers::clients::Cli;
use testcontainers::core::WaitFor;
use testcontainers::{clients, Container, GenericImage};

use command_interface_adaptor_impl::gateways::group_chat_repository::GroupChatRepositoryImpl;

pub static DOCKER: OnceLock<clients::Cli> = OnceLock::new();

pub fn init_logger() {
  env::set_var("RUST_LOG", "debug");
  let _ = env_logger::builder().is_test(true).try_init();
}

pub async fn create_journal_table(client: &Client, table_name: &str, gsi_name: &str) -> Result<CreateTableOutput> {
  let pkey_attribute_definition = AttributeDefinition::builder()
    .attribute_name("pkey")
    .attribute_type(ScalarAttributeType::S)
    .build()?;

  let skey_attribute_definition = AttributeDefinition::builder()
    .attribute_name("skey")
    .attribute_type(ScalarAttributeType::S)
    .build()?;

  let pkey_schema = KeySchemaElement::builder()
    .attribute_name("pkey")
    .key_type(KeyType::Hash)
    .build()?;

  let skey_schema = KeySchemaElement::builder()
    .attribute_name("skey")
    .key_type(KeyType::Range)
    .build()?;

  let aid_attribute_definition = AttributeDefinition::builder()
    .attribute_name("aid")
    .attribute_type(ScalarAttributeType::S)
    .build()?;

  let seq_nr_attribute_definition = AttributeDefinition::builder()
    .attribute_name("seq_nr")
    .attribute_type(ScalarAttributeType::N)
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
        .attribute_name("seq_nr")
        .key_type(KeyType::Range)
        .build()?,
    )
    .projection(Projection::builder().projection_type(ProjectionType::All).build())
    .provisioned_throughput(provisioned_throughput.clone())
    .build()?;

  let result = client
    .create_table()
    .table_name(table_name)
    .attribute_definitions(pkey_attribute_definition)
    .attribute_definitions(skey_attribute_definition)
    .attribute_definitions(aid_attribute_definition)
    .attribute_definitions(seq_nr_attribute_definition)
    .key_schema(pkey_schema)
    .key_schema(skey_schema)
    .global_secondary_indexes(gsi)
    .provisioned_throughput(provisioned_throughput)
    .send()
    .await?;

  Ok(result)
}

pub async fn create_snapshot_table(client: &Client, table_name: &str, gsi_name: &str) -> Result<CreateTableOutput> {
  let pkey_attribute_definition = AttributeDefinition::builder()
    .attribute_name("pkey")
    .attribute_type(ScalarAttributeType::S)
    .build()?;

  let pkey_schema = KeySchemaElement::builder()
    .attribute_name("pkey")
    .key_type(KeyType::Hash)
    .build()?;

  let skey_attribute_definition = AttributeDefinition::builder()
    .attribute_name("skey")
    .attribute_type(ScalarAttributeType::S)
    .build()?;

  let skey_schema = KeySchemaElement::builder()
    .attribute_name("skey")
    .key_type(KeyType::Range)
    .build()?;

  let aid_attribute_definition = AttributeDefinition::builder()
    .attribute_name("aid")
    .attribute_type(ScalarAttributeType::S)
    .build()?;

  let seq_nr_attribute_definition = AttributeDefinition::builder()
    .attribute_name("seq_nr")
    .attribute_type(ScalarAttributeType::N)
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
        .attribute_name("seq_nr")
        .key_type(KeyType::Range)
        .build()?,
    )
    .projection(Projection::builder().projection_type(ProjectionType::All).build())
    .provisioned_throughput(provisioned_throughput.clone())
    .build()?;

  let result = client
    .create_table()
    .table_name(table_name)
    .attribute_definitions(pkey_attribute_definition)
    .attribute_definitions(skey_attribute_definition)
    .attribute_definitions(aid_attribute_definition)
    .attribute_definitions(seq_nr_attribute_definition)
    .key_schema(pkey_schema)
    .key_schema(skey_schema)
    .global_secondary_indexes(gsi)
    .provisioned_throughput(provisioned_throughput)
    .send()
    .await?;

  Ok(result)
}
pub fn create_client(dynamodb_port: u16) -> Client {
  let region = Region::new("us-west-1");
  let config = aws_sdk_dynamodb::Config::builder()
    .region(Some(region))
    .endpoint_url(format!("http://localhost:{}", dynamodb_port))
    .credentials_provider(Credentials::new("x", "x", None, None, "default"))
    .behavior_version_latest()
    .build();

  Client::from_conf(config)
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

pub async fn get_repository<'a>(
  docker: &'a Cli,
) -> (
  GroupChatRepositoryImpl<EventStoreForDynamoDB<GroupChatId, GroupChat, GroupChatEvent>>,
  Container<'a, GenericImage>,
  Client,
) {
  init_logger();
  let wait_for = WaitFor::message_on_stdout("Ready.");
  let image = GenericImage::new("localstack/localstack", "2.1.0")
    .with_env_var("SERVICES", "dynamodb")
    .with_env_var("DEFAULT_REGION", "us-west-1")
    .with_env_var("EAGER_SERVICE_LOADING", "1")
    .with_env_var("DYNAMODB_SHARED_DB", "1")
    .with_env_var("DYNAMODB_IN_MEMORY", "1")
    .with_wait_for(wait_for);
  let dynamodb_node: Container<GenericImage> = docker.run::<GenericImage>(image);
  let port = dynamodb_node.get_host_port_ipv4(4566);
  log::debug!("DynamoDB port: {}", port);

  let test_time_factor = env::var("TEST_TIME_FACTOR")
    .unwrap_or("1".to_string())
    .parse::<u64>()
    .unwrap();

  sleep(Duration::from_millis(1000 * test_time_factor));

  let client = create_client(port);

  let journal_table_name = "journal";
  let journal_aid_index_name = "journal-aid-index";
  let _journal_table_output = create_journal_table(&client, journal_table_name, journal_aid_index_name).await;

  let snapshot_table_name = "snapshot";
  let snapshot_aid_index_name = "snapshot-aid-index";
  let _snapshot_table_output = create_snapshot_table(&client, snapshot_table_name, snapshot_aid_index_name).await;

  while !(wait_table(&client, journal_table_name).await) {
    log::info!("Waiting for journal table to be created");
    sleep(Duration::from_millis(1000 * test_time_factor));
  }

  while !(wait_table(&client, snapshot_table_name).await) {
    log::info!("Waiting for snapshot table to be created");
    sleep(Duration::from_millis(1000 * test_time_factor));
  }

  let epg = EventStoreForDynamoDB::new(
    client.clone(),
    journal_table_name.to_string(),
    journal_aid_index_name.to_string(),
    snapshot_table_name.to_string(),
    snapshot_aid_index_name.to_string(),
    64,
  );
  let repository = GroupChatRepositoryImpl::new(epg, 10);
  (repository, dynamodb_node, client)
}

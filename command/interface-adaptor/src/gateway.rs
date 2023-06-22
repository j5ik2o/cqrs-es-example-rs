use anyhow::Result;
use aws_sdk_dynamodb::types::{AttributeValue, Put, TransactWriteItem, Update};
use aws_sdk_dynamodb::Client;
use serde::{de, Serialize};

use palupunte_domain::aggregate::Aggregate;
use palupunte_domain::events::Event;

pub mod group_chat_repository;

pub struct EventPersistenceGateway<'a> {
  journal_table_name: String,
  snapshot_table_name: String,
  client: &'a Client,
}

impl<'a> EventPersistenceGateway<'a> {
  pub fn new(journal_table_name: String, snapshot_table_name: String, client: &'a Client) -> Self {
    Self {
      journal_table_name,
      snapshot_table_name,
      client,
    }
  }

  pub async fn get_snapshot_by_id<T>(&self, aid: String) -> Result<(T, usize)>
  where
    T: for<'de> de::Deserialize<'de>, {
    let response = self
      .client
      .get_item()
      .table_name(self.snapshot_table_name.clone())
      .key("pkey", AttributeValue::S(aid))
      .send()
      .await?;
    let item = response.item().unwrap();
    let payload = item.get("payload").unwrap().as_s().unwrap();
    let aggregate = serde_json::from_str::<T>(payload).unwrap();
    let seq_nr = item.get("seq_nr").unwrap().as_n().unwrap();
    Ok((aggregate, seq_nr.parse::<usize>().unwrap()))
  }

  pub async fn get_events_by_id_and_seq_nr<T>(&self, aid: String, seq_nr: usize) -> Result<Vec<T>>
  where
    T: for<'de> de::Deserialize<'de>, {
    let response = self
      .client
      .query()
      .table_name(self.journal_table_name.clone())
      .index_name("aid_index")
      .key_condition_expression("#aid = :aid AND #skey >= :skey")
      .expression_attribute_names("#aid", "aid")
      .expression_attribute_values(":aid", AttributeValue::S(aid))
      .expression_attribute_names("#skey", "skey")
      .expression_attribute_values(":skey", AttributeValue::N(seq_nr.to_string()))
      .send()
      .await?;
    let mut events = Vec::new();
    if let Some(items) = response.items {
      for item in items {
        let payload = item.get("payload").unwrap();
        let str = payload.as_s().unwrap();
        let event = serde_json::from_str::<T>(str).unwrap();
        events.push(event);
      }
    }
    Ok(events)
  }

  pub async fn store_event_with_snapshot_opt<A, E>(
    &mut self,
    event: &E,
    version: usize,
    aggregate: Option<&A>,
  ) -> Result<()>
  where
    A: ?Sized + Serialize + Aggregate,
    E: ?Sized + Serialize + Event, {
    match (event.is_created(), aggregate) {
      (true, Some(ar)) => {
        let aggregate_id = AttributeValue::S(event.aggregate_id().to_string());
        let put_snapshot = Put::builder()
          .table_name(self.snapshot_table_name.clone())
          .item("pkey", aggregate_id)
          .item("payload", AttributeValue::S(serde_json::to_string(ar)?))
          .item("seq_nr", AttributeValue::N(event.seq_nr().to_string()))
          .item("version", AttributeValue::N("1".to_string()))
          .condition_expression("attribute_not_exists(pkey)")
          .build();
        let put_journal = Put::builder()
          .table_name(self.journal_table_name.clone())
          .item("pkey", AttributeValue::S(event.id().to_string()))
          .item("skey", AttributeValue::N(event.seq_nr().to_string()))
          .item("aid", AttributeValue::S(event.aggregate_id().to_string()))
          .item("payload", AttributeValue::S(serde_json::to_string(event)?))
          .item(
            "occurred_at",
            AttributeValue::N(event.occurred_at().timestamp_millis().to_string()),
          )
          .build();
        let twi1 = TransactWriteItem::builder().put(put_snapshot).build();
        let twi2 = TransactWriteItem::builder().put(put_journal).build();
        let _ = self
          .client
          .transact_write_items()
          .set_transact_items(Some(vec![twi1, twi2]))
          .send()
          .await?;
      }
      (true, None) => {
        panic!("Aggregate is not found");
      }
      (false, Some(ar)) => {
        let aggregate_id = AttributeValue::S(event.aggregate_id().to_string());
        let update_snapshot = Update::builder()
          .table_name(self.snapshot_table_name.clone())
          .update_expression("SET #payload=:payload, #seq_nr=:seq_nr, #version=:after_version")
          .key("pkey", aggregate_id)
          .expression_attribute_names("#payload", "payload")
          .expression_attribute_names("#seq_nr", "seq_nr")
          .expression_attribute_names("#version", "version")
          .expression_attribute_values(":payload", AttributeValue::S(serde_json::to_string(ar)?))
          .expression_attribute_values(":seq_nr", AttributeValue::N(event.seq_nr().to_string()))
          .expression_attribute_values(":before_version", AttributeValue::N(version.to_string()))
          .expression_attribute_values(":after_version", AttributeValue::N((version + 1).to_string()))
          .condition_expression("#version=:before_version")
          .build();
        let put_journal = Put::builder()
          .table_name(self.journal_table_name.clone())
          .item("pkey", AttributeValue::S(event.id().to_string()))
          .item("skey", AttributeValue::N(event.seq_nr().to_string()))
          .item("aid", AttributeValue::S(event.aggregate_id().to_string()))
          .item("payload", AttributeValue::S(serde_json::to_string(event)?))
          .item(
            "occurred_at",
            AttributeValue::N(event.occurred_at().timestamp_millis().to_string()),
          )
          .build();
        let twi1 = TransactWriteItem::builder().update(update_snapshot).build();
        let twi2 = TransactWriteItem::builder().put(put_journal).build();
        let _ = self
          .client
          .transact_write_items()
          .set_transact_items(Some(vec![twi1, twi2]))
          .send()
          .await?;
      }
      (false, None) => {
        let aggregate_id = AttributeValue::S(event.aggregate_id().to_string());
        let update_snapshot = Update::builder()
          .table_name(self.snapshot_table_name.clone())
          .update_expression("SET #seq_nr=:seq_nr, #version=:after_version")
          .key("pkey", aggregate_id)
          .expression_attribute_names("#seq_nr", "seq_nr")
          .expression_attribute_names("#version", "version")
          .expression_attribute_values(":seq_nr", AttributeValue::N(event.seq_nr().to_string()))
          .expression_attribute_values(":before_version", AttributeValue::N(version.to_string()))
          .expression_attribute_values(":after_version", AttributeValue::N((version + 1).to_string()))
          .condition_expression("#version=:before_version")
          .build();
        let put_journal = Put::builder()
          .table_name(self.journal_table_name.clone())
          .item("pkey", AttributeValue::S(event.id().to_string()))
          .item("skey", AttributeValue::N(event.seq_nr().to_string()))
          .item("aid", AttributeValue::S(event.aggregate_id().to_string()))
          .item("payload", AttributeValue::S(serde_json::to_string(event)?))
          .item(
            "occurred_at",
            AttributeValue::N(event.occurred_at().timestamp_millis().to_string()),
          )
          .build();
        let twi1 = TransactWriteItem::builder().update(update_snapshot).build();
        let twi2 = TransactWriteItem::builder().put(put_journal).build();
        let _ = self
          .client
          .transact_write_items()
          .set_transact_items(Some(vec![twi1, twi2]))
          .send()
          .await?;
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::env;

  use anyhow::Result;
  use aws_sdk_dynamodb::config::{Credentials, Region};
  use aws_sdk_dynamodb::operation::create_table::CreateTableOutput;
  use aws_sdk_dynamodb::types::{
    AttributeDefinition, GlobalSecondaryIndex, KeySchemaElement, KeyType, Projection, ProjectionType,
    ProvisionedThroughput, ScalarAttributeType,
  };
  use aws_sdk_dynamodb::Client;
  use testcontainers::clients;
  use testcontainers::core::WaitFor;
  use testcontainers::images::generic::GenericImage;

  use palupunte_domain::aggregate::Aggregate;
  use palupunte_domain::events::GroupChatEvent;

  use palupunte_domain::group_chat::{GroupChat, GroupChatName, MemberID};

  use crate::gateway::EventPersistenceGateway;

  fn init_logger() {
    let _ = env::set_var("RUST_LOG", "info");
    let _ = env_logger::builder().is_test(true).try_init();
  }

  #[tokio::test]
  async fn test() {
    init_logger();
    let docker = clients::Cli::default();
    let wait_for = WaitFor::message_on_stdout("Port");
    let image = GenericImage::new("amazon/dynamodb-local", "1.18.0").with_wait_for(wait_for);
    let dynamodb_node = docker.run(image);

    let client = create_client(dynamodb_node.get_host_port_ipv4(8000));
    let journal_table_name = "journal";
    let _create_table_response = create_journal_table(&client, journal_table_name).await;

    let snapshot_table_name = "snapshot";
    let _create_table_response = create_snapshot_table(&client, snapshot_table_name).await;

    let lto = client.list_tables().send().await.unwrap();
    assert!(lto.table_names.unwrap().contains(&journal_table_name.to_string()));

    let mut epg =
      EventPersistenceGateway::new(journal_table_name.to_string(), snapshot_table_name.to_string(), &client);
    let group_name = GroupChatName::new("test".to_string());

    let (group_chat, event1) = GroupChat::new(group_name, vec![]);
    epg
      .store_event_with_snapshot_opt(&event1, 1, Some(&group_chat))
      .await
      .unwrap();
    let events = epg
      .get_events_by_id_and_seq_nr(group_chat.id().to_string(), 0)
      .await
      .unwrap();
    let mut actual = GroupChat::replay(events, None);
    assert_eq!(group_chat, actual);

    println!("group_chat = {:?}", group_chat);
    println!("actual = {:?}", actual);

    let event2 = actual.add_member_id(MemberID::new()).unwrap();
    epg
      .store_event_with_snapshot_opt(&event2, actual.version(), Some(&actual))
      .await
      .unwrap();

    let event3 = actual.add_member_id(MemberID::new()).unwrap();
    epg
      .store_event_with_snapshot_opt::<GroupChat, GroupChatEvent>(&event3, actual.version() + 1, None)
      .await
      .unwrap();

    let (mut snapshot, seq_nr) = epg
      .get_snapshot_by_id::<GroupChat>(actual.id().to_string())
      .await
      .unwrap();
    println!("snapshot = {:?}", snapshot);
    println!("seq_nr = {:?}", seq_nr);

    let events = epg
      .get_events_by_id_and_seq_nr::<GroupChatEvent>(actual.id().to_string(), seq_nr)
      .await
      .unwrap();

    let actual2 = GroupChat::replay(events, Some(snapshot.clone()));

    assert_eq!(actual, actual2);
  }

  async fn create_journal_table(client: &Client, table_name: &str) -> Result<CreateTableOutput> {
    let pkey_attribute_definition = AttributeDefinition::builder()
      .attribute_name("pkey")
      .attribute_type(ScalarAttributeType::S)
      .build();

    let skey_attribute_definition = AttributeDefinition::builder()
      .attribute_name("skey")
      .attribute_type(ScalarAttributeType::N)
      .build();

    let aid_attribute_definition = AttributeDefinition::builder()
      .attribute_name("aid")
      .attribute_type(ScalarAttributeType::S)
      .build();

    let pkey_schema = KeySchemaElement::builder()
      .attribute_name("pkey")
      .key_type(KeyType::Hash)
      .build();

    let skey_schema = KeySchemaElement::builder()
      .attribute_name("skey")
      .key_type(KeyType::Range)
      .build();

    let provisioned_throughput = ProvisionedThroughput::builder()
      .read_capacity_units(10)
      .write_capacity_units(5)
      .build();

    let gsi = GlobalSecondaryIndex::builder()
      .index_name("aid_index")
      .key_schema(
        KeySchemaElement::builder()
          .attribute_name("aid")
          .key_type(KeyType::Hash)
          .build(),
      )
      .key_schema(
        KeySchemaElement::builder()
          .attribute_name("skey")
          .key_type(KeyType::Range)
          .build(),
      )
      .projection(Projection::builder().projection_type(ProjectionType::All).build())
      .provisioned_throughput(provisioned_throughput.clone())
      .build();

    let result = client
      .create_table()
      .table_name(table_name)
      .attribute_definitions(pkey_attribute_definition)
      .attribute_definitions(skey_attribute_definition)
      .attribute_definitions(aid_attribute_definition)
      .key_schema(pkey_schema)
      .key_schema(skey_schema)
      .global_secondary_indexes(gsi)
      .provisioned_throughput(provisioned_throughput)
      .send()
      .await?;

    Ok(result)
  }
  async fn create_snapshot_table(client: &Client, table_name: &str) -> Result<CreateTableOutput> {
    let pkey_attribute_definition = AttributeDefinition::builder()
      .attribute_name("pkey")
      .attribute_type(ScalarAttributeType::S)
      .build();

    let pkey_schema = KeySchemaElement::builder()
      .attribute_name("pkey")
      .key_type(KeyType::Hash)
      .build();

    let provisioned_throughput = ProvisionedThroughput::builder()
      .read_capacity_units(10)
      .write_capacity_units(5)
      .build();

    let result = client
      .create_table()
      .table_name(table_name)
      .attribute_definitions(pkey_attribute_definition)
      .key_schema(pkey_schema)
      .provisioned_throughput(provisioned_throughput)
      .send()
      .await?;

    Ok(result)
  }

  fn create_client(dynamodb_port: u16) -> Client {
    let region = Region::new("us-west-1");
    let config = aws_sdk_dynamodb::Config::builder()
      .region(Some(region))
      .endpoint_url(format!("http://localhost:{}", dynamodb_port))
      .credentials_provider(Credentials::new("x", "x", None, None, "default"))
      .build();
    let client = Client::from_conf(config);
    client
  }
}

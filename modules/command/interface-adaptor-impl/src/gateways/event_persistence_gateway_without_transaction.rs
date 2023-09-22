use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;
use aws_sdk_dynamodb::operation::put_item::PutItemOutput;
use aws_sdk_dynamodb::operation::update_item::UpdateItemOutput;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use serde::{de, Serialize};

use command_domain::aggregate::{Aggregate, AggregateId};
use command_domain::Event;

use crate::gateways::*;

// 以下に基づいて実装した例。結局無駄な書き込みが防げないので、この実装は不採用。利用していない実装なので完全に無視してください。
// https://aws.amazon.com/jp/blogs/news/build-a-cqrs-event-store-with-amazon-dynamodb/

#[async_trait::async_trait]
impl EventPersistenceGateway for EventPersistenceGatewayWithoutTransaction {
  async fn get_snapshot_by_id<E, T, AID: AggregateId>(&self, aid: &AID) -> Result<(T, usize, usize)>
  where
    E: ?Sized + Serialize + Event + for<'de> de::Deserialize<'de>,
    T: ?Sized + Serialize + Aggregate + for<'de> de::Deserialize<'de>, {
    let response = self
      .client
      .query()
      .table_name(self.snapshot_table_name.clone())
      .index_name(self.snapshot_aid_index_name.clone())
      .key_condition_expression("#aid = :aid AND #seq_nr > :seq_nr")
      .expression_attribute_names("#aid", "aid")
      .expression_attribute_names("#seq_nr", "seq_nr")
      .expression_attribute_values(":aid", AttributeValue::S(aid.to_string()))
      .expression_attribute_values(":seq_nr", AttributeValue::N(0.to_string()))
      .scan_index_forward(false)
      .limit(1)
      .send()
      .await?;

    if let Some(items) = response.items {
      if items.len() == 1 {
        let item = items[0].clone();
        let payload = item.get("payload").unwrap().as_s().unwrap();
        let aggregate = serde_json::from_str::<T>(payload).unwrap();
        let seq_nr = item.get("seq_nr").unwrap().as_n().unwrap().parse::<usize>().unwrap();
        let version = item.get("version").unwrap().as_n().unwrap().parse::<usize>().unwrap();

        // 読み込むたびにこの書き込みが発生する。今のところcondition_expressionでべき等性を担保している。
        let event_str = item.get("last_event").unwrap().as_s().unwrap();
        let event: E = serde_json::from_str::<E>(event_str).unwrap();
        self.put_journal(&event).await?;

        // NOTE: 厳密に対策するには以下のようなトランザクションが結局必要ではないか？
        // if snapshot.last_event.exists() then
        //    tx.begin()
        //    move snapshot.last_event to journal
        //    delete snapshot.last_event
        //    tx.commit()
        // end

        Ok((aggregate, seq_nr, version))
      } else {
        Err(anyhow::anyhow!("No snapshot found for aggregate id: {}", aid))
      }
    } else {
      Err(anyhow::anyhow!("No snapshot found for aggregate id: {}", aid))
    }
  }

  async fn get_events_by_id_and_seq_nr<T, AID: AggregateId>(&self, aid: &AID, seq_nr: usize) -> Result<Vec<T>>
  where
    T: Debug + for<'de> de::Deserialize<'de>, {
    let response = self
      .client
      .query()
      .table_name(self.journal_table_name.clone())
      .index_name(self.journal_aid_index_name.clone())
      .key_condition_expression("#aid = :aid AND #seq_nr > :seq_nr")
      .expression_attribute_names("#aid", "aid")
      .expression_attribute_values(":aid", AttributeValue::S(aid.to_string()))
      .expression_attribute_names("#seq_nr", "seq_nr")
      .expression_attribute_values(":seq_nr", AttributeValue::N(seq_nr.to_string()))
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
    log::info!("epg: events = {:?}", events);
    Ok(events)
  }

  async fn store_event_with_snapshot_opt<A, E>(
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
        let _ = self.put_snapshot_with_event(event, ar).await?;
      }
      (true, None) => {
        panic!("Aggregate is not found");
      }
      (false, ar) => {
        let _ = self.update_snapshot(event, version, ar).await?;
      }
    }
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct EventPersistenceGatewayWithoutTransaction {
  journal_table_name: String,
  journal_aid_index_name: String,
  snapshot_table_name: String,
  snapshot_aid_index_name: String,
  client: Client,
  shard_count: u64,
  key_resolver: Arc<dyn KeyResolver>,
}

unsafe impl Send for EventPersistenceGatewayWithoutTransaction {}

unsafe impl Sync for EventPersistenceGatewayWithoutTransaction {}

impl EventPersistenceGatewayWithoutTransaction {
  pub fn new(
    client: Client,
    journal_table_name: String,
    journal_aid_index_name: String,
    snapshot_table_name: String,
    snapshot_aid_index_name: String,
    shard_count: u64,
  ) -> Self {
    Self::new_with_key_resolver(
      client,
      journal_table_name,
      journal_aid_index_name,
      snapshot_table_name,
      snapshot_aid_index_name,
      shard_count,
      Arc::new(DefaultPartitionKeyResolver),
    )
  }

  pub fn new_with_key_resolver(
    client: Client,
    journal_table_name: String,
    journal_aid_index_name: String,
    snapshot_table_name: String,
    snapshot_aid_index_name: String,
    shard_count: u64,
    key_resolver: Arc<dyn KeyResolver>,
  ) -> Self {
    Self {
      journal_table_name,
      journal_aid_index_name,
      snapshot_table_name,
      snapshot_aid_index_name,
      client,
      shard_count,
      key_resolver,
    }
  }

  async fn put_snapshot_with_event<E, A>(&mut self, event: &E, ar: &A) -> Result<PutItemOutput>
  where
    A: ?Sized + Serialize + Aggregate,
    E: ?Sized + Serialize + Event, {
    let result = self
      .client
      .put_item()
      .table_name(self.snapshot_table_name.to_string())
      .item(
        "pkey",
        AttributeValue::S(self.resolve_pkey(event.aggregate_id(), self.shard_count)),
      )
      // ロックを取る場合は常にskey=resolve_skey(aid, 0)で行う
      .item("skey", AttributeValue::S(self.resolve_skey(event.aggregate_id(), 0)))
      .item("payload", AttributeValue::S(serde_json::to_string(ar)?))
      .item("last_event", AttributeValue::S(serde_json::to_string(event)?))
      .item("aid", AttributeValue::S(event.aggregate_id().to_string()))
      .item("seq_nr", AttributeValue::N(ar.seq_nr().to_string()))
      .item("version", AttributeValue::N("1".to_string()))
      .condition_expression("attribute_not_exists(pkey) AND attribute_not_exists(skey)")
      .send()
      .await?;
    Ok(result)
  }

  async fn update_snapshot<E, A>(&mut self, event: &E, version: usize, ar_opt: Option<&A>) -> Result<UpdateItemOutput>
  where
    A: ?Sized + Serialize + Aggregate,
    E: ?Sized + Serialize + Event, {
    let mut update_snapshot = self
      .client
      .update_item()
      .table_name(self.snapshot_table_name.to_string())
      .update_expression("SET #last_event=:last_event, #version=:after_version")
      .key(
        "pkey",
        AttributeValue::S(self.resolve_pkey(event.aggregate_id(), self.shard_count)),
      )
      // ロックを取る場合は常にskey=resolve_skey(aid, 0)で行う
      .key("skey", AttributeValue::S(self.resolve_skey(event.aggregate_id(), 0)))
      .expression_attribute_names("#last_event", "last_event")
      .expression_attribute_names("#version", "version")
      .expression_attribute_values(":last_event", AttributeValue::S(serde_json::to_string(event)?))
      .expression_attribute_values(":before_version", AttributeValue::N(version.to_string()))
      .expression_attribute_values(":after_version", AttributeValue::N((version + 1).to_string()))
      .condition_expression("#version=:before_version");
    if let Some(ar) = ar_opt {
      update_snapshot = update_snapshot
        .update_expression("SET #payload=:payload, #seq_nr=:seq_nr, #version=:after_version")
        .expression_attribute_names("#seq_nr", "seq_nr")
        .expression_attribute_names("#payload", "payload")
        .expression_attribute_values(":seq_nr", AttributeValue::N(ar.seq_nr().to_string()))
        .expression_attribute_values(":payload", AttributeValue::S(serde_json::to_string(ar)?));
    }
    let result = update_snapshot.send().await?;
    Ok(result)
  }

  async fn put_journal<E>(&self, event: &E) -> Result<()>
  where
    E: ?Sized + Serialize + Event, {
    let pkey = self.resolve_pkey(event.aggregate_id(), self.shard_count);
    let skey = self.resolve_skey(event.aggregate_id(), event.seq_nr());
    let aid = event.aggregate_id().to_string();
    let seq_nr = event.seq_nr().to_string();
    let payload = serde_json::to_string(event)?;
    let occurred_at = event.occurred_at().timestamp_millis().to_string();

    // info!("pkey = {}", pkey);
    // info!("skey = {}", skey);
    // info!("aid = {}", aid);
    // info!("seq_nr = {}", seq_nr);
    // info!("payload = {}", payload);
    // info!("occurred_at = {}", occurred_at);

    let result = self
      .client
      .put_item()
      .table_name(self.journal_table_name.clone())
      .item("pkey", AttributeValue::S(pkey))
      .item("skey", AttributeValue::S(skey))
      .item("aid", AttributeValue::S(aid))
      .item("seq_nr", AttributeValue::N(seq_nr))
      .item("payload", AttributeValue::S(payload))
      .item("occurred_at", AttributeValue::N(occurred_at))
      .condition_expression("attribute_not_exists(pkey) AND attribute_not_exists(skey)")
      .send()
      .await;

    match result {
      Ok(_) => Ok(()),
      Err(error) => {
        log::warn!("put_journal error: {}", error);
        Ok(())
      }
    }
  }

  fn resolve_pkey<AID: AggregateId>(&self, id: &AID, shard_count: u64) -> String {
    self
      .key_resolver
      .resolve_pkey(&id.type_name(), &id.value(), shard_count)
  }

  fn resolve_skey<AID: AggregateId>(&self, id: &AID, seq_nr: usize) -> String {
    self.key_resolver.resolve_skey(&id.type_name(), &id.value(), seq_nr)
  }
}

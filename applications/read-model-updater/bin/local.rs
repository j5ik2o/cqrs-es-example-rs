use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use aws_lambda_events::dynamodb;
use aws_lambda_events::dynamodb::{StreamRecord, StreamViewType};
use aws_sdk_dynamodb::Client as DynamoDBClient;
use aws_sdk_dynamodbstreams::types::ShardIteratorType;
use aws_sdk_dynamodbstreams::Client as DynamoDBStreamsClient;
use chrono::Utc;
use command_domain::id_generate;
use command_interface_adaptor_if::GroupChatReadModelUpdateDao;
use command_interface_adaptor_impl::gateways::group_chat_read_model_dao_impl::GroupChatReadModelUpdateDaoImpl;

use rmu::{process_records, DynamoDbJournalReader, JournalReader};
use serde_dynamo::Item;
use sqlx::MySqlPool;

use read_model_updater::{create_aws_config, load_app_config};

/// ローカル版の Read Model Updater。
///
/// 集約 head テーブルの DynamoDB Streams を購読する。Lambda の代わりに同じ処理関数を
/// 呼び出し、処理位置は `read_model_positions` に記録する。
#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .with_target(false)
    .with_ansi(false)
    .without_time()
    .init();

  let app_settings = load_app_config().unwrap();
  let stream_settings = app_settings
    .stream
    .as_ref()
    .ok_or_else(|| anyhow::anyhow!("stream_settings is not found"))?
    .clone();

  let pool = MySqlPool::connect(&app_settings.database.url).await?;
  let aws_config = create_aws_config(&app_settings.aws).await;
  let dynamodb_client = DynamoDBClient::new(&aws_config);
  let dynamodb_streams_client = DynamoDBStreamsClient::new(&aws_config);
  let dao = GroupChatReadModelUpdateDaoImpl::new(pool);
  let journal_reader = DynamoDbJournalReader::new(dynamodb_client.clone(), stream_settings.journal_table_name.clone());

  loop {
    match stream_events_driver(
      &dynamodb_client,
      &dynamodb_streams_client,
      &dao,
      &journal_reader,
      &stream_settings.head_table_name,
      stream_settings.max_item_count,
      Duration::from_millis(stream_settings.poll_interval_millis),
    )
    .await
    {
      Ok(_) => {}
      Err(err) => {
        tracing::error!(
          "An error has occurred, but stream processing is restarted.\
          If this error persists, the read model condition may be incorrect.: {}",
          err
        );
        tokio::time::sleep(Duration::from_secs(1)).await;
      }
    }
  }
}

fn convert_to(src: aws_sdk_dynamodbstreams::types::AttributeValue) -> serde_dynamo::AttributeValue {
  match src {
    aws_sdk_dynamodbstreams::types::AttributeValue::B(b) => serde_dynamo::AttributeValue::B(b.into_inner()),
    aws_sdk_dynamodbstreams::types::AttributeValue::Bool(b) => serde_dynamo::AttributeValue::Bool(b),
    aws_sdk_dynamodbstreams::types::AttributeValue::Bs(bs) => {
      serde_dynamo::AttributeValue::Bs(bs.iter().map(|e| e.clone().into_inner()).collect())
    }
    aws_sdk_dynamodbstreams::types::AttributeValue::L(l) => {
      serde_dynamo::AttributeValue::L(l.iter().map(|v| convert_to(v.clone())).collect())
    }
    aws_sdk_dynamodbstreams::types::AttributeValue::M(m) => {
      serde_dynamo::AttributeValue::M(m.iter().map(|(k, v)| (k.clone(), convert_to(v.clone()))).collect())
    }
    aws_sdk_dynamodbstreams::types::AttributeValue::N(n) => serde_dynamo::AttributeValue::N(n.clone()),
    aws_sdk_dynamodbstreams::types::AttributeValue::Ns(ns) => serde_dynamo::AttributeValue::Ns(ns.clone()),
    aws_sdk_dynamodbstreams::types::AttributeValue::Null(b) => serde_dynamo::AttributeValue::Null(b),
    aws_sdk_dynamodbstreams::types::AttributeValue::S(s) => serde_dynamo::AttributeValue::S(s.clone()),
    aws_sdk_dynamodbstreams::types::AttributeValue::Ss(ss) => serde_dynamo::AttributeValue::Ss(ss.clone()),
    _ => panic!("not supported"),
  }
}

fn convert_item(
  item: &HashMap<String, aws_sdk_dynamodbstreams::types::AttributeValue>,
) -> HashMap<String, serde_dynamo::AttributeValue> {
  item.iter().map(|(k, v)| (k.clone(), convert_to(v.clone()))).collect()
}

#[derive(Debug, Clone)]
enum ShardState {
  Reading(String),
  Closed,
}

async fn stream_events_driver<D, J>(
  dynamodb_client: &DynamoDBClient,
  dynamodb_streams_client: &DynamoDBStreamsClient,
  dao: &D,
  journal_reader: &J,
  head_table_name: &str,
  max_item_count: i32,
  poll_interval: Duration,
) -> Result<()>
where
  D: GroupChatReadModelUpdateDao,
  J: JournalReader, {
  let describe_table_output = dynamodb_client
    .describe_table()
    .table_name(head_table_name)
    .send()
    .await?;
  let stream_arn = describe_table_output
    .table()
    .and_then(|table| table.latest_stream_arn())
    .ok_or_else(|| {
      anyhow::anyhow!(
        "the head table {} has no DynamoDB Streams; enable Streams with NEW_IMAGE",
        head_table_name
      )
    })?;

  let mut shard_states: HashMap<String, ShardState> = HashMap::new();
  loop {
    let describe_stream_output = dynamodb_streams_client
      .describe_stream()
      .stream_arn(stream_arn)
      .send()
      .await?;
    let shards = describe_stream_output
      .stream_description()
      .map(|description| description.shards().to_vec())
      .unwrap_or_default();
    for shard in shards {
      let Some(shard_id) = shard.shard_id() else {
        continue;
      };
      let shard_id = shard_id.to_string();
      if matches!(shard_states.get(&shard_id), Some(ShardState::Closed)) {
        continue;
      }
      if !shard_states.contains_key(&shard_id) {
        let get_shard_iterator_output = dynamodb_streams_client
          .get_shard_iterator()
          .stream_arn(stream_arn)
          .shard_id(&shard_id)
          .shard_iterator_type(ShardIteratorType::TrimHorizon)
          .send()
          .await?;
        match get_shard_iterator_output.shard_iterator() {
          Some(iterator) => {
            shard_states.insert(shard_id.clone(), ShardState::Reading(iterator.to_string()));
          }
          None => {
            shard_states.insert(shard_id.clone(), ShardState::Closed);
            continue;
          }
        }
      }
      loop {
        let iterator = match shard_states.get(&shard_id) {
          Some(ShardState::Reading(iterator)) => Some(iterator.clone()),
          _ => None,
        };
        let Some(iterator) = iterator else {
          break;
        };
        let get_records_output = dynamodb_streams_client
          .get_records()
          .shard_iterator(iterator)
          .limit(max_item_count)
          .send()
          .await?;
        let records = get_records_output.records();
        if !records.is_empty() {
          let event_records = event_records_from_stream_records(records)?;
          process_records(dao, journal_reader, event_records).await?;
        }
        match get_records_output.next_shard_iterator() {
          Some(next_iterator) => {
            shard_states.insert(shard_id.clone(), ShardState::Reading(next_iterator.to_string()));
          }
          None => {
            shard_states.insert(shard_id.clone(), ShardState::Closed);
            break;
          }
        }
        if records.is_empty() {
          break;
        }
      }
    }
    tokio::time::sleep(poll_interval).await;
  }
}

fn event_records_from_stream_records(
  records: &[aws_sdk_dynamodbstreams::types::Record],
) -> Result<Vec<dynamodb::EventRecord>> {
  records
    .iter()
    .map(|record| {
      let stream_record = record
        .dynamodb()
        .ok_or_else(|| anyhow::anyhow!("the stream record has no dynamodb payload"))?;
      let new_image: Item = convert_item(stream_record.new_image().unwrap_or(&HashMap::new())).into();
      let old_image: Item = convert_item(stream_record.old_image().unwrap_or(&HashMap::new())).into();
      let keys: Item = convert_item(stream_record.keys().unwrap_or(&HashMap::new())).into();
      let event_name = record
        .event_name()
        .map(|name| name.as_str().to_string())
        .unwrap_or_else(|| "INSERT".to_string());
      Ok(dynamodb::EventRecord {
        aws_region: record.aws_region().unwrap_or("ap-northeast-1").to_string(),
        change: StreamRecord {
          approximate_creation_date_time: Utc::now(),
          keys,
          new_image,
          old_image,
          sequence_number: stream_record.sequence_number().map(|value| value.to_string()),
          size_bytes: stream_record.size_bytes().unwrap_or_default(),
          stream_view_type: Some(StreamViewType::NewImage),
        },
        event_id: record
          .event_id()
          .map(|value| value.to_string())
          .unwrap_or_else(|| id_generate().to_string()),
        event_name,
        event_source: record.event_source().map(|value| value.to_string()),
        event_version: record.event_version().map(|value| value.to_string()),
        event_source_arn: None,
        user_identity: None,
        record_format: None,
        table_name: None,
      })
    })
    .collect()
}

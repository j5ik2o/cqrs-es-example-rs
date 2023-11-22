use std::collections::HashMap;
use std::mem::size_of;

use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_lambda_events::dynamodb;
use aws_lambda_events::dynamodb::{StreamRecord, StreamViewType};
use aws_sdk_dynamodb::Client as DynamoDBClient;
use aws_sdk_dynamodbstreams::config::{Credentials, Region};
use aws_sdk_dynamodbstreams::types::ShardIteratorType;
use aws_sdk_dynamodbstreams::Client as DynamoDBStreamsClient;
use chrono::Utc;
use command_domain::id_generate;
use command_interface_adaptor_impl::gateways::group_chat_read_model_dao_impl::GroupChatReadModelUpdateDaoImpl;
use http::{HeaderMap, HeaderValue};
use lambda_runtime::{Context, LambdaEvent};

use serde_dynamo::Item;
use sqlx::{MySql, MySqlPool, Pool};

use read_model_updater::{load_app_config, AwsSettings};

// ローカル版のRead Model Updater
#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .with_target(false)
    .with_ansi(false)
    .without_time()
    .init();

  let app_settings = load_app_config().unwrap();

  let pool = MySqlPool::connect(&app_settings.database.url).await?;
  let dynamodb_client = create_aws_client(&app_settings.aws).await;
  let dynamodb_streams_client = create_aws_dynamodb_streams_client(&app_settings.aws).await;
  if let Some(stream_settings) = &app_settings.stream {
    loop {
      match stream_events_driver(
        &dynamodb_client,
        &dynamodb_streams_client,
        pool.clone(),
        &stream_settings.journal_table_name,
        stream_settings.max_item_count,
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
          continue;
        }
      }
    }
  } else {
    Err(anyhow::anyhow!("stream_settings is not found"))
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

async fn stream_events_driver(
  dynamodb_client: &DynamoDBClient,
  dynamodb_streams_client: &DynamoDBStreamsClient,
  pool: Pool<MySql>,
  journal_table_name: &str,
  max_item_count: usize,
) -> Result<()> {
  let describe_table_out = dynamodb_client
    .describe_table()
    .table_name(journal_table_name)
    .send()
    .await?;
  let stream_arn = describe_table_out.table().unwrap().latest_stream_arn().unwrap();

  let dao = GroupChatReadModelUpdateDaoImpl::new(pool);
  let mut last_evaluated_shard_id: Option<String> = None;
  loop {
    tracing::info!("stream_arn = {:?}", stream_arn);
    tracing::info!("last_evaluated_shard_id = {:?}", last_evaluated_shard_id);
    tracing::info!("max_item_count = {:?}", max_item_count);

    let mut builder = dynamodb_streams_client.describe_stream().stream_arn(stream_arn);
    if let Some(shard_id) = last_evaluated_shard_id.clone() {
      builder = builder.exclusive_start_shard_id(shard_id);
    }
    let describe_stream_output = builder.send().await?;
    let shards = describe_stream_output.stream_description().unwrap().shards().to_vec();

    for shard in shards {
      tracing::info!("shard = {:?}", shard);
      let get_shard_iterator_output = dynamodb_streams_client
        .get_shard_iterator()
        .stream_arn(stream_arn)
        .shard_id(shard.shard_id().unwrap())
        .shard_iterator_type(ShardIteratorType::Latest)
        .send()
        .await?;
      let mut shard_iterator_opt = get_shard_iterator_output.shard_iterator().map(|s| s.to_owned());
      let mut processed_record_count = 0usize;
      while shard_iterator_opt.is_some() && processed_record_count < max_item_count {
        // tracing::info!("shard_iterator = {:?}", shard_iterator_opt.clone().unwrap());
        let get_records_output = dynamodb_streams_client
          .get_records()
          .shard_iterator(shard_iterator_opt.unwrap())
          .send()
          .await?;
        let records = get_records_output.records();
        for record in records {
          let stream_record = record.dynamodb.clone().unwrap();
          // tracing::info!("dynamodb stream event = {:?}", stream_record);

          let new_image = stream_record
            .new_image()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), convert_to(v.clone())))
            .collect::<HashMap<String, serde_dynamo::AttributeValue>>();

          let item: serde_dynamo::Item = new_image.clone().into();
          let keys = stream_record.keys().cloned().unwrap();
          let key_item: serde_dynamo::Item = keys
            .iter()
            .map(|(k, v)| (k.clone(), convert_to(v.clone())))
            .collect::<HashMap<String, serde_dynamo::AttributeValue>>()
            .into();

          let sequence_number = id_generate();
          let event_id = id_generate();

          let event = dynamodb::Event {
            records: vec![dynamodb::EventRecord {
              aws_region: "ap-northeast-1".to_string(),
              change: StreamRecord {
                approximate_creation_date_time: Utc::now(),
                keys: key_item,
                new_image: item.clone(),
                old_image: item.clone(),
                sequence_number: Some(sequence_number.to_string()),
                size_bytes: size_of::<Item>() as i64,
                stream_view_type: Some(StreamViewType::NewImage),
              },
              event_id: event_id.to_string(),
              event_name: "INSERT".to_string(),
              event_source: None,
              event_version: None,
              event_source_arn: Some(
                "arn:aws:dynamodb:us-east-1:123456789012:table/Example-Table/stream/2016-12-01T00:00:00.000"
                  .to_string(),
              ),
              user_identity: None,
              record_format: None,
              table_name: None,
            }],
          };

          let request_id = id_generate().to_string();
          let deadline_ms = (Utc::now().timestamp_millis() + 3000).to_string();

          let mut headers = HeaderMap::new();
          headers.insert(
            "lambda-runtime-aws-request-id",
            HeaderValue::from_str(&request_id).unwrap(),
          );
          headers.insert(
            "lambda-runtime-deadline-ms",
            HeaderValue::from_str(&deadline_ms).unwrap(),
          );

          let context = Context::try_from(headers).unwrap();
          let lambda_event = LambdaEvent::new(event, context);

          read_model_updater::update_read_model(&dao, lambda_event).await?;
        }
        processed_record_count += records.len();
        shard_iterator_opt = get_records_output.next_shard_iterator().map(|s| s.to_owned())
      }
    }

    let next_last_evaluated_shard_id = describe_stream_output
      .stream_description()
      .unwrap()
      .last_evaluated_shard_id()
      .map(|s| s.to_owned());

    if next_last_evaluated_shard_id.is_none() {
      break;
    }

    last_evaluated_shard_id = next_last_evaluated_shard_id;
  }

  Ok(())
}

async fn create_aws_client(aws_settings: &AwsSettings) -> DynamoDBClient {
  let region_name = aws_settings.region_name.clone();
  let region = Region::new(region_name);
  let region_provider_chain = RegionProviderChain::default_provider().or_else(region);

  let mut config_loader = aws_config::from_env().region(region_provider_chain);
  if let Some(endpoint_url) = aws_settings.endpoint_url.clone() {
    tracing::info!("endpoint_url = {}", endpoint_url);
    config_loader = config_loader.endpoint_url(endpoint_url);
  }

  match (
    aws_settings.access_key_id.clone(),
    aws_settings.secret_access_key.clone(),
  ) {
    (Some(access_key_id), Some(secret_access_key)) => {
      tracing::info!("access_key_id = {}", access_key_id);
      tracing::info!("secret_access_key = {}", secret_access_key);
      config_loader = config_loader.credentials_provider(Credentials::new(
        access_key_id,
        secret_access_key,
        None,
        None,
        "default",
      ));
    }
    _ => {}
  }

  let config = config_loader.load().await;

  DynamoDBClient::new(&config)
}

async fn create_aws_dynamodb_streams_client(aws_settings: &AwsSettings) -> DynamoDBStreamsClient {
  let region_name = aws_settings.region_name.clone();
  let region = Region::new(region_name);
  let region_provider_chain = RegionProviderChain::default_provider().or_else(region);
  let mut config_loader = aws_config::from_env().region(region_provider_chain);
  if let Some(endpoint_url) = aws_settings.endpoint_url.clone() {
    config_loader = config_loader.endpoint_url(endpoint_url);
  }
  match (
    aws_settings.access_key_id.clone(),
    aws_settings.secret_access_key.clone(),
  ) {
    (Some(access_key_id), Some(secret_access_key)) => {
      config_loader = config_loader.credentials_provider(Credentials::new(
        access_key_id,
        secret_access_key,
        None,
        None,
        "default",
      ));
    }
    _ => {}
  }
  let config = config_loader.load().await;

  DynamoDBStreamsClient::new(&config)
}

use std::error::Error;

use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodbstreams::{Client as DynamoDBStreamsClient, Config};
use aws_sdk_dynamodbstreams::config::{Credentials, Region};
use aws_sdk_dynamodbstreams::operation::describe_stream::builders::DescribeStreamFluentBuilder;
use aws_sdk_dynamodbstreams::types::ShardIteratorType;
use config::Environment;
use serde::{Deserialize, Serialize};

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .init();
    println!("Hello, world!");
}

async fn stream_events_driver(
    dynamodb_streams_client: &DynamoDBStreamsClient,
    stream_arn: &str,
    max_item_count: usize,
) -> Result<()> {
    let mut last_evaluated_shard_id: Option<String> = None;

    tracing::info!("stream_arn = {:?}", stream_arn);
    tracing::info!("last_evaluated_shard_id = {:?}", last_evaluated_shard_id);
    tracing::info!("max_item_count = {:?}", max_item_count);
    loop {
        let mut builder = dynamodb_streams_client
            .describe_stream()
            .stream_arn(stream_arn);
        if let Some(shardId) = last_evaluated_shard_id.clone() {
            builder = builder.exclusive_start_shard_id(shardId);
        }
        let describe_stream_output = builder.send().await?;
        let shards = describe_stream_output
            .stream_description()
            .unwrap()
            .shards()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        for shard in shards {
            tracing::info!("shard = {:?}", shard);
            let get_shard_iterator_output = dynamodb_streams_client
                .get_shard_iterator()
                .stream_arn(stream_arn)
                .shard_id(shard.shard_id().unwrap())
                .shard_iterator_type(ShardIteratorType::Latest)
                .send()
                .await?;
            let mut shard_iterator_opt = get_shard_iterator_output
                .shard_iterator()
                .map(|s| s.to_owned());
            let mut processed_record_count = 0usize;
            while { shard_iterator_opt.is_some() && processed_record_count < max_item_count } {
                tracing::info!("shard_iterator = {:?}", shard_iterator_opt.clone().unwrap());
                let get_records_output = dynamodb_streams_client
                    .get_records()
                    .shard_iterator(shard_iterator_opt.unwrap())
                    .send()
                    .await?;
                let records = get_records_output.records().unwrap();
                for record in records {
                    let event = record.dynamodb.clone().unwrap();
                    tracing::info!("dynamodb stream event = {:?}", event);
                    cqrs_es_example_rmu::update_read_model()
                    update_
                }
                processed_record_count += records.len();
                shard_iterator_opt = get_records_output
                    .next_shard_iterator()
                    .map(|s| s.to_owned())
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
    }

    Ok(())
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
    let client = DynamoDBStreamsClient::new(&config);
    client
}

#[derive(Deserialize, Debug)]
struct AwsSettings {
    region_name: String,
    endpoint_url: Option<String>,
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
}

fn load_app_config() -> Result<AwsSettings> {
    let config = config::Config::builder()
        .add_source(config::File::with_name("config/rmu"))
        .add_source(Environment::with_prefix("RMU"))
        .build()?;
    let app_config = config.try_deserialize()?;
    Ok(app_config)
}

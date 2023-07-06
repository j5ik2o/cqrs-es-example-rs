#[macro_use]
extern crate log;

use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_lambda_events::event::dynamodb;
use aws_sdk_dynamodbstreams::operation::describe_stream::builders::DescribeStreamFluentBuilder;
use aws_sdk_dynamodbstreams::types::ShardIteratorType;
use aws_sdk_dynamodbstreams::Client as DynamoDBStreamsClient;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Deserialize;
use tracing::instrument::WithSubscriber;

//
// static INIT: Once = Once::new();
//
// async fn update_read_model(event: LambdaEvent<dynamodb::Event>) -> Result<(), Error> {
//     info!("event: {:?}", event);
//     event.payload.records.iter().for_each(|record| {
//         record.change.new_image.iter().for_each(|(key, value)| {
//             let str = match value {
//                 AttributeValue::S(v) => v.clone(),
//                 _ => panic!("unexpected type"),
//             };
//             let parsed: Value = serde_json::from_str(&str).unwrap();
//             let type_value = &parsed["type"];
//             let type_value_str = type_value.as_str().unwrap();
//             match type_value_str {
//                 s if s.starts_with("Thread") => {
//                     let ev = serde_json::from_str::<ThreadEvent>(&str).unwrap();
//                     info!("key: {:?}, value: {:?}", key, ev);
//                 }
//                 _ => {}
//             }
//         });
//     });
//     Ok(())
// }

// fn extract_file_name(s: &str) -> Option<&str> {
//     let index = s.rfind("/src/")?;
//     Some(&s[(index)..])
// }
//
// pub fn setup_logger() {
//     INIT.call_once(|| {
//         let mut builder = Builder::from_env(Env::default().default_filter_or("info"));
//         builder
//             .format(move |buf, record| {
//                 let file_name = record
//                     .file()
//                     .and_then(extract_file_name)
//                     .unwrap_or_else(|| record.file().unwrap_or("unknown"));
//                 writeln!(
//                     buf,
//                     "{} [{}] - {} on {}{}:{}",
//                     chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
//                     record.level(),
//                     record.args(),
//                     record.module_path().unwrap_or("unknown"),
//                     file_name,
//                     record.line().unwrap_or(0),
//                 )
//             })
//             .target(Target::Stdout)
//             .init();
//     });
// }

async fn my_handler(event: LambdaEvent<dynamodb::Event>) -> Result<(), Error> {
    tracing::info!("test = {:?}", event);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .init();
    tracing::info!("main: start");
    let func = service_fn(my_handler);
    lambda_runtime::run(func).await?;
    tracing::info!("main: finished");
    Ok(())
}

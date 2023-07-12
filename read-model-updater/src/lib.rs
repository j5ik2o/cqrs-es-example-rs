use anyhow::Result;
use aws_lambda_events::dynamodb;
use config::Environment;
use lambda_runtime::LambdaEvent;
use serde::Deserialize;
use serde_dynamo::AttributeValue;
use serde_json::Value;

use cqrs_es_example_domain::thread::events::ThreadEvent;

use crate::thread_read_model_dao::ThreadReadModelDao;

pub mod thread_read_model_dao;

pub async fn update_read_model<D: ThreadReadModelDao>(thread_read_model_dao: &D, event: LambdaEvent<dynamodb::Event>) {
  for record in event.payload.records {
    let attribute_values = record.change.new_image.clone().into_inner();
    let payload_str = match attribute_values.get("payload").unwrap() {
      AttributeValue::S(v) => v.clone(),
      _ => panic!("unexpected type"),
    };
    let type_value_str = get_type_string(&payload_str);
    match type_value_str {
      s if s.starts_with("Thread") => {
        let ev = serde_json::from_str::<ThreadEvent>(&payload_str).unwrap();
        match &ev {
          ThreadEvent::ThreadCreated(body) => thread_read_model_dao.insert_thread(body).await.unwrap(),
          ThreadEvent::ThreadDeleted(body) => thread_read_model_dao.delete_thread(body).await.unwrap(),
          ThreadEvent::ThreadRenamed(body) => thread_read_model_dao.update_thread_name(body).await.unwrap(),
          ThreadEvent::ThreadMemberAdd(body) => thread_read_model_dao.insert_member(body).await.unwrap(),
          ThreadEvent::ThreadMemberRemoved(body) => thread_read_model_dao.delete_member(body).await.unwrap(),
          ThreadEvent::ThreadMessagePosted(body) => thread_read_model_dao.post_message(body).await.unwrap(),
          ThreadEvent::ThreadMessageDeleted(body) => thread_read_model_dao.delete_message(body).await.unwrap(),
        }
      }
      _ => {}
    }
  }
}

fn get_type_string(payload_str: &String) -> String {
  let parsed: Value = serde_json::from_str(&payload_str).unwrap();
  let type_value = &parsed["type"];
  let type_value_str = type_value.as_str().unwrap();
  type_value_str.to_string()
}

#[derive(Deserialize, Debug)]
pub struct AwsSettings {
  pub region_name: String,
  pub endpoint_url: Option<String>,
  pub access_key_id: Option<String>,
  pub secret_access_key: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
  pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct StreamSettings {
  pub journal_table_name: String,
  pub max_item_count: usize,
}

#[derive(Deserialize, Debug)]
pub struct AppSettings {
  pub aws: AwsSettings,
  pub stream: StreamSettings,
  pub database: DatabaseSettings,
}

pub fn load_app_config() -> Result<AppSettings> {
  let config = config::Config::builder()
    .add_source(config::File::with_name("config/read-model-updater").required(false))
    .add_source(Environment::with_prefix("APP").try_parsing(true).separator("__"))
    .build()?;
  let app_config = config.try_deserialize()?;
  Ok(app_config)
}

#[cfg(test)]
#[allow(deprecated)]
mod test {
  use aws_lambda_events::dynamodb::Event;
  use http::{HeaderMap, HeaderValue};
  use lambda_runtime::Context;
  use serde_json;

  use crate::thread_read_model_dao::MockThreadReadModelDao;

  use super::*;

  #[tokio::test]
  async fn example_dynamodb_event() {
    let data = include_bytes!("../fixtures/example-dynamodb-event.json");
    let parsed: Event = serde_json::from_slice(data).unwrap();
    let output: String = serde_json::to_string(&parsed).unwrap();
    // println!("output: {}", output);
    let reparsed: Event = serde_json::from_slice(output.as_bytes()).unwrap();
    assert_eq!(parsed, reparsed);

    //let event = parsed.records.pop().unwrap();
    //let date = Utc.ymd(2016, 12, 2).and_hms(1, 27, 0);
    //assert_eq!(date, event.change.approximate_creation_date_time);

    let mut headers = HeaderMap::new();
    headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
    headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
    let context = Context::try_from(headers).unwrap();
    let le = LambdaEvent::new(parsed, context);

    let dao = MockThreadReadModelDao;

    update_read_model(&dao, le).await;
  }
}

use anyhow::Result;
use aws_lambda_events::dynamodb;
use command_interface_adaptor_if::GroupChatReadModelUpdateDao;
use config::Environment;
use lambda_runtime::LambdaEvent;
use serde::Deserialize;
use serde_dynamo::AttributeValue;
use serde_json::Value;

use command_domain::group_chat::GroupChatEvent;
use command_domain::group_chat::MemberRole;

// NOTE: イベントのシーケンス番号とリードモデルのシーケンス番号がズレないことを前提にしているため
// DynamoDBを初期化した際は、必ずAurora側のデータベースも初期化すること
pub async fn update_read_model<D: GroupChatReadModelUpdateDao>(
  group_chat_read_model_dao: &D,
  event: LambdaEvent<dynamodb::Event>,
) -> Result<()> {
  tracing::info!("Rust function invoked: event = {:?}", event);
  for record in event.payload.records {
    let attribute_values = record.change.new_image.clone().into_inner();
    tracing::info!("attribute_values = {:?}", attribute_values);
    let payload_str = match attribute_values.get("payload").unwrap() {
      AttributeValue::S(v) => v.clone(),
      AttributeValue::B(v) => String::from_utf8(v.clone()).unwrap(),
      _ => panic!("unexpected type"),
    };
    tracing::info!("payload_str = {}", payload_str);
    let type_value_str = get_type_string(&payload_str);
    tracing::info!("type_value_str = {}", type_value_str);
    match type_value_str {
      s if s.starts_with("GroupChat") => {
        let ev = serde_json::from_str::<GroupChatEvent>(&payload_str).unwrap();
        tracing::info!("ev = {:?}", ev);
        match &ev {
          GroupChatEvent::GroupChatCreated(body) => {
            group_chat_read_model_dao
              .insert_group_chat(
                body.aggregate_id.clone(),
                body.name.clone(),
                body
                  .members
                  .administrator_id()
                  .breach_encapsulation_of_user_account_id()
                  .clone(),
                body.occurred_at,
              )
              .await
              .unwrap();
            group_chat_read_model_dao
              .insert_member(
                body.aggregate_id.clone(),
                body.members.administrator_id().breach_encapsulation_of_id().clone(),
                body
                  .members
                  .administrator_id()
                  .breach_encapsulation_of_user_account_id()
                  .clone(),
                MemberRole::Admin,
                body.occurred_at,
              )
              .await
              .unwrap();
          }
          GroupChatEvent::GroupChatDeleted(body) => group_chat_read_model_dao
            .delete_group_chat(body.aggregate_id.clone(), body.occurred_at.clone())
            .await
            .unwrap(),
          GroupChatEvent::GroupChatRenamed(body) => group_chat_read_model_dao
            .rename_group_chat(body.aggregate_id.clone(), body.name.clone(), body.occurred_at.clone())
            .await
            .unwrap(),
          GroupChatEvent::GroupChatMemberAdded(body) => group_chat_read_model_dao
            .insert_member(
              body.aggregate_id.clone(),
              body.member.breach_encapsulation_of_id().clone(),
              body.member.breach_encapsulation_of_user_account_id().clone(),
              body.member.breach_encapsulation_of_role().clone(),
              body.occurred_at,
            )
            .await
            .unwrap(),
          GroupChatEvent::GroupChatMemberRemoved(body) => group_chat_read_model_dao
            .delete_member(body.aggregate_id.clone(), body.user_account_id.clone())
            .await
            .unwrap(),
          GroupChatEvent::GroupChatMessagePosted(body) => group_chat_read_model_dao
            .insert_message(
              body.aggregate_id.clone(),
              body.message.clone(),
              body.occurred_at.clone(),
            )
            .await
            .unwrap(),
          GroupChatEvent::GroupChatMessageDeleted(body) => group_chat_read_model_dao
            .delete_message(
              body.aggregate_id.clone(),
              body.message_id.clone(),
              body.occurred_at.clone(),
            )
            .await
            .unwrap(),
        }
      }
      _ => {}
    }
  }
  tracing::info!("Rust function responds to event");
  Ok(())
}

/// DynamoDBのストリームから取得したイベントのペイロードからイベントタイプを取得する
fn get_type_string(payload_str: &String) -> String {
  let parsed: Value = serde_json::from_str(payload_str).unwrap();
  let type_value = &parsed["type"];
  let type_value_str = type_value.as_str().unwrap();
  type_value_str.to_string()
}

#[derive(Deserialize, Debug)]
pub struct AwsSettings {
  /// AWSリージョン名
  pub region_name: String,
  /// DynamoDBのエンドポイントURL(ローカル開発用)
  pub endpoint_url: Option<String>,
  /// アクセスキーID(ローカル開発用)
  pub access_key_id: Option<String>,
  /// シークレットアクセスキー(ローカル開発用)
  pub secret_access_key: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
  /// データベース接続URL
  pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct RedisSettings {
  pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct StreamSettings {
  /// コンシュームするジャーナルのテーブル名
  pub journal_table_name: String,
  /// ストリームから読み込む最大件数
  pub max_item_count: usize,
}

#[derive(Deserialize, Debug)]
pub struct AppSettings {
  pub aws: AwsSettings,
  pub stream: Option<StreamSettings>,
  pub database: DatabaseSettings,
}

// ---

pub fn load_app_config() -> Result<AppSettings> {
  let config = config::Config::builder()
    .add_source(config::File::with_name("config/read-model-updater").required(false))
    .add_source(Environment::with_prefix("APP").try_parsing(true).separator("__"))
    .build()?;
  let app_config = config.try_deserialize()?;
  Ok(app_config)
}

#[cfg(test)]
mod tests {
  use aws_lambda_events::dynamodb::Event;
  use chrono::Utc;
  use command_domain::id_generate;
  use command_interface_adaptor_impl::gateways::group_chat_read_model_dao_impl::MockGroupChatReadModelUpdateDao;
  use http::{HeaderMap, HeaderValue};
  use lambda_runtime::Context;
  use once_cell::sync::Lazy;

  use super::*;

  static REQUEST_ID: Lazy<String> = Lazy::new(|| id_generate().to_string());
  static DEADLINE_MS: Lazy<String> = Lazy::new(|| (Utc::now().timestamp_millis() + 3000).to_string());

  /// DynamoDBのイベント(ダミー)のペイロードからイベントタイプを取得する
  #[tokio::test]
  async fn example_dynamodb_event() {
    let data = include_bytes!("../fixtures/example-dynamodb-event.json");
    let parsed: Event = serde_json::from_slice(data).unwrap();
    let output: String = serde_json::to_string(&parsed).unwrap();
    // println!("output: {}", output);
    let reparsed: Event = serde_json::from_slice(output.as_bytes()).unwrap();
    assert_eq!(parsed, reparsed);

    let mut headers = HeaderMap::new();
    headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static(&REQUEST_ID));
    headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static(&DEADLINE_MS));
    let context = Context::try_from(headers).unwrap();
    let le = LambdaEvent::new(parsed, context);

    let dao = MockGroupChatReadModelUpdateDao;

    update_read_model(&dao, le).await.unwrap();
  }
}

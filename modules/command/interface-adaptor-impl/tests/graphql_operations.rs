use async_graphql::Request;
use serde_json::Value;
use serial_test::serial;

use command_domain::user_account::UserAccountId;
use command_interface_adaptor_impl::graphql::{create_schema, ApiSchema};

use crate::common::{get_repository, init_logger};

mod common;

async fn execute(schema: &ApiSchema, query: &str) -> Value {
  let response = schema.execute(Request::new(query)).await;
  assert!(response.errors.is_empty(), "GraphQL errors: {:?}", response.errors);
  response.data.into_json().unwrap()
}

fn extract_group_chat_id(data: &Value, field: &str) -> String {
  data[field]["groupChatId"].as_str().unwrap().to_string()
}

#[tokio::test]
#[serial]
async fn should_expose_all_write_api_mutations_and_health_check() {
  init_logger();
  let (repository, container, client) = get_repository().await;
  let schema = create_schema(repository);

  let health = execute(&schema, "query { healthCheck }").await;
  assert_eq!(health["healthCheck"].as_str(), Some("OK"));

  let admin_id = UserAccountId::new().to_string();
  let member_id = UserAccountId::new().to_string();

  let created = execute(
    &schema,
    &format!(
      r#"mutation {{ createGroupChat(input: {{ name: "chat", executorId: "{admin_id}" }}) {{ groupChatId }} }}"#
    ),
  )
  .await;
  let group_chat_id = extract_group_chat_id(&created, "createGroupChat");

  let renamed = execute(
    &schema,
    &format!(
      r#"mutation {{ renameGroupChat(input: {{ groupChatId: "{group_chat_id}", name: "renamed", executorId: "{admin_id}" }}) {{ groupChatId }} }}"#
    ),
  )
  .await;
  assert_eq!(extract_group_chat_id(&renamed, "renameGroupChat"), group_chat_id);

  let added = execute(
    &schema,
    &format!(
      r#"mutation {{ addMember(input: {{ groupChatId: "{group_chat_id}", userAccountId: "{member_id}", role: MEMBER, executorId: "{admin_id}" }}) {{ groupChatId }} }}"#
    ),
  )
  .await;
  assert_eq!(extract_group_chat_id(&added, "addMember"), group_chat_id);

  let posted = execute(
    &schema,
    &format!(
      r#"mutation {{ postMessage(input: {{ groupChatId: "{group_chat_id}", content: "hello", executorId: "{admin_id}" }}) {{ groupChatId messageId }} }}"#
    ),
  )
  .await;
  assert_eq!(extract_group_chat_id(&posted, "postMessage"), group_chat_id);
  let message_id = posted["postMessage"]["messageId"].as_str().unwrap().to_string();

  let edited = execute(
    &schema,
    &format!(
      r#"mutation {{ editMessage(input: {{ groupChatId: "{group_chat_id}", messageId: "{message_id}", content: "edited", executorId: "{admin_id}" }}) {{ groupChatId }} }}"#
    ),
  )
  .await;
  assert_eq!(extract_group_chat_id(&edited, "editMessage"), group_chat_id);

  let deleted_message = execute(
    &schema,
    &format!(
      r#"mutation {{ deleteMessage(input: {{ groupChatId: "{group_chat_id}", messageId: "{message_id}", executorId: "{admin_id}" }}) {{ groupChatId }} }}"#
    ),
  )
  .await;
  assert_eq!(extract_group_chat_id(&deleted_message, "deleteMessage"), group_chat_id);

  let removed = execute(
    &schema,
    &format!(
      r#"mutation {{ removeMember(input: {{ groupChatId: "{group_chat_id}", userAccountId: "{member_id}", executorId: "{admin_id}" }}) {{ groupChatId }} }}"#
    ),
  )
  .await;
  assert_eq!(extract_group_chat_id(&removed, "removeMember"), group_chat_id);

  let deleted_chat = execute(
    &schema,
    &format!(
      r#"mutation {{ deleteGroupChat(input: {{ groupChatId: "{group_chat_id}", executorId: "{admin_id}" }}) {{ groupChatId }} }}"#
    ),
  )
  .await;
  assert_eq!(extract_group_chat_id(&deleted_chat, "deleteGroupChat"), group_chat_id);

  drop(client);
  container.stop().await.unwrap();
  drop(container);
}

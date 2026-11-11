use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use chrono::{Duration, TimeZone, Utc};
use command_domain::group_chat::{GroupChatId, GroupChatName, MemberId, MemberRole, Message, MessageId};
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::GroupChatReadModelUpdateDao;
use command_interface_adaptor_impl::gateways::group_chat_read_model_dao_impl::GroupChatReadModelUpdateDaoImpl;
use query_interface_adaptor::controllers::create_router;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerRequest, GenericImage, ImageExt};
use tower::ServiceExt;

fn mysql_image() -> ContainerRequest<GenericImage> {
  let port = ContainerPort::from(3306);
  GenericImage::new("mysql", "26.10")
    .with_exposed_port(port)
    .with_wait_for(WaitFor::message_on_stdout("Ready for start up"))
    .with_startup_timeout(std::time::Duration::from_secs(180))
    .with_env_var("MYSQL_ROOT_PASSWORD", "password")
    .with_env_var("MYSQL_DATABASE", "ceer")
    .with_env_var("MYSQL_USER", "ceer")
    .with_env_var("MYSQL_PASSWORD", "ceer")
}

mod embedded {
  use refinery::embed_migrations;

  embed_migrations!("../../tools/refinery/migrations");
}

fn make_database_url_for_migration(port: u16) -> String {
  format!("mysql://root:password@localhost:{}/ceer", port)
}

fn make_database_url_for_application(port: u16) -> String {
  format!("mysql://ceer:ceer@localhost:{}/ceer", port)
}

fn refinery_migrate(port: u16) {
  let url = make_database_url_for_migration(port);
  let opts = refinery_core::mysql::Opts::from_url(&url).unwrap();
  let mut pool_result;
  while {
    pool_result = refinery_core::mysql::Pool::new(opts.clone());
    pool_result.is_err()
  } {
    std::thread::sleep(std::time::Duration::from_secs(1));
  }
  let pool = pool_result.unwrap();
  let mut attempts = 0;
  loop {
    let result = pool
      .get_conn()
      .map_err(|error| format!("{:?}", error))
      .and_then(|mut conn| {
        embedded::migrations::runner()
          .run(&mut conn)
          .map_err(|error| format!("{:?}", error))
      });
    match result {
      Ok(_) => break,
      Err(error) if attempts < 5 => {
        eprintln!("retry migration after error: {}", error);
        attempts += 1;
        std::thread::sleep(std::time::Duration::from_secs(2));
      }
      Err(error) => panic!("failed to migrate the read model: {}", error),
    }
  }
}

async fn execute(router: &Router, query: &str) -> Value {
  let body = json!({ "query": query }).to_string();
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/query")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
  serde_json::from_slice(&bytes).unwrap()
}

fn error_code(response: &Value) -> Option<&str> {
  response["errors"][0]["extensions"]["code"].as_str()
}

#[tokio::test]
async fn should_scope_read_model_by_user_account_when_querying_through_graphql() {
  let mysql_node = mysql_image().start().await.unwrap();
  let mysql_port = mysql_node.get_host_port_ipv4(3306).await.unwrap();
  refinery_migrate(mysql_port);

  let pool = MySqlPool::connect(&make_database_url_for_application(mysql_port))
    .await
    .unwrap();
  let update_dao = GroupChatReadModelUpdateDaoImpl::new(pool.clone());

  let admin_id = UserAccountId::new();
  let member_id = UserAccountId::new();
  let other_admin_id = UserAccountId::new();
  let outsider_id = UserAccountId::new();

  let created_at = Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap();
  let message_created_at = created_at + Duration::seconds(10);
  let second_message_created_at = created_at + Duration::seconds(20);
  let deleted_message_created_at = created_at + Duration::seconds(30);

  let group_chat_id = GroupChatId::new();
  update_dao
    .insert_group_chat(
      group_chat_id.clone(),
      GroupChatName::new("chat-1").unwrap(),
      admin_id.clone(),
      created_at,
    )
    .await
    .unwrap();
  update_dao
    .insert_member(
      group_chat_id.clone(),
      MemberId::new(),
      admin_id.clone(),
      MemberRole::Admin,
      created_at,
    )
    .await
    .unwrap();
  update_dao
    .insert_member(
      group_chat_id.clone(),
      MemberId::new(),
      member_id.clone(),
      MemberRole::Member,
      created_at + Duration::seconds(1),
    )
    .await
    .unwrap();

  let other_group_chat_id = GroupChatId::new();
  update_dao
    .insert_group_chat(
      other_group_chat_id.clone(),
      GroupChatName::new("chat-2").unwrap(),
      other_admin_id.clone(),
      created_at + Duration::seconds(1),
    )
    .await
    .unwrap();
  update_dao
    .insert_member(
      other_group_chat_id.clone(),
      MemberId::new(),
      other_admin_id.clone(),
      MemberRole::Admin,
      created_at + Duration::seconds(1),
    )
    .await
    .unwrap();

  let message = Message::new(MessageId::new(), "hello".to_string(), member_id.clone());
  let second_message = Message::new(MessageId::new(), "world".to_string(), member_id.clone());
  let deleted_message = Message::new(MessageId::new(), "deleted".to_string(), admin_id.clone());
  update_dao
    .insert_message(group_chat_id.clone(), message.clone(), message_created_at)
    .await
    .unwrap();
  update_dao
    .insert_message(group_chat_id.clone(), second_message.clone(), second_message_created_at)
    .await
    .unwrap();
  update_dao
    .insert_message(
      group_chat_id.clone(),
      deleted_message.clone(),
      deleted_message_created_at,
    )
    .await
    .unwrap();
  update_dao
    .delete_message(
      deleted_message.breach_encapsulation_of_id().clone(),
      deleted_message_created_at + Duration::seconds(1),
    )
    .await
    .unwrap();

  let router = create_router(pool);

  let group_chat = execute(
    &router,
    &format!(
      r#"{{ getGroupChat(groupChatId: "{}", userAccountId: "{}") {{ id name ownerId createdAt updatedAt }} }}"#,
      group_chat_id, member_id
    ),
  )
  .await;
  assert_eq!(
    group_chat["data"]["getGroupChat"],
    json!({
        "id": group_chat_id.to_string(),
        "name": "chat-1",
        "ownerId": admin_id.to_string(),
        "createdAt": "2026-01-02T03:04:05",
        "updatedAt": "2026-01-02T03:04:05"
    })
  );

  let group_chats = execute(
    &router,
    &format!(r#"{{ getGroupChats(userAccountId: "{}") {{ id name }} }}"#, member_id),
  )
  .await;
  assert_eq!(
    group_chats["data"]["getGroupChats"],
    json!([{ "id": group_chat_id.to_string(), "name": "chat-1" }])
  );

  let other_group_chats = execute(
    &router,
    &format!(
      r#"{{ getGroupChats(userAccountId: "{}") {{ id name }} }}"#,
      other_admin_id
    ),
  )
  .await;
  assert_eq!(
    other_group_chats["data"]["getGroupChats"],
    json!([{ "id": other_group_chat_id.to_string(), "name": "chat-2" }])
  );

  let member = execute(
    &router,
    &format!(
      r#"{{ getMember(groupChatId: "{}", userAccountId: "{}") {{ groupChatId userAccountId role }} }}"#,
      group_chat_id, member_id
    ),
  )
  .await;
  assert_eq!(
    member["data"]["getMember"],
    json!({
        "groupChatId": group_chat_id.to_string(),
        "userAccountId": member_id.to_string(),
        "role": "member"
    })
  );

  let members = execute(
    &router,
    &format!(
      r#"{{ getMembers(groupChatId: "{}", userAccountId: "{}") {{ userAccountId role }} }}"#,
      group_chat_id, admin_id
    ),
  )
  .await;
  assert_eq!(
    members["data"]["getMembers"],
    json!([
        { "userAccountId": admin_id.to_string(), "role": "admin" },
        { "userAccountId": member_id.to_string(), "role": "member" }
    ])
  );

  let message_id = message.breach_encapsulation_of_id().to_string();
  let message_response = execute(
    &router,
    &format!(
      r#"{{ getMessage(messageId: "{}", userAccountId: "{}") {{ id text }} }}"#,
      message_id, member_id
    ),
  )
  .await;
  assert_eq!(
    message_response["data"]["getMessage"],
    json!({ "id": message_id, "text": "hello" })
  );

  let messages = execute(
    &router,
    &format!(
      r#"{{ getMessages(groupChatId: "{}", userAccountId: "{}") {{ id text }} }}"#,
      group_chat_id, member_id
    ),
  )
  .await;
  assert_eq!(
    messages["data"]["getMessages"],
    json!([
        { "id": message.breach_encapsulation_of_id().to_string(), "text": "hello" },
        { "id": second_message.breach_encapsulation_of_id().to_string(), "text": "world" }
    ])
  );

  let visible_to_outsider = execute(
    &router,
    &format!(r#"{{ getGroupChats(userAccountId: "{}") {{ id }} }}"#, outsider_id),
  )
  .await;
  assert_eq!(visible_to_outsider["data"]["getGroupChats"], json!([]));

  let group_chat_error = execute(
    &router,
    &format!(
      r#"{{ getGroupChat(groupChatId: "{}", userAccountId: "{}") {{ id }} }}"#,
      group_chat_id, outsider_id
    ),
  )
  .await;
  assert_eq!(error_code(&group_chat_error), Some("404"));

  let member_error = execute(
    &router,
    &format!(
      r#"{{ getMember(groupChatId: "{}", userAccountId: "{}") {{ id }} }}"#,
      group_chat_id, outsider_id
    ),
  )
  .await;
  assert_eq!(error_code(&member_error), Some("404"));

  let member_list = execute(
    &router,
    &format!(
      r#"{{ getMembers(groupChatId: "{}", userAccountId: "{}") {{ id }} }}"#,
      group_chat_id, outsider_id
    ),
  )
  .await;
  assert_eq!(member_list["data"]["getMembers"], json!([]));

  let message_error = execute(
    &router,
    &format!(
      r#"{{ getMessage(messageId: "{}", userAccountId: "{}") {{ id }} }}"#,
      message_id, outsider_id
    ),
  )
  .await;
  assert_eq!(error_code(&message_error), Some("404"));

  let message_list = execute(
    &router,
    &format!(
      r#"{{ getMessages(groupChatId: "{}", userAccountId: "{}") {{ id }} }}"#,
      group_chat_id, outsider_id
    ),
  )
  .await;
  assert_eq!(message_list["data"]["getMessages"], json!([]));
}

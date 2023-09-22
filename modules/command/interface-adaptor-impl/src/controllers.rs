use std::str::FromStr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use utoipa::ToSchema;

use command_domain::group_chat::GroupChatEvent;
use command_domain::group_chat::{GroupChatId, GroupChatName, MemberRole, MessageId};
use command_domain::user_account::UserAccountId;
use command_domain::Event;
use command_interface_adaptor_if::{GroupChatPresenter, GroupChatRepository};
use command_processor::command_processor::GroupChatCommandProcessor;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CommandResponseFailureBody {
  /// Error message
  msg: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GroupChatCommandResponseSuccessBody {
  pub group_chat_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MessageCommandResponseSuccessBody {
  pub message_id: String,
}

/// Create group chat request body
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateGroupChatRequestBody {
  /// group chat name
  #[schema(example = "group-chat-example-1", required = true)]
  pub name: String,
  /// user account id to execute this command
  #[schema(example = "01H42K4ABWQ5V2XQEP3A48VE0Z", required = true)]
  pub executor_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteGroupChatRequestBody {
  /// target group chat id
  #[schema(example = "01H7C6927YNHXQZFTJKZK6Y9A7", required = true)]
  pub group_chat_id: String,
  /// user account id to execute this command
  #[schema(example = "01H42K4ABWQ5V2XQEP3A48VE0Z", required = true)]
  pub executor_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RenameGroupChatRequestBody {
  /// target group chat id
  #[schema(example = "01H7C6927YNHXQZFTJKZK6Y9A7", required = true)]
  pub group_chat_id: String,
  /// new group chat name
  #[schema(example = "group-chat-example-2", required = true)]
  pub name: String,
  /// user account id to execute this command
  #[schema(example = "01H42K4ABWQ5V2XQEP3A48VE0Z", required = true)]
  pub executor_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddMemberRequestBody {
  /// target group chat id
  #[schema(example = "01H7C6927YNHXQZFTJKZK6Y9A7", required = true)]
  pub group_chat_id: String,
  /// user account id to be added
  #[schema(example = "01H7C6DWMK1BKS1JYH1XZE529V", required = true)]
  pub user_account_id: String,
  /// user role of the user account
  #[schema(example = "admin or member", required = true)]
  pub role: String,
  /// user account id to execute this command
  #[schema(example = "01H42K4ABWQ5V2XQEP3A48VE0Z", required = true)]
  pub executor_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RemoveMemberRequestBody {
  /// target group chat id
  #[schema(example = "01H7C6927YNHXQZFTJKZK6Y9A7", required = true)]
  pub group_chat_id: String,
  /// user account id to be removed
  #[schema(example = "01H7C6DWMK1BKS1JYH1XZE529V", required = true)]
  pub user_account_id: String,
  /// user account id to execute this command
  #[schema(example = "01H42K4ABWQ5V2XQEP3A48VE0Z", required = true)]
  pub executor_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostMessageRequestBody {
  /// target group chat id
  #[schema(example = "01H7C6927YNHXQZFTJKZK6Y9A7", required = true)]
  pub group_chat_id: String,
  /// user account id to post this message
  #[schema(example = "01H7C6DWMK1BKS1JYH1XZE529V", required = true)]
  pub user_account_id: String,
  /// message
  pub message: String,
  /// user account id to execute this command
  #[schema(example = "01H42K4ABWQ5V2XQEP3A48VE0Z", required = true)]
  pub executor_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteMessageRequestBody {
  /// target group chat id
  #[schema(example = "01H7C6927YNHXQZFTJKZK6Y9A7", required = true)]
  pub group_chat_id: String,
  /// user account id to delete this message
  #[schema(example = "01H7C6DWMK1BKS1JYH1XZE529V", required = true)]
  pub user_account_id: String,
  /// message id to be deleted
  #[schema(example = "test message-1", required = true)]
  pub message_id: String,
  /// user account id to execute this command
  #[schema(example = "01H42K4ABWQ5V2XQEP3A48VE0Z", required = true)]
  pub executor_id: String,
}

pub struct AppState<TR: GroupChatRepository> {
  group_chat_repository: TR,
}

impl<TR: GroupChatRepository> AppState<TR> {
  fn new(group_chat_repository: TR) -> Self {
    Self { group_chat_repository }
  }
}

pub type AppDate<TR> = Arc<RwLock<AppState<TR>>>;

pub struct GroupChatIdPresenter {
  group_chat_id: Option<GroupChatId>,
}

impl GroupChatIdPresenter {
  pub fn new() -> Self {
    Self { group_chat_id: None }
  }

  pub fn group_chat_id(&self) -> &GroupChatId {
    self.group_chat_id.as_ref().unwrap()
  }
}

impl GroupChatPresenter for GroupChatIdPresenter {
  fn present(&mut self, group_chat_event: GroupChatEvent) {
    self.group_chat_id = Some(group_chat_event.aggregate_id().clone());
  }
}

pub struct MessageIdPresenter {
  message_id: Option<MessageId>,
}

impl MessageIdPresenter {
  pub fn new() -> Self {
    Self { message_id: None }
  }

  pub fn message_id(&self) -> &MessageId {
    self.message_id.as_ref().unwrap()
  }
}

impl GroupChatPresenter for MessageIdPresenter {
  fn present(&mut self, group_chat_event: GroupChatEvent) {
    match group_chat_event {
      GroupChatEvent::GroupChatMessagePosted(message_posted) => {
        self.message_id = Some(message_posted.message.breach_encapsulation_of_id().clone());
      }
      _ => panic!("Unexpected event type: {:?}", group_chat_event),
    }
  }
}

/// create group chat.
#[utoipa::path(
  post,
  path = "/group-chats/create",
  responses(
    (status = 200, description = "Group chat successfully created.", body = GroupChatCommandResponseSuccessBody),
    (status = 500, description = "Group chat creation failed.", body = CommandResponseFailureBody),
  )
)]
pub async fn create_group_chat<TR: GroupChatRepository>(
  State(state): State<AppDate<TR>>,
  Json(payload): Json<CreateGroupChatRequestBody>,
) -> impl IntoResponse {
  let mut lock = state.write().await;
  let mut command_processor = GroupChatCommandProcessor::new(&mut lock.group_chat_repository);
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();

  let group_chat = match GroupChatName::from_str(&payload.name) {
    Ok(group_chat) => group_chat,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  let executor_id = match UserAccountId::from_str(&payload.executor_id) {
    Ok(executor_id) => executor_id,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  match command_processor
    .create_group_chat(&mut group_chat_id_presenter, group_chat, executor_id)
    .await
  {
    Ok(_) => (
      StatusCode::OK,
      Json(GroupChatCommandResponseSuccessBody {
        group_chat_id: group_chat_id_presenter.group_chat_id().to_string(),
      }),
    )
      .into_response(),
    Err(error) => {
      log::error!("error = {}", error);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response()
    }
  }
}

/// delete group chat.
#[utoipa::path(
  post,
  path = "/group-chats/delete",
  responses(
    (status = 200, description = "Group chat successfully deleted.", body = GroupChatCommandResponseSuccessBody),
    (status = 500, description = "Group chat deletion failed.", body = CommandResponseFailureBody),
  )
)]
async fn delete_group_chat<TR: GroupChatRepository>(
  State(state): State<AppDate<TR>>,
  Json(payload): Json<DeleteGroupChatRequestBody>,
) -> impl IntoResponse {
  let mut lock = state.write().await;
  let mut command_processor = GroupChatCommandProcessor::new(&mut lock.group_chat_repository);
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();

  let group_chat_id = match GroupChatId::from_str(&payload.group_chat_id) {
    Ok(group_chat_id) => group_chat_id,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  let executor_id = match UserAccountId::from_str(&payload.executor_id) {
    Ok(executor_id) => executor_id,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  match command_processor
    .delete_group_chat(&mut group_chat_id_presenter, group_chat_id, executor_id)
    .await
  {
    Ok(_) => (
      StatusCode::OK,
      Json(GroupChatCommandResponseSuccessBody {
        group_chat_id: group_chat_id_presenter.group_chat_id().to_string(),
      }),
    )
      .into_response(),
    Err(error) => {
      log::error!("error = {}", error);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(CommandResponseFailureBody { msg: error.to_string() }).into_response(),
      )
        .into_response()
    }
  }
}

/// rename group chat.
#[utoipa::path(
  post,
  path = "/group-chats/rename",
  responses(
    (status = 200, description = "Group chat successfully renamed.", body = GroupChatCommandResponseSuccessBody),
    (status = 500, description = "Group chat rename failed.", body = CommandResponseFailureBody),
  )
)]
async fn rename_group_chat<TR: GroupChatRepository>(
  State(state): State<AppDate<TR>>,
  Json(payload): Json<RenameGroupChatRequestBody>,
) -> impl IntoResponse {
  let mut lock = state.write().await;
  let mut command_processor = GroupChatCommandProcessor::new(&mut lock.group_chat_repository);
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();

  let group_chat_id = match GroupChatId::from_str(&payload.group_chat_id) {
    Ok(group_chat_id) => group_chat_id,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  let group_chat = match GroupChatName::from_str(&payload.name) {
    Ok(group_chat) => group_chat,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  let executor_id = match UserAccountId::from_str(&payload.executor_id) {
    Ok(executor_id) => executor_id,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  match command_processor
    .rename_group_chat(&mut group_chat_id_presenter, group_chat_id, group_chat, executor_id)
    .await
  {
    Ok(_) => (
      StatusCode::OK,
      Json(GroupChatCommandResponseSuccessBody {
        group_chat_id: group_chat_id_presenter.group_chat_id().to_string(),
      }),
    )
      .into_response(),
    Err(error) => {
      log::error!("error = {}", error);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response()
    }
  }
}

/// add a member to the group chat.
#[utoipa::path(
  post,
  path = "/group-chats/add-member",
  responses(
    (status = 200, description = "Group chat member successfully added.", body = GroupChatCommandResponseSuccessBody),
    (status = 500, description = "Group chat member addition failed.", body = CommandResponseFailureBody),
  )
)]
async fn add_member<TR: GroupChatRepository>(
  State(state): State<AppDate<TR>>,
  Json(payload): Json<AddMemberRequestBody>,
) -> impl IntoResponse {
  let mut lock = state.write().await;
  let mut command_processor = GroupChatCommandProcessor::new(&mut lock.group_chat_repository);
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();

  let group_chat_id = match GroupChatId::from_str(&payload.group_chat_id) {
    Ok(group_chat_id) => group_chat_id,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  let user_account_id = match UserAccountId::from_str(&payload.user_account_id) {
    Ok(user_account_id) => user_account_id,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  let role = match MemberRole::from_str(&payload.role) {
    Ok(role) => role,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  let executor_id = match UserAccountId::from_str(&payload.executor_id) {
    Ok(executor_id) => executor_id,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  match command_processor
    .add_member(
      &mut group_chat_id_presenter,
      group_chat_id,
      user_account_id,
      role,
      executor_id,
    )
    .await
  {
    Ok(_) => (
      StatusCode::OK,
      Json(GroupChatCommandResponseSuccessBody {
        group_chat_id: group_chat_id_presenter.group_chat_id().to_string(),
      }),
    )
      .into_response(),
    Err(error) => {
      log::error!("error = {}", error);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response()
    }
  }
}

/// remove the member from the group chat.
#[utoipa::path(
  post,
  path = "/group-chats/remove-member",
  responses(
    (status = 200, description = "Group chat member successfully removed.", body = GroupChatCommandResponseSuccessBody),
    (status = 500, description = "Group chat member remove failed.", body = CommandResponseFailureBody),
  )
)]
async fn remove_member<TR: GroupChatRepository>(
  State(state): State<AppDate<TR>>,
  Json(payload): Json<RemoveMemberRequestBody>,
) -> impl IntoResponse {
  let mut lock = state.write().await;
  let mut command_processor = GroupChatCommandProcessor::new(&mut lock.group_chat_repository);
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();

  let group_chat_id = match GroupChatId::from_str(&payload.group_chat_id) {
    Ok(group_chat_id) => group_chat_id,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  let user_account_id = match UserAccountId::from_str(&payload.user_account_id) {
    Ok(user_account_id) => user_account_id,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  let executor_id = match UserAccountId::from_str(&payload.executor_id) {
    Ok(executor_id) => executor_id,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  match command_processor
    .remove_member(
      &mut group_chat_id_presenter,
      group_chat_id,
      user_account_id,
      executor_id,
    )
    .await
  {
    Ok(_) => (
      StatusCode::OK,
      Json(GroupChatCommandResponseSuccessBody {
        group_chat_id: group_chat_id_presenter.group_chat_id().to_string(),
      }),
    )
      .into_response(),
    Err(error) => {
      log::error!("error = {}", error);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response()
    }
  }
}

/// post a message to the group chat.
#[utoipa::path(
  post,
  path = "/group-chats/post-message",
  responses(
    (status = 200, description = "Group chat message successfully posted.", body = MessageCommandResponseSuccessBody),
    (status = 500, description = "Group chat message post failed.", body = CommandResponseFailureBody),
  )
)]
async fn post_message<TR: GroupChatRepository>(
  State(_state): State<AppDate<TR>>,
  Json(_payload): Json<PostMessageRequestBody>,
) -> impl IntoResponse {
  todo!() // 必須課題 難易度:高
}

/// delete the message from the group chat.
#[utoipa::path(
  post,
  path = "/group-chats/delete-message",
  responses(
    (status = 200, description = "Group chat message successfully deleted.", body = GroupChatCommandResponseSuccessBody),
    (status = 500, description = "Group chat message deletion failed.", body = CommandResponseFailureBody),
  )
)]
async fn delete_message<TR: GroupChatRepository>(
  State(state): State<AppDate<TR>>,
  Json(payload): Json<DeleteMessageRequestBody>,
) -> impl IntoResponse {
  let mut lock = state.write().await;
  let mut command_processor = GroupChatCommandProcessor::new(&mut lock.group_chat_repository);
  let mut group_chat_id_presenter = GroupChatIdPresenter::new();

  let group_chat_id = match GroupChatId::from_str(&payload.group_chat_id) {
    Ok(group_chat_id) => group_chat_id,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  let message_id = match MessageId::from_str(&payload.message_id) {
    Ok(message_id) => message_id,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  let executor_id = match UserAccountId::from_str(&payload.executor_id) {
    Ok(executor_id) => executor_id,
    Err(error) => {
      log::warn!("error = {}", error);
      return (
        StatusCode::BAD_REQUEST,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response();
    }
  };

  match command_processor
    .delete_message(
      &mut group_chat_id_presenter,
      group_chat_id,
      message_id.clone(),
      executor_id,
    )
    .await
  {
    Ok(_) => (
      StatusCode::OK,
      Json(GroupChatCommandResponseSuccessBody {
        group_chat_id: group_chat_id_presenter.group_chat_id().to_string(),
      }),
    )
      .into_response(),
    Err(error) => {
      log::error!("error = {}: message_id = {}", error, message_id);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(CommandResponseFailureBody { msg: error.to_string() }),
      )
        .into_response()
    }
  }
}

pub enum EndpointPaths {
  Root,
  HealthAlive,
  HealthReady,
  CreateGroupChat,
  DeleteGroupChat,
  RenameGroupChat,
  AddMember,
  RemoveMember,
  PostMessage,
  DeleteMessage,
}

impl EndpointPaths {
  pub fn as_str(&self) -> &'static str {
    match *self {
      EndpointPaths::Root => "/",
      EndpointPaths::HealthAlive => "/health/alive",
      EndpointPaths::HealthReady => "/health/ready",
      EndpointPaths::CreateGroupChat => "/group-chats/create",
      EndpointPaths::DeleteGroupChat => "/group-chats/delete",
      EndpointPaths::RenameGroupChat => "/group-chats/rename",
      EndpointPaths::AddMember => "/group-chats/add-member",
      EndpointPaths::RemoveMember => "/group-chats/remove-member",
      EndpointPaths::PostMessage => "/group-chats/post-message",
      EndpointPaths::DeleteMessage => "/group-chats/delete-message",
    }
  }
}

async fn hello_write_api() -> &'static str {
  "Hello, Write API!"
}

async fn alive<TR: GroupChatRepository>(State(_state): State<AppDate<TR>>) -> impl IntoResponse {
  (StatusCode::OK, "OK")
}

async fn ready<TR: GroupChatRepository>(State(_state): State<AppDate<TR>>) -> impl IntoResponse {
  (StatusCode::OK, "OK")
}

pub fn create_router<TR: GroupChatRepository>(repository: TR) -> Router {
  let state = Arc::new(RwLock::new(AppState::new(repository)));
  let router = Router::new()
    .route(EndpointPaths::Root.as_str(), get(hello_write_api))
    .route(EndpointPaths::HealthAlive.as_str(), get(alive))
    .route(EndpointPaths::HealthReady.as_str(), get(ready))
    .route(EndpointPaths::CreateGroupChat.as_str(), post(create_group_chat))
    .route(EndpointPaths::DeleteGroupChat.as_str(), post(delete_group_chat))
    .route(EndpointPaths::RenameGroupChat.as_str(), post(rename_group_chat))
    .route(EndpointPaths::AddMember.as_str(), post(add_member))
    .route(EndpointPaths::RemoveMember.as_str(), post(remove_member))
    .route(EndpointPaths::PostMessage.as_str(), post(post_message))
    .route(EndpointPaths::DeleteMessage.as_str(), post(delete_message))
    .with_state(state);
  router
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use axum::body::Body;
  use axum::http::Request;
  use axum::routing::get;
  use axum::Router;
  use tokio::sync::RwLock;
  use tower::ServiceExt;

  use crate::gateways::group_chat_repository::MockGroupChatRepository;

  use super::*;

  #[tokio::test]
  async fn test_root() {
    let router = Router::new().route(EndpointPaths::Root.as_str(), get(hello_write_api));

    let response = router
      .oneshot(
        Request::builder()
          .uri(EndpointPaths::Root.as_str())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"Hello, Write API!");
  }

  #[tokio::test]
  async fn test_alive() {
    let repository = MockGroupChatRepository::new();
    let app_state = AppState::new(repository);

    let router = Router::new()
      .route(EndpointPaths::HealthAlive.as_str(), get(alive))
      .with_state(Arc::new(RwLock::new(app_state)));

    let response = router
      .oneshot(
        Request::builder()
          .uri(EndpointPaths::HealthAlive.as_str())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"OK");
  }

  #[tokio::test]
  async fn test_ready() {
    let repository = MockGroupChatRepository::new();
    let app_state = AppState::new(repository);

    let router = Router::new()
      .route(EndpointPaths::HealthReady.as_str(), get(ready))
      .with_state(Arc::new(RwLock::new(app_state)));

    let response = router
      .oneshot(
        Request::builder()
          .uri(EndpointPaths::HealthReady.as_str())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"OK");
  }
}

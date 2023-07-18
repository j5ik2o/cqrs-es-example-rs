use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use cqrs_es_example_command_processor::command_processor::ThreadCommandProcessor;
use cqrs_es_example_domain::thread::{MemberRole, Message, MessageId, ThreadId, ThreadName, ThreadRepository};
use cqrs_es_example_domain::user_account::UserAccountId;

#[derive(Debug, Serialize, Deserialize)]
pub enum ThreadCommandResponseBody {
  Success { id: ThreadId },
  Failure { msg: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateThreadRequestBody {
  pub name: ThreadName,
  pub executor_id: UserAccountId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteThreadRequestBody {
  pub thread_id: ThreadId,
  pub executor_id: UserAccountId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenameThreadRequestBody {
  pub thread_id: ThreadId,
  pub name: ThreadName,
  pub executor_id: UserAccountId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddMemberRequestBody {
  pub thread_id: ThreadId,
  pub user_account_id: UserAccountId,
  pub role: MemberRole,
  pub executor_id: UserAccountId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveMemberRequestBody {
  pub thread_id: ThreadId,
  pub user_account_id: UserAccountId,
  pub executor_id: UserAccountId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostMessageRequestBody {
  pub thread_id: ThreadId,
  pub user_account_id: UserAccountId,
  pub message: Message,
  pub executor_id: UserAccountId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteMessageRequestBody {
  pub thread_id: ThreadId,
  pub user_account_id: UserAccountId,
  pub message_id: MessageId,
  pub executor_id: UserAccountId,
}

struct AppState<TR: ThreadRepository> {
  thread_repository: TR,
}

impl<TR: ThreadRepository> AppState<TR> {
  fn new(thread_repository: TR) -> Self {
    Self { thread_repository }
  }
}

type AppDate<TR> = Arc<RwLock<AppState<TR>>>;

async fn create_thread<TR: ThreadRepository>(
  State(state): State<AppDate<TR>>,
  Json(payload): Json<CreateThreadRequestBody>,
) -> impl IntoResponse {
  let mut lock = state.write().await;
  let mut command_processor = ThreadCommandProcessor::new(&mut lock.thread_repository);
  match command_processor.create_thread(payload.name, payload.executor_id).await {
    Ok(id) => (StatusCode::CREATED, Json(ThreadCommandResponseBody::Success { id })),
    Err(error) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ThreadCommandResponseBody::Failure { msg: error.to_string() }),
    ),
  }
}

async fn delete_thread<TR: ThreadRepository>(
  State(state): State<AppDate<TR>>,
  Json(payload): Json<DeleteThreadRequestBody>,
) -> impl IntoResponse {
  let mut lock = state.write().await;
  let mut command_processor = ThreadCommandProcessor::new(&mut lock.thread_repository);
  match command_processor
    .delete_thread(payload.thread_id, payload.executor_id)
    .await
  {
    Ok(id) => (StatusCode::OK, Json(ThreadCommandResponseBody::Success { id })),
    Err(error) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ThreadCommandResponseBody::Failure { msg: error.to_string() }),
    ),
  }
}

async fn rename_thread<TR: ThreadRepository>(
  State(state): State<AppDate<TR>>,
  Json(payload): Json<RenameThreadRequestBody>,
) -> impl IntoResponse {
  let mut lock = state.write().await;
  let mut command_processor = ThreadCommandProcessor::new(&mut lock.thread_repository);
  match command_processor
    .rename_thread(payload.thread_id, payload.name, payload.executor_id)
    .await
  {
    Ok(id) => (StatusCode::OK, Json(ThreadCommandResponseBody::Success { id })),
    Err(error) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ThreadCommandResponseBody::Failure { msg: error.to_string() }),
    ),
  }
}

async fn add_member<TR: ThreadRepository>(
  State(state): State<AppDate<TR>>,
  Json(payload): Json<AddMemberRequestBody>,
) -> impl IntoResponse {
  let mut lock = state.write().await;
  let mut command_processor = ThreadCommandProcessor::new(&mut lock.thread_repository);
  match command_processor
    .add_member(
      payload.thread_id,
      payload.user_account_id,
      payload.role,
      payload.executor_id,
    )
    .await
  {
    Ok(id) => (StatusCode::OK, Json(ThreadCommandResponseBody::Success { id })),
    Err(error) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ThreadCommandResponseBody::Failure { msg: error.to_string() }),
    ),
  }
}

async fn remove_member<TR: ThreadRepository>(
  State(state): State<AppDate<TR>>,
  Json(payload): Json<RemoveMemberRequestBody>,
) -> impl IntoResponse {
  let mut lock = state.write().await;
  let mut command_processor = ThreadCommandProcessor::new(&mut lock.thread_repository);
  match command_processor
    .remove_member(payload.thread_id, payload.user_account_id, payload.executor_id)
    .await
  {
    Ok(id) => (StatusCode::OK, Json(ThreadCommandResponseBody::Success { id })),
    Err(error) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ThreadCommandResponseBody::Failure { msg: error.to_string() }),
    ),
  }
}

async fn post_message<TR: ThreadRepository>(
  State(state): State<AppDate<TR>>,
  Json(payload): Json<PostMessageRequestBody>,
) -> impl IntoResponse {
  let mut lock = state.write().await;
  let mut command_processor = ThreadCommandProcessor::new(&mut lock.thread_repository);
  match command_processor
    .post_message(payload.thread_id, payload.message, payload.executor_id)
    .await
  {
    Ok(id) => (StatusCode::OK, Json(ThreadCommandResponseBody::Success { id })),
    Err(error) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ThreadCommandResponseBody::Failure { msg: error.to_string() }),
    ),
  }
}

async fn delete_message<TR: ThreadRepository>(
  State(state): State<AppDate<TR>>,
  Json(payload): Json<DeleteMessageRequestBody>,
) -> impl IntoResponse {
  let mut lock = state.write().await;
  let mut command_processor = ThreadCommandProcessor::new(&mut lock.thread_repository);
  match command_processor
    .delete_message(payload.thread_id, payload.message_id, payload.executor_id)
    .await
  {
    Ok(id) => (StatusCode::OK, Json(ThreadCommandResponseBody::Success { id })),
    Err(error) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ThreadCommandResponseBody::Failure { msg: error.to_string() }),
    ),
  }
}

pub enum EndpointPaths {
  CreateThread,
  DeleteThread,
  RenameThread,
  AddMember,
  RemoveMember,
  PostMessage,
  DeleteMessage,
}

impl EndpointPaths {
  pub fn as_str(&self) -> &'static str {
    match *self {
      EndpointPaths::CreateThread => "/threads/create",
      EndpointPaths::DeleteThread => "/threads/delete",
      EndpointPaths::RenameThread => "/threads/rename",
      EndpointPaths::AddMember => "/threads/add_member",
      EndpointPaths::RemoveMember => "/threads/remove_member",
      EndpointPaths::PostMessage => "/threads/post_message",
      EndpointPaths::DeleteMessage => "/threads/delete_message",
    }
  }
}

async fn hello_write_api() -> &'static str {
  "Hello, Write API!"
}

async fn alive<TR: ThreadRepository>(State(_state): State<AppDate<TR>>) -> impl IntoResponse {
  (StatusCode::OK, "OK")
}

async fn ready<TR: ThreadRepository>(State(_state): State<AppDate<TR>>) -> impl IntoResponse {
  (StatusCode::OK, "OK")
}

pub fn create_router<TR: ThreadRepository>(repository: TR) -> Router {
  let state = Arc::new(RwLock::new(AppState::new(repository)));
  let router = Router::new()
    .route("/", get(hello_write_api))
    .route("/health/alive", get(alive))
    .route("/health/ready", get(ready))
    .route(EndpointPaths::CreateThread.as_str(), post(create_thread))
    .route(EndpointPaths::DeleteThread.as_str(), post(delete_thread))
    .route(EndpointPaths::RenameThread.as_str(), post(rename_thread))
    .route(EndpointPaths::AddMember.as_str(), post(add_member))
    .route(EndpointPaths::RemoveMember.as_str(), post(remove_member))
    .route(EndpointPaths::PostMessage.as_str(), post(post_message))
    .route(EndpointPaths::DeleteMessage.as_str(), post(delete_message))
    .with_state(state);
  router
}

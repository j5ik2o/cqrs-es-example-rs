use axum::http::{header, Method, Request, StatusCode};
use hyper::Body;
use tower::util::ServiceExt;

use common::*;
use cqrs_es_example_command_interface_adaptor::controllers::{
  create_router, AddMemberRequestBody, CreateThreadRequestBody, DeleteMessageRequestBody, DeleteThreadRequestBody,
  EndpointPaths, PostMessageRequestBody, RemoveMemberRequestBody, RenameThreadRequestBody, ThreadCommandResponseBody,
};
use cqrs_es_example_domain::thread::{MemberRole, Message, MessageId, ThreadId, ThreadName, ThreadRepository};
use cqrs_es_example_domain::user_account::UserAccountId;

mod common;

async fn create_thread<TR: ThreadRepository>(
  repository: &TR,
  name: ThreadName,
  executor_id: UserAccountId,
) -> ThreadId {
  let create_thread_body = CreateThreadRequestBody { name, executor_id };
  let create_thread_request = Request::builder()
    .uri(EndpointPaths::CreateThread.as_str())
    .method(Method::POST)
    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
    .body(Body::from(serde_json::to_string(&create_thread_body).unwrap()))
    .unwrap();
  let create_thread_response = create_router(repository.clone())
    .oneshot(create_thread_request)
    .await
    .unwrap();
  assert_eq!(create_thread_response.status(), StatusCode::CREATED);
  let create_thread_response_body_bytes = hyper::body::to_bytes(create_thread_response.into_body()).await.unwrap();
  let create_thread_response_body_str = String::from_utf8(create_thread_response_body_bytes.to_vec()).unwrap();
  let thread_command_response_body =
    serde_json::from_str::<ThreadCommandResponseBody>(&create_thread_response_body_str).unwrap();
  let ThreadCommandResponseBody::Success { id } = thread_command_response_body else { panic!("missing thread id"); };
  id
}

async fn delete_thread<TR: ThreadRepository>(
  repository: &TR,
  thread_id: ThreadId,
  executor_id: UserAccountId,
) -> ThreadId {
  let delete_thread_body = DeleteThreadRequestBody { thread_id, executor_id };
  let delete_thread_request = Request::builder()
    .uri(EndpointPaths::DeleteThread.as_str())
    .method(Method::POST)
    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
    .body(Body::from(serde_json::to_string(&delete_thread_body).unwrap()))
    .unwrap();
  let delete_thread_response = create_router(repository.clone())
    .oneshot(delete_thread_request)
    .await
    .unwrap();
  assert_eq!(delete_thread_response.status(), StatusCode::OK);
  let delete_thread_response_body_bytes = hyper::body::to_bytes(delete_thread_response.into_body()).await.unwrap();
  let delete_thread_response_body_str = String::from_utf8(delete_thread_response_body_bytes.to_vec()).unwrap();
  let delete_command_response_body =
    serde_json::from_str::<ThreadCommandResponseBody>(&delete_thread_response_body_str).unwrap();
  let ThreadCommandResponseBody::Success { id } = delete_command_response_body else { panic!("missing thread id"); };
  id
}

async fn rename_thread<TR: ThreadRepository>(
  repository: &TR,
  thread_id: ThreadId,
  name: ThreadName,
  executor_id: UserAccountId,
) -> ThreadId {
  let rename_thread_body = RenameThreadRequestBody {
    thread_id,
    name,
    executor_id,
  };
  let rename_thread_request = Request::builder()
    .uri(EndpointPaths::RenameThread.as_str())
    .method(Method::POST)
    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
    .body(Body::from(serde_json::to_string(&rename_thread_body).unwrap()))
    .unwrap();
  let rename_thread_response = create_router(repository.clone())
    .oneshot(rename_thread_request)
    .await
    .unwrap();
  assert_eq!(rename_thread_response.status(), StatusCode::OK);
  let rename_thread_response_body_bytes = hyper::body::to_bytes(rename_thread_response.into_body()).await.unwrap();
  let rename_thread_response_body_str = String::from_utf8(rename_thread_response_body_bytes.to_vec()).unwrap();
  let rename_thread_response_body =
    serde_json::from_str::<ThreadCommandResponseBody>(&rename_thread_response_body_str).unwrap();
  let ThreadCommandResponseBody::Success { id } = rename_thread_response_body else { panic!("missing thread id"); };
  id
}

async fn add_member<TR: ThreadRepository>(
  repository: &TR,
  thread_id: ThreadId,
  user_account_id: UserAccountId,
  role: MemberRole,
  executor_id: UserAccountId,
) -> ThreadId {
  let add_member_body = AddMemberRequestBody {
    thread_id,
    user_account_id,
    role,
    executor_id,
  };
  let add_member_request = Request::builder()
    .uri(EndpointPaths::AddMember.as_str())
    .method(Method::POST)
    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
    .body(Body::from(serde_json::to_string(&add_member_body).unwrap()))
    .unwrap();
  let add_member_response = create_router(repository.clone())
    .oneshot(add_member_request)
    .await
    .unwrap();
  assert_eq!(add_member_response.status(), StatusCode::OK);
  let add_member_response_body_bytes = hyper::body::to_bytes(add_member_response.into_body()).await.unwrap();
  let add_member_response_body_str = String::from_utf8(add_member_response_body_bytes.to_vec()).unwrap();
  let add_member_response_body =
    serde_json::from_str::<ThreadCommandResponseBody>(&add_member_response_body_str).unwrap();
  let ThreadCommandResponseBody::Success { id } = add_member_response_body else { panic!("missing thread id"); };
  id
}

async fn remove_member<TR: ThreadRepository>(
  repository: &TR,
  thread_id: ThreadId,
  user_account_id: UserAccountId,
  executor_id: UserAccountId,
) -> ThreadId {
  let remove_member_body = RemoveMemberRequestBody {
    thread_id,
    user_account_id,
    executor_id,
  };
  let remove_member_request = Request::builder()
    .uri(EndpointPaths::RemoveMember.as_str())
    .method(Method::POST)
    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
    .body(Body::from(serde_json::to_string(&remove_member_body).unwrap()))
    .unwrap();
  let remove_member_response = create_router(repository.clone())
    .oneshot(remove_member_request)
    .await
    .unwrap();
  assert_eq!(remove_member_response.status(), StatusCode::OK);
  let remove_member_response_body_bytes = hyper::body::to_bytes(remove_member_response.into_body()).await.unwrap();
  let remove_member_response_body_str = String::from_utf8(remove_member_response_body_bytes.to_vec()).unwrap();
  let remove_member_response_body =
    serde_json::from_str::<ThreadCommandResponseBody>(&remove_member_response_body_str).unwrap();
  let ThreadCommandResponseBody::Success { id } = remove_member_response_body else { panic!("missing thread id"); };
  id
}

async fn post_message<TR: ThreadRepository>(
  repository: &TR,
  thread_id: ThreadId,
  message: Message,
  user_account_id: UserAccountId,
  executor_id: UserAccountId,
) -> ThreadId {
  let post_message_body = PostMessageRequestBody {
    thread_id,
    user_account_id,
    message,
    executor_id,
  };
  let post_message_request = Request::builder()
    .uri(EndpointPaths::PostMessage.as_str())
    .method(Method::POST)
    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
    .body(Body::from(serde_json::to_string(&post_message_body).unwrap()))
    .unwrap();
  let post_message_response = create_router(repository.clone())
    .oneshot(post_message_request)
    .await
    .unwrap();
  assert_eq!(post_message_response.status(), StatusCode::OK);
  let post_message_response_body_bytes = hyper::body::to_bytes(post_message_response.into_body()).await.unwrap();
  let post_message_response_body_str = String::from_utf8(post_message_response_body_bytes.to_vec()).unwrap();
  let post_message_response_body =
    serde_json::from_str::<ThreadCommandResponseBody>(&post_message_response_body_str).unwrap();
  let ThreadCommandResponseBody::Success { id } = post_message_response_body else { panic!("missing thread id"); };
  id
}

async fn delete_message<TR: ThreadRepository>(
  repository: &TR,
  thread_id: ThreadId,
  message_id: MessageId,
  user_account_id: UserAccountId,
  executor_id: UserAccountId,
) -> ThreadId {
  let delete_message_body = DeleteMessageRequestBody {
    thread_id,
    user_account_id,
    message_id,
    executor_id,
  };
  let delete_message_request = Request::builder()
    .uri(EndpointPaths::DeleteMessage.as_str())
    .method(Method::POST)
    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
    .body(Body::from(serde_json::to_string(&delete_message_body).unwrap()))
    .unwrap();
  let delete_message_response = create_router(repository.clone())
    .oneshot(delete_message_request)
    .await
    .unwrap();
  assert_eq!(delete_message_response.status(), StatusCode::OK);
  let delete_message_response_body_bytes = hyper::body::to_bytes(delete_message_response.into_body())
    .await
    .unwrap();
  let delete_message_response_body_str = String::from_utf8(delete_message_response_body_bytes.to_vec()).unwrap();
  let delete_message_response_body =
    serde_json::from_str::<ThreadCommandResponseBody>(&delete_message_response_body_str).unwrap();
  let ThreadCommandResponseBody::Success { id } = delete_message_response_body else { panic!("missing thread id"); };
  id
}

#[tokio::test]
async fn test_create_thread() {
  with_repository(|repository| async move {
    let user_account_id = UserAccountId::new();
    let _ = create_thread(&repository, ThreadName::new("ABC".to_string()), user_account_id.clone()).await;
  })
  .await;
}

#[tokio::test]
async fn test_delete_thread() {
  with_repository(|repository| async move {
    // Given
    let user_account_id = UserAccountId::new();
    let id1 = create_thread(&repository, ThreadName::new("ABC".to_string()), user_account_id.clone()).await;

    // When
    let id2 = delete_thread(&repository, id1.clone(), user_account_id.clone()).await;

    // Then
    assert_eq!(id1, id2);
  })
  .await;
}

#[tokio::test]
async fn test_rename_thread() {
  with_repository(|repository| async move {
    // Given
    let user_account_id = UserAccountId::new();
    let id1 = create_thread(&repository, ThreadName::new("ABC".to_string()), user_account_id.clone()).await;

    // When
    let id2 = rename_thread(
      &repository,
      id1.clone(),
      ThreadName::new("DEF".to_string()),
      user_account_id,
    )
    .await;

    // Then
    assert_eq!(id1, id2);
  })
  .await;
}

#[tokio::test]
async fn test_add_member() {
  with_repository(|repository| async move {
    // Given
    let user_account_id = UserAccountId::new();
    let id1 = create_thread(&repository, ThreadName::new("ABC".to_string()), user_account_id.clone()).await;
    let user_account_id2 = UserAccountId::new();

    // When
    let id2 = add_member(
      &repository,
      id1.clone(),
      user_account_id2,
      MemberRole::Member,
      user_account_id,
    )
    .await;

    // Then
    assert_eq!(id1, id2);
  })
  .await;
}

#[tokio::test]
async fn test_remove_member() {
  with_repository(|repository| async move {
    // Given
    let user_account_id = UserAccountId::new();
    let id1 = create_thread(&repository, ThreadName::new("ABC".to_string()), user_account_id.clone()).await;
    let user_account_id2 = UserAccountId::new();

    let id2 = add_member(
      &repository,
      id1.clone(),
      user_account_id2.clone(),
      MemberRole::Member,
      user_account_id.clone(),
    )
    .await;

    // When
    let id3 = remove_member(&repository, id1.clone(), user_account_id2, user_account_id).await;

    // Then
    assert_eq!(id1, id2);
    assert_eq!(id2, id3);
  })
  .await;
}

#[tokio::test]
async fn test_post_message() {
  with_repository(|repository| async move {
    let user_account_id = UserAccountId::new();
    let id1 = create_thread(&repository, ThreadName::new("ABC".to_string()), user_account_id.clone()).await;

    let id2 = post_message(
      &repository,
      id1.clone(),
      Message::new("Hello".to_string(), user_account_id.clone()),
      user_account_id.clone(),
      user_account_id.clone(),
    )
    .await;

    assert_eq!(id1, id2);
  })
  .await;
}

#[tokio::test]
async fn test_delete_message() {
  with_repository(|repository| async move {
    let user_account_id = UserAccountId::new();
    let id1 = create_thread(&repository, ThreadName::new("ABC".to_string()), user_account_id.clone()).await;

    let message = Message::new("Hello".to_string(), user_account_id.clone());
    let id2 = post_message(
      &repository,
      id1.clone(),
      message.clone(),
      user_account_id.clone(),
      user_account_id.clone(),
    )
    .await;

    let id3 = delete_message(
      &repository,
      id1.clone(),
      message.id.clone(),
      user_account_id.clone(),
      user_account_id.clone(),
    )
    .await;

    assert_eq!(id1, id2);
    assert_eq!(id2, id3);
  })
  .await;
}

#[test]
fn test() {
  let body = CreateThreadRequestBody {
    name: ThreadName::new("test".to_string()),
    executor_id: UserAccountId::new(),
  };

  let json = serde_json::to_string(&body);
  println!("{}", json.unwrap());
}

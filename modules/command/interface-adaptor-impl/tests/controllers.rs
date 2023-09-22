use axum::http::{header, Method, Request, StatusCode};
use hyper::Body;
use std::str::FromStr;
use tower::util::ServiceExt;

use command_domain::group_chat::{GroupChatId, GroupChatName, MemberRole, MessageId};
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::GroupChatRepository;
use command_interface_adaptor_impl::controllers::{
  create_router, AddMemberRequestBody, CreateGroupChatRequestBody, DeleteGroupChatRequestBody,
  DeleteMessageRequestBody, EndpointPaths, GroupChatCommandResponseSuccessBody, MessageCommandResponseSuccessBody,
  PostMessageRequestBody, RemoveMemberRequestBody, RenameGroupChatRequestBody,
};
use common::*;

mod common;

async fn create_group_chat<TR: GroupChatRepository>(
  repository: &TR,
  name: GroupChatName,
  executor_id: UserAccountId,
) -> GroupChatId {
  let create_group_chat_body = CreateGroupChatRequestBody {
    name: name.to_string(),
    executor_id: executor_id.to_string(),
  };
  let create_group_chat_request = Request::builder()
    .uri(EndpointPaths::CreateGroupChat.as_str())
    .method(Method::POST)
    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
    .body(Body::from(serde_json::to_string(&create_group_chat_body).unwrap()))
    .unwrap();
  let create_group_chat_response = create_router(repository.clone())
    .oneshot(create_group_chat_request)
    .await
    .unwrap();
  assert_eq!(create_group_chat_response.status(), StatusCode::OK);
  let create_group_chat_response_body_bytes = hyper::body::to_bytes(create_group_chat_response.into_body())
    .await
    .unwrap();
  let create_group_chat_response_body_str = String::from_utf8(create_group_chat_response_body_bytes.to_vec()).unwrap();
  let group_chat_command_response_body =
    serde_json::from_str::<GroupChatCommandResponseSuccessBody>(&create_group_chat_response_body_str).unwrap();
  GroupChatId::from_str(&group_chat_command_response_body.group_chat_id).unwrap()
}

async fn delete_group_chat<TR: GroupChatRepository>(
  repository: &TR,
  group_chat_id: GroupChatId,
  executor_id: UserAccountId,
) -> GroupChatId {
  let delete_group_chat_body = DeleteGroupChatRequestBody {
    group_chat_id: group_chat_id.to_string(),
    executor_id: executor_id.to_string(),
  };
  let delete_group_chat_request = Request::builder()
    .uri(EndpointPaths::DeleteGroupChat.as_str())
    .method(Method::POST)
    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
    .body(Body::from(serde_json::to_string(&delete_group_chat_body).unwrap()))
    .unwrap();
  let delete_group_chat_response = create_router(repository.clone())
    .oneshot(delete_group_chat_request)
    .await
    .unwrap();
  assert_eq!(delete_group_chat_response.status(), StatusCode::OK);
  let delete_group_chat_response_body_bytes = hyper::body::to_bytes(delete_group_chat_response.into_body())
    .await
    .unwrap();
  let delete_group_chat_response_body_str = String::from_utf8(delete_group_chat_response_body_bytes.to_vec()).unwrap();
  let delete_command_response_body =
    serde_json::from_str::<GroupChatCommandResponseSuccessBody>(&delete_group_chat_response_body_str).unwrap();
  GroupChatId::from_str(&delete_command_response_body.group_chat_id).unwrap()
}

async fn rename_group_chat<TR: GroupChatRepository>(
  repository: &TR,
  group_chat_id: GroupChatId,
  name: GroupChatName,
  executor_id: UserAccountId,
) -> GroupChatId {
  let rename_group_chat_body = RenameGroupChatRequestBody {
    group_chat_id: group_chat_id.to_string(),
    name: name.to_string(),
    executor_id: executor_id.to_string(),
  };
  let rename_group_chat_request = Request::builder()
    .uri(EndpointPaths::RenameGroupChat.as_str())
    .method(Method::POST)
    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
    .body(Body::from(serde_json::to_string(&rename_group_chat_body).unwrap()))
    .unwrap();
  let rename_group_chat_response = create_router(repository.clone())
    .oneshot(rename_group_chat_request)
    .await
    .unwrap();
  assert_eq!(rename_group_chat_response.status(), StatusCode::OK);
  let rename_group_chat_response_body_bytes = hyper::body::to_bytes(rename_group_chat_response.into_body())
    .await
    .unwrap();
  let rename_group_chat_response_body_str = String::from_utf8(rename_group_chat_response_body_bytes.to_vec()).unwrap();
  let rename_group_chat_response_body =
    serde_json::from_str::<GroupChatCommandResponseSuccessBody>(&rename_group_chat_response_body_str).unwrap();
  GroupChatId::from_str(&rename_group_chat_response_body.group_chat_id).unwrap()
}

async fn add_member<TR: GroupChatRepository>(
  repository: &TR,
  group_chat_id: GroupChatId,
  user_account_id: UserAccountId,
  role: MemberRole,
  executor_id: UserAccountId,
) -> GroupChatId {
  let add_member_body = AddMemberRequestBody {
    group_chat_id: group_chat_id.to_string(),
    user_account_id: user_account_id.to_string(),
    role: role.to_string(),
    executor_id: executor_id.to_string(),
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
    serde_json::from_str::<GroupChatCommandResponseSuccessBody>(&add_member_response_body_str).unwrap();
  GroupChatId::from_str(&add_member_response_body.group_chat_id).unwrap()
}

async fn remove_member<TR: GroupChatRepository>(
  repository: &TR,
  group_chat_id: GroupChatId,
  user_account_id: UserAccountId,
  executor_id: UserAccountId,
) -> GroupChatId {
  let remove_member_body = RemoveMemberRequestBody {
    group_chat_id: group_chat_id.to_string(),
    user_account_id: user_account_id.to_string(),
    executor_id: executor_id.to_string(),
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
    serde_json::from_str::<GroupChatCommandResponseSuccessBody>(&remove_member_response_body_str).unwrap();
  GroupChatId::from_str(&remove_member_response_body.group_chat_id).unwrap()
}

async fn post_message<TR: GroupChatRepository>(
  repository: &TR,
  group_chat_id: GroupChatId,
  message: String,
  user_account_id: UserAccountId,
  executor_id: UserAccountId,
) -> MessageId {
  let post_message_body = PostMessageRequestBody {
    group_chat_id: group_chat_id.to_string(),
    user_account_id: user_account_id.to_string(),
    message,
    executor_id: executor_id.to_string(),
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
    serde_json::from_str::<MessageCommandResponseSuccessBody>(&post_message_response_body_str).unwrap();
  MessageId::from_str(&post_message_response_body.message_id).unwrap()
}

async fn delete_message<TR: GroupChatRepository>(
  repository: &TR,
  group_chat_id: GroupChatId,
  message_id: MessageId,
  user_account_id: UserAccountId,
  executor_id: UserAccountId,
) -> GroupChatId {
  let delete_message_body = DeleteMessageRequestBody {
    group_chat_id: group_chat_id.to_string(),
    user_account_id: user_account_id.to_string(),
    message_id: message_id.to_string(),
    executor_id: executor_id.to_string(),
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
    serde_json::from_str::<GroupChatCommandResponseSuccessBody>(&delete_message_response_body_str).unwrap();
  GroupChatId::from_str(&delete_message_response_body.group_chat_id).unwrap()
}

/// Router経由での結合テスト
#[cfg(test)]
mod tests {
  use super::*;
  use testcontainers::clients::Cli;

  #[tokio::test]
  async fn test() {
    test_create_group_chat().await;
    test_delete_group_chat().await;
    test_rename_group_chat().await;
    test_add_member().await;
    test_remove_member().await;
    // test_post_message().await;
    // test_delete_message().await;
  }

  async fn test_create_group_chat() {
    let docker = Cli::default();
    let (repository, container, client) = get_repository(&docker).await;
    let user_account_id = UserAccountId::new();
    let _ = create_group_chat(&repository, GroupChatName::new("ABC").unwrap(), user_account_id.clone()).await;

    drop(client);
    container.stop();
    drop(container);
  }

  async fn test_delete_group_chat() {
    let docker = Cli::default();
    let (repository, container, client) = get_repository(&docker).await;
    // Given
    let user_account_id = UserAccountId::new();
    let id1 = create_group_chat(&repository, GroupChatName::new("ABC").unwrap(), user_account_id.clone()).await;

    // When
    let id2 = delete_group_chat(&repository, id1.clone(), user_account_id.clone()).await;

    // Then
    assert_eq!(id1, id2);

    drop(client);
    container.stop();
    drop(container);
  }

  async fn test_rename_group_chat() {
    let docker = Cli::default();
    let (repository, container, client) = get_repository(&docker).await;
    // Given
    let user_account_id = UserAccountId::new();
    let id1 = create_group_chat(&repository, GroupChatName::new("ABC").unwrap(), user_account_id.clone()).await;

    // When
    let id2 = rename_group_chat(
      &repository,
      id1.clone(),
      GroupChatName::new("DEF").unwrap(),
      user_account_id,
    )
    .await;

    // Then
    assert_eq!(id1, id2);

    drop(client);
    container.stop();
    drop(container);
  }

  async fn test_add_member() {
    let docker = Cli::default();
    let (repository, container, client) = get_repository(&docker).await;
    // Given
    let user_account_id = UserAccountId::new();
    let id1 = create_group_chat(&repository, GroupChatName::new("ABC").unwrap(), user_account_id.clone()).await;
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

    drop(client);
    container.stop();
    drop(container);
  }

  async fn test_remove_member() {
    let docker = Cli::default();
    let (repository, container, client) = get_repository(&docker).await;
    // Given
    let user_account_id = UserAccountId::new();
    let id1 = create_group_chat(&repository, GroupChatName::new("ABC").unwrap(), user_account_id.clone()).await;
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

    drop(client);
    container.stop();
    drop(container);
  }

  async fn test_post_message() {
    let docker = Cli::default();
    let (repository, container, client) = get_repository(&docker).await;
    let user_account_id = UserAccountId::new();
    let id1 = create_group_chat(&repository, GroupChatName::new("ABC").unwrap(), user_account_id.clone()).await;

    let _id2 = post_message(
      &repository,
      id1.clone(),
      "Hello".to_string(),
      user_account_id.clone(),
      user_account_id.clone(),
    )
    .await;

    // assert_eq!(id1, id2);
    drop(client);
    container.stop();
    drop(container);
  }

  async fn test_delete_message() {
    let docker = Cli::default();
    let (repository, container, client) = get_repository(&docker).await;
    let user_account_id = UserAccountId::new();
    let id1 = create_group_chat(&repository, GroupChatName::new("ABC").unwrap(), user_account_id.clone()).await;

    let message_id = post_message(
      &repository,
      id1.clone(),
      "Hello".to_string(),
      user_account_id.clone(),
      user_account_id.clone(),
    )
    .await;

    let t = repository.find_by_id(&id1.clone()).await.unwrap();
    assert!(t.messages().contains(&message_id));

    let id3 = delete_message(
      &repository,
      id1.clone(),
      message_id.clone(),
      user_account_id.clone(),
      user_account_id.clone(),
    )
    .await;

    assert_eq!(id1, id3);

    drop(client);
    container.stop();
    drop(container);
  }
}

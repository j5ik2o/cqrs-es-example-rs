use std::sync::Arc;

use async_graphql::futures_util::Stream;
use async_graphql::futures_util::StreamExt;
use async_graphql::{
  Context, EmptyMutation, Error, ErrorExtensions, FieldResult, Object, Schema, SchemaBuilder, Subscription,
};
use redis::Client;
use sqlx::MySqlPool;

use crate::gateways::{
  GroupChat, GroupChatDao, GroupChatDaoError, GroupChatDaoImpl, Member, MemberDao, MemberDaoError, MemberDaoImpl,
  Message, MessageDao, MessageDaoError, MessageDaoImpl,
};

pub struct ServiceContext {
  group_chat_dao: Arc<dyn GroupChatDao>,
  member_dao: Arc<dyn MemberDao>,
  message_dao: Arc<dyn MessageDao>,
}

impl ServiceContext {
  pub fn new(
    group_chat_dao: Arc<dyn GroupChatDao>,
    member_dao: Arc<dyn MemberDao>,
    message_dao: Arc<dyn MessageDao>,
  ) -> Self {
    Self {
      group_chat_dao,
      member_dao,
      message_dao,
    }
  }

  pub fn get_group_chat_dao(&self) -> Arc<dyn GroupChatDao> {
    self.group_chat_dao.clone()
  }

  pub fn get_member_dao(&self) -> Arc<dyn MemberDao> {
    self.member_dao.clone()
  }

  pub fn get_message_dao(&self) -> Arc<dyn MessageDao> {
    self.message_dao.clone()
  }
}

/// クエリ
pub struct QueryRoot;

#[Object]
impl QueryRoot {
  /// 指定されたグループチャットIDのグループチャットを取得する。
  ///
  /// # 引数
  /// - `group_chat_id` - グループチャットID
  /// - `user_account_id` - 閲覧アカウントID
  ///
  /// # 戻り値
  /// - `GroupChat` - グループチャット
  async fn get_group_chat<'ctx>(
    &self,
    ctx: &Context<'ctx>,
    group_chat_id: String,
    user_account_id: String,
  ) -> FieldResult<GroupChat> {
    let ctx = ctx.data::<ServiceContext>().unwrap();
    ctx
      .group_chat_dao
      .get_group_chat(group_chat_id, user_account_id)
      .await
      .map_err(group_chat_dao_error_handling)
  }

  /// 指定されたアカウントIDが参加するグループチャット一覧を取得する。
  ///
  /// # 引数
  /// - `user_account_id` - 閲覧アカウントID
  ///
  /// # 戻り値
  /// - `Vec<GroupChat>` - グループチャット一覧
  async fn get_group_chats<'ctx>(&self, ctx: &Context<'ctx>, user_account_id: String) -> FieldResult<Vec<GroupChat>> {
    let ctx = ctx.data::<ServiceContext>().unwrap();
    ctx
      .group_chat_dao
      .get_group_chats(user_account_id)
      .await
      .map_err(group_chat_dao_error_handling)
  }

  /// 指定されたアカウントIDのメンバーを取得する
  ///
  /// # 引数
  /// - `group_chat_id` - グループチャットID
  /// - `user_account_id` - 閲覧アカウントID
  ///
  /// # 戻り値
  /// - `Member` - [Member]
  async fn get_member<'ctx>(
    &self,
    ctx: &Context<'ctx>,
    group_chat_id: String,
    user_account_id: String,
  ) -> FieldResult<Member> {
    let ctx = ctx.data::<ServiceContext>().unwrap();
    ctx
      .member_dao
      .get_member(group_chat_id, user_account_id)
      .await
      .map_err(member_dao_error_handling)
  }

  /// 指定されたグループチャットIDのメンバー一覧を取得する
  ///
  /// # 引数
  /// - `group_chat_id` - グループチャットID
  /// - `user_account_id` - 閲覧アカウントID
  ///
  /// # 戻り値
  /// - `Vec<Member>` - メンバー一覧
  async fn get_members<'ctx>(
    &self,
    ctx: &Context<'ctx>,
    group_chat_id: String,
    user_account_id: String,
  ) -> FieldResult<Vec<Member>> {
    let ctx = ctx.data::<ServiceContext>().unwrap();
    ctx
      .member_dao
      .get_members(group_chat_id, user_account_id)
      .await
      .map_err(member_dao_error_handling)
  }

  /// 指定されたメッセージIDのメッセージを取得する
  ///
  /// # 引数
  /// - `message_id` - メッセージID
  /// - `user_account_id` - 閲覧アカウントID
  ///
  /// # 戻り値
  /// - `Message` - メッセージ
  async fn get_message<'ctx>(
    &self,
    ctx: &Context<'ctx>,
    message_id: String,
    user_account_id: String,
  ) -> FieldResult<Message> {
    let ctx = ctx.data::<ServiceContext>().unwrap();
    ctx
      .message_dao
      .get_message(message_id, user_account_id)
      .await
      .map_err(message_dao_error_handling)
  }

  /// 指定されたグループチャットIDのメッセージ一覧を取得する
  ///
  /// # 引数
  /// - `group_chat_id` - グループチャットID
  /// - `user_account_id` - 閲覧アカウントID
  ///
  /// # 戻り値
  /// - `Vec<Message>` - メッセージ一覧
  async fn get_messages<'ctx>(
    &self,
    ctx: &Context<'ctx>,
    group_chat_id: String,
    user_account_id: String,
  ) -> FieldResult<Vec<Message>> {
    let ctx = ctx.data::<ServiceContext>().unwrap();
    ctx
      .message_dao
      .get_messages(group_chat_id, user_account_id)
      .await
      .map_err(message_dao_error_handling)
  }
}

fn group_chat_dao_error_handling(error: GroupChatDaoError) -> Error {
  match error {
    GroupChatDaoError::NotFoundError(_) => Error::new(error.to_string()).extend_with(|_, e| e.set("code", "404")),
    GroupChatDaoError::OtherError(_) => Error::new(error.to_string()).extend_with(|_, e| e.set("code", "500")),
  }
}

fn member_dao_error_handling(error: MemberDaoError) -> Error {
  match error {
    MemberDaoError::NotFoundError(_) => Error::new(error.to_string()).extend_with(|_, e| e.set("code", "404")),
    MemberDaoError::OtherError(_) => Error::new(error.to_string()).extend_with(|_, e| e.set("code", "500")),
  }
}

fn message_dao_error_handling(error: MessageDaoError) -> Error {
  match error {
    MessageDaoError::NotFoundError(_) => Error::new(error.to_string()).extend_with(|_, e| e.set("code", "404")),
    MessageDaoError::OtherError(_) => Error::new(error.to_string()).extend_with(|_, e| e.set("code", "500")),
  }
}

/// 以下のサブスクリプションは未実装です。
pub struct SubscriptionRoot;

/// https://github.com/async-graphql/examples/blob/c8219078a4b7aa6d84d22e9b79f033088897be4b/poem/subscription-redis/src/main.rs
#[Subscription]
impl SubscriptionRoot {
  async fn group_chats<'ctx>(
    &self,
    ctx: &Context<'ctx>,
    group_chat_id: String,
  ) -> anyhow::Result<impl Stream<Item = String>> {
    let client = ctx.data_unchecked::<Client>();
    let mut conn = client.get_async_connection().await?.into_pubsub();
    conn.subscribe(format!("group_chat_id={}", group_chat_id)).await?;
    Ok(
      conn
        .into_on_message()
        .filter_map(|msg| async move { msg.get_payload().ok() }),
    )
  }
}

/// ----

pub type ApiSchema = Schema<QueryRoot, EmptyMutation, SubscriptionRoot>;

pub fn create_schema_builder() -> SchemaBuilder<QueryRoot, EmptyMutation, SubscriptionRoot> {
  Schema::build(QueryRoot, EmptyMutation, SubscriptionRoot)
}

pub fn create_schema(pool: MySqlPool) -> ApiSchema {
  let group_chat_dao = GroupChatDaoImpl::new(pool.clone());
  let member_dao = MemberDaoImpl::new(pool.clone());
  let message_dao = MessageDaoImpl::new(pool);
  let ctx = ServiceContext::new(Arc::new(group_chat_dao), Arc::new(member_dao), Arc::new(message_dao));
  create_schema_builder().data(ctx).finish()
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use async_graphql::async_trait::async_trait;

  use chrono::NaiveDateTime;

  use crate::gateways::GroupChatDao;
  use crate::resolvers::{create_schema_builder, ServiceContext};

  use super::*;

  struct MockGroupChatDaoImpl;

  #[async_trait]
  impl GroupChatDao for MockGroupChatDaoImpl {
    async fn get_group_chat(
      &self,
      group_chat_id: String,
      _account_id: String,
    ) -> Result<Option<GroupChat>, GroupChatDaoError> {
      let t1 = GroupChat::new(
        group_chat_id,
        "mock group chat".to_string(),
        "mock owner".to_string(),
        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
      );
      Ok(Some(t1))
    }

    async fn get_group_chats(&self, user_account_id: String) -> Result<Vec<GroupChat>, GroupChatDaoError> {
      let t1 = GroupChat::new(
        "1".to_string(),
        "mock group chat".to_string(),
        user_account_id,
        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
      );
      Ok(vec![t1])
    }
  }

  struct MockMemberDaoImpl;

  #[async_trait]
  impl MemberDao for MockMemberDaoImpl {
    async fn get_member(
      &self,
      group_chat_id: String,
      user_account_id: String,
    ) -> Result<Option<Member>, MemberDaoError> {
      let m1 = Member::new(
        "1".to_string(),
        group_chat_id,
        user_account_id,
        "mock member".to_string(),
        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
      );
      Ok(Some(m1))
    }

    async fn get_members(
      &self,
      group_chat_id: String,
      _user_account_id: String,
    ) -> Result<Vec<Member>, MemberDaoError> {
      let m1 = Member::new(
        "1".to_string(),
        group_chat_id,
        "mock member".to_string(),
        "mock member".to_string(),
        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
      );
      Ok(vec![m1])
    }
  }

  struct MockMessageDaoImpl;

  #[async_trait]
  impl MessageDao for MockMessageDaoImpl {
    async fn get_message(
      &self,
      message_id: String,
      user_account_id: String,
    ) -> Result<Option<Message>, MessageDaoError> {
      let m1 = Message::new(
        message_id,
        "mock group chat".to_string(),
        user_account_id,
        "mock message".to_string(),
        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
      );
      Ok(Some(m1))
    }

    async fn get_messages(
      &self,
      group_chat_id: String,
      user_account_id: String,
    ) -> Result<Vec<Message>, MessageDaoError> {
      let m1 = Message::new(
        "1".to_string(),
        group_chat_id,
        user_account_id,
        "mock message".to_string(),
        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
      );
      Ok(vec![m1])
    }
  }

  fn create_schema_on_test() -> ApiSchema {
    let ctx = ServiceContext::new(
      Arc::new(MockGroupChatDaoImpl),
      Arc::new(MockMemberDaoImpl),
      Arc::new(MockMessageDaoImpl),
    );

    create_schema_builder().data(ctx).finish()
  }

  #[tokio::test]
  async fn test_get_group_chat() {
    let result = create_schema_on_test()
      .execute(r#"{ getGroupChat(groupChatId: "group_chat_id", userAccountId: "user_account_id") { id name } }"#)
      .await
      .into_result()
      .unwrap()
      .data;
    assert_eq!(
      result,
      async_graphql::value!({
          "getGroupChat": {
              "id": "group_chat_id",
              "name": "mock group chat"
          }
      })
    );
  }

  #[tokio::test]
  async fn test_get_group_chats() {
    let result = create_schema_on_test()
      .execute(r#"{ getGroupChats(userAccountId: "user_account_id") { id name } }"#)
      .await
      .into_result()
      .unwrap()
      .data;

    assert_eq!(
      result,
      async_graphql::value!({
          "getGroupChats": [{
              "id": "1",
              "name": "mock group chat"
          }]
      })
    );
  }

  #[tokio::test]
  async fn test_get_member() {
    let result = create_schema_on_test()
      .execute(
        r#"{ getMember(groupChatId: "group_chat_id", userAccountId: "user_account_id") { id, groupChatId, userAccountId, role } }"#,
      )
      .await
      .into_result()
      .unwrap()
      .data;

    assert_eq!(
      result,
      async_graphql::value!({
          "getMember": {
              "id": "1",
              "groupChatId": "group_chat_id",
              "userAccountId": "user_account_id",
              "role": "mock member"
          }
      })
    );
  }

  #[tokio::test]
  async fn test_get_members() {
    let result = create_schema_on_test()
      .execute(
        r#"{ getMembers(groupChatId: "group_chat_id", userAccountId: "user_account_id") { id, groupChatId, userAccountId, role } }"#,
      )
      .await
      .into_result()
      .unwrap()
      .data;

    assert_eq!(
      result,
      async_graphql::value!({
          "getMembers": [{
              "id": "1",
              "groupChatId": "group_chat_id",
              "userAccountId": "mock member",
              "role": "mock member"
          }]
      })
    );
  }

  #[tokio::test]
  async fn test_get_message() {
    let result = create_schema_on_test()
      .execute(
        r#"{ getMessage(messageId: "message_id", userAccountId: "user_account_id") { id, groupChatId, text, userAccountId } }"#,
      )
      .await
      .into_result()
      .unwrap()
      .data;

    assert_eq!(
      result,
      async_graphql::value!({
          "getMessage": {
              "id": "message_id",
              "groupChatId": "mock group chat",
              "text": "mock message",
              "userAccountId": "user_account_id"
          }
      })
    );
  }

  #[tokio::test]
  async fn test_get_messages() {
    let result = create_schema_on_test()
        .execute(r#"{ getMessages(groupChatId: "group_chat_id", userAccountId: "user_account_id") { id, groupChatId, text, userAccountId } }"#)
        .await
        .into_result()
        .unwrap()
        .data;

    assert_eq!(
      result,
      async_graphql::value!({
          "getMessages": [{
              "id": "1",
              "groupChatId": "group_chat_id",
              "text": "mock message",
              "userAccountId": "user_account_id"
          }]
      })
    );
  }
}

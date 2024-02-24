use crate::gateways::group_chat_repository::GroupChatRepositoryImpl;
use async_graphql::{
  Context, EmptySubscription, Error, FieldResult, InputObject, Object, Schema, SchemaBuilder, SimpleObject,
};
use command_domain::group_chat::{
  GroupChat, GroupChatEvent, GroupChatId, GroupChatName, MemberRole, Message, MessageId,
};
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::GroupChatRepository;
use command_processor::command_processor::GroupChatCommandProcessor;
use event_store_adapter_rs::EventStoreForDynamoDB;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ServiceContext<TR: GroupChatRepository> {
  group_chat_command_processor: Arc<Mutex<GroupChatCommandProcessor<TR>>>,
}

impl<TR: GroupChatRepository> ServiceContext<TR> {
  pub fn new(group_chat_command_processor: GroupChatCommandProcessor<TR>) -> Self {
    Self {
      group_chat_command_processor: Arc::new(Mutex::new(group_chat_command_processor)),
    }
  }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
  async fn health_check(&self) -> String {
    "OK".to_string()
  }
}

#[derive(Debug, Clone, SimpleObject)]
struct GroupChatResult {
  group_chat_id: String,
}

#[derive(Debug, Clone, SimpleObject)]
struct MessageResult {
  group_chat_id: String,
  message_id: String,
}

#[derive(Debug, Clone, InputObject)]
struct CreateGroupChatInput {
  name: String,
  executor_id: String,
}

#[derive(Debug, Clone, InputObject)]
struct DeleteGroupChatInput {
  group_chat_id: String,
  executor_id: String,
}

#[derive(Debug, Clone, InputObject)]
struct RenameGroupChatInput {
  group_chat_id: String,
  name: String,
  executor_id: String,
}

#[derive(Debug, Clone, InputObject)]
struct AddMemberInput {
  group_chat_id: String,
  user_account_id: String,
  role: String,
  executor_id: String,
}

#[derive(Debug, Clone, InputObject)]
struct RemoveMemberInput {
  group_chat_id: String,
  user_account_id: String,
  executor_id: String,
}

#[derive(Debug, Clone, InputObject)]
struct PostMessageInput {
  group_chat_id: String,
  content: String,
  executor_id: String,
}

#[derive(Debug, Clone, InputObject)]
struct DeleteMessageInput {
  group_chat_id: String,
  message_id: String,
  executor_id: String,
}

pub struct MutationRoot;

pub type ES = EventStoreForDynamoDB<GroupChatId, GroupChat, GroupChatEvent>;

#[Object]
impl MutationRoot {
  async fn create_group_chat<'ctx>(
    &self,
    ctx: &Context<'ctx>,
    input: CreateGroupChatInput,
  ) -> FieldResult<GroupChatResult> {
    let service_ctx = ctx.data::<ServiceContext<GroupChatRepositoryImpl<ES>>>().unwrap();

    let group_chat_name = match GroupChatName::from_str(&input.name) {
      Ok(group_chat) => group_chat,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let executor_id = match UserAccountId::from_str(&input.executor_id) {
      Ok(executor_id) => executor_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let mut processor = service_ctx.group_chat_command_processor.lock().await;
    match processor.create_group_chat(group_chat_name, executor_id).await {
      Ok(group_chat_id) => Ok(GroupChatResult {
        group_chat_id: group_chat_id.to_string(),
      }),
      Err(error) => {
        log::warn!("error = {}", error);
        Err(Error::new(error.to_string()))
      }
    }
  }

  async fn delete_group_chat<'ctx>(
    &self,
    ctx: &Context<'ctx>,
    input: DeleteGroupChatInput,
  ) -> FieldResult<GroupChatResult> {
    let service_ctx = ctx.data::<ServiceContext<GroupChatRepositoryImpl<ES>>>().unwrap();
    let group_chat_id = match GroupChatId::from_str(&input.group_chat_id) {
      Ok(group_chat_id) => group_chat_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let executor_id = match UserAccountId::from_str(&input.executor_id) {
      Ok(executor_id) => executor_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let mut processor = service_ctx.group_chat_command_processor.lock().await;
    match processor.delete_group_chat(group_chat_id, executor_id).await {
      Ok(group_chat_id) => Ok(GroupChatResult {
        group_chat_id: group_chat_id.to_string(),
      }),
      Err(error) => {
        log::warn!("error = {}", error);
        Err(Error::new(error.to_string()))
      }
    }
  }

  async fn rename_group_chat<'ctx>(
    &self,
    ctx: &Context<'ctx>,
    input: RenameGroupChatInput,
  ) -> FieldResult<GroupChatResult> {
    let service_ctx = ctx.data::<ServiceContext<GroupChatRepositoryImpl<ES>>>().unwrap();
    let group_chat_id = match GroupChatId::from_str(&input.group_chat_id) {
      Ok(group_chat_id) => group_chat_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let group_chat_name = match GroupChatName::from_str(&input.name) {
      Ok(group_chat_name) => group_chat_name,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let executor_id = match UserAccountId::from_str(&input.executor_id) {
      Ok(executor_id) => executor_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let mut processor = service_ctx.group_chat_command_processor.lock().await;
    match processor
      .rename_group_chat(group_chat_id, group_chat_name, executor_id)
      .await
    {
      Ok(group_chat_id) => Ok(GroupChatResult {
        group_chat_id: group_chat_id.to_string(),
      }),
      Err(error) => {
        log::warn!("error = {}", error);
        Err(Error::new(error.to_string()))
      }
    }
  }

  async fn add_member<'ctx>(&self, ctx: &Context<'ctx>, input: AddMemberInput) -> FieldResult<GroupChatResult> {
    let service_ctx = ctx.data::<ServiceContext<GroupChatRepositoryImpl<ES>>>().unwrap();
    let group_chat_id = match GroupChatId::from_str(&input.group_chat_id) {
      Ok(group_chat_id) => group_chat_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let user_account_id = match UserAccountId::from_str(&input.user_account_id) {
      Ok(user_account_id) => user_account_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let role = match MemberRole::from_str(&input.role) {
      Ok(role) => role,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let executor_id = match UserAccountId::from_str(&input.executor_id) {
      Ok(executor_id) => executor_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let mut processor = service_ctx.group_chat_command_processor.lock().await;
    match processor
      .add_member(group_chat_id, user_account_id, role, executor_id)
      .await
    {
      Ok(group_chat_id) => Ok(GroupChatResult {
        group_chat_id: group_chat_id.to_string(),
      }),
      Err(error) => {
        log::warn!("error = {}", error);
        Err(Error::new(error.to_string()))
      }
    }
  }

  async fn remove_member<'ctx>(&self, ctx: &Context<'ctx>, input: RemoveMemberInput) -> FieldResult<GroupChatResult> {
    let service_ctx = ctx.data::<ServiceContext<GroupChatRepositoryImpl<ES>>>().unwrap();
    let group_chat_id = match GroupChatId::from_str(&input.group_chat_id) {
      Ok(group_chat_id) => group_chat_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let user_account_id = match UserAccountId::from_str(&input.user_account_id) {
      Ok(user_account_id) => user_account_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let executor_id = match UserAccountId::from_str(&input.executor_id) {
      Ok(executor_id) => executor_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let mut processor = service_ctx.group_chat_command_processor.lock().await;
    match processor
      .remove_member(group_chat_id, user_account_id, executor_id)
      .await
    {
      Ok(group_chat_id) => Ok(GroupChatResult {
        group_chat_id: group_chat_id.to_string(),
      }),
      Err(error) => {
        log::warn!("error = {}", error);
        Err(Error::new(error.to_string()))
      }
    }
  }

  async fn post_message<'ctx>(&self, ctx: &Context<'ctx>, input: PostMessageInput) -> FieldResult<MessageResult> {
    let service_ctx = ctx.data::<ServiceContext<GroupChatRepositoryImpl<ES>>>().unwrap();
    let group_chat_id = match GroupChatId::from_str(&input.group_chat_id) {
      Ok(group_chat_id) => group_chat_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let content = input.content.clone();
    let executor_id = match UserAccountId::from_str(&input.executor_id) {
      Ok(executor_id) => executor_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let mut processor = service_ctx.group_chat_command_processor.lock().await;
    let message = Message::new(content, executor_id.clone());
    match processor.post_message(group_chat_id, message, executor_id).await {
      Ok((group_chat_id, message_id)) => Ok(MessageResult {
        group_chat_id: group_chat_id.to_string(),
        message_id: message_id.to_string(),
      }),
      Err(error) => {
        log::warn!("error = {}", error);
        Err(Error::new(error.to_string()))
      }
    }
  }

  async fn delete_message<'ctx>(&self, ctx: &Context<'ctx>, input: DeleteMessageInput) -> FieldResult<GroupChatResult> {
    let service_ctx = ctx.data::<ServiceContext<GroupChatRepositoryImpl<ES>>>().unwrap();
    let group_chat_id = match GroupChatId::from_str(&input.group_chat_id) {
      Ok(group_chat_id) => group_chat_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let message_id = match MessageId::from_str(&input.message_id) {
      Ok(message_id) => message_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let executor_id = match UserAccountId::from_str(&input.executor_id) {
      Ok(executor_id) => executor_id,
      Err(error) => {
        log::warn!("error = {}", error);
        return Err(Error::new(error.to_string()));
      }
    };
    let mut processor = service_ctx.group_chat_command_processor.lock().await;
    match processor.delete_message(group_chat_id, message_id, executor_id).await {
      Ok(group_chat_id) => Ok(GroupChatResult {
        group_chat_id: group_chat_id.to_string(),
      }),
      Err(error) => {
        log::warn!("error = {}", error);
        Err(Error::new(error.to_string()))
      }
    }
  }
}

pub type ApiSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema_builder() -> SchemaBuilder<QueryRoot, MutationRoot, EmptySubscription> {
  Schema::build(QueryRoot, MutationRoot, EmptySubscription)
}

pub fn create_schema(group_chat_repository: GroupChatRepositoryImpl<ES>) -> ApiSchema {
  let processor = GroupChatCommandProcessor::new(group_chat_repository);
  let ctx = ServiceContext::new(processor);
  create_schema_builder().data(ctx).finish()
}

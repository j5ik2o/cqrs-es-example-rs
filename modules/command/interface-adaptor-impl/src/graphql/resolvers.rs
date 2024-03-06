use async_graphql::{Context, Error, ErrorExtensions, FieldResult, Object};
use event_store_adapter_rs::types::EventStoreWriteError;
use std::str::FromStr;

use command_domain::group_chat::{GroupChatId, GroupChatName, MemberRole, Message, MessageId};
use command_domain::group_chat_error::GroupChatError;
use command_domain::user_account::UserAccountId;
use command_processor::group_chat_command_processor::CommandProcessError;

use crate::gateways::group_chat_repository::GroupChatRepositoryImpl;
use crate::graphql::inputs::{
  AddMemberInput, CreateGroupChatInput, DeleteGroupChatInput, DeleteMessageInput, EditMessageInput, PostMessageInput,
  RemoveMemberInput, RenameGroupChatInput,
};
use crate::graphql::outputs::{GroupChatOut, MessageOut};
use crate::graphql::{MutationRoot, ServiceContext, ES};

#[Object]
impl MutationRoot {
  async fn create_group_chat<'ctx>(
    &self,
    ctx: &Context<'ctx>,
    input: CreateGroupChatInput,
  ) -> FieldResult<GroupChatOut> {
    let service_ctx = ctx.data::<ServiceContext<GroupChatRepositoryImpl<ES>>>().unwrap();

    let group_chat_name = validate_group_chat_name(&input.name)?;
    let executor_id = validate_user_account_id(&input.executor_id)?;

    let mut processor = service_ctx.group_chat_command_processor.lock().await;
    processor
      .create_group_chat(group_chat_name, executor_id)
      .await
      .map(|group_chat_id| GroupChatOut::new(group_chat_id.to_string()))
      .map_err(error_handling)
  }

  async fn delete_group_chat<'ctx>(
    &self,
    ctx: &Context<'ctx>,
    input: DeleteGroupChatInput,
  ) -> FieldResult<GroupChatOut> {
    let service_ctx = ctx.data::<ServiceContext<GroupChatRepositoryImpl<ES>>>().unwrap();

    let group_chat_id = validate_group_chat_id(&input.group_chat_id)?;
    let executor_id = validate_user_account_id(&input.executor_id)?;

    let mut processor = service_ctx.group_chat_command_processor.lock().await;
    processor
      .delete_group_chat(group_chat_id, executor_id)
      .await
      .map(|group_chat_id| GroupChatOut::new(group_chat_id.to_string()))
      .map_err(error_handling)
  }

  async fn rename_group_chat<'ctx>(
    &self,
    ctx: &Context<'ctx>,
    input: RenameGroupChatInput,
  ) -> FieldResult<GroupChatOut> {
    let service_ctx = ctx.data::<ServiceContext<GroupChatRepositoryImpl<ES>>>().unwrap();

    let group_chat_id = validate_group_chat_id(&input.group_chat_id)?;
    let group_chat_name = validate_group_chat_name(&input.name)?;
    let executor_id = validate_user_account_id(&input.executor_id)?;

    let mut processor = service_ctx.group_chat_command_processor.lock().await;
    processor
      .rename_group_chat(group_chat_id, group_chat_name, executor_id)
      .await
      .map(|group_chat_id| GroupChatOut::new(group_chat_id.to_string()))
      .map_err(error_handling)
  }

  async fn add_member<'ctx>(&self, ctx: &Context<'ctx>, input: AddMemberInput) -> FieldResult<GroupChatOut> {
    let service_ctx = ctx.data::<ServiceContext<GroupChatRepositoryImpl<ES>>>().unwrap();

    let group_chat_id = validate_group_chat_id(&input.group_chat_id)?;
    let user_account_id = validate_user_account_id(&input.user_account_id)?;
    let role = validate_member_role(&input.role)?;
    let executor_id = validate_user_account_id(&input.executor_id)?;

    let mut processor = service_ctx.group_chat_command_processor.lock().await;
    processor
      .add_member(group_chat_id, user_account_id, role, executor_id)
      .await
      .map(|group_chat_id| GroupChatOut::new(group_chat_id.to_string()))
      .map_err(error_handling)
  }

  async fn remove_member<'ctx>(&self, ctx: &Context<'ctx>, input: RemoveMemberInput) -> FieldResult<GroupChatOut> {
    let service_ctx = ctx.data::<ServiceContext<GroupChatRepositoryImpl<ES>>>().unwrap();

    let group_chat_id = validate_group_chat_id(&input.group_chat_id)?;
    let user_account_id = validate_user_account_id(&input.user_account_id)?;
    let executor_id = validate_user_account_id(&input.executor_id)?;

    let mut processor = service_ctx.group_chat_command_processor.lock().await;

    processor
      .remove_member(group_chat_id, user_account_id, executor_id)
      .await
      .map(|group_chat_id| GroupChatOut::new(group_chat_id.to_string()))
      .map_err(error_handling)
  }

  async fn post_message<'ctx>(&self, ctx: &Context<'ctx>, input: PostMessageInput) -> FieldResult<MessageOut> {
    let service_ctx = ctx.data::<ServiceContext<GroupChatRepositoryImpl<ES>>>().unwrap();

    let group_chat_id = validate_group_chat_id(&input.group_chat_id)?;
    let executor_id = validate_user_account_id(&input.executor_id)?;
    let message = validate_message(&input.content, executor_id.clone())?;

    let mut processor = service_ctx.group_chat_command_processor.lock().await;
    processor
      .post_message(group_chat_id, message, executor_id)
      .await
      .map(|(group_chat_id, message_id)| MessageOut::new(group_chat_id.to_string(), message_id.to_string()))
      .map_err(error_handling)
  }

  async fn edit_message<'ctx>(&self, ctx: &Context<'ctx>, input: EditMessageInput) -> FieldResult<MessageOut> {
    todo!()
  }

  async fn delete_message<'ctx>(&self, ctx: &Context<'ctx>, input: DeleteMessageInput) -> FieldResult<GroupChatOut> {
    let service_ctx = ctx.data::<ServiceContext<GroupChatRepositoryImpl<ES>>>().unwrap();

    let group_chat_id = validate_group_chat_id(&input.group_chat_id)?;
    let message_id = validate_message_id(&input.message_id)?;
    let executor_id = validate_user_account_id(&input.executor_id)?;

    let mut processor = service_ctx.group_chat_command_processor.lock().await;
    processor
      .delete_message(group_chat_id, message_id, executor_id)
      .await
      .map(|group_chat_id| GroupChatOut::new(group_chat_id.to_string()))
      .map_err(error_handling)
  }
}

fn error_handling(error: anyhow::Error) -> Error {
  if let Some(CommandProcessError::NotFoundError) = error.downcast_ref::<CommandProcessError>() {
    return Error::new(error.to_string()).extend_with(|_, e| e.set("code", "404"));
  }
  if let Some(EventStoreWriteError::OptimisticLockError(cause)) = error.downcast_ref::<EventStoreWriteError>() {
    return Error::new(error.to_string())
      .extend_with(|_, e| e.set("code", "409"))
      .extend_with(|_, e| e.set("cause", cause.to_string()));
  }
  if let Some(group_chat_error) = error.downcast_ref::<GroupChatError>() {
    return Error::new(group_chat_error.to_string()).extend_with(|_, e| e.set("code", "422"));
  }
  return Error::new(error.to_string()).extend_with(|_, e| e.set("code", "500"));
}

fn validate_group_chat_id(value: &str) -> Result<GroupChatId, Error> {
  GroupChatId::from_str(value).map_err(|error| Error::new(error.to_string()).extend_with(|_, e| e.set("code", "400")))
}

fn validate_group_chat_name(value: &str) -> Result<GroupChatName, Error> {
  GroupChatName::from_str(value).map_err(|error| Error::new(error.to_string()).extend_with(|_, e| e.set("code", "400")))
}

fn validate_member_role(value: &str) -> Result<MemberRole, Error> {
  MemberRole::from_str(value).map_err(|error| Error::new(error.to_string()).extend_with(|_, e| e.set("code", "400")))
}

fn validate_message_id(value: &str) -> Result<MessageId, Error> {
  MessageId::from_str(value).map_err(|error| Error::new(error.to_string()).extend_with(|_, e| e.set("code", "400")))
}

fn validate_message(value: &str, sender_id: UserAccountId) -> Result<Message, Error> {
  Message::validate(&value, sender_id.clone())
    .map_err(|error| Error::new(error.to_string()).extend_with(|_, e| e.set("code", "400")))
}

fn validate_user_account_id(value: &str) -> Result<UserAccountId, Error> {
  UserAccountId::from_str(value).map_err(|error| Error::new(error.to_string()).extend_with(|_, e| e.set("code", "400")))
}

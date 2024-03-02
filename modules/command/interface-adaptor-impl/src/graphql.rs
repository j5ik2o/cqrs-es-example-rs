use std::str::FromStr;
use std::sync::Arc;

use async_graphql::{EmptySubscription, ErrorExtensions, Object, Schema, SchemaBuilder};
use event_store_adapter_rs::EventStoreForDynamoDB;
use tokio::sync::Mutex;

use command_domain::group_chat::{GroupChat, GroupChatEvent, GroupChatId};
use command_interface_adaptor_if::GroupChatRepository;
use command_processor::group_chat_command_processor::GroupChatCommandProcessor;

use crate::gateways::group_chat_repository::GroupChatRepositoryImpl;

pub mod inputs;
pub mod outputs;
pub mod resolvers;

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

pub struct MutationRoot;

pub type ES = EventStoreForDynamoDB<GroupChatId, GroupChat, GroupChatEvent>;

pub type ApiSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema_builder() -> SchemaBuilder<QueryRoot, MutationRoot, EmptySubscription> {
  Schema::build(QueryRoot, MutationRoot, EmptySubscription)
}

pub fn create_schema(group_chat_repository: GroupChatRepositoryImpl<ES>) -> ApiSchema {
  let processor = GroupChatCommandProcessor::new(group_chat_repository);
  let ctx = ServiceContext::new(processor);
  create_schema_builder().data(ctx).finish()
}

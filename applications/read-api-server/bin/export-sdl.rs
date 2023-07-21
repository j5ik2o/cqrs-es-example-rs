use anyhow::Result;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};

use cqrs_es_example_query_interface_adaptor_impl::resolvers::QueryRoot;

#[tokio::main]
async fn main() -> Result<()> {
  let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
  println!("{}", schema.sdl());
  Ok(())
}

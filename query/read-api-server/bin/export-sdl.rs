use anyhow::Result;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};

use cqrs_es_example_read_api_server::QueryRoot;

#[tokio::main]
async fn main() -> Result<()> {
  let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
  println!("{}", schema.sdl());
  Ok(())
}

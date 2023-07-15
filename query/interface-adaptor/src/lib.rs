use anyhow::Result;
use async_graphql::futures_util::{Stream, StreamExt};
use async_graphql::{Context, Object, SimpleObject, Subscription};
use chrono::NaiveDateTime;
use redis::Client;
use sqlx::MySqlPool;

#[derive(SimpleObject)]
pub struct Thread {
  id: String,
  name: String,
  owner_id: String,
  created_at: NaiveDateTime,
}

impl Thread {}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
  async fn get_thread<'ctx>(&self, ctx: &Context<'ctx>, thread_id: String) -> Result<Thread> {
    let pool = ctx.data::<MySqlPool>().unwrap();

    let thread = sqlx::query_as!(
      Thread,
      "SELECT id, name, owner_id, created_at FROM threads WHERE id = ?",
      thread_id
    )
    .fetch_one(pool)
    .await?;

    Ok(thread)
  }
}

pub struct SubscriptionRoot;

/// https://github.com/async-graphql/examples/blob/c8219078a4b7aa6d84d22e9b79f033088897be4b/poem/subscription-redis/src/main.rs
#[Subscription]
impl SubscriptionRoot {
  async fn threads<'ctx>(&self, ctx: &Context<'ctx>, thread_id: String) -> Result<impl Stream<Item = String>> {
    let client = ctx.data_unchecked::<Client>();
    let mut conn = client.get_async_connection().await?.into_pubsub();
    conn.subscribe(format!("thread_id={}", thread_id)).await?;
    Ok(
      conn
        .into_on_message()
        .filter_map(|msg| async move { msg.get_payload().ok() }),
    )
  }
}

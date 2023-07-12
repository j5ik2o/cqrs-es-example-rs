use anyhow::Result;
use async_graphql::{Object, SimpleObject, Subscription};
use async_graphql::futures_util::StreamExt;
use chrono::NaiveDateTime;
use config::Environment;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ApiSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct RedisSettings {
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct AppSettings {
    pub api: ApiSettings,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
}

pub fn load_app_config() -> Result<AppSettings> {
    let config = config::Config::builder()
    .add_source(config::File::with_name("config/read-api-server").required(false))
    .add_source(Environment::with_prefix("APP").try_parsing(true).separator("__"))
    .build()?;
  let app_config = config.try_deserialize()?;
  Ok(app_config)
}

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


struct SubscriptionRoot;

/// https://github.com/async-graphql/examples/blob/c8219078a4b7aa6d84d22e9b79f033088897be4b/poem/subscription-redis/src/main.rs
#[Subscription]
impl SubscriptionRoot {
    async fn threads<'ctx>(&self, ctx: &Context<'ctx>, thread_id: String) -> Result<impl Stream<Item=String>> {
        let client = ctx.data_unchecked::<Client>();
        let mut conn = client.get_async_connection().await?.into_pubsub();
        conn.subscribe(format!("thread_id={}", thread_id)).await?;
        Ok(conn
            .into_on_message()
            .filter_map(|msg| async move { msg.get_payload().ok() }))
    }
}
use anyhow::Result;
use async_graphql::{Object, SimpleObject};
use chrono::NaiveDateTime;
use config::Environment;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ApiSettings {
  host: String,
  port: u16,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
  pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct AppSettings {
  pub api: ApiSettings,
  pub database: DatabaseSettings,
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

use anyhow::Result;
use config::Environment;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ApiSettings {
  pub host: String,
  pub port: u16,
  pub allow_origins: Vec<String>,
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
  // pub redis: RedisSettings,
}

pub fn load_app_config() -> Result<AppSettings> {
  let source = Environment::with_prefix("APP")
    .try_parsing(true)
    .separator("__")
    .with_list_parse_key("api.allow_origins")
    .list_separator(",");
  let config = config::Config::builder()
    .add_source(config::File::with_name("config/read-api-server").required(false))
    .add_source(source)
    .build()?;
  let app_config = config.try_deserialize()?;
  Ok(app_config)
}

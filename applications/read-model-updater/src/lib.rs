use config::{ConfigError, Environment};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AppSettings {
  pub aws: AwsSettings,
  pub stream: Option<StreamSettings>,
  pub database: DatabaseSettings,
}

#[derive(Deserialize, Debug)]
pub struct AwsSettings {
  /// AWSリージョン名
  pub region_name: String,
  /// DynamoDBのエンドポイントURL(ローカル開発用)
  pub endpoint_url: Option<String>,
  /// アクセスキーID(ローカル開発用)
  pub access_key_id: Option<String>,
  /// シークレットアクセスキー(ローカル開発用)
  pub secret_access_key: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
  /// データベース接続URL
  pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct RedisSettings {
  pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct StreamSettings {
  /// コンシュームするジャーナルのテーブル名
  pub journal_table_name: String,
  /// ストリームから読み込む最大件数
  pub max_item_count: usize,
}

pub fn load_app_config() -> Result<AppSettings, ConfigError> {
  let config = config::Config::builder()
    .add_source(config::File::with_name("config/read-model-updater").required(false))
    .add_source(Environment::with_prefix("APP").try_parsing(true).separator("__"))
    .build()?;
  let app_config = config.try_deserialize()?;
  Ok(app_config)
}

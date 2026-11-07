use aws_config::meta::region::RegionProviderChain;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::config::{Credentials, Region};
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

#[derive(Deserialize, Debug, Clone)]
pub struct StreamSettings {
  /// 購読する集約 head テーブル名
  pub head_table_name: String,
  /// 欠番を補うために読む journal テーブル名
  pub journal_table_name: String,
  /// Streams から読み込む最大件数
  pub max_item_count: i32,
  /// 新しいレコードを待つ間隔(ミリ秒)
  #[serde(default = "default_poll_interval_millis")]
  pub poll_interval_millis: u64,
}

fn default_poll_interval_millis() -> u64 {
  1000
}

pub fn load_app_config() -> Result<AppSettings, ConfigError> {
  let config = config::Config::builder()
    .add_source(config::File::with_name("config/read-model-updater").required(false))
    .add_source(Environment::with_prefix("APP").try_parsing(true).separator("__"))
    .build()?;
  let app_config = config.try_deserialize()?;
  Ok(app_config)
}

/// アプリ共通の AWS SDK 設定を組み立てる。
pub async fn create_aws_config(aws_settings: &AwsSettings) -> SdkConfig {
  let region = Region::new(aws_settings.region_name.clone());
  let region_provider_chain = RegionProviderChain::default_provider().or_else(region);
  let mut config_loader = aws_config::from_env().region(region_provider_chain);
  if let Some(endpoint_url) = aws_settings.endpoint_url.clone() {
    tracing::info!("endpoint_url = {}", endpoint_url);
    config_loader = config_loader.endpoint_url(endpoint_url);
  }
  if let (Some(access_key_id), Some(secret_access_key)) = (
    aws_settings.access_key_id.clone(),
    aws_settings.secret_access_key.clone(),
  ) {
    config_loader = config_loader.credentials_provider(Credentials::new(
      access_key_id,
      secret_access_key,
      None,
      None,
      "default",
    ));
  }
  config_loader.load().await
}

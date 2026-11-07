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
  #[serde(default = "default_max_item_count")]
  pub max_item_count: i32,
  /// 新しいレコードを待つ初期間隔(ミリ秒)。待ち時間は指数バックオフする。
  #[serde(default = "default_poll_interval_millis")]
  pub poll_interval_millis: u64,
  /// ポーリング間隔の上限(ミリ秒)
  #[serde(default = "default_poll_max_interval_millis")]
  pub poll_max_interval_millis: u64,
  /// 1 トランザクションで確定するイベント数
  #[serde(default = "default_max_events_per_transaction")]
  pub max_events_per_transaction: usize,
  /// 1 レコードの最大試行回数
  #[serde(default = "default_retry_max_attempts")]
  pub retry_max_attempts: usize,
  /// 1 レコードの再試行に使える最大経過時間(ミリ秒)
  #[serde(default = "default_retry_max_elapsed_millis")]
  pub retry_max_elapsed_millis: u64,
  /// 再試行の初期待ち時間(ミリ秒)
  #[serde(default = "default_retry_initial_delay_millis")]
  pub retry_initial_delay_millis: u64,
  /// 再試行の待ち時間の上限(ミリ秒)
  #[serde(default = "default_retry_max_delay_millis")]
  pub retry_max_delay_millis: u64,
}

impl StreamSettings {
  pub fn retry_policy(&self) -> rmu::RetryPolicy {
    rmu::RetryPolicy::new(
      self.retry_max_attempts,
      std::time::Duration::from_millis(self.retry_max_elapsed_millis),
      std::time::Duration::from_millis(self.retry_initial_delay_millis),
      std::time::Duration::from_millis(self.retry_max_delay_millis),
    )
  }

  pub fn poll_backoff(&self) -> rmu::Backoff {
    rmu::Backoff::new(
      std::time::Duration::from_millis(self.poll_interval_millis),
      std::time::Duration::from_millis(self.poll_max_interval_millis),
    )
  }
}

fn default_max_item_count() -> i32 {
  100
}

fn default_poll_interval_millis() -> u64 {
  1000
}

fn default_poll_max_interval_millis() -> u64 {
  30_000
}

fn default_max_events_per_transaction() -> usize {
  1
}

fn default_retry_max_attempts() -> usize {
  5
}

fn default_retry_max_elapsed_millis() -> u64 {
  30_000
}

fn default_retry_initial_delay_millis() -> u64 {
  200
}

fn default_retry_max_delay_millis() -> u64 {
  5_000
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_default_max_item_count_and_poll_interval() {
    let source = config::File::from_str(
      "head_table_name = \"head\"\njournal_table_name = \"journal\"\n",
      config::FileFormat::Toml,
    );
    let settings: StreamSettings = config::Config::builder()
      .add_source(source)
      .build()
      .unwrap()
      .try_deserialize()
      .unwrap();

    assert_eq!(100, settings.max_item_count);
    assert_eq!(1000, settings.poll_interval_millis);
    assert_eq!(30_000, settings.poll_max_interval_millis);
    assert_eq!(1, settings.max_events_per_transaction);
    assert_eq!(5, settings.retry_max_attempts);
    assert_eq!(30_000, settings.retry_max_elapsed_millis);
    assert_eq!(200, settings.retry_initial_delay_millis);
    assert_eq!(5_000, settings.retry_max_delay_millis);
  }
}

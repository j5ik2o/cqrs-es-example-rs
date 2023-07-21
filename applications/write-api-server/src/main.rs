use std::fmt::Debug;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Once;

use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::config::{Credentials, Region};
use aws_sdk_dynamodb::Client;
use config::{Config, Environment};
use cqrs_es_example_command_interface_adaptor_impl::controllers::create_router;
use cqrs_es_example_command_interface_adaptor_impl::gateways::event_persistence_gateway::EventPersistenceGateway;
use cqrs_es_example_command_interface_adaptor_impl::gateways::thread_repository::ThreadRepositoryImpl;
use serde::Deserialize;
use tracing_log::LogTracer;

#[derive(Deserialize, Debug)]
struct AppSettings {
  api: ApiSettings,
  persistence: PersistenceSettings,
  aws: AwsSettings,
}

#[derive(Deserialize, Debug)]
struct ApiSettings {
  host: String,
  port: u16,
}

#[derive(Deserialize, Debug)]
struct PersistenceSettings {
  journal_table_name: String,
  journal_aid_index_name: String,
  snapshot_table_name: String,
  snapshot_aid_index_name: String,
  shard_count: u64,
}

#[derive(Deserialize, Debug)]
struct AwsSettings {
  region_name: String,
  endpoint_url: Option<String>,
  access_key_id: Option<String>,
  secret_access_key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_target(false)
    .with_ansi(false)
    .without_time()
    .init();

  let app_settings = load_app_config().unwrap();
  let aws_client = create_aws_client(&app_settings.aws).await;
  let egg = EventPersistenceGateway::new(
    aws_client,
    app_settings.persistence.journal_table_name.clone(),
    app_settings.persistence.journal_aid_index_name.clone(),
    app_settings.persistence.snapshot_table_name.clone(),
    app_settings.persistence.snapshot_aid_index_name.clone(),
    app_settings.persistence.shard_count,
  );
  let repository = ThreadRepositoryImpl::new(egg);
  let socket_addr = SocketAddr::new(IpAddr::from_str(&app_settings.api.host).unwrap(), app_settings.api.port);
  tracing::info!("Server listening on {}", socket_addr);
  axum::Server::bind(&socket_addr)
    .serve(create_router(repository).into_make_service())
    .await?;
  Ok(())
}

fn load_app_config() -> Result<AppSettings> {
  let config = Config::builder()
    .add_source(config::File::with_name("config/write-api-server").required(false))
    .add_source(Environment::with_prefix("APP").try_parsing(true).separator("__"))
    .build()?;
  tracing::info!("config = {:#?}", config);
  let app_config = config.try_deserialize()?;
  Ok(app_config)
}

async fn create_aws_client(aws_settings: &AwsSettings) -> Client {
  tracing::info!("create_aws_client: start");
  let region_name = aws_settings.region_name.clone();
  let region = Region::new(region_name);
  let region_provider_chain = RegionProviderChain::default_provider().or_else(region);

  let mut config_loader = aws_config::from_env().region(region_provider_chain);
  if let Some(endpoint_url) = aws_settings.endpoint_url.clone() {
    tracing::info!("endpoint_url = {}", endpoint_url);
    config_loader = config_loader.endpoint_url(endpoint_url);
  }

  match (
    aws_settings.access_key_id.clone(),
    aws_settings.secret_access_key.clone(),
  ) {
    (Some(access_key_id), Some(secret_access_key)) => {
      tracing::info!("access_key_id = {}", access_key_id);
      tracing::info!("secret_access_key = {}", secret_access_key);
      config_loader = config_loader.credentials_provider(Credentials::new(
        access_key_id,
        secret_access_key,
        None,
        None,
        "default",
      ));
    }
    _ => {}
  }

  let config = config_loader.load().await;
  tracing::info!("create_aws_client: SdkConfig = {:#?}", config);
  let client = Client::new(&config);
  tracing::info!("create_aws_client: finish");
  client
}

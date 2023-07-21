use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use anyhow::Result;
use sqlx::MySqlPool;

use cqrs_es_example_query_interface_adaptor_impl::controllers::create_router;
use cqrs_es_example_read_api_server::load_app_config;

#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_target(false)
    .with_ansi(false)
    .without_time()
    .init();

  let app_settings = load_app_config().unwrap();
  let pool = MySqlPool::connect(&app_settings.database.url).await?;
  let router = create_router(pool);

  let socket_addr = SocketAddr::new(IpAddr::from_str(&app_settings.api.host).unwrap(), app_settings.api.port);
  tracing::info!("Server listening on {}", socket_addr);
  axum::Server::bind(&socket_addr)
    .serve(router.into_make_service())
    .await?;
  Ok(())
}

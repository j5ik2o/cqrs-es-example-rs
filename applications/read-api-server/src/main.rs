use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use anyhow::Result;
use axum::headers::HeaderValue;
use hyper::header::CONTENT_TYPE;
use sqlx::MySqlPool;
use tower_http::cors::{AllowMethods, CorsLayer};

use query_interface_adaptor::controllers::create_router;
use read_api_server::{load_app_config, AppSettings};

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

  let router = create_router(pool).layer(create_cors_layer(&app_settings));

  let socket_addr = SocketAddr::new(IpAddr::from_str(&app_settings.api.host).unwrap(), app_settings.api.port);
  tracing::info!("Server listening on http://{}", socket_addr);
  axum::Server::bind(&socket_addr)
    .serve(router.into_make_service())
    .await?;
  Ok(())
}

fn create_cors_layer(app_settings: &AppSettings) -> CorsLayer {
  let origins = app_settings
    .api
    .allow_origins
    .iter()
    .map(|origin| origin.parse::<HeaderValue>().unwrap())
    .collect::<Vec<_>>();

  CorsLayer::new()
    .allow_origin(origins)
    .allow_headers(vec![CONTENT_TYPE])
    .allow_methods(AllowMethods::any())
}

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use anyhow::Result;
use async_graphql::http::GraphiQLSource;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::response;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};
use sqlx::MySqlPool;

use cqrs_es_example_query_interface_adaptor_impl::*;
use cqrs_es_example_read_api_server::load_app_config;

type ApiSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

async fn graphql_handler(schema: Extension<ApiSchema>, req: GraphQLRequest) -> GraphQLResponse {
  schema.execute(req.into_inner()).await.into()
}

async fn graphql() -> impl IntoResponse {
  response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

async fn hello_read_api() -> &'static str {
    "Hello, Read API!"
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

  let pool = MySqlPool::connect(&app_settings.database.url).await?;
  let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
    .data(pool)
    .finish();

  let app = Router::new()
    .route("/", get(hello_read_api))
    .route("/health/alive", get(alive))
    .route("/health/ready", get(ready))
    .route("/graphql", get(graphql).post(graphql_handler))
    .layer(Extension(schema));

  let socket_addr = SocketAddr::new(IpAddr::from_str(&app_settings.api.host).unwrap(), app_settings.api.port);
  tracing::info!("Server listening on {}", socket_addr);

  axum::Server::bind(&socket_addr).serve(app.into_make_service()).await?;
  Ok(())
}

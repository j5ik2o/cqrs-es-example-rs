use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::http::StatusCode;
use axum::response;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};
use sqlx::MySqlPool;

use crate::resolvers::{create_schema, ApiSchema};

pub async fn hello_read_api() -> &'static str {
  "Hello, Read API!"
}

pub async fn alive() -> impl IntoResponse {
  (StatusCode::OK, "OK")
}

pub async fn ready() -> impl IntoResponse {
  (StatusCode::OK, "OK")
}

async fn graphql_handler(schema: Extension<ApiSchema>, req: GraphQLRequest) -> GraphQLResponse {
  schema.execute(req.into_inner()).await.into()
}

async fn graphql() -> impl IntoResponse {
  response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub enum EndpointPaths {
  Root,
  HealthAlive,
  HealthReady,
  GraphQL,
}

impl EndpointPaths {
  pub fn as_str(&self) -> &'static str {
    match *self {
      EndpointPaths::Root => "/",
      EndpointPaths::HealthAlive => "/health/alive",
      EndpointPaths::HealthReady => "/health/ready",
      EndpointPaths::GraphQL => "/graphql",
    }
  }
}

pub fn create_router(pool: MySqlPool) -> Router {
  let schema = create_schema(pool);
  Router::new()
    .route(EndpointPaths::Root.as_str(), get(hello_read_api))
    .route(EndpointPaths::HealthAlive.as_str(), get(alive))
    .route(EndpointPaths::HealthReady.as_str(), get(ready))
    .route(EndpointPaths::GraphQL.as_str(), get(graphql).post(graphql_handler))
    .layer(Extension(schema))
}

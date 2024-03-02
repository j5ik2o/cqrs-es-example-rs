use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{response, Extension, Router};

use crate::gateways::group_chat_repository::GroupChatRepositoryImpl;

use crate::graphql::{create_schema, ApiSchema, ES};

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
      EndpointPaths::GraphQL => "/query",
    }
  }
}

async fn hello_write_api() -> &'static str {
  "Hello, Write API!"
}

pub async fn alive() -> impl IntoResponse {
  (StatusCode::OK, "OK")
}

pub async fn ready() -> impl IntoResponse {
  (StatusCode::OK, "OK")
}

/// GraphQLのリクエストを受け付けるエンドポイント。
async fn graphql_handler(schema: Extension<ApiSchema>, req: GraphQLRequest) -> GraphQLResponse {
  schema.execute(req.into_inner()).await.into()
}

/// GraphQL IDEのためのエンドポイント。
async fn graphql() -> impl IntoResponse {
  response::Html(
    GraphiQLSource::build()
      .endpoint(EndpointPaths::GraphQL.as_str())
      .finish(),
  )
}

pub fn create_router(repository: GroupChatRepositoryImpl<ES>) -> Router {
  let schema = create_schema(repository);
  Router::new()
    .route(EndpointPaths::Root.as_str(), get(hello_write_api))
    .route(EndpointPaths::HealthAlive.as_str(), get(alive))
    .route(EndpointPaths::HealthReady.as_str(), get(ready))
    .route(EndpointPaths::GraphQL.as_str(), get(graphql).post(graphql_handler))
    .layer(Extension(schema))
}

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::http::StatusCode;
use axum::routing::get_service;
use axum::{
  extract::Extension,
  response::{self, IntoResponse},
  routing::get,
  Router,
};
use sqlx::MySqlPool;
use tower_http::services::ServeDir;

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

pub enum EndpointPaths {
  Root,
  Assets,
  HealthAlive,
  HealthReady,
  GraphQL,
}

impl EndpointPaths {
  pub fn as_str(&self) -> &'static str {
    match *self {
      EndpointPaths::Root => "/",
      EndpointPaths::Assets => "/assets",
      EndpointPaths::HealthAlive => "/health/alive",
      EndpointPaths::HealthReady => "/health/ready",
      EndpointPaths::GraphQL => "/query",
    }
  }
}

/// [Router]を生成する関数。
pub fn create_router(pool: MySqlPool) -> Router {
  let schema = create_schema(pool);
  let serve_dir = ServeDir::new(&EndpointPaths::Assets.as_str()[1..]);
  let service = get_service(serve_dir);
  let r = Router::new()
    .route(EndpointPaths::Root.as_str(), get(hello_read_api))
    .route(EndpointPaths::HealthAlive.as_str(), get(alive))
    .route(EndpointPaths::HealthReady.as_str(), get(ready))
    .route(EndpointPaths::GraphQL.as_str(), get(graphql).post(graphql_handler))
    .nest_service(EndpointPaths::Assets.as_str(), service)
    .layer(Extension(schema));
  r
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::body::Body;
  use axum::http::Request;
  use axum::Router;
  use tower::ServiceExt;

  #[tokio::test]
  async fn test_root() {
    let router = Router::new().route(EndpointPaths::Root.as_str(), get(hello_read_api));

    let response = router
      .oneshot(
        Request::builder()
          .uri(EndpointPaths::Root.as_str())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"Hello, Read API!");
  }

  #[tokio::test]
  async fn test_alive() {
    let router = Router::new().route(EndpointPaths::HealthAlive.as_str(), get(alive));

    let response = router
      .oneshot(
        Request::builder()
          .uri(EndpointPaths::HealthAlive.as_str())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"OK");
  }

  #[tokio::test]
  async fn test_ready() {
    let router = Router::new().route(EndpointPaths::HealthReady.as_str(), get(ready));

    let response = router
      .oneshot(
        Request::builder()
          .uri(EndpointPaths::HealthReady.as_str())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"OK");
  }
}

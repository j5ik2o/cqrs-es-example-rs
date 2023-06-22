use async_graphql::http::GraphiQLSource;
use async_graphql::Object;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::response;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};
use hyper::Server;

struct QueryRoot;

#[Object]
impl QueryRoot {
  async fn total_photos(&self, n: u32) -> u32 {
    n
  }
}

type ApiSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

async fn graphql_handler(schema: Extension<ApiSchema>, req: GraphQLRequest) -> GraphQLResponse {
  schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
  response::Html(GraphiQLSource::build().endpoint("/").finish())
}

#[tokio::main]
async fn main() {
  let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

  let app = Router::new()
    .route("/", get(graphiql).post(graphql_handler))
    .layer(Extension(schema));

  println!("GraphiQL IDE: http://localhost:8000");

  Server::bind(&"127.0.0.1:8000".parse().unwrap())
    .serve(app.into_make_service())
    .await
    .unwrap();
}

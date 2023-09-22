use utoipa::OpenApi;
use write_api_server::ApiDoc;

fn main() {
  print!("{}", ApiDoc::openapi().to_yaml().unwrap());
}

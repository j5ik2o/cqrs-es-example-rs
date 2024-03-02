use anyhow::Result;

use command_interface_adaptor_impl::graphql::create_schema_builder;

#[tokio::main]
async fn main() -> Result<()> {
  let schema = create_schema_builder().finish();
  println!("{}", schema.sdl());
  Ok(())
}

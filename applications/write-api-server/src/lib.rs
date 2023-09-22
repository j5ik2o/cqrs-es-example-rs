use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::Modify;
use utoipa::OpenApi;

use command_interface_adaptor_impl::controllers;

#[derive(OpenApi)]
#[openapi(
paths(
controllers::create_group_chat,
controllers::delete_group_chat,
controllers::rename_group_chat,
controllers::add_member,
controllers::remove_member,
controllers::post_message,
controllers::delete_message,
),
components(
schemas(controllers::CreateGroupChatRequestBody,
controllers::DeleteGroupChatRequestBody,
controllers::RenameGroupChatRequestBody,
controllers::AddMemberRequestBody,
controllers::RemoveMemberRequestBody,
controllers::PostMessageRequestBody,
controllers::DeleteMessageRequestBody,
controllers::GroupChatCommandResponseSuccessBody,
controllers::MessageCommandResponseSuccessBody,
controllers::CommandResponseFailureBody,
)
),
modifiers(&SecurityAddon),
tags(
(name = "write-api-server", description = "Write API Server")
)
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    if let Some(components) = openapi.components.as_mut() {
      components.add_security_scheme(
        "api_key",
        SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("palupunte"))),
      )
    }
  }
}

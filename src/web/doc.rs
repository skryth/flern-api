use utoipa::{Modify, OpenApi};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};


pub struct CookieAuthModifier;

impl Modify for CookieAuthModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(schema) = openapi.components.as_mut() {
            schema.add_security_scheme("cookie", SecurityScheme::ApiKey(
                    ApiKey::Cookie(ApiKeyValue::with_description("SID", "JWT token for current user"))
            ));
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::web::routes::user::user_signup_handler, 
        crate::web::routes::user::user_signin_handler,
        crate::web::routes::user::user_list_handler,
        crate::web::routes::user::user_update_handler,
        crate::web::routes::user::user_delete_handler,
        crate::web::routes::modules::modules_list_handler,
    ),
    modifiers(&CookieAuthModifier),
)]
pub struct ApiDoc;

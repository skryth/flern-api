use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use tower_cookies::Cookies;

use crate::{
    Config, auth,
    model::{CrudRepository, ResourceTyped, entity::UserEntity},
    web::{AppState, RequestContext, context::AuthenticatedUser, error::WebError},
};

pub static AUTH_TOKEN: &str = "SID";

pub async fn extract_context_fn(
    State(state): State<AppState>,
    cookies: Cookies,
    mut req: Request,
    next: Next,
) -> Result<Response, WebError> {
    let token = match cookies.get(AUTH_TOKEN) {
        Some(token) => token,
        None => {
            req.extensions_mut().insert(RequestContext::new(None));
            return Ok(next.run(req).await);
        }
    };

    let claims = auth::process_token(token.value(), Config::get_or_init(false).await.app().jwt())
        .map_err(|e| WebError::auth_cookie_invalid(AUTH_TOKEN, e))?;

    let id = claims
        .claims
        .sub
        .parse::<uuid::Uuid>()
        .expect("Unable to parse `sub` in token");

    let role = UserEntity::find_by_id(state.pool(), &AuthenticatedUser::admin(), id)
        .await
        .map_err(|e| WebError::resource_fetch_error(UserEntity::get_resource_type(), e))?;

    match role {
        Some(role) => {
            let role = role.role();
            req.extensions_mut()
                .insert(RequestContext::new(Some(AuthenticatedUser::new(id, role))));

            Ok(next.run(req).await)
        }
        None => {
            req.extensions_mut().insert(RequestContext::new(None));
            Ok(next.run(req).await)
        }
    }
}

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post, put},
};
use chrono::Duration;
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies, cookie::SameSite};
use uuid::Uuid;

use crate::{
    auth::{self, hash_password, verify_password, UserClaims}, model::{
        check_access, entity::{UserEntity, UserEntityCreateUpdate}, CrudRepository, DatabaseError, PaginatableRepository, ResourceTyped
    }, web::{
        error::ErrorResponse, middlewares::{self, AUTH_TOKEN}, routes::PaginationQuery, AppState, AuthenticatedUser, RequestContext, UserRole, WebError, WebResult
    }, Config
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UserCreateUpdateBody {
    pub username: String,
    pub password: String,
}

pub fn routes<S>(state: AppState) -> Router<S> {
    let protected = Router::new()
        .route("/page", get(user_list_handler))
        .route("/verify", get(user_verify_handler))
        .route(
            "/{id}",
            put(user_update_handler).delete(user_delete_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            middlewares::extract_context_fn,
        ));

    Router::new()
        .route("/signup", post(user_signup_handler))
        .route("/signin", post(user_signin_handler))
        .merge(protected)
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/api/v1/account/signup",
    request_body = UserCreateUpdateBody,
    description = "Creates new user in database",
    responses(
        (status = 200, description = "User created successfully", body = UserEntity),
        (status = 409, description = "User already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "account"
)]
async fn user_signup_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<UserCreateUpdateBody>,
) -> WebResult<impl IntoResponse> {
    let admin = AuthenticatedUser::admin();
    let found = UserEntity::find_by_username(state.pool(), &admin, &payload.username)
        .await
        .map_err(|e| WebError::resource_fetch_error(UserEntity::get_resource_type(), e))?;

    if found.is_some() {
        return Err(WebError::registration_conflict());
    }

    let hash = hash_password(&payload.password).map_err(|e| WebError::server_crypt_error(e))?;
    let payload = UserEntityCreateUpdate {
        username: payload.username,
        password_hash: hash,
    };

    let created = UserEntity::create(state.pool(), &admin, payload)
        .await
        .map_err(|e| WebError::resource_fetch_error(UserEntity::get_resource_type(), e))?;

    let timestamp = (chrono::Utc::now() + Duration::days(1)).timestamp();
    let jwt_token = Config::get_or_init(false).await.app().jwt();

    let claims = UserClaims {
        sub: created.id().to_string(),
        exp: timestamp,
    };
    let token = auth::generate_token(claims, jwt_token)
        .map_err(|e| WebError::server_crypt_error(e.into()))?;
    let mut cookie = Cookie::new(AUTH_TOKEN, token);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookies.add(cookie);

    Ok((StatusCode::OK, Json(created)))
}

#[utoipa::path(
    post,
    path = "/api/v1/account/signin",
    description = "Authorizes user in the system",
    request_body = UserCreateUpdateBody,
    responses(
        (status = 200, description = "User signed in", body = UserEntity),
        (status = 401, description = "Credentials invalid", body = ErrorResponse),
        (status = 404, description = "Specified user is not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "account",
)]
async fn user_signin_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<UserCreateUpdateBody>,
) -> WebResult<impl IntoResponse> {
    let admin = AuthenticatedUser::admin();
    let found = UserEntity::find_by_username(state.pool(), &admin, &payload.username)
        .await
        .map_err(|e| WebError::resource_fetch_error(UserEntity::get_resource_type(), e))?;

    if found.is_none() {
        return Err(WebError::auth_invalid_credentials());
    }

    let found = found.unwrap();

    let is_verified = verify_password(found.hash(), &payload.password)
        .map_err(|e| WebError::server_crypt_error(e))?;

    if !is_verified {
        return Err(WebError::auth_invalid_credentials());
    }

    let timestamp = (chrono::Utc::now() + Duration::days(1)).timestamp();
    let jwt_token = Config::get_or_init(false).await.app().jwt();
    let claims = UserClaims {
        sub: found.id().to_string(),
        exp: timestamp,
    };

    let token = auth::generate_token(claims, jwt_token)
        .map_err(|e| WebError::server_crypt_error(e.into()))?;

    let mut cookie = Cookie::new(AUTH_TOKEN, token);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookies.add(cookie);

    Ok((StatusCode::OK, Json(found)))
}

async fn user_verify_handler(ctx: RequestContext) -> WebResult<impl IntoResponse> {
    let user = ctx.maybe_user();

    if user.is_none() {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/account/page",
    request_body = PaginationQuery,
    responses(
        (status = 200, description = "Returns requested page", body = crate::model::Page<UserEntity>),
        (status = 401, description = "You're not an admin to do this", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "account",
    security(
        ("cookie" = [])
    )
)]
async fn user_list_handler(
    ctx: RequestContext,
    Query(page): Query<PaginationQuery>,
    State(state): State<AppState>,
) -> WebResult<impl IntoResponse> {
    let user = ctx.user()?;
    if user.user_role() != UserRole::Admin {
        return Err(WebError::resource_forbidden(UserEntity::get_resource_type()));
    }

    let users = UserEntity::page(state.pool(), &user, page.limit, page.offset)
        .await
        .map_err(|e| WebError::resource_fetch_error(UserEntity::get_resource_type(), e))?;

    Ok((StatusCode::OK, Json(users)))
}

#[utoipa::path(
    put,
    path = "/api/v1/account/{id}",
    request_body = UserCreateUpdateBody,
    responses(
        (status = 200, description = "User updated successfully", body = UserEntity),
        (status = 401, description = "You're not authorized to do this", body = ErrorResponse),
        (status = 403, description = "You doesn't have enough permissions to do this", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "account",
    security(
        ("cookie" = [])
    )
)]
async fn user_update_handler(
    ctx: RequestContext,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UserCreateUpdateBody>,
) -> WebResult<impl IntoResponse> {
    let user = ctx.user()?;

    let found = UserEntity::find_by_id(state.pool(), &user, id)
        .await
        .map_err(|e| WebError::resource_fetch_error(UserEntity::get_resource_type(), e))?;

    if found.is_none() {
        return Err(WebError::resource_not_found(UserEntity::get_resource_type()));
    }
    let found = found.unwrap();
    check_access(state.pool(), &user, &found, user.user_id())
        .await
        .map_err(|e| {
            if let DatabaseError::Forbidden = e {
                WebError::resource_forbidden(UserEntity::get_resource_type())
            } else {
                WebError::resource_fetch_error(UserEntity::get_resource_type(), e)
            }
        })?;

    let conflict_found = UserEntity::find_by_username(state.pool(), &user, &payload.username)
        .await
        .map_err(|e| WebError::resource_fetch_error(UserEntity::get_resource_type(), e))?;

    if conflict_found.is_some() {
        return Err(WebError::registration_conflict());
    }

    let payload = UserEntityCreateUpdate {
        username: payload.username,
        password_hash: String::new(), // not in use
    };

    let updated = found
        .update(state.pool(), &user, payload)
        .await
        .map_err(|e| WebError::resource_fetch_error(UserEntity::get_resource_type(), e))?;

    Ok((StatusCode::OK, Json(updated)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/account/{id}",
    description = "Deletes specified user",
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 401, description = "You're not authorized", body = ErrorResponse),
        (status = 403, description = "You're not allowed to do this", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "account",
    security(
        ("cookie" = [])
    )
)]
async fn user_delete_handler(
    ctx: RequestContext,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> WebResult<impl IntoResponse> {
    let user = ctx.user()?;

    let found = UserEntity::find_by_id(state.pool(), &user, id)
        .await
        .map_err(|e| WebError::resource_fetch_error(UserEntity::get_resource_type(), e))?;

    if found.is_none() {
        return Err(WebError::resource_not_found(UserEntity::get_resource_type()));
    }

    let found = found.unwrap();
    check_access(state.pool(), &user, &found, user.user_id())
        .await
        .map_err(|e| {
            if let DatabaseError::Forbidden = e {
                WebError::resource_forbidden(UserEntity::get_resource_type())
            } else {
                WebError::resource_fetch_error(UserEntity::get_resource_type(), e)
            }
        })?;

    found
        .delete(state.pool(), &user)
        .await
        .map_err(|e| WebError::resource_fetch_error(UserEntity::get_resource_type(), e))?;

    Ok(StatusCode::OK)
}

use axum::{
    extract::{Path, State}, http::StatusCode, middleware, response::IntoResponse, routing::{get, post}, Json, Router
};

use crate::{
    model::{
        CrudRepository, ResourceTyped,
        entity::{
            Lesson, ProgressToken, ProgressTokenCreate, UserEntity, UserProgress, UserTaskAttempt,
        },
    },
    web::{
        AppState, AuthenticatedUser, RequestContext, WebError, WebResult,
        dto::progress::UserProgressResponse, error::ErrorResponse, middlewares,
    },
};

pub fn routes<S>(state: AppState) -> Router<S> {
    Router::new()
        .route("/{token}", get(progress_get_handler))
        .route("/share", post(progress_token_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            middlewares::extract_context_fn,
        ))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/api/v1/progress/{token}",
    description = "Get current user's progress",
    params(
        ("token" = String, Path, description = "Token recieved from /api/v1/progress/share")
    ),
    responses(
        (status = 200, description = "Progress found", body = UserProgressResponse),
        (status = 401, description = "You're not authorized to do this", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "progress",
)]
async fn progress_get_handler(
    Path(token): Path<String>,
    State(state): State<AppState>,
) -> WebResult<impl IntoResponse> {
    let admin = AuthenticatedUser::admin();

    // cleanup old tokens
    ProgressToken::cleanup_expired(state.pool(), &admin)
        .await
        .map(|n| tracing::debug!("progress_tokens: cleaned up {} expired tokens", n))
        .map_err(|e| WebError::resource_fetch_error(ProgressToken::get_resource_type(), e))?;


    // find database progress
    let progress_token = ProgressToken::find_by_token(state.pool(), &admin, &token)
        .await
        .map_err(|e| WebError::resource_fetch_error(ProgressToken::get_resource_type(), e))?;

    if progress_token.is_none() {
        return Err(WebError::resource_not_found(
            ProgressToken::get_resource_type(),
        ));
    }
    let token = progress_token.unwrap();

    // check the token is not expired
    if *token.expires_at() < chrono::Utc::now() {
        // delete token here
        token
            .delete(state.pool(), &admin)
            .await
            .map_err(|e| WebError::resource_fetch_error(ProgressToken::get_resource_type(), e))?;

        return Err(WebError::user_bad_request(
            "This token has been expired".to_string(),
        ));
    }

    // check that this user really exists
    let token_user =
        UserEntity::find_by_id(state.pool(), &admin, token.user_id())
            .await
            .map_err(|e| WebError::resource_fetch_error(UserEntity::get_resource_type(), e))?;

    if token_user.is_none() {
        return Err(WebError::resource_not_found(UserEntity::get_resource_type()));
    }
    let target_user = token_user.unwrap();
    let token_user = AuthenticatedUser::new(target_user.id(), target_user.role());

    // run all dat shit in parallel
    let (total_lessons, completed_lessons, total_answers, correct_answers) = tokio::try_join!(
        Lesson::count(state.pool(), &token_user),
        UserProgress::count_completed(state.pool(), &token_user),
        UserTaskAttempt::count(state.pool(), &token_user),
        UserTaskAttempt::count_correct(state.pool(), &token_user),
    )
    .map_err(|e| WebError::resource_fetch_error(UserProgress::get_resource_type(), e))?;

    let res = UserProgressResponse::new(
        total_lessons,
        completed_lessons,
        correct_answers,
        total_answers,
        target_user.username().to_string(),
    );

    Ok((StatusCode::OK, Json(res)))
}

#[utoipa::path(
    post,
    path = "/api/v1/progress/share",
    description = "Generate a share token for the current's user progress",
    responses(
        (status = 200, description = "Token generated", body = ProgressToken),
        (status = 401, description = "You're not authorized to do this", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "progress",
    security(
        ("cookie" = [])
    )
)]
async fn progress_token_handler(
    ctx: RequestContext,
    State(state): State<AppState>,
) -> WebResult<impl IntoResponse> {
    let user = ctx.user()?;
    let token = crate::auth::token::generate_token();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);
    let token_create = ProgressTokenCreate {
        user_id: user.user_id(),
        token,
        expires_at,
    };

    let progress = ProgressToken::create(state.pool(), &user, token_create)
        .await
        .map_err(|e| WebError::resource_fetch_error(ProgressToken::get_resource_type(), e))?;

    Ok((StatusCode::OK, Json(progress)))
}

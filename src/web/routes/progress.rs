use axum::{extract::State, http::StatusCode, middleware, response::IntoResponse, routing::get, Json, Router};

use crate::{model::{entity::{Lesson, UserProgress, UserTaskAttempt}, CrudRepository, ResourceTyped}, web::{dto::progress::UserProgressResponse, error::ErrorResponse, middlewares, AppState, RequestContext, WebError, WebResult}};

pub fn routes<S>(state: AppState) -> Router<S> {
    Router::new()
        .route("/", get(progress_get_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            middlewares::extract_context_fn,
        ))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/api/v1/progress/",
    description = "Get current user's progress",
    responses(
        (status = 200, description = "Progress found", body = UserProgressResponse),
        (status = 401, description = "You're not authorized to do this", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "progress",
    security(
        ("cookie" = [])
    )
)]
async fn progress_get_handler(
    ctx: RequestContext,
    State(state): State<AppState>,
) -> WebResult<impl IntoResponse> {
    let user = ctx.user()?;

    // run all dat shit in parallel
    let (total_lessons, completed_lessons, total_answers, correct_answers) =
        tokio::try_join!(
            Lesson::count(state.pool(), &user),
            UserProgress::count_completed(state.pool(), &user),
            UserTaskAttempt::count(state.pool(), &user),
            UserTaskAttempt::count_correct(state.pool(), &user),
        )
        .map_err(|e| WebError::resource_fetch_error(UserProgress::get_resource_type(), e))?;

    let res = UserProgressResponse::new(total_lessons, completed_lessons, correct_answers, total_answers);

    Ok((StatusCode::OK, Json(res))) 
}

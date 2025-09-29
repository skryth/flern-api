use std::{path::PathBuf, str::FromStr};

use crate::{
    model::{
        entity::{Answer, LessonTask, UserProgress, UserProgressCreate, UserTaskAttempt, UserTaskAttemptCreate}, CrudRepository, ResourceTyped
    },
    web::{
        dto::tasks::{TaskCheckRequest, TaskCheckResponse}, error::ErrorResponse, middlewares, AppState, RequestContext, WebError, WebResult
    }, Config,
};
use axum::{
    Json, Router, extract::State, http::StatusCode, middleware, response::IntoResponse,
    routing::post,
};

pub fn routes<S>(state: AppState) -> Router<S> {
    Router::new()
        .route("/check", post(tasks_check_answer_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            middlewares::extract_context_fn,
        ))
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/api/v1/tasks/check",
    description = "Check if provided answer is correct and mark according lesson as completed",
    request_body = TaskCheckRequest,
    responses(
        (status = 200, description = "Answer checked", body = TaskCheckResponse),
        (status = 404, description = "Answer not found", body = ErrorResponse),
        (status = 401, description = "You're not authorized to do this", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("cookie" = [])
    ),
    tag = "tasks"
)]
async fn tasks_check_answer_handler(
    State(state): State<AppState>,
    ctx: RequestContext,
    Json(req): Json<TaskCheckRequest>,
) -> WebResult<impl IntoResponse> {
    let user = ctx.user()?;
    let answer = Answer::find_by_id(state.pool(), &user, req.answer_id)
        .await
        .map_err(|e| WebError::resource_fetch_error(Answer::get_resource_type(), e))?;

    if answer.is_none() {
        return Err(WebError::resource_not_found(Answer::get_resource_type()));
    }
    let answer = answer.unwrap();

    let is_correct = match req.task_type.as_str() {
        "string_cmp" => {
            if req.user_answer.is_none() {
                return Err(WebError::user_bad_request(format!(
                    "invalid user_answer field passed. You should pass some value in it if you're checking string_cmp task"
                )));
            }
            let user_answer = req.user_answer.unwrap();

            answer.answer_text() == user_answer.trim()
        }
        _ => answer.is_correct(),
    };

    let task = LessonTask::find_by_id(state.pool(), &user, answer.task_id())
        .await
        .map_err(|e| WebError::resource_fetch_error(LessonTask::get_resource_type(), e))?;
    if task.is_none() {
        return Err(WebError::resource_not_found(LessonTask::get_resource_type()));
    }
    let task = task.unwrap();

    // TODO: add progress mark in db

    // Current implementation allows multiple tasks per lesson
    // but our frontend couldn't do that before deadline, so currently
    // we are building according to lesson -> task, not lesson -> task(s)[]
    // according to our decision /tasks/check route will mark lesson as done too
    if is_correct {
        UserProgress::create(
            state.pool(),
            &user,
            UserProgressCreate::new(user.user_id(), task.lesson_id(), true),
        )
        .await
        .map_err(|e| WebError::resource_fetch_error(UserProgress::get_resource_type(), e))?;
    }
    let utc = UserTaskAttemptCreate::new(user.user_id(), task.id(), answer.id(), is_correct);
    UserTaskAttempt::create(state.pool(), &user, utc)
        .await
        .map_err(|e| WebError::resource_fetch_error(UserTaskAttempt::get_resource_type(), e))?;

    // Map database path to a download URL
    let base_url = Config::get_or_init()
        .await
        .app()
        .host_url()
        .trim_end_matches('/');
    let image_path = PathBuf::from_str(answer.image()).unwrap();
    let image_url = format!("{}/api/v1/static/{}", base_url, image_path.display());

    Ok((
        StatusCode::OK,
        Json(TaskCheckResponse {
            is_correct,
            explanation: task.explanation().to_string(),
            image: image_url,
        }),
    ))
}

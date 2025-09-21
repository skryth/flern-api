use crate::{
    model::{
        CrudRepository, ResourceTyped,
        entity::{Answer, LessonTask},
    },
    web::{
        AppState, RequestContext, WebError, WebResult,
        dto::tasks::{TaskCheckRequest, TaskCheckResponse},
        error::ErrorResponse,
        middlewares,
    },
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
    description = "Check if provided answer is correct",
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

            answer.answer_text() == user_answer.to_lowercase().trim()
        }
        _ => answer.is_correct(),
    };

    // TODO: add progress mark in db

    let task = LessonTask::find_by_id(state.pool(), &user, answer.task_id())
        .await
        .map_err(|e| WebError::resource_fetch_error(LessonTask::get_resource_type(), e))?;
    if task.is_none() {
        return Err(WebError::resource_not_found(LessonTask::get_resource_type()));
    }
    let task = task.unwrap();

    Ok((
        StatusCode::OK,
        Json(TaskCheckResponse {
            is_correct,
            explanation: task.explanation().to_string(),
            image: answer.image().to_string(), // TODO: Map to URL instead of path
        }),
    ))
}

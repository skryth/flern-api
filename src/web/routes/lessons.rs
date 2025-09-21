use axum::extract::Path;
use axum::routing::post;
use axum::Json;
use axum::{extract::State, middleware, response::IntoResponse, routing::get, Router};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::model::entity::{Answer, Lesson, LessonTask, LessonWithStatusRow, UserProgress, UserProgressCreate};
use crate::model::{CrudRepository, ResourceTyped};
use crate::web::dto::lessons::LessonResponse;
use crate::web::dto::tasks::TaskResponse;
use crate::web::error::ErrorResponse;
use crate::web::{middlewares, AppState, RequestContext, WebError, WebResult};

pub fn routes<S>(state: AppState) -> Router<S> {
    Router::new()
        .route("/{id}", get(lessons_get_handler))
        .route("/{id}/done", post(lessons_mark_done_handler))
        .route("/{id}/tasks", get(lessons_get_tasks_handler))
        .route("/{id}/next", get(lessons_get_next_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            middlewares::extract_context_fn,
        ))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/api/v1/lessons/{lesson_id}",
    description = "Fetch comprehensive info about lesson including its content",
    params(
        ("lesson_id" = Uuid, Path, description = "ID of the lesson to get")
    ),
    responses(
        (status = 200, description = "Lesson found", body = LessonResponse),
        (status = 404, description = "Lesson not found", body = ErrorResponse),
        (status = 401, description = "You're not authorized to do this", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("cookie" = [])
    ),
    tag = "lessons"
)]
async fn lessons_get_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ctx: RequestContext,
) -> WebResult<impl IntoResponse> {
    let user = ctx.user()?; 
    let lesson = LessonWithStatusRow::find_by_id(state.pool(), &user, id)
        .await
        .map(LessonResponse::from)
        .map_err(|e| WebError::resource_fetch_error(Lesson::get_resource_type(), e))?;

    Ok((StatusCode::OK, Json(lesson)))
}

#[utoipa::path(
    post,
    path = "/api/v1/lessons/{lesson_id}/done",
    description = "Mark lesson as done",
    params(
        ("lesson_id" = Uuid, Path, description = "ID of the lesson to mark")
    ),
    responses(
        (status = 200, description = "Lesson marked"),
        (status = 404, description = "Lesson not found", body = ErrorResponse),
        (status = 401, description = "You're not allowed to do this", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("cookie" = [])
    ),
    tag = "lessons"
)]
async fn lessons_mark_done_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ctx: RequestContext,
) -> WebResult<impl IntoResponse> {
    let user = ctx.user()?;
    let exists = Lesson::find_by_id(state.pool(), &user, id)
        .await
        .map_err(|e| WebError::resource_fetch_error(Lesson::get_resource_type(), e))?
        .is_some();

    if !exists {
        return Err(WebError::resource_not_found(Lesson::get_resource_type()));
    }

    UserProgress::create(state.pool(), &user, UserProgressCreate::new(
        user.user_id(),
        id,
        true
    ))
    .await
    .map_err(|e| WebError::resource_fetch_error(crate::model::ResourceType::UserProgress, e))?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/lessons/{lesson_id}/tasks",
    description = "Get all tasks for this lesson",
    params(
        ("lesson_id" = Uuid, Path, description = "ID of the lesson to get tasks for")
    ),
    responses(
        (status = 200, description = "Tasks found", body = Vec<TaskResponse>),
        (status = 404, description = "Lesson not found", body = ErrorResponse),
        (status = 401, description = "You're not authorized to do this", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("cookie" = [])
    ),
    tag = "tasks"
)]
async fn lessons_get_tasks_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ctx: RequestContext,
) -> WebResult<impl IntoResponse> {
    let user = ctx.user()?;
    let exists = Lesson::find_by_id(state.pool(), &user, id)
        .await
        .map_err(|e| WebError::resource_fetch_error(LessonTask::get_resource_type(), e))?
        .is_some();

    if !exists {
        return Err(WebError::resource_not_found(LessonTask::get_resource_type()));
    }

    let tasks = LessonTask::find_all_by_lesson(state.pool(), &user, id)
        .await
        .map_err(|e| WebError::resource_fetch_error(LessonTask::get_resource_type(), e))?;

    // fetch all answers
    let mut responses = Vec::new();
    for task in tasks {
        let answers = Answer::find_all_by_task(state.pool(), &user, task.id())
            .await
            .map_err(|e| WebError::resource_fetch_error(Answer::get_resource_type(), e))?;

        let response = TaskResponse::from_entity(task, answers);
        responses.push(response);
    }

    Ok((StatusCode::OK, Json(responses)))
} 

#[utoipa::path(
    get,
    path = "/api/v1/lessons/{lesson_id}/next",
    description = "Returns next in order uncompleted lesson",
    params(
        ("lesson_id" = Uuid, Path, description = "ID of the current lesson")
    ),
    responses(
        (status = 200, description = "Found next lesson", body = LessonResponse),
        (status = 404, description = "Next lesson not found", body = ErrorResponse),
        (status = 401, description = "You're not authorized to do this", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(
        ("cookie" = [])
    ),
    tag = "lessons"
)]
async fn lessons_get_next_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ctx: RequestContext,
) -> WebResult<impl IntoResponse> {
    let user = ctx.user()?;
    let next = LessonWithStatusRow::find_next_uncompleted(state.pool(), &user, id)
        .await
        .map_err(|e| WebError::resource_fetch_error(Lesson::get_resource_type(), e))?;


    if let Some(next) = next {
        Ok((StatusCode::OK, Json(LessonResponse::from(next))))
    } else {
        Err(WebError::resource_not_found(Lesson::get_resource_type()))
    }
}

use axum::extract::Path;
use axum::routing::post;
use axum::Json;
use axum::{extract::State, middleware, response::IntoResponse, routing::get, Router};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::model::entity::{Lesson, LessonWithStatusRow, UserProgress, UserProgressCreate};
use crate::model::{CrudRepository, ResourceTyped};
use crate::web::dto::lessons::LessonResponse;
use crate::web::error::ErrorResponse;
use crate::web::{middlewares, AppState, RequestContext, WebError, WebResult};

pub fn routes<S>(state: AppState) -> Router<S> {
    Router::new()
        .route("/{id}", get(lessons_get_handler))
        .route("/{id}/done", post(lessons_mark_done_handler))
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

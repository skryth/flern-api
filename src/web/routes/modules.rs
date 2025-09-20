use axum::{
    Json, Router, extract::State, http::StatusCode, middleware, response::IntoResponse,
    routing::get,
};

use crate::{
    model::{
        entity::{Module, ModuleWithLessons}, ResourceTyped
    },
    web::{error::ErrorResponse, middlewares, AppState, RequestContext, WebError, WebResult},
};

pub fn routes<S>(state: AppState) -> Router<S> {
    Router::new()
        .route("/", get(modules_list_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            middlewares::extract_context_fn,
        ))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/api/v1/modules/",
    description = "List ALL modules objects with lessons. See success response body",
    responses(
        (status = 200, description = "Successfully collected modules", body = Vec<ModuleWithLessons>),
        (status = 401, description = "You had to be authorized to do this", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    tag = "modules",
    security(
        ("cookie" = [])
    )
)]
async fn modules_list_handler(
    ctx: RequestContext,
    State(state): State<AppState>,
) -> WebResult<impl IntoResponse> {
    let user = ctx.user()?;
    let modules = ModuleWithLessons::fetch_all(state.pool(), &user)
        .await
        .map_err(|e| WebError::resource_fetch_error(Module::get_resource_type(), e))?;

    Ok((StatusCode::OK, Json(modules)))
}

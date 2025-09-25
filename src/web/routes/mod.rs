use crate::{web::{doc::ApiDoc, AppState}, Config};
use axum::Router;
use serde::Deserialize;
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::CorsLayer, services::ServeDir};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod user;
pub mod modules;
pub mod lessons;
pub mod tasks;

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct PaginationQuery {
    limit: i64,
    offset: i64,
}

pub fn build_app<S: Send + Sync + Clone + 'static>(state: AppState, config: &'static Config) -> Router<S> {
    let mut router = Router::new()
        .nest("/api/v1/account/", user::routes(state.clone()))
        .nest("/api/v1/modules/", modules::routes(state.clone()))
        .nest("/api/v1/lessons/", lessons::routes(state.clone()))
        .nest("/api/v1/tasks/", tasks::routes(state.clone()))
        .nest_service("/api/v1/static/", ServeDir::new("uploads"))
        .layer(CookieManagerLayer::default())
        .layer(CorsLayer::very_permissive())
        .with_state(state);

    if config.app().docs() {
        let openapi = ApiDoc::openapi();

        router = router
            .merge(
                SwaggerUi::new("/api/v1/docs")
                    .url("/api-doc/openapi.json", openapi),
            );
    }

    router
}

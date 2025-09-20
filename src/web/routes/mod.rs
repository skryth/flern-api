use crate::web::{doc::ApiDoc, AppState};
use axum::Router;
use serde::Deserialize;
use tower_cookies::CookieManagerLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod user;
pub mod modules;

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct PaginationQuery {
    limit: i64,
    offset: i64,
}

pub fn build_app<S: Send + Sync + Clone + 'static>(state: AppState) -> Router<S> {
    let mut router = Router::new()
        .nest("/api/v1/account/", user::routes(state.clone()))
        .nest("/api/v1/modules/", modules::routes(state.clone()))
        .layer(CookieManagerLayer::default())
        .with_state(state);

    if cfg!(debug_assertions) {
        let openapi = ApiDoc::openapi();

        router = router
            .merge(
                SwaggerUi::new("/api/v1/docs")
                    .url("/api-doc/openapi.json", openapi),
            );
    }

    router
}

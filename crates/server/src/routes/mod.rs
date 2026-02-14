mod ai;
mod project;
mod script;
mod story;
mod timeline;

use axum::Router;

use crate::state::AppState;

/// Build the `/api` router with all sub-routes.
pub fn api_router() -> Router<AppState> {
    Router::new()
        .merge(project::router())
        .merge(story::router())
        .merge(timeline::router())
        .merge(script::router())
        .merge(ai::router())
}

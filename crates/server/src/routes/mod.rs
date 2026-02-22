mod ai;
mod diffusion;
mod export;
mod models;
mod project;
mod reference;
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
        .merge(diffusion::router())
        .merge(export::router())
        .merge(reference::router())
        .merge(models::router())
}

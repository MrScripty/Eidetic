mod ai;
mod commands;
mod diffusion;
mod export;
mod models;
mod project;
mod projections;
mod reference;
mod script;
mod story;
mod support;
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
        .merge(commands::router())
        .merge(projections::router())
        .merge(diffusion::router())
        .merge(export::router())
        .merge(reference::router())
        .merge(models::router())
}

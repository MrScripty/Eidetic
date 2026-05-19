mod ai;
mod commands;
mod commands_semantic;
mod commands_timeline;
mod diffusion;
mod export;
mod models;
mod project;
mod projections;
mod projections_semantic;
mod reference;
mod script;
mod support;
mod timeline;

use axum::Router;

use crate::state::AppState;

/// Build the `/api` router with all sub-routes.
pub fn api_router() -> Router<AppState> {
    Router::new()
        .merge(project::router())
        .merge(timeline::router())
        .merge(script::router())
        .merge(ai::router())
        .merge(commands::router())
        .merge(commands_semantic::router())
        .merge(projections::router())
        .merge(projections_semantic::router())
        .merge(diffusion::router())
        .merge(export::router())
        .merge(reference::router())
        .merge(models::router())
}

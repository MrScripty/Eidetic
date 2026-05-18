use axum::Router;
use axum::extract::{Path, State};
use axum::routing::get;
use uuid::Uuid;

use eidetic_core::timeline::node::NodeId;

use crate::error::{ApiError, ApiJson, json_value};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/nodes/{id}/content", get(get_node_content))
}

async fn get_node_content(State(state): State<AppState>, Path(id): Path<Uuid>) -> ApiJson {
    let guard = state.project.lock();
    let Some(project) = guard.as_ref() else {
        return Err(ApiError::no_project());
    };

    match project.timeline.node(NodeId(id)) {
        Ok(node) => json_value(&node.content),
        Err(e) => Err(ApiError::bad_request(e.to_string())),
    }
}

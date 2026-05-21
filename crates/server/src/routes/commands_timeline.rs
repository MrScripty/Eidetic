use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use eidetic_core::contracts::{
    CommandEnvelope, DeleteTimelineNodeCommand, DeleteTimelineRelationshipCommand,
    SetTimelineNodeLockCommand, SetTimelineNodeNotesCommand, SetTimelineNodeRangeCommand,
};

use crate::error::{ApiError, ApiJson};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/commands/timeline/node-range",
            post(set_timeline_node_range),
        )
        .route("/commands/timeline/create-node", post(create_timeline_node))
        .route(
            "/commands/timeline/create-relationship",
            post(create_timeline_relationship),
        )
        .route(
            "/commands/timeline/delete-relationship",
            post(delete_timeline_relationship),
        )
        .route(
            "/commands/timeline/apply-children",
            post(apply_timeline_children),
        )
        .route("/commands/timeline/split-node", post(split_timeline_node))
        .route("/commands/timeline/node-lock", post(set_timeline_node_lock))
        .route(
            "/commands/timeline/node-notes",
            post(set_timeline_node_notes),
        )
        .route("/commands/timeline/delete-node", post(delete_timeline_node))
}

async fn set_timeline_node_range(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetTimelineNodeRangeCommand>>,
) -> ApiJson {
    let response = crate::command_service::set_timeline_node_range(&state, command)
        .await
        .map_err(ApiError::from)?;
    crate::error::json_value(response)
}

async fn create_timeline_node(
    State(state): State<AppState>,
    Json(command): Json<crate::command_service::CreateTimelineNodeRequestCommand>,
) -> ApiJson {
    let response = crate::command_service::create_timeline_node(&state, command)
        .await
        .map_err(ApiError::from)?;
    crate::error::json_value(response)
}

async fn apply_timeline_children(
    State(state): State<AppState>,
    Json(command): Json<crate::command_service::ApplyTimelineChildrenRequestCommand>,
) -> ApiJson {
    let response = crate::command_service::apply_timeline_children(&state, command)
        .await
        .map_err(ApiError::from)?;
    crate::error::json_value(response)
}

async fn create_timeline_relationship(
    State(state): State<AppState>,
    Json(command): Json<crate::command_service::CreateTimelineRelationshipRequestCommand>,
) -> ApiJson {
    let response = crate::command_service::create_timeline_relationship(&state, command)
        .await
        .map_err(ApiError::from)?;
    crate::error::json_value(response)
}

async fn delete_timeline_relationship(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<DeleteTimelineRelationshipCommand>>,
) -> ApiJson {
    let response = crate::command_service::delete_timeline_relationship(&state, command)
        .await
        .map_err(ApiError::from)?;
    crate::error::json_value(response)
}

async fn split_timeline_node(
    State(state): State<AppState>,
    Json(command): Json<crate::command_service::SplitTimelineNodeRequestCommand>,
) -> ApiJson {
    crate::command_service::split_timeline_node(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn set_timeline_node_lock(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetTimelineNodeLockCommand>>,
) -> ApiJson {
    crate::command_service::set_timeline_node_lock(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn set_timeline_node_notes(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetTimelineNodeNotesCommand>>,
) -> ApiJson {
    crate::command_service::set_timeline_node_notes(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn delete_timeline_node(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<DeleteTimelineNodeCommand>>,
) -> ApiJson {
    crate::command_service::delete_timeline_node(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

#[cfg(test)]
#[path = "commands_timeline_range_tests.rs"]
mod range_tests;

#[cfg(test)]
#[path = "commands_timeline_create_tests.rs"]
mod create_tests;

#[cfg(test)]
#[path = "commands_timeline_children_tests.rs"]
mod children_tests;

#[cfg(test)]
#[path = "commands_timeline_relationship_tests.rs"]
mod relationship_tests;

#[cfg(test)]
#[path = "commands_timeline_split_tests.rs"]
mod split_tests;

#[cfg(test)]
#[path = "commands_timeline_state_tests.rs"]
mod state_tests;

#[cfg(test)]
#[path = "commands_timeline_delete_tests.rs"]
mod delete_tests;

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use eidetic_core::contracts::{
    ApplyTimelineChildrenCommand, BibleNodeDetailProjection, CommandEnvelope,
    CreateBibleGraphNodeCommand, CreateTimelineNodeCommand, DeleteTimelineNodeCommand,
    ProjectionEnvelope, ScriptDocumentProjection, SetBibleGraphEdgeCommand,
    SetBibleGraphFieldCommand, SetBibleGraphSnapshotFieldCommand, SetObjectFieldCommand,
    SetScriptBlockCommand, SetScriptLockCommand, SetTimelineNodeRangeCommand,
    SplitTimelineNodeCommand, TimelineRenderProjection,
};
use eidetic_core::contracts::{BibleGraphNodeListProjection, EnsureCanonicalBibleRootsCommand};
use serde::Serialize;

use crate::bible_graph_command::{self, BibleGraphCommandError};
use crate::error::{ApiError, ApiJson};
use crate::history_store::{self, RecordChangeOutcome};
use crate::object_field_command::{self, ObjectFieldCommandError};
use crate::revision_projection::ObjectFieldProjection;
use crate::script_document_command::{self, ScriptDocumentCommandError};
use crate::state::{AppState, ServerEvent};
use crate::timeline_command::{self, TimelineCommandError};
use crate::validation;
use crate::ydoc::DocCommand;

use super::support::{active_project_path, map_history_error};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/commands/object-field", post(set_object_field))
        .route("/commands/bible-graph/node", post(create_bible_graph_node))
        .route("/commands/bible-graph/field", post(set_bible_graph_field))
        .route("/commands/bible-graph/edge", post(set_bible_graph_edge))
        .route(
            "/commands/bible-graph/snapshot-field",
            post(set_bible_graph_snapshot_field),
        )
        .route(
            "/commands/bible-graph/canonical-roots",
            post(ensure_canonical_bible_roots),
        )
        .route("/commands/script/block", post(set_script_block))
        .route("/commands/script/lock", post(set_script_lock))
        .route(
            "/commands/timeline/node-range",
            post(set_timeline_node_range),
        )
        .route("/commands/timeline/create-node", post(create_timeline_node))
        .route(
            "/commands/timeline/apply-children",
            post(apply_timeline_children),
        )
        .route("/commands/timeline/split-node", post(split_timeline_node))
        .route("/commands/timeline/delete-node", post(delete_timeline_node))
}

#[derive(Debug, Serialize)]
struct ObjectFieldCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<ObjectFieldProjection>,
}

#[derive(Debug, Serialize)]
struct BibleGraphNodeCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<BibleNodeDetailProjection>,
}

#[derive(Debug, Serialize)]
struct BibleGraphRootsCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<BibleGraphNodeListProjection>,
}

#[derive(Debug, Serialize)]
struct ScriptDocumentCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<ScriptDocumentProjection>,
}

#[derive(Debug, Serialize)]
struct TimelineCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<TimelineRenderProjection>,
}

async fn set_object_field(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetObjectFieldCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = tokio::task::spawn_blocking(move || apply_command_at_path(path, command))
        .await
        .map_err(|e| ApiError::internal(format!("object field command task failed: {e}")))??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    crate::error::json_value(response)
}

async fn create_bible_graph_node(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<CreateBibleGraphNodeCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = tokio::task::spawn_blocking(move || create_bible_node_at_path(path, command))
        .await
        .map_err(|e| ApiError::internal(format!("bible graph command task failed: {e}")))??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    crate::error::json_value(response)
}

async fn set_bible_graph_field(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetBibleGraphFieldCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response =
        tokio::task::spawn_blocking(move || set_bible_graph_field_at_path(path, command))
            .await
            .map_err(|e| ApiError::internal(format!("bible graph field task failed: {e}")))??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    crate::error::json_value(response)
}

async fn set_bible_graph_edge(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetBibleGraphEdgeCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = tokio::task::spawn_blocking(move || set_bible_graph_edge_at_path(path, command))
        .await
        .map_err(|e| ApiError::internal(format!("bible graph edge task failed: {e}")))??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    crate::error::json_value(response)
}

async fn set_bible_graph_snapshot_field(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetBibleGraphSnapshotFieldCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response =
        tokio::task::spawn_blocking(move || set_bible_graph_snapshot_field_at_path(path, command))
            .await
            .map_err(|e| {
                ApiError::internal(format!("bible graph snapshot field task failed: {e}"))
            })??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    crate::error::json_value(response)
}

async fn ensure_canonical_bible_roots(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<EnsureCanonicalBibleRootsCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = tokio::task::spawn_blocking(move || ensure_roots_at_path(path, command))
        .await
        .map_err(|e| ApiError::internal(format!("bible graph roots task failed: {e}")))??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    crate::error::json_value(response)
}

async fn set_script_block(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetScriptBlockCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = tokio::task::spawn_blocking(move || set_script_block_at_path(path, command))
        .await
        .map_err(|e| ApiError::internal(format!("script block command task failed: {e}")))??;

    let _ = state.events_tx.send(ServerEvent::ScriptChanged);
    crate::error::json_value(response)
}

async fn set_script_lock(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetScriptLockCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = tokio::task::spawn_blocking(move || set_script_lock_at_path(path, command))
        .await
        .map_err(|e| ApiError::internal(format!("script lock command task failed: {e}")))??;

    let _ = state.events_tx.send(ServerEvent::ScriptChanged);
    crate::error::json_value(response)
}

async fn set_timeline_node_range(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetTimelineNodeRangeCommand>>,
) -> ApiJson {
    let response = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        let projection = timeline_command::apply_set_timeline_node_range(project, &command)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome: RecordChangeOutcome::Recorded,
            projection,
        }
    };

    let _ = state.events_tx.send(ServerEvent::TimelineChanged);
    state.trigger_save();
    crate::error::json_value(response)
}

async fn create_timeline_node(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<CreateTimelineNodeCommand>>,
) -> ApiJson {
    validation::validate_name(&command.payload.name, "node name")?;
    let created_node_id = command.payload.node_id;
    let response = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        let projection = timeline_command::apply_create_timeline_node(project, &command)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome: RecordChangeOutcome::Recorded,
            projection,
        }
    };

    let _ = state.doc_tx.try_send(DocCommand::EnsureNode {
        node_id: created_node_id,
    });
    let _ = state.events_tx.send(ServerEvent::TimelineChanged);
    let _ = state.events_tx.send(ServerEvent::HierarchyChanged);
    state.trigger_save();
    crate::error::json_value(response)
}

async fn apply_timeline_children(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<ApplyTimelineChildrenCommand>>,
) -> ApiJson {
    for child in &command.payload.children {
        validation::validate_name(&child.name, "child node name")?;
    }
    let children = command.payload.children.clone();
    let response = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        let projection = timeline_command::apply_timeline_children(project, &command)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome: RecordChangeOutcome::Recorded,
            projection,
        }
    };

    for child in children {
        let _ = state.doc_tx.try_send(DocCommand::EnsureNode {
            node_id: child.node_id,
        });
        if !child.outline.is_empty() {
            let _ = state.doc_tx.try_send(DocCommand::WriteNodeContent {
                node_id: child.node_id,
                field: crate::ydoc::ContentField::Notes,
                text: child.outline,
                author: "human:command".into(),
            });
        }
    }
    let _ = state.events_tx.send(ServerEvent::TimelineChanged);
    let _ = state.events_tx.send(ServerEvent::HierarchyChanged);
    state.trigger_save();
    crate::error::json_value(response)
}

async fn split_timeline_node(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SplitTimelineNodeCommand>>,
) -> ApiJson {
    let response = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        let projection = timeline_command::apply_split_timeline_node(project, &command)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome: RecordChangeOutcome::Recorded,
            projection,
        }
    };

    let _ = state.events_tx.send(ServerEvent::TimelineChanged);
    let _ = state.events_tx.send(ServerEvent::HierarchyChanged);
    state.trigger_save();
    crate::error::json_value(response)
}

async fn delete_timeline_node(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<DeleteTimelineNodeCommand>>,
) -> ApiJson {
    let removed_node_id = command.payload.node_id;
    let response = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        let projection = timeline_command::apply_delete_timeline_node(project, &command)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome: RecordChangeOutcome::Recorded,
            projection,
        }
    };

    let _ = state.doc_tx.try_send(DocCommand::RemoveNode {
        node_id: removed_node_id,
    });
    let _ = state.events_tx.send(ServerEvent::TimelineChanged);
    let _ = state.events_tx.send(ServerEvent::HierarchyChanged);
    state.trigger_save();
    crate::error::json_value(response)
}

fn apply_command_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<SetObjectFieldCommand>,
) -> Result<ObjectFieldCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    history_store::create_schema(&conn).map_err(map_history_error)?;
    let (outcome, projection) =
        object_field_command::apply_set_object_field(&mut conn, &command, 0)
            .map_err(map_object_field_error)?;
    let object_kind = projection.object_kind.clone();
    let object_id = projection.object_id.clone();
    let projection = crate::revision_projection::load_object_field_projection_envelope(
        &conn,
        object_kind,
        &object_id,
    )
    .map_err(map_history_error)?;

    Ok(ObjectFieldCommandResponse {
        outcome,
        projection,
    })
}

fn create_bible_node_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<CreateBibleGraphNodeCommand>,
) -> Result<BibleGraphNodeCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let (outcome, projection) =
        bible_graph_command::apply_create_bible_graph_node(&mut conn, &command, 0)
            .map_err(map_bible_graph_error)?;

    Ok(BibleGraphNodeCommandResponse {
        outcome,
        projection,
    })
}

fn set_bible_graph_field_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<SetBibleGraphFieldCommand>,
) -> Result<BibleGraphNodeCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let (outcome, projection) =
        bible_graph_command::apply_set_bible_graph_field(&mut conn, &command, 0)
            .map_err(map_bible_graph_error)?;

    Ok(BibleGraphNodeCommandResponse {
        outcome,
        projection,
    })
}

fn set_bible_graph_edge_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<SetBibleGraphEdgeCommand>,
) -> Result<BibleGraphNodeCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let (outcome, projection) =
        bible_graph_command::apply_set_bible_graph_edge(&mut conn, &command, 0)
            .map_err(map_bible_graph_error)?;

    Ok(BibleGraphNodeCommandResponse {
        outcome,
        projection,
    })
}

fn set_bible_graph_snapshot_field_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<SetBibleGraphSnapshotFieldCommand>,
) -> Result<BibleGraphNodeCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let (outcome, projection) =
        bible_graph_command::apply_set_bible_graph_snapshot_field(&mut conn, &command, 0)
            .map_err(map_bible_graph_error)?;

    Ok(BibleGraphNodeCommandResponse {
        outcome,
        projection,
    })
}

fn ensure_roots_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<EnsureCanonicalBibleRootsCommand>,
) -> Result<BibleGraphRootsCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let (outcome, projection) =
        bible_graph_command::apply_ensure_canonical_bible_roots(&mut conn, &command, 0)
            .map_err(map_bible_graph_error)?;

    Ok(BibleGraphRootsCommandResponse {
        outcome,
        projection,
    })
}

fn set_script_block_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<SetScriptBlockCommand>,
) -> Result<ScriptDocumentCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let (outcome, projection) =
        script_document_command::apply_set_script_block(&mut conn, &command, 0)
            .map_err(map_script_document_error)?;

    Ok(ScriptDocumentCommandResponse {
        outcome,
        projection,
    })
}

fn set_script_lock_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<SetScriptLockCommand>,
) -> Result<ScriptDocumentCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let (outcome, projection) =
        script_document_command::apply_set_script_lock(&mut conn, &command, 0)
            .map_err(map_script_document_error)?;

    Ok(ScriptDocumentCommandResponse {
        outcome,
        projection,
    })
}

fn map_object_field_error(error: ObjectFieldCommandError) -> ApiError {
    match error {
        ObjectFieldCommandError::InvalidCommand(message) => ApiError::bad_request(message),
        ObjectFieldCommandError::History(error) => map_history_error(error),
    }
}

fn map_bible_graph_error(error: BibleGraphCommandError) -> ApiError {
    match error {
        BibleGraphCommandError::InvalidCommand(message) => ApiError::bad_request(message),
        BibleGraphCommandError::Store(error) => map_history_error(error),
    }
}

fn map_script_document_error(error: ScriptDocumentCommandError) -> ApiError {
    match error {
        ScriptDocumentCommandError::InvalidCommand(message) => ApiError::bad_request(message),
        ScriptDocumentCommandError::Store(error) => map_history_error(error),
    }
}

fn map_timeline_command_error(error: TimelineCommandError) -> ApiError {
    match error {
        TimelineCommandError::Core(error) => ApiError::bad_request(error.to_string()),
    }
}

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;

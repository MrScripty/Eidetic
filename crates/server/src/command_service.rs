use eidetic_core::contracts::{
    CommandEnvelope, CommandId, CreateStoryArcCommand, DeleteStoryArcCommand, ProjectionEnvelope,
    ScriptDocumentProjection, SetObjectFieldCommand, SetScriptBlockCommand, SetScriptLockCommand,
    SetStoryArcMetadataCommand, StoryArcListProjection,
};
use eidetic_core::story::arc::ArcId;
use serde::{Deserialize, Serialize};

use crate::backend_error::BackendError;
use crate::command_service_support::{
    active_project_path, derived_command_uuid, map_history_error,
};
use crate::history_store::{self, RecordChangeOutcome};
use crate::object_field_command::{self, ObjectFieldCommandError};
use crate::revision_projection::ObjectFieldProjection;
use crate::script_document_command::{self, ScriptDocumentCommandError};
use crate::state::{AppState, ServerEvent};
use crate::story_arc_command::{self, StoryArcCommandError};
use crate::story_arc_store;

pub use crate::command_service_bible::{
    BibleGraphNodeCommandResponse, BibleGraphRootsCommandResponse,
    CreateBibleGraphNodeRequestCommand, SetBibleGraphEdgeRequestCommand,
    SetBibleGraphSnapshotFieldRequestCommand, create_bible_graph_node,
    ensure_canonical_bible_roots, set_bible_graph_edge, set_bible_graph_field,
    set_bible_graph_snapshot_field,
};
pub use crate::command_service_semantic::{
    BibleReferenceProposalCommandResponse, PropagationProposalCommandResponse,
    accept_bible_reference_proposal, accept_propagation_proposal, create_bible_reference_proposal,
    create_propagation_proposal, reject_bible_reference_proposal, reject_propagation_proposal,
    update_propagation_proposal,
};
pub use crate::command_service_timeline::{
    CreateTimelineNodeRequestCommand, CreateTimelineRelationshipRequestCommand,
    SplitTimelineNodeRequestCommand, TimelineCommandResponse, create_timeline_node,
    create_timeline_relationship, delete_timeline_node, delete_timeline_relationship,
    set_timeline_node_lock, set_timeline_node_notes, set_timeline_node_range, split_timeline_node,
};

#[derive(Debug, Serialize)]
pub struct ObjectFieldCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<ObjectFieldProjection>,
}

#[derive(Debug, Serialize)]
pub struct ScriptDocumentCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<ScriptDocumentProjection>,
}

#[derive(Debug, Serialize)]
pub struct StoryArcCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<StoryArcListProjection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateStoryArcRequestCommand {
    id: CommandId,
    payload: CreateStoryArcRequestPayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateStoryArcRequestPayload {
    #[serde(default)]
    arc_id: Option<ArcId>,
    #[serde(default)]
    parent_arc_id: Option<ArcId>,
    name: String,
    #[serde(default)]
    description: String,
    arc_type: eidetic_core::story::arc::ArcType,
    color: eidetic_core::story::arc::Color,
}

impl CreateStoryArcRequestCommand {
    fn into_core_command(self) -> CommandEnvelope<CreateStoryArcCommand> {
        CommandEnvelope {
            id: self.id,
            payload: CreateStoryArcCommand {
                arc_id: self
                    .payload
                    .arc_id
                    .unwrap_or_else(|| ArcId(derived_command_uuid(self.id, b"story.arc"))),
                parent_arc_id: self.payload.parent_arc_id,
                name: self.payload.name,
                description: self.payload.description,
                arc_type: self.payload.arc_type,
                color: self.payload.color,
            },
        }
    }
}

pub async fn set_object_field(
    state: &AppState,
    command: CommandEnvelope<SetObjectFieldCommand>,
) -> Result<ObjectFieldCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response = tokio::task::spawn_blocking(move || apply_object_field_at_path(path, command))
        .await
        .map_err(|error| {
            BackendError::internal(format!("object field command task failed: {error}"))
        })??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    Ok(response)
}

pub async fn set_script_block(
    state: &AppState,
    command: CommandEnvelope<SetScriptBlockCommand>,
) -> Result<ScriptDocumentCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response = tokio::task::spawn_blocking(move || set_script_block_at_path(path, command))
        .await
        .map_err(|error| {
            BackendError::internal(format!("script block command task failed: {error}"))
        })??;

    let _ = state.events_tx.send(ServerEvent::ScriptChanged);
    Ok(response)
}

pub async fn set_script_lock(
    state: &AppState,
    command: CommandEnvelope<SetScriptLockCommand>,
) -> Result<ScriptDocumentCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response = tokio::task::spawn_blocking(move || set_script_lock_at_path(path, command))
        .await
        .map_err(|error| {
            BackendError::internal(format!("script lock command task failed: {error}"))
        })??;

    let _ = state.events_tx.send(ServerEvent::ScriptChanged);
    Ok(response)
}

pub async fn create_story_arc(
    state: &AppState,
    command: CreateStoryArcRequestCommand,
) -> Result<StoryArcCommandResponse, BackendError> {
    let command = command.into_core_command();
    let path = active_project_path(state)?;
    let response = tokio::task::spawn_blocking(move || create_story_arc_at_path(path, command))
        .await
        .map_err(|error| {
            BackendError::internal(format!("story arc create task failed: {error}"))
        })??;

    send_story_changed(state, response.outcome);
    Ok(response)
}

pub async fn update_story_arc(
    state: &AppState,
    command: CommandEnvelope<SetStoryArcMetadataCommand>,
) -> Result<StoryArcCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response = tokio::task::spawn_blocking(move || update_story_arc_at_path(path, command))
        .await
        .map_err(|error| {
            BackendError::internal(format!("story arc update task failed: {error}"))
        })??;

    send_story_changed(state, response.outcome);
    Ok(response)
}

pub async fn delete_story_arc(
    state: &AppState,
    command: CommandEnvelope<DeleteStoryArcCommand>,
) -> Result<StoryArcCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response = tokio::task::spawn_blocking(move || delete_story_arc_at_path(path, command))
        .await
        .map_err(|error| {
            BackendError::internal(format!("story arc delete task failed: {error}"))
        })??;

    send_story_changed(state, response.outcome);
    Ok(response)
}

fn create_story_arc_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<CreateStoryArcCommand>,
) -> Result<StoryArcCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    story_arc_store::create_schema(&conn).map_err(map_history_error)?;
    let outcome = story_arc_command::record_create_story_arc_history(&mut conn, &command, 0)
        .map_err(map_story_arc_command_error)?;
    story_arc_response(conn, outcome)
}

fn update_story_arc_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<SetStoryArcMetadataCommand>,
) -> Result<StoryArcCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    story_arc_store::create_schema(&conn).map_err(map_history_error)?;
    let outcome = story_arc_command::record_set_story_arc_metadata_history(&mut conn, &command, 0)
        .map_err(map_story_arc_command_error)?;
    story_arc_response(conn, outcome)
}

fn delete_story_arc_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<DeleteStoryArcCommand>,
) -> Result<StoryArcCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    story_arc_store::create_schema(&conn).map_err(map_history_error)?;
    let outcome = story_arc_command::record_delete_story_arc_history(&mut conn, &command, 0)
        .map_err(map_story_arc_command_error)?;
    story_arc_response(conn, outcome)
}

fn story_arc_response(
    conn: rusqlite::Connection,
    outcome: RecordChangeOutcome,
) -> Result<StoryArcCommandResponse, BackendError> {
    let projection =
        story_arc_store::load_arc_list_projection_envelope(&conn).map_err(map_history_error)?;
    Ok(StoryArcCommandResponse {
        outcome,
        projection,
    })
}

fn send_story_changed(state: &AppState, outcome: RecordChangeOutcome) {
    if outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
    }
}

fn apply_object_field_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<SetObjectFieldCommand>,
) -> Result<ObjectFieldCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
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

fn set_script_block_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<SetScriptBlockCommand>,
) -> Result<ScriptDocumentCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
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
) -> Result<ScriptDocumentCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let (outcome, projection) =
        script_document_command::apply_set_script_lock(&mut conn, &command, 0)
            .map_err(map_script_document_error)?;

    Ok(ScriptDocumentCommandResponse {
        outcome,
        projection,
    })
}

fn map_object_field_error(error: ObjectFieldCommandError) -> BackendError {
    match error {
        ObjectFieldCommandError::InvalidCommand(message) => BackendError::bad_request(message),
        ObjectFieldCommandError::History(error) => map_history_error(error),
    }
}

fn map_script_document_error(error: ScriptDocumentCommandError) -> BackendError {
    match error {
        ScriptDocumentCommandError::InvalidCommand(message) => BackendError::bad_request(message),
        ScriptDocumentCommandError::Store(error) => map_history_error(error),
    }
}

fn map_story_arc_command_error(error: StoryArcCommandError) -> BackendError {
    match error {
        StoryArcCommandError::InvalidCommand(message) => BackendError::bad_request(message),
        StoryArcCommandError::NotFound(message) => BackendError::not_found(message),
        StoryArcCommandError::History(error) => map_history_error(error),
    }
}

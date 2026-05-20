use std::path::PathBuf;

use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphNodeId, BibleGraphNodeListProjection,
    BibleGraphPartKey, BibleGraphSchemaKey, BibleGraphSnapshotFieldId, BibleGraphSnapshotId,
    BibleNodeDetailProjection, CommandEnvelope, CommandId, CreateBibleGraphNodeCommand,
    CreateStoryArcCommand, DeleteStoryArcCommand, EnsureCanonicalBibleRootsCommand, FieldValue,
    ProjectionEnvelope, ScriptDocumentProjection, SetBibleGraphEdgeCommand,
    SetBibleGraphFieldCommand, SetBibleGraphSnapshotFieldCommand, SetObjectFieldCommand,
    SetScriptBlockCommand, SetScriptLockCommand, SetStoryArcMetadataCommand,
    StoryArcListProjection,
};
use eidetic_core::story::arc::ArcId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::backend_error::BackendError;
use crate::bible_graph_command::{self, BibleGraphCommandError};
use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::object_field_command::{self, ObjectFieldCommandError};
use crate::revision_projection::ObjectFieldProjection;
use crate::script_document_command::{self, ScriptDocumentCommandError};
use crate::state::{AppState, ServerEvent};
use crate::story_arc_command::{self, StoryArcCommandError};
use crate::story_arc_store;

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

#[derive(Debug, Serialize)]
pub struct BibleGraphNodeCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<BibleNodeDetailProjection>,
}

#[derive(Debug, Serialize)]
pub struct BibleGraphRootsCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<BibleGraphNodeListProjection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateBibleGraphNodeRequestCommand {
    id: CommandId,
    payload: CreateBibleGraphNodeRequestPayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateBibleGraphNodeRequestPayload {
    #[serde(default)]
    node_id: Option<BibleGraphNodeId>,
    #[serde(default)]
    parent_id: Option<BibleGraphNodeId>,
    schema_key: BibleGraphSchemaKey,
    name: String,
    #[serde(default)]
    sort_order: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetBibleGraphEdgeRequestCommand {
    id: CommandId,
    payload: SetBibleGraphEdgeRequestPayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SetBibleGraphEdgeRequestPayload {
    #[serde(default)]
    edge_id: Option<BibleGraphEdgeId>,
    from_node_id: BibleGraphNodeId,
    to_node_id: BibleGraphNodeId,
    edge_kind: BibleGraphEdgeKind,
    label: String,
    #[serde(default = "default_directed")]
    directed: bool,
    #[serde(default)]
    sort_order: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetBibleGraphSnapshotFieldRequestCommand {
    id: CommandId,
    payload: SetBibleGraphSnapshotFieldRequestPayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SetBibleGraphSnapshotFieldRequestPayload {
    #[serde(default)]
    snapshot_id: Option<BibleGraphSnapshotId>,
    node_id: BibleGraphNodeId,
    at_ms: u64,
    label: String,
    #[serde(default)]
    snapshot_sort_order: u32,
    #[serde(default)]
    field_id: Option<BibleGraphSnapshotFieldId>,
    part_key: BibleGraphPartKey,
    part_name: String,
    field_key: eidetic_core::contracts::BibleGraphFieldKey,
    #[serde(default)]
    value: Option<FieldValue>,
    #[serde(default)]
    field_sort_order: u32,
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

impl CreateBibleGraphNodeRequestCommand {
    fn into_core_command(
        self,
    ) -> Result<CommandEnvelope<CreateBibleGraphNodeCommand>, BackendError> {
        let node_id = match self.payload.node_id {
            Some(node_id) => node_id,
            None => BibleGraphNodeId::new(format!(
                "node.{}.{}",
                self.payload.schema_key.as_str(),
                derived_command_uuid(self.id, b"bible.node")
            ))
            .map_err(|error| BackendError::bad_request(error.to_string()))?,
        };
        Ok(CommandEnvelope {
            id: self.id,
            payload: CreateBibleGraphNodeCommand {
                node_id,
                parent_id: self.payload.parent_id,
                schema_key: self.payload.schema_key,
                name: self.payload.name,
                sort_order: self.payload.sort_order,
            },
        })
    }
}

impl SetBibleGraphEdgeRequestCommand {
    fn into_core_command(self) -> Result<CommandEnvelope<SetBibleGraphEdgeCommand>, BackendError> {
        let edge_id = match self.payload.edge_id {
            Some(edge_id) => edge_id,
            None => BibleGraphEdgeId::new(format!(
                "edge.{}",
                derived_command_uuid(self.id, b"bible.edge")
            ))
            .map_err(|error| BackendError::bad_request(error.to_string()))?,
        };
        Ok(CommandEnvelope {
            id: self.id,
            payload: SetBibleGraphEdgeCommand {
                edge_id,
                from_node_id: self.payload.from_node_id,
                to_node_id: self.payload.to_node_id,
                edge_kind: self.payload.edge_kind,
                label: self.payload.label,
                directed: self.payload.directed,
                sort_order: self.payload.sort_order,
            },
        })
    }
}

impl SetBibleGraphSnapshotFieldRequestCommand {
    fn into_core_command(
        self,
    ) -> Result<CommandEnvelope<SetBibleGraphSnapshotFieldCommand>, BackendError> {
        let snapshot_id = match self.payload.snapshot_id {
            Some(snapshot_id) => snapshot_id,
            None => BibleGraphSnapshotId::new(format!(
                "snapshot.{}",
                derived_command_uuid(self.id, b"bible.snapshot")
            ))
            .map_err(|error| BackendError::bad_request(error.to_string()))?,
        };
        let field_id = match self.payload.field_id {
            Some(field_id) => field_id,
            None => BibleGraphSnapshotFieldId::new(format!(
                "snapshot-field.{}",
                derived_command_uuid(self.id, b"bible.snapshot.field")
            ))
            .map_err(|error| BackendError::bad_request(error.to_string()))?,
        };
        Ok(CommandEnvelope {
            id: self.id,
            payload: SetBibleGraphSnapshotFieldCommand {
                snapshot_id,
                node_id: self.payload.node_id,
                at_ms: self.payload.at_ms,
                label: self.payload.label,
                snapshot_sort_order: self.payload.snapshot_sort_order,
                field_id,
                part_key: self.payload.part_key,
                part_name: self.payload.part_name,
                field_key: self.payload.field_key,
                value: self.payload.value,
                field_sort_order: self.payload.field_sort_order,
            },
        })
    }
}

fn default_directed() -> bool {
    true
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

pub async fn set_bible_graph_field(
    state: &AppState,
    command: CommandEnvelope<SetBibleGraphFieldCommand>,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response =
        tokio::task::spawn_blocking(move || set_bible_graph_field_at_path(path, command))
            .await
            .map_err(|error| {
                BackendError::internal(format!("bible graph field task failed: {error}"))
            })??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    Ok(response)
}

pub async fn set_bible_graph_edge(
    state: &AppState,
    command: SetBibleGraphEdgeRequestCommand,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let command = command.into_core_command()?;
    let path = active_project_path(state)?;
    let response = tokio::task::spawn_blocking(move || set_bible_graph_edge_at_path(path, command))
        .await
        .map_err(|error| {
            BackendError::internal(format!("bible graph edge task failed: {error}"))
        })??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    Ok(response)
}

pub async fn set_bible_graph_snapshot_field(
    state: &AppState,
    command: SetBibleGraphSnapshotFieldRequestCommand,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let command = command.into_core_command()?;
    let path = active_project_path(state)?;
    let response =
        tokio::task::spawn_blocking(move || set_bible_graph_snapshot_field_at_path(path, command))
            .await
            .map_err(|error| {
                BackendError::internal(format!("bible graph snapshot field task failed: {error}"))
            })??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    Ok(response)
}

pub async fn create_bible_graph_node(
    state: &AppState,
    command: CreateBibleGraphNodeRequestCommand,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let command = command.into_core_command()?;
    let path = active_project_path(state)?;
    let response = tokio::task::spawn_blocking(move || create_bible_node_at_path(path, command))
        .await
        .map_err(|error| {
            BackendError::internal(format!("bible graph command task failed: {error}"))
        })??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    Ok(response)
}

pub async fn ensure_canonical_bible_roots(
    state: &AppState,
    command: CommandEnvelope<EnsureCanonicalBibleRootsCommand>,
) -> Result<BibleGraphRootsCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response = tokio::task::spawn_blocking(move || ensure_roots_at_path(path, command))
        .await
        .map_err(|error| {
            BackendError::internal(format!("bible graph roots task failed: {error}"))
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

fn active_project_path(state: &AppState) -> Result<PathBuf, BackendError> {
    if state.project.lock().is_none() {
        return Err(BackendError::no_project());
    }
    state
        .project_database
        .active_path()
        .ok_or_else(BackendError::no_project)
}

fn create_story_arc_at_path(
    path: PathBuf,
    command: CommandEnvelope<CreateStoryArcCommand>,
) -> Result<StoryArcCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    story_arc_store::create_schema(&conn).map_err(map_history_error)?;
    let outcome = story_arc_command::record_create_story_arc_history(&mut conn, &command, 0)
        .map_err(map_story_arc_command_error)?;
    story_arc_response(conn, outcome)
}

fn create_bible_node_at_path(
    path: PathBuf,
    command: CommandEnvelope<CreateBibleGraphNodeCommand>,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let (outcome, projection) =
        bible_graph_command::apply_create_bible_graph_node(&mut conn, &command, 0)
            .map_err(map_bible_graph_error)?;

    Ok(BibleGraphNodeCommandResponse {
        outcome,
        projection,
    })
}

fn ensure_roots_at_path(
    path: PathBuf,
    command: CommandEnvelope<EnsureCanonicalBibleRootsCommand>,
) -> Result<BibleGraphRootsCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let (outcome, projection) =
        bible_graph_command::apply_ensure_canonical_bible_roots(&mut conn, &command, 0)
            .map_err(map_bible_graph_error)?;

    Ok(BibleGraphRootsCommandResponse {
        outcome,
        projection,
    })
}

fn set_bible_graph_field_at_path(
    path: PathBuf,
    command: CommandEnvelope<SetBibleGraphFieldCommand>,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let (outcome, projection) =
        bible_graph_command::apply_set_bible_graph_field(&mut conn, &command, 0)
            .map_err(map_bible_graph_error)?;

    Ok(BibleGraphNodeCommandResponse {
        outcome,
        projection,
    })
}

fn set_bible_graph_edge_at_path(
    path: PathBuf,
    command: CommandEnvelope<SetBibleGraphEdgeCommand>,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let (outcome, projection) =
        bible_graph_command::apply_set_bible_graph_edge(&mut conn, &command, 0)
            .map_err(map_bible_graph_error)?;

    Ok(BibleGraphNodeCommandResponse {
        outcome,
        projection,
    })
}

fn set_bible_graph_snapshot_field_at_path(
    path: PathBuf,
    command: CommandEnvelope<SetBibleGraphSnapshotFieldCommand>,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let (outcome, projection) =
        bible_graph_command::apply_set_bible_graph_snapshot_field(&mut conn, &command, 0)
            .map_err(map_bible_graph_error)?;

    Ok(BibleGraphNodeCommandResponse {
        outcome,
        projection,
    })
}

fn update_story_arc_at_path(
    path: PathBuf,
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
    path: PathBuf,
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

fn derived_command_uuid(command_id: CommandId, role: &[u8]) -> Uuid {
    let mut bytes = *command_id.0.as_bytes();
    for (index, byte) in role.iter().enumerate() {
        let slot = index % bytes.len();
        bytes[slot] = bytes[slot].wrapping_add(*byte).rotate_left(1);
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

fn apply_object_field_at_path(
    path: PathBuf,
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
    path: PathBuf,
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
    path: PathBuf,
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

fn map_history_error(error: HistoryStoreError) -> BackendError {
    match error {
        HistoryStoreError::InvalidValue(message) => BackendError::conflict(message),
        HistoryStoreError::InvalidId(message) => BackendError::bad_request(message),
        HistoryStoreError::MissingColumn(message) => BackendError::internal(message),
        HistoryStoreError::Sqlite(error) => BackendError::internal(error.to_string()),
        HistoryStoreError::Json(error) => BackendError::bad_request(error.to_string()),
    }
}

fn map_bible_graph_error(error: BibleGraphCommandError) -> BackendError {
    match error {
        BibleGraphCommandError::InvalidCommand(message) => BackendError::bad_request(message),
        BibleGraphCommandError::Store(error) => map_history_error(error),
    }
}

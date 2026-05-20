use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphNodeId, BibleGraphNodeListProjection, BibleGraphSnapshotFieldId,
    BibleGraphSnapshotId, CommandId, EnsureCanonicalBibleRootsCommand,
};
use eidetic_core::contracts::{
    BibleNodeDetailProjection, CommandEnvelope, CreateBibleGraphNodeCommand, CreateStoryArcCommand,
    DeleteStoryArcCommand, ProjectionEnvelope, ScriptDocumentProjection, SetBibleGraphEdgeCommand,
    SetBibleGraphFieldCommand, SetBibleGraphSnapshotFieldCommand, SetObjectFieldCommand,
    SetScriptBlockCommand, SetScriptLockCommand, SetStoryArcMetadataCommand,
    StoryArcListProjection,
};
use eidetic_core::story::arc::ArcId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::bible_graph_command::{self, BibleGraphCommandError};
use crate::error::{ApiError, ApiJson};
use crate::history_store::{self, RecordChangeOutcome};
use crate::object_field_command::{self, ObjectFieldCommandError};
use crate::revision_projection::ObjectFieldProjection;
use crate::script_document_command::{self, ScriptDocumentCommandError};
use crate::state::{AppState, ServerEvent};
use crate::story_arc_command::{self, StoryArcCommandError};
use crate::story_arc_store;

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
        .route("/commands/story/create-arc", post(create_story_arc))
        .route("/commands/story/update-arc", post(update_story_arc))
        .route("/commands/story/delete-arc", post(delete_story_arc))
        .merge(super::commands_timeline::router())
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
struct StoryArcCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<StoryArcListProjection>,
}

#[derive(Debug, Deserialize)]
struct CreateStoryArcRouteCommand {
    id: CommandId,
    payload: CreateStoryArcRoutePayload,
}

#[derive(Debug, Deserialize)]
struct CreateStoryArcRoutePayload {
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

#[derive(Debug, Deserialize)]
struct CreateBibleGraphNodeRouteCommand {
    id: CommandId,
    payload: CreateBibleGraphNodeRoutePayload,
}

#[derive(Debug, Deserialize)]
struct CreateBibleGraphNodeRoutePayload {
    #[serde(default)]
    node_id: Option<BibleGraphNodeId>,
    #[serde(default)]
    parent_id: Option<BibleGraphNodeId>,
    schema_key: eidetic_core::contracts::BibleGraphSchemaKey,
    name: String,
    #[serde(default)]
    sort_order: u32,
}

#[derive(Debug, Deserialize)]
struct SetBibleGraphEdgeRouteCommand {
    id: CommandId,
    payload: SetBibleGraphEdgeRoutePayload,
}

#[derive(Debug, Deserialize)]
struct SetBibleGraphEdgeRoutePayload {
    #[serde(default)]
    edge_id: Option<BibleGraphEdgeId>,
    from_node_id: BibleGraphNodeId,
    to_node_id: BibleGraphNodeId,
    edge_kind: eidetic_core::contracts::BibleGraphEdgeKind,
    label: String,
    #[serde(default = "default_directed")]
    directed: bool,
    #[serde(default)]
    sort_order: u32,
}

#[derive(Debug, Deserialize)]
struct SetBibleGraphSnapshotFieldRouteCommand {
    id: CommandId,
    payload: SetBibleGraphSnapshotFieldRoutePayload,
}

#[derive(Debug, Deserialize)]
struct SetBibleGraphSnapshotFieldRoutePayload {
    #[serde(default)]
    snapshot_id: Option<BibleGraphSnapshotId>,
    node_id: BibleGraphNodeId,
    at_ms: u64,
    label: String,
    #[serde(default)]
    snapshot_sort_order: u32,
    #[serde(default)]
    field_id: Option<BibleGraphSnapshotFieldId>,
    part_key: eidetic_core::contracts::BibleGraphPartKey,
    part_name: String,
    field_key: eidetic_core::contracts::BibleGraphFieldKey,
    #[serde(default)]
    value: Option<eidetic_core::contracts::FieldValue>,
    #[serde(default)]
    field_sort_order: u32,
}

impl CreateStoryArcRouteCommand {
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

impl CreateBibleGraphNodeRouteCommand {
    fn into_core_command(self) -> Result<CommandEnvelope<CreateBibleGraphNodeCommand>, ApiError> {
        let node_id = match self.payload.node_id {
            Some(node_id) => node_id,
            None => BibleGraphNodeId::new(format!(
                "node.{}.{}",
                self.payload.schema_key.as_str(),
                derived_command_uuid(self.id, b"bible.node")
            ))
            .map_err(|error| ApiError::bad_request(error.to_string()))?,
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

impl SetBibleGraphEdgeRouteCommand {
    fn into_core_command(self) -> Result<CommandEnvelope<SetBibleGraphEdgeCommand>, ApiError> {
        let edge_id = match self.payload.edge_id {
            Some(edge_id) => edge_id,
            None => BibleGraphEdgeId::new(format!(
                "edge.{}",
                derived_command_uuid(self.id, b"bible.edge")
            ))
            .map_err(|error| ApiError::bad_request(error.to_string()))?,
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

impl SetBibleGraphSnapshotFieldRouteCommand {
    fn into_core_command(
        self,
    ) -> Result<CommandEnvelope<SetBibleGraphSnapshotFieldCommand>, ApiError> {
        let snapshot_id = match self.payload.snapshot_id {
            Some(snapshot_id) => snapshot_id,
            None => BibleGraphSnapshotId::new(format!(
                "snapshot.{}",
                derived_command_uuid(self.id, b"bible.snapshot")
            ))
            .map_err(|error| ApiError::bad_request(error.to_string()))?,
        };
        let field_id = match self.payload.field_id {
            Some(field_id) => field_id,
            None => BibleGraphSnapshotFieldId::new(format!(
                "snapshot-field.{}",
                derived_command_uuid(self.id, b"bible.snapshot.field")
            ))
            .map_err(|error| ApiError::bad_request(error.to_string()))?,
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
    Json(command): Json<CreateBibleGraphNodeRouteCommand>,
) -> ApiJson {
    let command = command.into_core_command()?;
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
    Json(command): Json<SetBibleGraphEdgeRouteCommand>,
) -> ApiJson {
    let command = command.into_core_command()?;
    let path = active_project_path(&state)?;
    let response = tokio::task::spawn_blocking(move || set_bible_graph_edge_at_path(path, command))
        .await
        .map_err(|e| ApiError::internal(format!("bible graph edge task failed: {e}")))??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    crate::error::json_value(response)
}

async fn set_bible_graph_snapshot_field(
    State(state): State<AppState>,
    Json(command): Json<SetBibleGraphSnapshotFieldRouteCommand>,
) -> ApiJson {
    let command = command.into_core_command()?;
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

async fn create_story_arc(
    State(state): State<AppState>,
    Json(command): Json<CreateStoryArcRouteCommand>,
) -> ApiJson {
    let command = command.into_core_command();
    let path = active_project_path(&state)?;
    if state.project.lock().is_none() {
        return Err(ApiError::no_project());
    }
    let response = {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        story_arc_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = story_arc_command::record_create_story_arc_history(&mut conn, &command, 0)
            .map_err(map_story_arc_command_error)?;
        let projection =
            story_arc_store::load_arc_list_projection_envelope(&conn).map_err(map_history_error)?;
        StoryArcCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
    }
    crate::error::json_value(response)
}

async fn update_story_arc(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetStoryArcMetadataCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    if state.project.lock().is_none() {
        return Err(ApiError::no_project());
    }
    let response = {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        story_arc_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome =
            story_arc_command::record_set_story_arc_metadata_history(&mut conn, &command, 0)
                .map_err(map_story_arc_command_error)?;
        let projection =
            story_arc_store::load_arc_list_projection_envelope(&conn).map_err(map_history_error)?;
        StoryArcCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
    }
    crate::error::json_value(response)
}

async fn delete_story_arc(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<DeleteStoryArcCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    if state.project.lock().is_none() {
        return Err(ApiError::no_project());
    }
    let response = {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        story_arc_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = story_arc_command::record_delete_story_arc_history(&mut conn, &command, 0)
            .map_err(map_story_arc_command_error)?;
        let projection =
            story_arc_store::load_arc_list_projection_envelope(&conn).map_err(map_history_error)?;
        StoryArcCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
    }
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

fn map_story_arc_command_error(error: StoryArcCommandError) -> ApiError {
    match error {
        StoryArcCommandError::InvalidCommand(message) => ApiError::bad_request(message),
        StoryArcCommandError::NotFound(message) => ApiError::not_found(message),
        StoryArcCommandError::History(error) => map_history_error(error),
    }
}

#[cfg(test)]
#[path = "commands_object_story_tests.rs"]
mod object_story_tests;

#[cfg(test)]
#[path = "commands_bible_tests.rs"]
mod bible_tests;

#[cfg(test)]
#[path = "commands_script_tests.rs"]
mod script_tests;

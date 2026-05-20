use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphNodeId, BibleGraphNodeListProjection,
    BibleGraphPartKey, BibleGraphSchemaKey, BibleGraphSnapshotFieldId, BibleGraphSnapshotId,
    BibleNodeDetailProjection, CommandEnvelope, CommandId, CreateBibleGraphNodeCommand,
    EnsureCanonicalBibleRootsCommand, FieldValue, ProjectionEnvelope, SetBibleGraphEdgeCommand,
    SetBibleGraphFieldCommand, SetBibleGraphSnapshotFieldCommand,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::bible_graph_command::{self, BibleGraphCommandError};
use crate::error::{ApiError, ApiJson};
use crate::history_store::RecordChangeOutcome;
use crate::state::{AppState, ServerEvent};

use super::support::{active_project_path, map_history_error};

pub fn router() -> Router<AppState> {
    Router::new()
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateBibleGraphNodeRouteCommand {
    id: CommandId,
    payload: CreateBibleGraphNodeRoutePayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateBibleGraphNodeRoutePayload {
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
struct SetBibleGraphEdgeRouteCommand {
    id: CommandId,
    payload: SetBibleGraphEdgeRoutePayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SetBibleGraphEdgeRoutePayload {
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
struct SetBibleGraphSnapshotFieldRouteCommand {
    id: CommandId,
    payload: SetBibleGraphSnapshotFieldRoutePayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
    part_key: BibleGraphPartKey,
    part_name: String,
    field_key: eidetic_core::contracts::BibleGraphFieldKey,
    #[serde(default)]
    value: Option<FieldValue>,
    #[serde(default)]
    field_sort_order: u32,
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

fn map_bible_graph_error(error: BibleGraphCommandError) -> ApiError {
    match error {
        BibleGraphCommandError::InvalidCommand(message) => ApiError::bad_request(message),
        BibleGraphCommandError::Store(error) => map_history_error(error),
    }
}

#[cfg(test)]
#[path = "commands_bible_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "commands_bible_snapshot_tests.rs"]
mod snapshot_tests;

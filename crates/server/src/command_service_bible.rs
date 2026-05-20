use std::path::PathBuf;

use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphNodeId, BibleGraphNodeListProjection,
    BibleGraphPartKey, BibleGraphSchemaKey, BibleGraphSnapshotFieldId, BibleGraphSnapshotId,
    BibleNodeDetailProjection, CommandEnvelope, CommandId, CreateBibleGraphNodeCommand,
    EnsureCanonicalBibleRootsCommand, FieldValue, ProjectionEnvelope, SetBibleGraphEdgeCommand,
    SetBibleGraphFieldCommand, SetBibleGraphSnapshotFieldCommand,
};
use serde::{Deserialize, Serialize};

use crate::backend_error::BackendError;
use crate::bible_graph_command::{self, BibleGraphCommandError};
use crate::command_service_support::{
    active_project_path, derived_command_uuid, map_history_error,
};
use crate::history_store::RecordChangeOutcome;
use crate::state::{AppState, ServerEvent};

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

fn map_bible_graph_error(error: BibleGraphCommandError) -> BackendError {
    match error {
        BibleGraphCommandError::InvalidCommand(message) => BackendError::bad_request(message),
        BibleGraphCommandError::Store(error) => map_history_error(error),
    }
}

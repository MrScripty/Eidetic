use std::path::PathBuf;

use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphNodeCategory, BibleGraphNodeId,
    BibleGraphNodeListProjection, BibleGraphPartKey, BibleGraphSchemaKey,
    BibleGraphSnapshotFieldId, BibleGraphSnapshotId, BibleNodeDetailProjection, CommandEnvelope,
    CommandId, CreateBibleGraphNodeCommand, DeleteBibleGraphEdgeCommand,
    DeleteBibleGraphNodeCommand, EnsureCanonicalBibleRootsCommand, FieldValue, ProjectionEnvelope,
    SetBibleGraphEdgeCommand, SetBibleGraphFieldCommand, SetBibleGraphNodeNameCommand,
    SetBibleGraphNodeTextCommand, SetBibleGraphSnapshotFieldCommand,
    builtin_bible_graph_schema_list_projection,
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

impl BibleGraphNodeCommandResponse {
    pub fn node_id(&self) -> &BibleGraphNodeId {
        &self.projection.payload.node.id
    }
}

#[derive(Debug, Serialize)]
pub struct BibleGraphRootsCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<BibleGraphNodeListProjection>,
}

#[derive(Debug, Serialize)]
pub struct BibleGraphNodeListCommandResponse {
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

pub async fn set_bible_graph_node_name(
    state: &AppState,
    command: CommandEnvelope<SetBibleGraphNodeNameCommand>,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response =
        tokio::task::spawn_blocking(move || set_bible_graph_node_name_at_path(path, command))
            .await
            .map_err(|error| {
                BackendError::internal(format!(
                    "bible graph node name command task failed: {error}"
                ))
            })??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    Ok(response)
}

pub async fn set_bible_graph_node_text(
    state: &AppState,
    command: CommandEnvelope<SetBibleGraphNodeTextCommand>,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response =
        tokio::task::spawn_blocking(move || set_bible_graph_node_text_at_path(path, command))
            .await
            .map_err(|error| {
                BackendError::internal(format!(
                    "bible graph node text command task failed: {error}"
                ))
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

pub async fn delete_bible_graph_edge(
    state: &AppState,
    command: CommandEnvelope<DeleteBibleGraphEdgeCommand>,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response =
        tokio::task::spawn_blocking(move || delete_bible_graph_edge_at_path(path, command))
            .await
            .map_err(|error| {
                BackendError::internal(format!("bible graph edge delete task failed: {error}"))
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

pub async fn create_connected_bible_graph_node(
    state: &AppState,
    parent_id: BibleGraphNodeId,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response =
        tokio::task::spawn_blocking(move || create_connected_bible_node_at_path(path, parent_id))
            .await
            .map_err(|error| {
                BackendError::internal(format!(
                    "connected bible graph node create task failed: {error}"
                ))
            })??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    Ok(response)
}

pub async fn delete_bible_graph_node(
    state: &AppState,
    command: CommandEnvelope<DeleteBibleGraphNodeCommand>,
) -> Result<BibleGraphNodeListCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response =
        tokio::task::spawn_blocking(move || delete_bible_graph_node_at_path(path, command))
            .await
            .map_err(|error| {
                BackendError::internal(format!("bible graph node delete task failed: {error}"))
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

fn create_connected_bible_node_at_path(
    path: PathBuf,
    parent_id: BibleGraphNodeId,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let command = create_connected_bible_node_command(&conn, parent_id)?;
    let (outcome, projection) =
        bible_graph_command::apply_create_bible_graph_node(&mut conn, &command, 0)
            .map_err(map_bible_graph_error)?;

    Ok(BibleGraphNodeCommandResponse {
        outcome,
        projection,
    })
}

fn create_connected_bible_node_command(
    conn: &rusqlite::Connection,
    parent_id: BibleGraphNodeId,
) -> Result<CommandEnvelope<CreateBibleGraphNodeCommand>, BackendError> {
    crate::bible_graph_store::create_schema(conn).map_err(map_history_error)?;
    let parent = crate::bible_graph_store::load_node(conn, &parent_id)
        .map_err(map_history_error)?
        .ok_or_else(|| {
            BackendError::bad_request(format!(
                "bible graph parent node does not exist: {}",
                parent_id.as_str()
            ))
        })?;
    let category = category_for_connected_child(&parent);
    let schema = builtin_bible_graph_schema_list_projection()
        .payload
        .schemas
        .into_iter()
        .find(|schema| schema.category == category)
        .ok_or_else(|| {
            BackendError::bad_request(format!(
                "no creatable bible graph schema for {}",
                category.display_name()
            ))
        })?;
    let sort_order = crate::bible_graph_store::active_child_count(conn, &parent_id)
        .map_err(map_history_error)?
        .try_into()
        .unwrap_or(u32::MAX);
    let command_id = CommandId::new();
    let node_id = BibleGraphNodeId::new(format!(
        "node.{}.{}",
        schema.schema_key.as_str(),
        derived_command_uuid(command_id, b"bible.node")
    ))
    .map_err(|error| BackendError::bad_request(error.to_string()))?;

    Ok(CommandEnvelope {
        id: command_id,
        payload: CreateBibleGraphNodeCommand {
            node_id,
            parent_id: Some(parent_id),
            schema_key: schema.schema_key,
            name: schema.default_node_name,
            sort_order,
        },
    })
}

fn category_for_connected_child(
    parent: &eidetic_core::contracts::BibleGraphNode,
) -> BibleGraphNodeCategory {
    if parent.system_owned && parent.schema_key.as_str().starts_with("canonical.") {
        BibleGraphNodeCategory::for_node(parent)
    } else {
        BibleGraphNodeCategory::Detail
    }
}

fn delete_bible_graph_node_at_path(
    path: PathBuf,
    command: CommandEnvelope<DeleteBibleGraphNodeCommand>,
) -> Result<BibleGraphNodeListCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let (outcome, projection) =
        bible_graph_command::apply_delete_bible_graph_node(&mut conn, &command, 0)
            .map_err(map_bible_graph_error)?;

    Ok(BibleGraphNodeListCommandResponse {
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

fn set_bible_graph_node_name_at_path(
    path: PathBuf,
    command: CommandEnvelope<SetBibleGraphNodeNameCommand>,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let (outcome, projection) =
        bible_graph_command::apply_set_bible_graph_node_name(&mut conn, &command, 0)
            .map_err(map_bible_graph_error)?;
    Ok(BibleGraphNodeCommandResponse {
        outcome,
        projection,
    })
}

fn set_bible_graph_node_text_at_path(
    path: PathBuf,
    command: CommandEnvelope<SetBibleGraphNodeTextCommand>,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let (outcome, projection) =
        bible_graph_command::apply_set_bible_graph_node_text(&mut conn, &command, 0)
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

fn delete_bible_graph_edge_at_path(
    path: PathBuf,
    command: CommandEnvelope<DeleteBibleGraphEdgeCommand>,
) -> Result<BibleGraphNodeCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let (outcome, projection) =
        bible_graph_command::apply_delete_bible_graph_edge(&mut conn, &command, 0)
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

#[cfg(test)]
mod tests {
    use super::create_connected_bible_node_command;
    use eidetic_core::contracts::{
        BIBLE_GRAPH_NODE_TEXT_FIELD_KEY, BIBLE_GRAPH_NODE_TEXT_PART_KEY, CanonicalBibleRoot,
        CommandEnvelope, EnsureCanonicalBibleRootsCommand, FieldValue,
        SetBibleGraphNodeNameCommand, SetBibleGraphNodeTextCommand,
    };
    use rusqlite::Connection;

    #[test]
    fn connected_node_command_uses_parent_category_schema_and_next_sort_order() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::bible_graph_command::apply_ensure_canonical_bible_roots(
            &mut conn,
            &CommandEnvelope::new(EnsureCanonicalBibleRootsCommand {}),
            0,
        )
        .unwrap();

        let parent_id = CanonicalBibleRoot::Characters.node_id();
        let first = create_connected_bible_node_command(&conn, parent_id.clone()).unwrap();
        crate::bible_graph_command::apply_create_bible_graph_node(&mut conn, &first, 0).unwrap();
        let second = create_connected_bible_node_command(&conn, parent_id.clone()).unwrap();

        assert!(
            first
                .payload
                .node_id
                .as_str()
                .starts_with("node.character.")
        );
        assert_eq!(first.payload.parent_id, Some(parent_id.clone()));
        assert_eq!(first.payload.schema_key.as_str(), "character");
        assert_eq!(first.payload.name, "New Character");
        assert_eq!(first.payload.sort_order, 0);
        assert_eq!(second.payload.parent_id, Some(parent_id));
        assert_eq!(second.payload.sort_order, 1);
    }

    #[test]
    fn connected_node_command_supports_all_canonical_root_categories() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::bible_graph_command::apply_ensure_canonical_bible_roots(
            &mut conn,
            &CommandEnvelope::new(EnsureCanonicalBibleRootsCommand {}),
            0,
        )
        .unwrap();

        let culture =
            create_connected_bible_node_command(&conn, CanonicalBibleRoot::Cultures.node_id())
                .unwrap();
        let rule = create_connected_bible_node_command(&conn, CanonicalBibleRoot::Rules.node_id())
            .unwrap();
        let reference =
            create_connected_bible_node_command(&conn, CanonicalBibleRoot::References.node_id())
                .unwrap();

        assert_eq!(culture.payload.schema_key.as_str(), "culture");
        assert_eq!(culture.payload.name, "New Culture");
        assert_eq!(rule.payload.schema_key.as_str(), "rule");
        assert_eq!(rule.payload.name, "New Rule");
        assert_eq!(reference.payload.schema_key.as_str(), "reference");
        assert_eq!(reference.payload.name, "New Reference");
    }

    #[test]
    fn connected_node_command_creates_detail_under_non_root_nodes() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::bible_graph_command::apply_ensure_canonical_bible_roots(
            &mut conn,
            &CommandEnvelope::new(EnsureCanonicalBibleRootsCommand {}),
            0,
        )
        .unwrap();

        let character =
            create_connected_bible_node_command(&conn, CanonicalBibleRoot::Characters.node_id())
                .unwrap();
        crate::bible_graph_command::apply_create_bible_graph_node(&mut conn, &character, 0)
            .unwrap();
        let detail =
            create_connected_bible_node_command(&conn, character.payload.node_id.clone()).unwrap();

        assert_eq!(detail.payload.parent_id, Some(character.payload.node_id));
        assert_eq!(detail.payload.schema_key.as_str(), "detail");
        assert_eq!(detail.payload.name, "New Detail");
    }

    #[test]
    fn set_node_name_command_updates_node_projection() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::bible_graph_command::apply_ensure_canonical_bible_roots(
            &mut conn,
            &CommandEnvelope::new(EnsureCanonicalBibleRootsCommand {}),
            0,
        )
        .unwrap();
        let character =
            create_connected_bible_node_command(&conn, CanonicalBibleRoot::Characters.node_id())
                .unwrap();
        crate::bible_graph_command::apply_create_bible_graph_node(&mut conn, &character, 0)
            .unwrap();

        let (_, projection) = crate::bible_graph_command::apply_set_bible_graph_node_name(
            &mut conn,
            &CommandEnvelope::new(SetBibleGraphNodeNameCommand {
                node_id: character.payload.node_id,
                name: "Ada".to_string(),
            }),
            0,
        )
        .unwrap();

        assert_eq!(projection.payload.node.name, "Ada");
    }

    #[test]
    fn set_node_text_command_updates_backend_owned_node_content() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::bible_graph_command::apply_ensure_canonical_bible_roots(
            &mut conn,
            &CommandEnvelope::new(EnsureCanonicalBibleRootsCommand {}),
            0,
        )
        .unwrap();
        let character =
            create_connected_bible_node_command(&conn, CanonicalBibleRoot::Characters.node_id())
                .unwrap();
        crate::bible_graph_command::apply_create_bible_graph_node(&mut conn, &character, 0)
            .unwrap();

        let (_, projection) = crate::bible_graph_command::apply_set_bible_graph_node_text(
            &mut conn,
            &CommandEnvelope::new(SetBibleGraphNodeTextCommand {
                node_id: character.payload.node_id,
                text: "Ada keeps a coded notebook.".to_string(),
            }),
            0,
        )
        .unwrap();

        let content_part = projection
            .payload
            .parts
            .iter()
            .find(|part| part.part.part_key.as_str() == BIBLE_GRAPH_NODE_TEXT_PART_KEY)
            .expect("node text content part should be projected");
        let text_field = content_part
            .fields
            .iter()
            .find(|field| field.field_key.as_str() == BIBLE_GRAPH_NODE_TEXT_FIELD_KEY)
            .expect("node text field should be projected");
        assert_eq!(
            text_field.value,
            Some(FieldValue::Text("Ada keeps a coded notebook.".to_string()))
        );
    }
}

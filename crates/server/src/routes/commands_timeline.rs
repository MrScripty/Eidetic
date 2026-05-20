use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use eidetic_core::contracts::{
    ApplyTimelineChildCommand, ApplyTimelineChildrenCommand, CommandEnvelope, CommandId,
    CreateTimelineNodeCommand, CreateTimelineRelationshipCommand, DeleteTimelineNodeCommand,
    DeleteTimelineRelationshipCommand, ObjectKind, ProjectionEnvelope, SetTimelineNodeLockCommand,
    SetTimelineNodeNotesCommand, SetTimelineNodeRangeCommand, SplitTimelineNodeCommand,
    TimelineRenderProjection,
};
use eidetic_core::timeline::Timeline;
use eidetic_core::timeline::node::NodeId;
use eidetic_core::timeline::relationship::RelationshipId;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{ApiError, ApiJson};
use crate::history_store::{self, RecordChangeOutcome};
use crate::state::{AppState, ServerEvent};
use crate::timeline_command::{self, TimelineCommandError};
use crate::validation;
use crate::ydoc::DocCommand;
use crate::{timeline_node_store, timeline_relationship_store};

use super::support::{active_project_path, map_history_error};

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

#[derive(Debug, Serialize)]
struct TimelineCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<TimelineRenderProjection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateTimelineNodeRouteCommand {
    id: CommandId,
    payload: CreateTimelineNodeRoutePayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateTimelineNodeRoutePayload {
    #[serde(default)]
    node_id: Option<NodeId>,
    parent_id: Option<NodeId>,
    level: eidetic_core::timeline::node::StoryLevel,
    name: String,
    start_ms: u64,
    end_ms: u64,
    beat_type: Option<eidetic_core::timeline::node::BeatType>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApplyTimelineChildrenRouteCommand {
    id: CommandId,
    payload: ApplyTimelineChildrenRoutePayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApplyTimelineChildrenRoutePayload {
    parent_id: NodeId,
    #[serde(default)]
    child_plan_id: Option<eidetic_core::ai::backend::ChildPlanId>,
    children: Vec<ApplyTimelineChildRoutePayload>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApplyTimelineChildRoutePayload {
    #[serde(default)]
    node_id: Option<NodeId>,
    name: String,
    outline: String,
    weight: f32,
    beat_type: Option<eidetic_core::timeline::node::BeatType>,
    #[serde(default)]
    characters: Vec<String>,
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    props: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SplitTimelineNodeRouteCommand {
    id: CommandId,
    payload: SplitTimelineNodeRoutePayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SplitTimelineNodeRoutePayload {
    node_id: NodeId,
    at_ms: u64,
    #[serde(default)]
    left_node_id: Option<NodeId>,
    #[serde(default)]
    right_node_id: Option<NodeId>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateTimelineRelationshipRouteCommand {
    id: CommandId,
    payload: CreateTimelineRelationshipRoutePayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateTimelineRelationshipRoutePayload {
    #[serde(default)]
    relationship_id: Option<RelationshipId>,
    from_node_id: NodeId,
    to_node_id: NodeId,
    relationship_type: eidetic_core::timeline::relationship::RelationshipType,
}

impl CreateTimelineNodeRouteCommand {
    fn into_core_command(self) -> CommandEnvelope<CreateTimelineNodeCommand> {
        CommandEnvelope {
            id: self.id,
            payload: CreateTimelineNodeCommand {
                node_id: self
                    .payload
                    .node_id
                    .unwrap_or_else(|| NodeId(derived_command_uuid(self.id, b"timeline.node"))),
                parent_id: self.payload.parent_id,
                level: self.payload.level,
                name: self.payload.name,
                start_ms: self.payload.start_ms,
                end_ms: self.payload.end_ms,
                beat_type: self.payload.beat_type,
            },
        }
    }
}

impl ApplyTimelineChildrenRouteCommand {
    fn into_core_command(self) -> CommandEnvelope<ApplyTimelineChildrenCommand> {
        CommandEnvelope {
            id: self.id,
            payload: ApplyTimelineChildrenCommand {
                parent_id: self.payload.parent_id,
                child_plan_id: self.payload.child_plan_id,
                children: self
                    .payload
                    .children
                    .into_iter()
                    .enumerate()
                    .map(|(index, child)| ApplyTimelineChildCommand {
                        node_id: child.node_id.unwrap_or_else(|| {
                            NodeId(derived_indexed_command_uuid(
                                self.id,
                                b"timeline.child",
                                index,
                            ))
                        }),
                        name: child.name,
                        outline: child.outline,
                        weight: child.weight,
                        beat_type: child.beat_type,
                        characters: child.characters,
                        location: child.location,
                        props: child.props,
                    })
                    .collect(),
            },
        }
    }
}

impl SplitTimelineNodeRouteCommand {
    fn into_core_command(self) -> CommandEnvelope<SplitTimelineNodeCommand> {
        CommandEnvelope {
            id: self.id,
            payload: SplitTimelineNodeCommand {
                node_id: self.payload.node_id,
                at_ms: self.payload.at_ms,
                left_node_id: self.payload.left_node_id.unwrap_or_else(|| {
                    NodeId(derived_command_uuid(self.id, b"timeline.split.left"))
                }),
                right_node_id: self.payload.right_node_id.unwrap_or_else(|| {
                    NodeId(derived_command_uuid(self.id, b"timeline.split.right"))
                }),
            },
        }
    }
}

impl CreateTimelineRelationshipRouteCommand {
    fn into_core_command(self) -> CommandEnvelope<CreateTimelineRelationshipCommand> {
        CommandEnvelope {
            id: self.id,
            payload: CreateTimelineRelationshipCommand {
                relationship_id: self.payload.relationship_id.unwrap_or_else(|| {
                    RelationshipId(derived_command_uuid(self.id, b"timeline.relationship"))
                }),
                from_node_id: self.payload.from_node_id,
                to_node_id: self.payload.to_node_id,
                relationship_type: self.payload.relationship_type,
            },
        }
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

fn derived_indexed_command_uuid(command_id: CommandId, role: &[u8], index: usize) -> Uuid {
    let mut bytes = *derived_command_uuid(command_id, role).as_bytes();
    for (offset, byte) in index.to_le_bytes().iter().enumerate() {
        let slot = bytes.len() - 1 - (offset % bytes.len());
        bytes[slot] ^= *byte;
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

async fn set_timeline_node_range(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetTimelineNodeRangeCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = {
        let project = timeline_command_project(&state, &path).await?;
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_set_timeline_node_range_history(
            &mut conn, &project, &command, 0,
        )
        .map_err(map_timeline_command_error)?;
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        state.trigger_save();
    }
    crate::error::json_value(response)
}

async fn timeline_command_project(
    state: &AppState,
    path: &std::path::Path,
) -> Result<eidetic_core::Project, ApiError> {
    if state.project.lock().is_none() {
        return Err(ApiError::no_project());
    }
    match crate::persistence::load_project(path).await {
        Ok((project, _)) => Ok(project),
        Err(_) => state
            .project
            .lock()
            .clone()
            .ok_or_else(ApiError::no_project),
    }
}

async fn create_timeline_node(
    State(state): State<AppState>,
    Json(command): Json<CreateTimelineNodeRouteCommand>,
) -> ApiJson {
    validation::validate_name(&command.payload.name, "node name")?;
    let command = command.into_core_command();
    let path = active_project_path(&state)?;
    let created_node_id = command.payload.node_id;
    let response = {
        let project = timeline_command_project(&state, &path).await?;
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome =
            timeline_command::record_create_timeline_node_history(&mut conn, &project, &command, 0)
                .map_err(map_timeline_command_error)?;
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.doc_tx.try_send(DocCommand::EnsureNode {
            node_id: created_node_id,
        });
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        let _ = state.events_tx.send(ServerEvent::HierarchyChanged);
        state.trigger_save();
    }
    crate::error::json_value(response)
}

async fn apply_timeline_children(
    State(state): State<AppState>,
    Json(command): Json<ApplyTimelineChildrenRouteCommand>,
) -> ApiJson {
    for child in &command.payload.children {
        validation::validate_name(&child.name, "child node name")?;
        validation::validate_positive_finite_f32(child.weight, "child weight")?;
    }
    let command = command.into_core_command();
    let path = active_project_path(&state)?;
    let children = command.payload.children.clone();
    let response = {
        let project = timeline_command_project(&state, &path).await?;
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_apply_timeline_children_history(
            &mut conn, &project, &command, 0,
        )
        .map_err(map_timeline_command_error)?;
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let has_bible_references = children_have_bible_references(&children);
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
        if has_bible_references {
            let _ = state.events_tx.send(ServerEvent::SemanticProposalsChanged);
        }
        state.trigger_save();
    }
    crate::error::json_value(response)
}

fn children_have_bible_references(children: &[ApplyTimelineChildCommand]) -> bool {
    children.iter().any(|child| {
        child
            .characters
            .iter()
            .any(|value| !value.as_str().trim().is_empty())
            || child
                .location
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            || child
                .props
                .iter()
                .any(|value| !value.as_str().trim().is_empty())
    })
}

async fn create_timeline_relationship(
    State(state): State<AppState>,
    Json(command): Json<CreateTimelineRelationshipRouteCommand>,
) -> ApiJson {
    let command = command.into_core_command();
    let path = active_project_path(&state)?;
    let response = {
        let project = timeline_command_project(&state, &path).await?;
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_create_timeline_relationship_history(
            &mut conn, &project, &command, 0,
        )
        .map_err(map_timeline_command_error)?;
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        state.trigger_save();
    }
    crate::error::json_value(response)
}

async fn delete_timeline_relationship(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<DeleteTimelineRelationshipCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = {
        let project = timeline_command_project(&state, &path).await?;
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_delete_timeline_relationship_history(
            &mut conn, &project, &command, 0,
        )
        .map_err(map_timeline_command_error)?;
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        state.trigger_save();
    }
    crate::error::json_value(response)
}

async fn split_timeline_node(
    State(state): State<AppState>,
    Json(command): Json<SplitTimelineNodeRouteCommand>,
) -> ApiJson {
    let command = command.into_core_command();
    let path = active_project_path(&state)?;
    let response = {
        let project = timeline_command_project(&state, &path).await?;
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome =
            timeline_command::record_split_timeline_node_history(&mut conn, &project, &command, 0)
                .map_err(map_timeline_command_error)?;
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        let _ = state.events_tx.send(ServerEvent::HierarchyChanged);
        state.trigger_save();
    }
    crate::error::json_value(response)
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
    let path = active_project_path(&state)?;
    let removed_node_id = command.payload.node_id;
    let response = {
        let project = timeline_command_project(&state, &path).await?;
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome =
            timeline_command::record_delete_timeline_node_history(&mut conn, &project, &command, 0)
                .map_err(map_timeline_command_error)?;
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.doc_tx.try_send(DocCommand::RemoveNode {
            node_id: removed_node_id,
        });
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        let _ = state.events_tx.send(ServerEvent::HierarchyChanged);
        state.trigger_save();
    }
    crate::error::json_value(response)
}

fn map_timeline_command_error(error: TimelineCommandError) -> ApiError {
    match error {
        TimelineCommandError::Core(error) => ApiError::bad_request(error.to_string()),
        TimelineCommandError::History(error) => map_history_error(error),
    }
}

fn timeline_render_projection_from_current_state(
    conn: &Connection,
    fallback: &Timeline,
) -> Result<ProjectionEnvelope<TimelineRenderProjection>, TimelineCommandError> {
    let mut timeline = fallback.clone();
    let use_persisted_nodes = timeline_nodes_are_authoritative(conn)?;
    let use_persisted_relationships = timeline_relationships_are_authoritative(conn)?;
    let nodes = timeline_node_store::load_nodes(conn)?;
    if use_persisted_nodes || !nodes.is_empty() {
        timeline.nodes = nodes;
        timeline.node_arcs = timeline_node_store::load_node_arcs(conn)?;
    }
    let relationships = timeline_relationship_store::load_relationships(conn)?;
    if use_persisted_relationships || !relationships.is_empty() || fallback.relationships.is_empty()
    {
        timeline.relationships = relationships;
    }

    Ok(ProjectionEnvelope::initial(
        TimelineRenderProjection::from_timeline(&timeline),
    ))
}

fn timeline_nodes_are_authoritative(conn: &Connection) -> Result<bool, TimelineCommandError> {
    let node_revisions =
        history_store::load_revision_summary_for_kind(conn, ObjectKind::TimelineNode)?
            .revision_count;
    Ok(node_revisions > 0)
}

fn timeline_relationships_are_authoritative(
    conn: &Connection,
) -> Result<bool, TimelineCommandError> {
    let relationship_revisions =
        history_store::load_revision_summary_for_kind(conn, ObjectKind::TimelineRelationship)?
            .revision_count;
    Ok(relationship_revisions > 0)
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

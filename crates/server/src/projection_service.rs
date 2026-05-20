use std::path::PathBuf;

use eidetic_core::ai::backend::ChildPlanListProjection;
use eidetic_core::contracts::{
    BibleGraphNodeId, BibleGraphNodeListProjection, BibleGraphSchemaListProjection,
    BibleNodeDetailProjection, BibleReferenceProposalListProjection, BibleRenderGraphProjection,
    ChangeReviewProjection, ObjectKind, ProjectionEnvelope, PropagationProposalListProjection,
    ScriptDocumentId, ScriptDocumentProjection, SelectedNodeEditorProjection,
    SemanticDependencyProjection, StoryArcListProjection, StoryArcProgressionProjection,
    TimelineRenderProjection, builtin_bible_graph_schema_list_projection,
};
use eidetic_core::story::progression::analyze_all_arcs;
use eidetic_core::timeline::Timeline;
use eidetic_core::timeline::node::NodeId;
use serde::Deserialize;

use crate::backend_error::BackendError;
use crate::bible_graph_store;
use crate::child_plan_projection_store;
use crate::child_plan_store::ChildPlanStoreError;
use crate::history_store::{self, HistoryStoreError};
use crate::propagation_proposal_store::{self, PropagationProposalStoreError};
use crate::script_store;
use crate::semantic_dependency_store::{
    self, DependencyDirection, DependencyEndpointFilter, SemanticDependencyFilter,
    SemanticDependencyStoreError,
};
use crate::semantic_proposal_store::{self, SemanticProposalStoreError};
use crate::state::AppState;
use crate::story_arc_store;
use crate::timeline_node_store;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectFieldProjectionRequest {
    pub object_kind: ObjectKind,
    pub object_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptDocumentProjectionRequest {
    pub document_id: ScriptDocumentId,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BibleGraphNodeProjectionRequest {
    pub node_id: BibleGraphNodeId,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedNodeEditorProjectionRequest {
    pub node_id: Option<NodeId>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticDependencyProjectionRequest {
    pub source_kind: Option<String>,
    pub source_id: Option<String>,
    pub source_part_key: Option<String>,
    pub source_field_key: Option<String>,
    pub target_kind: Option<String>,
    pub target_id: Option<String>,
    pub target_part_key: Option<String>,
    pub target_field_key: Option<String>,
}

pub async fn object_field_projection(
    state: &AppState,
    request: ObjectFieldProjectionRequest,
) -> Result<serde_json::Value, BackendError> {
    if request.object_id.trim().is_empty() {
        return Err(BackendError::bad_request("object_id is required"));
    }

    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || {
        load_object_field_projection_value_at_path(path, request.object_kind, request.object_id)
    })
    .await
    .map_err(|error| {
        BackendError::internal(format!("object field projection task failed: {error}"))
    })?
}

pub async fn script_document_projection(
    state: &AppState,
    request: ScriptDocumentProjectionRequest,
) -> Result<ProjectionEnvelope<ScriptDocumentProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || {
        load_script_document_projection_at_path(path, request.document_id)
    })
    .await
    .map_err(|error| {
        BackendError::internal(format!("script document projection task failed: {error}"))
    })?
}

pub async fn bible_graph_node_projection(
    state: &AppState,
    request: BibleGraphNodeProjectionRequest,
) -> Result<ProjectionEnvelope<BibleNodeDetailProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_bible_node_projection_at_path(path, request.node_id))
        .await
        .map_err(|error| {
            BackendError::internal(format!("bible graph projection task failed: {error}"))
        })?
}

pub async fn bible_graph_node_list_projection(
    state: &AppState,
) -> Result<ProjectionEnvelope<BibleGraphNodeListProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_bible_node_list_projection_at_path(path))
        .await
        .map_err(|error| {
            BackendError::internal(format!("bible graph node list task failed: {error}"))
        })?
}

pub fn bible_graph_schema_list_projection(
    state: &AppState,
) -> Result<ProjectionEnvelope<BibleGraphSchemaListProjection>, BackendError> {
    let _ = active_project_path(state)?;
    Ok(builtin_bible_graph_schema_list_projection())
}

pub async fn bible_render_graph_projection(
    state: &AppState,
) -> Result<ProjectionEnvelope<BibleRenderGraphProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_bible_render_graph_projection_at_path(path))
        .await
        .map_err(|error| {
            BackendError::internal(format!("bible render graph task failed: {error}"))
        })?
}

pub async fn bible_reference_proposal_list_projection(
    state: &AppState,
) -> Result<ProjectionEnvelope<BibleReferenceProposalListProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_bible_reference_proposal_list_at_path(path))
        .await
        .map_err(|error| {
            BackendError::internal(format!("semantic proposal projection task failed: {error}"))
        })?
}

pub async fn propagation_proposal_list_projection(
    state: &AppState,
) -> Result<ProjectionEnvelope<PropagationProposalListProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_propagation_proposal_list_at_path(path))
        .await
        .map_err(|error| {
            BackendError::internal(format!(
                "propagation proposal projection task failed: {error}"
            ))
        })?
}

pub async fn semantic_dependency_projection(
    state: &AppState,
    request: SemanticDependencyProjectionRequest,
) -> Result<ProjectionEnvelope<SemanticDependencyProjection>, BackendError> {
    let path = active_project_path(state)?;
    let filter = semantic_dependency_filter_from_request(request)?;
    tokio::task::spawn_blocking(move || load_semantic_dependency_projection_at_path(path, filter))
        .await
        .map_err(|error| {
            BackendError::internal(format!(
                "semantic dependency projection task failed: {error}"
            ))
        })?
}

pub async fn child_plan_list_projection(
    state: &AppState,
) -> Result<ProjectionEnvelope<ChildPlanListProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_child_plan_list_projection_at_path(path))
        .await
        .map_err(|error| {
            BackendError::internal(format!("child plan projection task failed: {error}"))
        })?
}

pub async fn story_arc_list_projection(
    state: &AppState,
) -> Result<ProjectionEnvelope<StoryArcListProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_story_arc_list_projection_at_path(path))
        .await
        .map_err(|error| {
            BackendError::internal(format!("story arc list projection task failed: {error}"))
        })?
}

pub async fn story_arc_progression_projection(
    state: &AppState,
) -> Result<ProjectionEnvelope<StoryArcProgressionProjection>, BackendError> {
    let path = active_project_path(state)?;
    let arcs = tokio::task::spawn_blocking(move || load_story_arcs_at_path(path))
        .await
        .map_err(|error| {
            BackendError::internal(format!("story arc progression task failed: {error}"))
        })??;
    let guard = state.project.lock();
    let Some(project) = guard.as_ref() else {
        return Err(BackendError::no_project());
    };
    let mut projection_project = project.clone();
    projection_project.arcs = arcs;

    Ok(ProjectionEnvelope::initial(
        StoryArcProgressionProjection::new(analyze_all_arcs(&projection_project)),
    ))
}

pub async fn change_review_projection(
    state: &AppState,
) -> Result<ProjectionEnvelope<ChangeReviewProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_change_review_projection_at_path(path))
        .await
        .map_err(|error| {
            BackendError::internal(format!("change review projection task failed: {error}"))
        })?
}

pub async fn timeline_render_projection(
    state: &AppState,
) -> Result<ProjectionEnvelope<TimelineRenderProjection>, BackendError> {
    let path = active_project_path(state)?;
    let (project, _) = crate::persistence::load_project(&path)
        .await
        .map_err(BackendError::internal)?;

    Ok(ProjectionEnvelope::initial(
        TimelineRenderProjection::from_timeline(&project.timeline),
    ))
}

pub async fn selected_node_editor_projection(
    state: &AppState,
    request: SelectedNodeEditorProjectionRequest,
) -> Result<ProjectionEnvelope<SelectedNodeEditorProjection>, BackendError> {
    let path = active_project_path(state)?;
    let (project, _) = crate::persistence::load_project(&path)
        .await
        .map_err(BackendError::internal)?;
    let fallback_timeline = project.timeline;
    tokio::task::spawn_blocking(move || {
        load_selected_node_editor_at_path(path, fallback_timeline, request.node_id)
    })
    .await
    .map_err(|error| {
        BackendError::internal(format!("selected node projection task failed: {error}"))
    })?
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

fn load_object_field_projection_value_at_path(
    path: PathBuf,
    object_kind: ObjectKind,
    object_id: String,
) -> Result<serde_json::Value, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    history_store::create_schema(&conn).map_err(map_history_error)?;
    let projection = crate::revision_projection::load_object_field_projection_envelope(
        &conn,
        object_kind,
        &object_id,
    )
    .map_err(map_history_error)?;
    serde_json::to_value(projection).map_err(|e| BackendError::internal(e.to_string()))
}

fn load_bible_node_projection_at_path(
    path: PathBuf,
    node_id: BibleGraphNodeId,
) -> Result<ProjectionEnvelope<BibleNodeDetailProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    bible_graph_store::create_schema(&conn).map_err(map_history_error)?;
    bible_graph_store::load_node_detail_projection_envelope(&conn, &node_id)
        .map_err(map_history_error)?
        .ok_or_else(|| BackendError::not_found("bible graph node not found"))
}

fn load_bible_node_list_projection_at_path(
    path: PathBuf,
) -> Result<ProjectionEnvelope<BibleGraphNodeListProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    bible_graph_store::create_schema(&conn).map_err(map_history_error)?;
    bible_graph_store::load_node_list_projection_envelope(&conn).map_err(map_history_error)
}

fn load_bible_render_graph_projection_at_path(
    path: PathBuf,
) -> Result<ProjectionEnvelope<BibleRenderGraphProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    bible_graph_store::create_schema(&conn).map_err(map_history_error)?;
    bible_graph_store::load_render_graph_projection_envelope(&conn).map_err(map_history_error)
}

fn load_bible_reference_proposal_list_at_path(
    path: PathBuf,
) -> Result<ProjectionEnvelope<BibleReferenceProposalListProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    semantic_proposal_store::load_bible_reference_proposal_list_projection(&conn)
        .map_err(map_semantic_proposal_error)
}

fn load_propagation_proposal_list_at_path(
    path: PathBuf,
) -> Result<ProjectionEnvelope<PropagationProposalListProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
        .map_err(map_propagation_proposal_error)
}

fn load_semantic_dependency_projection_at_path(
    path: PathBuf,
    filter: SemanticDependencyFilter,
) -> Result<ProjectionEnvelope<SemanticDependencyProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    semantic_dependency_store::load_semantic_dependency_projection(&conn, &filter)
        .map_err(map_semantic_dependency_error)
}

fn load_child_plan_list_projection_at_path(
    path: PathBuf,
) -> Result<ProjectionEnvelope<ChildPlanListProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    child_plan_projection_store::load_child_plan_list_projection(&conn)
        .map_err(map_child_plan_error)
}

fn load_script_document_projection_at_path(
    path: PathBuf,
    document_id: ScriptDocumentId,
) -> Result<ProjectionEnvelope<ScriptDocumentProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    script_store::create_schema(&conn).map_err(map_history_error)?;
    script_store::load_document_projection_envelope(&conn, &document_id)
        .map_err(map_history_error)?
        .ok_or_else(|| BackendError::not_found("script document not found"))
}

fn load_story_arc_list_projection_at_path(
    path: PathBuf,
) -> Result<ProjectionEnvelope<StoryArcListProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    story_arc_store::create_schema(&conn).map_err(map_history_error)?;
    story_arc_store::load_arc_list_projection_envelope(&conn).map_err(map_history_error)
}

fn load_story_arcs_at_path(
    path: PathBuf,
) -> Result<Vec<eidetic_core::story::arc::StoryArc>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    story_arc_store::create_schema(&conn).map_err(map_history_error)?;
    story_arc_store::load_arcs(&conn).map_err(map_history_error)
}

fn load_change_review_projection_at_path(
    path: PathBuf,
) -> Result<ProjectionEnvelope<ChangeReviewProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    crate::change_review_projection::load_change_review_projection_envelope(&conn)
        .map_err(map_history_error)
}

fn load_selected_node_editor_at_path(
    path: PathBuf,
    fallback_timeline: Timeline,
    node_id: Option<NodeId>,
) -> Result<ProjectionEnvelope<SelectedNodeEditorProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    history_store::create_schema(&conn).map_err(map_history_error)?;
    let mut timeline = fallback_timeline;
    let nodes = timeline_node_store::load_nodes(&conn).map_err(map_history_error)?;
    let node_summary =
        history_store::load_revision_summary_for_kind(&conn, ObjectKind::TimelineNode)
            .map_err(map_history_error)?;
    if node_summary.revision_count > 0 || !nodes.is_empty() {
        timeline.nodes = nodes;
        timeline.node_arcs =
            timeline_node_store::load_node_arcs(&conn).map_err(map_history_error)?;
    }

    let projection = SelectedNodeEditorProjection::from_timeline(&timeline, node_id)
        .ok_or_else(|| BackendError::not_found("timeline node not found"))?;

    Ok(ProjectionEnvelope::initial(projection))
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

fn map_semantic_proposal_error(error: SemanticProposalStoreError) -> BackendError {
    match error {
        SemanticProposalStoreError::InvalidCommand(message) => BackendError::bad_request(message),
        SemanticProposalStoreError::NotFound(message) => BackendError::not_found(message),
        SemanticProposalStoreError::History(error) => map_history_error(error),
        SemanticProposalStoreError::Sqlite(error) => BackendError::internal(error.to_string()),
    }
}

fn map_propagation_proposal_error(error: PropagationProposalStoreError) -> BackendError {
    match error {
        PropagationProposalStoreError::InvalidCommand(message) => {
            BackendError::bad_request(message)
        }
        PropagationProposalStoreError::NotFound(message) => BackendError::not_found(message),
        PropagationProposalStoreError::History(error) => map_history_error(error),
        PropagationProposalStoreError::Sqlite(error) => BackendError::internal(error.to_string()),
        PropagationProposalStoreError::Json(error) => BackendError::bad_request(error.to_string()),
        PropagationProposalStoreError::Contract(error) => {
            BackendError::bad_request(error.to_string())
        }
        PropagationProposalStoreError::BibleGraphContract(error) => {
            BackendError::bad_request(error.to_string())
        }
        PropagationProposalStoreError::BibleGraphCommand(error) => {
            BackendError::bad_request(error.to_string())
        }
        PropagationProposalStoreError::ScriptContract(error) => {
            BackendError::bad_request(error.to_string())
        }
        PropagationProposalStoreError::SemanticDependencyContract(error) => {
            BackendError::bad_request(error.to_string())
        }
        PropagationProposalStoreError::Target(error) => {
            BackendError::bad_request(error.to_string())
        }
        PropagationProposalStoreError::ScriptDocumentCommand(error) => {
            BackendError::bad_request(error.to_string())
        }
    }
}

fn semantic_dependency_filter_from_request(
    request: SemanticDependencyProjectionRequest,
) -> Result<SemanticDependencyFilter, BackendError> {
    let source = endpoint_filter(
        request.source_kind,
        request.source_id,
        request.source_part_key,
        request.source_field_key,
    )?;
    let target = endpoint_filter(
        request.target_kind,
        request.target_id,
        request.target_part_key,
        request.target_field_key,
    )?;

    match (source, target) {
        (Some(endpoint), None) => Ok(SemanticDependencyFilter {
            endpoint,
            direction: DependencyDirection::Source,
        }),
        (None, Some(endpoint)) => Ok(SemanticDependencyFilter {
            endpoint,
            direction: DependencyDirection::Target,
        }),
        (None, None) => Err(BackendError::bad_request(
            "semantic dependency projection requires a source or target filter",
        )),
        (Some(_), Some(_)) => Err(BackendError::bad_request(
            "semantic dependency projection accepts only one source or target filter",
        )),
    }
}

fn endpoint_filter(
    kind: Option<String>,
    id: Option<String>,
    part_key: Option<String>,
    field_key: Option<String>,
) -> Result<Option<DependencyEndpointFilter>, BackendError> {
    match (kind, id) {
        (None, None) if part_key.is_none() && field_key.is_none() => Ok(None),
        (Some(kind), Some(id)) if !kind.trim().is_empty() && !id.trim().is_empty() => {
            validate_endpoint_filter(&kind, part_key.as_deref(), field_key.as_deref())?;
            Ok(Some(DependencyEndpointFilter {
                kind,
                id,
                part_key,
                field_key,
            }))
        }
        _ => Err(BackendError::bad_request(
            "semantic dependency filter requires kind and id",
        )),
    }
}

fn validate_endpoint_filter(
    kind: &str,
    part_key: Option<&str>,
    field_key: Option<&str>,
) -> Result<(), BackendError> {
    if !matches!(
        kind,
        "timeline_node" | "bible_node" | "bible_field" | "script_segment" | "script_block"
    ) {
        return Err(BackendError::bad_request(format!(
            "unknown semantic dependency endpoint kind: {kind}"
        )));
    }
    if (part_key.is_some() || field_key.is_some()) && kind != "bible_field" {
        return Err(BackendError::bad_request(
            "semantic dependency part and field filters require bible_field kind",
        ));
    }
    Ok(())
}

fn map_semantic_dependency_error(error: SemanticDependencyStoreError) -> BackendError {
    match error {
        SemanticDependencyStoreError::InvalidCommand(message) => BackendError::bad_request(message),
        SemanticDependencyStoreError::History(error) => map_history_error(error),
        SemanticDependencyStoreError::Sqlite(error) => BackendError::internal(error.to_string()),
        SemanticDependencyStoreError::Json(error) => BackendError::bad_request(error.to_string()),
        SemanticDependencyStoreError::Contract(error) => {
            BackendError::bad_request(error.to_string())
        }
        SemanticDependencyStoreError::BibleGraphContract(error) => {
            BackendError::bad_request(error.to_string())
        }
        SemanticDependencyStoreError::ScriptContract(error) => {
            BackendError::bad_request(error.to_string())
        }
    }
}

fn map_child_plan_error(error: ChildPlanStoreError) -> BackendError {
    match error {
        ChildPlanStoreError::InvalidCommand(message) => BackendError::bad_request(message),
        ChildPlanStoreError::NotFound(message) => BackendError::not_found(message),
        ChildPlanStoreError::History(error) => map_history_error(error),
        ChildPlanStoreError::Sqlite(error) => BackendError::internal(error.to_string()),
    }
}

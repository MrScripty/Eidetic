use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use eidetic_core::contracts::{
    BibleGraphNodeId, BibleGraphNodeListProjection, BibleGraphSchemaListProjection,
    BibleNodeDetailProjection, BibleRenderGraphProjection, ChangeReviewProjection, ObjectKind,
    ProjectionEnvelope, ScriptDocumentId, ScriptDocumentProjection, SelectedNodeEditorProjection,
    StoryArcListProjection, StoryArcProgressionProjection, TimelineRenderProjection,
    builtin_bible_graph_schema_list_projection,
};
use eidetic_core::story::progression::analyze_all_arcs;
use eidetic_core::timeline::Timeline;
use eidetic_core::timeline::node::NodeId;
use serde::Deserialize;

use crate::bible_graph_store;
use crate::error::{ApiError, ApiJson};
use crate::history_store;
use crate::script_store;
use crate::state::AppState;
use crate::story_arc_store;
use crate::timeline_node_store;

use super::support::{active_project_path, map_history_error};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/projections/object-field",
            get(get_object_field_projection),
        )
        .route(
            "/projections/bible-graph/node",
            get(get_bible_graph_node_projection),
        )
        .route(
            "/projections/bible-graph/nodes",
            get(get_bible_graph_node_list_projection),
        )
        .route(
            "/projections/bible-graph/schemas",
            get(get_bible_graph_schema_list_projection),
        )
        .route(
            "/projections/bible-graph/render",
            get(get_bible_render_graph_projection),
        )
        .route(
            "/projections/script/document",
            get(get_script_document_projection),
        )
        .route(
            "/projections/story/arcs",
            get(get_story_arc_list_projection),
        )
        .route(
            "/projections/story/arc-progression",
            get(get_story_arc_progression_projection),
        )
        .route(
            "/projections/timeline/render",
            get(get_timeline_render_projection),
        )
        .route(
            "/projections/timeline/selected-node",
            get(get_selected_node_editor_projection),
        )
        .route(
            "/projections/history/changes",
            get(get_change_review_projection),
        )
}

#[derive(Debug, Deserialize)]
struct BibleGraphNodeProjectionQuery {
    node_id: BibleGraphNodeId,
}

#[derive(Debug, Deserialize)]
struct ScriptDocumentProjectionQuery {
    document_id: ScriptDocumentId,
}

#[derive(Debug, Deserialize)]
struct SelectedNodeEditorProjectionQuery {
    node_id: Option<NodeId>,
}

async fn get_object_field_projection(
    State(state): State<AppState>,
    Query(query): Query<crate::projection_service::ObjectFieldProjectionRequest>,
) -> ApiJson {
    crate::projection_service::object_field_projection(&state, query)
        .await
        .map_err(ApiError::from)
        .map(axum::Json)
}

async fn get_bible_graph_node_projection(
    State(state): State<AppState>,
    Query(query): Query<BibleGraphNodeProjectionQuery>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let projection = tokio::task::spawn_blocking(move || {
        load_bible_node_projection_at_path(path, query.node_id)
    })
    .await
    .map_err(|e| ApiError::internal(format!("bible graph projection task failed: {e}")))??;

    crate::error::json_value(projection)
}

async fn get_bible_graph_node_list_projection(State(state): State<AppState>) -> ApiJson {
    let path = active_project_path(&state)?;
    let projection = tokio::task::spawn_blocking(move || load_bible_node_list_at_path(path))
        .await
        .map_err(|e| ApiError::internal(format!("bible graph node list task failed: {e}")))??;

    crate::error::json_value(projection)
}

async fn get_bible_graph_schema_list_projection(State(state): State<AppState>) -> ApiJson {
    let _ = active_project_path(&state)?;
    crate::error::json_value(load_bible_schema_list_projection())
}

async fn get_bible_render_graph_projection(State(state): State<AppState>) -> ApiJson {
    let path = active_project_path(&state)?;
    let projection = tokio::task::spawn_blocking(move || load_bible_render_graph_at_path(path))
        .await
        .map_err(|e| ApiError::internal(format!("bible render graph task failed: {e}")))??;

    crate::error::json_value(projection)
}

async fn get_change_review_projection(State(state): State<AppState>) -> ApiJson {
    let path = active_project_path(&state)?;
    let projection = tokio::task::spawn_blocking(move || load_change_review_at_path(path))
        .await
        .map_err(|e| ApiError::internal(format!("change review projection task failed: {e}")))??;

    crate::error::json_value(projection)
}

async fn get_script_document_projection(
    State(state): State<AppState>,
    Query(query): Query<ScriptDocumentProjectionQuery>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let projection = tokio::task::spawn_blocking(move || {
        load_script_document_projection_at_path(path, query.document_id)
    })
    .await
    .map_err(|e| ApiError::internal(format!("script document projection task failed: {e}")))??;

    crate::error::json_value(projection)
}

async fn get_timeline_render_projection(State(state): State<AppState>) -> ApiJson {
    let path = active_project_path(&state)?;
    let (project, _) = crate::persistence::load_project(&path)
        .await
        .map_err(ApiError::internal)?;

    crate::error::json_value(ProjectionEnvelope::initial(
        TimelineRenderProjection::from_timeline(&project.timeline),
    ))
}

async fn get_selected_node_editor_projection(
    State(state): State<AppState>,
    Query(query): Query<SelectedNodeEditorProjectionQuery>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let (project, _) = crate::persistence::load_project(&path)
        .await
        .map_err(ApiError::internal)?;
    let fallback_timeline = project.timeline;
    let projection = tokio::task::spawn_blocking(move || {
        load_selected_node_editor_at_path(path, fallback_timeline, query.node_id)
    })
    .await
    .map_err(|e| ApiError::internal(format!("selected node projection task failed: {e}")))??;

    crate::error::json_value(projection)
}

async fn get_story_arc_list_projection(State(state): State<AppState>) -> ApiJson {
    let path = active_project_path(&state)?;
    let projection = tokio::task::spawn_blocking(move || load_story_arc_list_at_path(path))
        .await
        .map_err(|e| ApiError::internal(format!("story arc list task failed: {e}")))??;

    crate::error::json_value(projection)
}

async fn get_story_arc_progression_projection(State(state): State<AppState>) -> ApiJson {
    let path = active_project_path(&state)?;
    let arcs = tokio::task::spawn_blocking(move || load_story_arcs_at_path(path))
        .await
        .map_err(|e| ApiError::internal(format!("story arc progression task failed: {e}")))??;
    let guard = state.project.lock();
    let Some(project) = guard.as_ref() else {
        return Err(ApiError::no_project());
    };
    let mut projection_project = project.clone();
    projection_project.arcs = arcs;

    crate::error::json_value(ProjectionEnvelope::initial(
        StoryArcProgressionProjection::new(analyze_all_arcs(&projection_project)),
    ))
}

fn load_bible_node_projection_at_path(
    path: std::path::PathBuf,
    node_id: BibleGraphNodeId,
) -> Result<ProjectionEnvelope<BibleNodeDetailProjection>, ApiError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    bible_graph_store::create_schema(&conn).map_err(map_history_error)?;
    bible_graph_store::load_node_detail_projection_envelope(&conn, &node_id)
        .map_err(map_history_error)?
        .ok_or_else(|| ApiError::not_found("bible graph node not found"))
}

fn load_bible_node_list_at_path(
    path: std::path::PathBuf,
) -> Result<ProjectionEnvelope<BibleGraphNodeListProjection>, ApiError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    bible_graph_store::create_schema(&conn).map_err(map_history_error)?;
    bible_graph_store::load_node_list_projection_envelope(&conn).map_err(map_history_error)
}

fn load_bible_schema_list_projection() -> ProjectionEnvelope<BibleGraphSchemaListProjection> {
    builtin_bible_graph_schema_list_projection()
}

fn load_bible_render_graph_at_path(
    path: std::path::PathBuf,
) -> Result<ProjectionEnvelope<BibleRenderGraphProjection>, ApiError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    bible_graph_store::create_schema(&conn).map_err(map_history_error)?;
    bible_graph_store::load_render_graph_projection_envelope(&conn).map_err(map_history_error)
}

fn load_change_review_at_path(
    path: std::path::PathBuf,
) -> Result<ProjectionEnvelope<ChangeReviewProjection>, ApiError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    crate::change_review_projection::load_change_review_projection_envelope(&conn)
        .map_err(map_history_error)
}

fn load_story_arc_list_at_path(
    path: std::path::PathBuf,
) -> Result<ProjectionEnvelope<StoryArcListProjection>, ApiError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    story_arc_store::create_schema(&conn).map_err(map_history_error)?;
    story_arc_store::load_arc_list_projection_envelope(&conn).map_err(map_history_error)
}

fn load_story_arcs_at_path(
    path: std::path::PathBuf,
) -> Result<Vec<eidetic_core::story::arc::StoryArc>, ApiError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    story_arc_store::create_schema(&conn).map_err(map_history_error)?;
    story_arc_store::load_arcs(&conn).map_err(map_history_error)
}

fn load_script_document_projection_at_path(
    path: std::path::PathBuf,
    document_id: ScriptDocumentId,
) -> Result<ProjectionEnvelope<ScriptDocumentProjection>, ApiError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    script_store::create_schema(&conn).map_err(map_history_error)?;
    script_store::load_document_projection_envelope(&conn, &document_id)
        .map_err(map_history_error)?
        .ok_or_else(|| ApiError::not_found("script document not found"))
}

fn load_selected_node_editor_at_path(
    path: std::path::PathBuf,
    fallback_timeline: Timeline,
    node_id: Option<NodeId>,
) -> Result<ProjectionEnvelope<SelectedNodeEditorProjection>, ApiError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
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
        .ok_or_else(|| ApiError::not_found("timeline node not found"))?;

    Ok(ProjectionEnvelope::initial(projection))
}

#[cfg(test)]
#[path = "projections_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "projections_story_tests.rs"]
mod story_tests;

#[cfg(test)]
#[path = "projections_history_tests.rs"]
mod history_tests;

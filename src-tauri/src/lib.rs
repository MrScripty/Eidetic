use eidetic_core::ai::backend::ChildPlanListProjection;
use eidetic_core::contracts::{
    AcceptBibleReferenceProposalCommand, AcceptPropagationProposalCommand,
    BibleGraphNodeListProjection, BibleGraphSchemaListProjection, BibleNodeDetailProjection,
    BibleReferenceProposalListProjection, BibleRenderGraphProjection, ChangeReviewProjection,
    CommandEnvelope, CreateBibleReferenceProposalCommand, CreatePropagationProposalCommand,
    DeleteStoryArcCommand, EnsureCanonicalBibleRootsCommand, ProjectionEnvelope,
    PropagationProposalListProjection, RejectBibleReferenceProposalCommand,
    RejectPropagationProposalCommand, ScriptDocumentProjection, SelectedNodeEditorProjection,
    SemanticDependencyProjection, SetBibleGraphFieldCommand, SetObjectFieldCommand,
    SetScriptBlockCommand, SetScriptLockCommand, SetStoryArcMetadataCommand,
    SetTimelineNodeLockCommand, StoryArcListProjection, StoryArcProgressionProjection,
    TimelineRenderProjection, UpdatePropagationProposalCommand,
};
use eidetic_server::backend_error::BackendError;
use eidetic_server::command_service::{self, CreateStoryArcRequestCommand};
use eidetic_server::project_service::{
    self, CreateProjectRequest, LoadProjectRequest, SaveProjectRequest, UpdateProjectRequest,
};
use eidetic_server::projection_service::{
    self, BibleGraphNodeProjectionRequest, ObjectFieldProjectionRequest,
    ScriptDocumentProjectionRequest, SelectedNodeEditorProjectionRequest,
    SemanticDependencyProjectionRequest,
};
use eidetic_server::state::AppState;
use serde::Serialize;
use tauri::Manager;

#[derive(Serialize)]
struct DesktopHealth {
    status: &'static str,
    boundary: &'static str,
}

#[derive(Debug, Serialize)]
struct CommandError {
    kind: &'static str,
    message: String,
}

impl From<BackendError> for CommandError {
    fn from(error: BackendError) -> Self {
        let kind = match error {
            BackendError::NotFound(_) => "not_found",
            BackendError::BadRequest(_) => "bad_request",
            BackendError::Conflict(_) => "conflict",
            BackendError::Internal(_) => "internal",
        };

        Self {
            kind,
            message: error.message().to_string(),
        }
    }
}

#[tauri::command]
fn desktop_health() -> DesktopHealth {
    DesktopHealth {
        status: "ok",
        boundary: "tauri",
    }
}

#[tauri::command]
async fn project_create(
    app: tauri::AppHandle,
    name: String,
    template: String,
) -> Result<serde_json::Value, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    project_service::create_project(&state, CreateProjectRequest { name, template })
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
fn project_get(app: tauri::AppHandle) -> Result<serde_json::Value, CommandError> {
    let state = app.state::<AppState>();
    project_service::get_project(&state).map_err(CommandError::from)
}

#[tauri::command]
fn project_update(
    app: tauri::AppHandle,
    name: Option<String>,
    premise: Option<String>,
) -> Result<serde_json::Value, CommandError> {
    let state = app.state::<AppState>();
    project_service::update_project(&state, UpdateProjectRequest { name, premise })
        .map_err(CommandError::from)
}

#[tauri::command]
async fn project_save(
    app: tauri::AppHandle,
    path: Option<String>,
) -> Result<serde_json::Value, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    project_service::save_project(&state, SaveProjectRequest { path })
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn project_load(
    app: tauri::AppHandle,
    path: String,
) -> Result<serde_json::Value, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    project_service::load_project(&state, LoadProjectRequest { path })
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn project_list() -> serde_json::Value {
    project_service::list_projects().await
}

#[tauri::command]
async fn command_object_field(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetObjectFieldCommand>,
) -> Result<command_service::ObjectFieldCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_object_field(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_script_block(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetScriptBlockCommand>,
) -> Result<command_service::ScriptDocumentCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_script_block(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_script_lock(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetScriptLockCommand>,
) -> Result<command_service::ScriptDocumentCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_script_lock(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_story_create(
    app: tauri::AppHandle,
    command: CreateStoryArcRequestCommand,
) -> Result<command_service::StoryArcCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::create_story_arc(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_story_update(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetStoryArcMetadataCommand>,
) -> Result<command_service::StoryArcCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::update_story_arc(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_story_delete(
    app: tauri::AppHandle,
    command: CommandEnvelope<DeleteStoryArcCommand>,
) -> Result<command_service::StoryArcCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::delete_story_arc(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_bible_graph_node(
    app: tauri::AppHandle,
    command: command_service::CreateBibleGraphNodeRequestCommand,
) -> Result<command_service::BibleGraphNodeCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::create_bible_graph_node(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_bible_graph_field(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetBibleGraphFieldCommand>,
) -> Result<command_service::BibleGraphNodeCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_bible_graph_field(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_bible_graph_edge(
    app: tauri::AppHandle,
    command: command_service::SetBibleGraphEdgeRequestCommand,
) -> Result<command_service::BibleGraphNodeCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_bible_graph_edge(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_bible_graph_snapshot_field(
    app: tauri::AppHandle,
    command: command_service::SetBibleGraphSnapshotFieldRequestCommand,
) -> Result<command_service::BibleGraphNodeCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_bible_graph_snapshot_field(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_bible_graph_roots(
    app: tauri::AppHandle,
    command: CommandEnvelope<EnsureCanonicalBibleRootsCommand>,
) -> Result<command_service::BibleGraphRootsCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::ensure_canonical_bible_roots(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_bible_reference_proposal_create(
    app: tauri::AppHandle,
    command: CommandEnvelope<CreateBibleReferenceProposalCommand>,
) -> Result<command_service::BibleReferenceProposalCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::create_bible_reference_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_bible_reference_proposal_reject(
    app: tauri::AppHandle,
    command: CommandEnvelope<RejectBibleReferenceProposalCommand>,
) -> Result<command_service::BibleReferenceProposalCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::reject_bible_reference_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_bible_reference_proposal_accept(
    app: tauri::AppHandle,
    command: CommandEnvelope<AcceptBibleReferenceProposalCommand>,
) -> Result<command_service::BibleReferenceProposalCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::accept_bible_reference_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_propagation_proposal_create(
    app: tauri::AppHandle,
    command: CommandEnvelope<CreatePropagationProposalCommand>,
) -> Result<command_service::PropagationProposalCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::create_propagation_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_propagation_proposal_reject(
    app: tauri::AppHandle,
    command: CommandEnvelope<RejectPropagationProposalCommand>,
) -> Result<command_service::PropagationProposalCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::reject_propagation_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_propagation_proposal_update(
    app: tauri::AppHandle,
    command: CommandEnvelope<UpdatePropagationProposalCommand>,
) -> Result<command_service::PropagationProposalCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::update_propagation_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_propagation_proposal_accept(
    app: tauri::AppHandle,
    command: CommandEnvelope<AcceptPropagationProposalCommand>,
) -> Result<command_service::PropagationProposalCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::accept_propagation_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn command_timeline_node_lock(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetTimelineNodeLockCommand>,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_timeline_node_lock(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn projection_object_field(
    app: tauri::AppHandle,
    query: ObjectFieldProjectionRequest,
) -> Result<serde_json::Value, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::object_field_projection(&state, query)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn projection_script_document(
    app: tauri::AppHandle,
    query: ScriptDocumentProjectionRequest,
) -> Result<ProjectionEnvelope<ScriptDocumentProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::script_document_projection(&state, query)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn projection_bible_graph_node(
    app: tauri::AppHandle,
    query: BibleGraphNodeProjectionRequest,
) -> Result<ProjectionEnvelope<BibleNodeDetailProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::bible_graph_node_projection(&state, query)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn projection_bible_graph_nodes(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<BibleGraphNodeListProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::bible_graph_node_list_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
fn projection_bible_graph_schemas(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<BibleGraphSchemaListProjection>, CommandError> {
    let state = app.state::<AppState>();
    projection_service::bible_graph_schema_list_projection(&state).map_err(CommandError::from)
}

#[tauri::command]
async fn projection_bible_render_graph(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<BibleRenderGraphProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::bible_render_graph_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn projection_bible_reference_proposals(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<BibleReferenceProposalListProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::bible_reference_proposal_list_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn projection_propagation_proposals(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<PropagationProposalListProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::propagation_proposal_list_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn projection_semantic_dependencies(
    app: tauri::AppHandle,
    query: SemanticDependencyProjectionRequest,
) -> Result<ProjectionEnvelope<SemanticDependencyProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::semantic_dependency_projection(&state, query)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn projection_child_plans(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<ChildPlanListProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::child_plan_list_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn projection_story_arcs(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<StoryArcListProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::story_arc_list_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn projection_story_arc_progression(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<StoryArcProgressionProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::story_arc_progression_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn projection_change_review(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<ChangeReviewProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::change_review_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn projection_timeline_render(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<TimelineRenderProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::timeline_render_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
async fn projection_selected_node(
    app: tauri::AppHandle,
    query: SelectedNodeEditorProjectionRequest,
) -> Result<ProjectionEnvelope<SelectedNodeEditorProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::selected_node_editor_projection(&state, query)
        .await
        .map_err(CommandError::from)
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_state = tauri::async_runtime::block_on(AppState::new());
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop_health,
            project_create,
            project_get,
            project_update,
            project_save,
            project_load,
            project_list,
            command_object_field,
            command_script_block,
            command_script_lock,
            command_story_create,
            command_story_update,
            command_story_delete,
            command_bible_graph_node,
            command_bible_graph_field,
            command_bible_graph_edge,
            command_bible_graph_snapshot_field,
            command_bible_graph_roots,
            command_bible_reference_proposal_create,
            command_bible_reference_proposal_reject,
            command_bible_reference_proposal_accept,
            command_propagation_proposal_create,
            command_propagation_proposal_reject,
            command_propagation_proposal_update,
            command_propagation_proposal_accept,
            command_timeline_node_lock,
            projection_object_field,
            projection_script_document,
            projection_bible_graph_node,
            projection_bible_graph_nodes,
            projection_bible_graph_schemas,
            projection_bible_render_graph,
            projection_bible_reference_proposals,
            projection_propagation_proposals,
            projection_semantic_dependencies,
            projection_child_plans,
            projection_story_arcs,
            projection_story_arc_progression,
            projection_change_review,
            projection_timeline_render,
            projection_selected_node
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Eidetic desktop application");
}

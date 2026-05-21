use eidetic_core::ai::backend::ChildPlanListProjection;
use eidetic_core::contracts::{
    BibleReferenceProposalListProjection, ProjectionEnvelope, PropagationProposalListProjection,
    SemanticDependencyProjection,
};
use eidetic_server::projection_service::{self, SemanticDependencyProjectionRequest};
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[tauri::command]
pub async fn projection_bible_reference_proposals(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<BibleReferenceProposalListProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::bible_reference_proposal_list_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn projection_propagation_proposals(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<PropagationProposalListProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::propagation_proposal_list_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn projection_semantic_dependencies(
    app: tauri::AppHandle,
    query: SemanticDependencyProjectionRequest,
) -> Result<ProjectionEnvelope<SemanticDependencyProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::semantic_dependency_projection(&state, query)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn projection_child_plans(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<ChildPlanListProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::child_plan_list_projection(&state)
        .await
        .map_err(CommandError::from)
}

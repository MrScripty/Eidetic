use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use eidetic_core::contracts::{
    AcceptBibleReferenceProposalCommand, AcceptPropagationProposalCommand, CommandEnvelope,
    CreateBibleReferenceProposalCommand, CreatePropagationProposalCommand, ProjectionEnvelope,
    RecordSemanticDependencyCommand, RejectBibleReferenceProposalCommand,
    RejectPropagationProposalCommand, SemanticDependencyProjection,
    UpdatePropagationProposalCommand,
};
use serde::Serialize;

use crate::error::{ApiError, ApiJson};
use crate::history_store::RecordChangeOutcome;
use crate::semantic_dependency_store::{
    self, DependencyDirection, DependencyEndpointFilter, SemanticDependencyFilter,
    SemanticDependencyStoreError,
};
use crate::state::AppState;

use super::support::{active_project_path, map_history_error};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/commands/semantic/bible-reference-proposal",
            post(create_bible_reference_proposal),
        )
        .route(
            "/commands/semantic/bible-reference-proposal/reject",
            post(reject_bible_reference_proposal),
        )
        .route(
            "/commands/semantic/bible-reference-proposal/accept",
            post(accept_bible_reference_proposal),
        )
        .route(
            "/commands/semantic/dependency",
            post(record_semantic_dependency),
        )
        .route(
            "/commands/semantic/propagation-proposal",
            post(create_propagation_proposal),
        )
        .route(
            "/commands/semantic/propagation-proposal/reject",
            post(reject_propagation_proposal),
        )
        .route(
            "/commands/semantic/propagation-proposal/update",
            post(update_propagation_proposal),
        )
        .route(
            "/commands/semantic/propagation-proposal/accept",
            post(accept_propagation_proposal),
        )
}

#[derive(Debug, Serialize)]
struct SemanticDependencyCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<SemanticDependencyProjection>,
}

async fn create_bible_reference_proposal(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<CreateBibleReferenceProposalCommand>>,
) -> ApiJson {
    crate::command_service::create_bible_reference_proposal(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn reject_bible_reference_proposal(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<RejectBibleReferenceProposalCommand>>,
) -> ApiJson {
    crate::command_service::reject_bible_reference_proposal(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn accept_bible_reference_proposal(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<AcceptBibleReferenceProposalCommand>>,
) -> ApiJson {
    crate::command_service::accept_bible_reference_proposal(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn record_semantic_dependency(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<RecordSemanticDependencyCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response =
        tokio::task::spawn_blocking(move || record_semantic_dependency_at_path(path, command))
            .await
            .map_err(|e| {
                ApiError::internal(format!("semantic dependency command task failed: {e}"))
            })??;

    crate::error::json_value(response)
}

async fn create_propagation_proposal(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<CreatePropagationProposalCommand>>,
) -> ApiJson {
    crate::command_service::create_propagation_proposal(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn reject_propagation_proposal(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<RejectPropagationProposalCommand>>,
) -> ApiJson {
    crate::command_service::reject_propagation_proposal(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn update_propagation_proposal(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<UpdatePropagationProposalCommand>>,
) -> ApiJson {
    crate::command_service::update_propagation_proposal(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn accept_propagation_proposal(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<AcceptPropagationProposalCommand>>,
) -> ApiJson {
    crate::command_service::accept_propagation_proposal(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

fn record_semantic_dependency_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<RecordSemanticDependencyCommand>,
) -> Result<SemanticDependencyCommandResponse, ApiError> {
    let filter = SemanticDependencyFilter {
        endpoint: DependencyEndpointFilter::from_endpoint(&command.payload.dependency.source),
        direction: DependencyDirection::Source,
    };
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let outcome = semantic_dependency_store::record_semantic_dependency(&mut conn, &command, 0)
        .map_err(map_semantic_dependency_error)?;
    let projection = semantic_dependency_store::load_semantic_dependency_projection(&conn, &filter)
        .map_err(map_semantic_dependency_error)?;

    Ok(SemanticDependencyCommandResponse {
        outcome,
        projection,
    })
}

fn map_semantic_dependency_error(error: SemanticDependencyStoreError) -> ApiError {
    match error {
        SemanticDependencyStoreError::InvalidCommand(message) => ApiError::bad_request(message),
        SemanticDependencyStoreError::History(error) => map_history_error(error),
        SemanticDependencyStoreError::Sqlite(error) => ApiError::internal(error.to_string()),
        SemanticDependencyStoreError::Json(error) => ApiError::bad_request(error.to_string()),
        SemanticDependencyStoreError::Contract(error) => ApiError::bad_request(error.to_string()),
        SemanticDependencyStoreError::BibleGraphContract(error) => {
            ApiError::bad_request(error.to_string())
        }
        SemanticDependencyStoreError::ScriptContract(error) => {
            ApiError::bad_request(error.to_string())
        }
    }
}

#[cfg(test)]
#[path = "commands_semantic_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "commands_semantic_propagation_tests.rs"]
mod propagation_tests;

#[cfg(test)]
#[path = "commands_semantic_propagation_update_tests.rs"]
mod propagation_update_tests;

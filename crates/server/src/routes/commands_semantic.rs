use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use eidetic_core::contracts::{
    AcceptBibleReferenceProposalCommand, AcceptPropagationProposalCommand,
    BibleReferenceProposalListProjection, CommandEnvelope, CreateBibleReferenceProposalCommand,
    CreatePropagationProposalCommand, ProjectionEnvelope, PropagationProposalListProjection,
    RecordSemanticDependencyCommand, RejectBibleReferenceProposalCommand,
    RejectPropagationProposalCommand, SemanticDependencyProjection,
};
use serde::Serialize;

use crate::error::{ApiError, ApiJson};
use crate::history_store::RecordChangeOutcome;
use crate::propagation_proposal_accept;
use crate::propagation_proposal_review;
use crate::propagation_proposal_store::{self, PropagationProposalStoreError};
use crate::semantic_dependency_store::{
    self, DependencyDirection, DependencyEndpointFilter, SemanticDependencyFilter,
    SemanticDependencyStoreError,
};
use crate::semantic_proposal_accept;
use crate::semantic_proposal_store::{self, SemanticProposalStoreError};
use crate::state::{AppState, ServerEvent};

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
            "/commands/semantic/propagation-proposal/accept",
            post(accept_propagation_proposal),
        )
}

#[derive(Debug, Serialize)]
struct BibleReferenceProposalCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<BibleReferenceProposalListProjection>,
}

#[derive(Debug, Serialize)]
struct SemanticDependencyCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<SemanticDependencyProjection>,
}

#[derive(Debug, Serialize)]
struct PropagationProposalCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<PropagationProposalListProjection>,
}

async fn create_bible_reference_proposal(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<CreateBibleReferenceProposalCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response =
        tokio::task::spawn_blocking(move || create_bible_reference_proposal_at_path(path, command))
            .await
            .map_err(|e| {
                ApiError::internal(format!("semantic proposal command task failed: {e}"))
            })??;

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::SemanticProposalsChanged);
    }
    crate::error::json_value(response)
}

async fn reject_bible_reference_proposal(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<RejectBibleReferenceProposalCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response =
        tokio::task::spawn_blocking(move || reject_bible_reference_proposal_at_path(path, command))
            .await
            .map_err(|e| {
                ApiError::internal(format!("semantic proposal reject task failed: {e}"))
            })??;

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::SemanticProposalsChanged);
    }
    crate::error::json_value(response)
}

async fn accept_bible_reference_proposal(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<AcceptBibleReferenceProposalCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response =
        tokio::task::spawn_blocking(move || accept_bible_reference_proposal_at_path(path, command))
            .await
            .map_err(|e| {
                ApiError::internal(format!("semantic proposal accept task failed: {e}"))
            })??;

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::SemanticProposalsChanged);
        let _ = state.events_tx.send(ServerEvent::BibleChanged);
    }
    crate::error::json_value(response)
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
    let path = active_project_path(&state)?;
    let response =
        tokio::task::spawn_blocking(move || create_propagation_proposal_at_path(path, command))
            .await
            .map_err(|e| {
                ApiError::internal(format!("propagation proposal command task failed: {e}"))
            })??;

    crate::error::json_value(response)
}

async fn reject_propagation_proposal(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<RejectPropagationProposalCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response =
        tokio::task::spawn_blocking(move || reject_propagation_proposal_at_path(path, command))
            .await
            .map_err(|e| {
                ApiError::internal(format!("propagation proposal reject task failed: {e}"))
            })??;

    crate::error::json_value(response)
}

async fn accept_propagation_proposal(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<AcceptPropagationProposalCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response =
        tokio::task::spawn_blocking(move || accept_propagation_proposal_at_path(path, command))
            .await
            .map_err(|e| {
                ApiError::internal(format!("propagation proposal accept task failed: {e}"))
            })??;

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::BibleChanged);
        let _ = state.events_tx.send(ServerEvent::ScriptChanged);
    }
    crate::error::json_value(response)
}

fn create_bible_reference_proposal_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<CreateBibleReferenceProposalCommand>,
) -> Result<BibleReferenceProposalCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let outcome =
        semantic_proposal_store::record_create_bible_reference_proposal(&mut conn, &command, 0)
            .map_err(map_semantic_proposal_error)?;
    let projection = semantic_proposal_store::load_bible_reference_proposal_list_projection(&conn)
        .map_err(map_semantic_proposal_error)?;

    Ok(BibleReferenceProposalCommandResponse {
        outcome,
        projection,
    })
}

fn reject_bible_reference_proposal_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<RejectBibleReferenceProposalCommand>,
) -> Result<BibleReferenceProposalCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let outcome =
        semantic_proposal_store::record_reject_bible_reference_proposal(&mut conn, &command, 0)
            .map_err(map_semantic_proposal_error)?;
    let projection = semantic_proposal_store::load_bible_reference_proposal_list_projection(&conn)
        .map_err(map_semantic_proposal_error)?;

    Ok(BibleReferenceProposalCommandResponse {
        outcome,
        projection,
    })
}

fn accept_bible_reference_proposal_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<AcceptBibleReferenceProposalCommand>,
) -> Result<BibleReferenceProposalCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let outcome =
        semantic_proposal_accept::record_accept_bible_reference_proposal(&mut conn, &command, 0)
            .map_err(map_semantic_proposal_error)?;
    let projection = semantic_proposal_store::load_bible_reference_proposal_list_projection(&conn)
        .map_err(map_semantic_proposal_error)?;

    Ok(BibleReferenceProposalCommandResponse {
        outcome,
        projection,
    })
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

fn create_propagation_proposal_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<CreatePropagationProposalCommand>,
) -> Result<PropagationProposalCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let outcome =
        propagation_proposal_store::record_create_propagation_proposal(&mut conn, &command, 0)
            .map_err(map_propagation_proposal_error)?;
    let projection = propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
        .map_err(map_propagation_proposal_error)?;

    Ok(PropagationProposalCommandResponse {
        outcome,
        projection,
    })
}

fn reject_propagation_proposal_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<RejectPropagationProposalCommand>,
) -> Result<PropagationProposalCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let outcome =
        propagation_proposal_review::record_reject_propagation_proposal(&mut conn, &command, 0)
            .map_err(map_propagation_proposal_error)?;
    let projection = propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
        .map_err(map_propagation_proposal_error)?;

    Ok(PropagationProposalCommandResponse {
        outcome,
        projection,
    })
}

fn accept_propagation_proposal_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<AcceptPropagationProposalCommand>,
) -> Result<PropagationProposalCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let outcome =
        propagation_proposal_accept::record_accept_propagation_proposal(&mut conn, &command, 0)
            .map_err(map_propagation_proposal_error)?;
    let projection = propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
        .map_err(map_propagation_proposal_error)?;

    Ok(PropagationProposalCommandResponse {
        outcome,
        projection,
    })
}

fn map_semantic_proposal_error(error: SemanticProposalStoreError) -> ApiError {
    match error {
        SemanticProposalStoreError::InvalidCommand(message) => ApiError::bad_request(message),
        SemanticProposalStoreError::NotFound(message) => ApiError::not_found(message),
        SemanticProposalStoreError::History(error) => map_history_error(error),
        SemanticProposalStoreError::Sqlite(error) => ApiError::internal(error.to_string()),
    }
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

fn map_propagation_proposal_error(error: PropagationProposalStoreError) -> ApiError {
    match error {
        PropagationProposalStoreError::InvalidCommand(message) => ApiError::bad_request(message),
        PropagationProposalStoreError::NotFound(message) => ApiError::not_found(message),
        PropagationProposalStoreError::History(error) => map_history_error(error),
        PropagationProposalStoreError::Sqlite(error) => ApiError::internal(error.to_string()),
        PropagationProposalStoreError::Json(error) => ApiError::bad_request(error.to_string()),
        PropagationProposalStoreError::Contract(error) => ApiError::bad_request(error.to_string()),
        PropagationProposalStoreError::BibleGraphContract(error) => {
            ApiError::bad_request(error.to_string())
        }
        PropagationProposalStoreError::ScriptContract(error) => {
            ApiError::bad_request(error.to_string())
        }
        PropagationProposalStoreError::SemanticDependencyContract(error) => {
            ApiError::bad_request(error.to_string())
        }
        PropagationProposalStoreError::Target(error) => ApiError::bad_request(error.to_string()),
        PropagationProposalStoreError::ScriptDocumentCommand(error) => {
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

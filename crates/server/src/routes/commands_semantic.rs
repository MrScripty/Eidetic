use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use eidetic_core::contracts::{
    AcceptBibleReferenceProposalCommand, BibleReferenceProposalListProjection, CommandEnvelope,
    CreateBibleReferenceProposalCommand, ProjectionEnvelope, RejectBibleReferenceProposalCommand,
};
use serde::Serialize;

use crate::error::{ApiError, ApiJson};
use crate::history_store::RecordChangeOutcome;
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
}

#[derive(Debug, Serialize)]
struct BibleReferenceProposalCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<BibleReferenceProposalListProjection>,
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

fn map_semantic_proposal_error(error: SemanticProposalStoreError) -> ApiError {
    match error {
        SemanticProposalStoreError::InvalidCommand(message) => ApiError::bad_request(message),
        SemanticProposalStoreError::NotFound(message) => ApiError::not_found(message),
        SemanticProposalStoreError::History(error) => map_history_error(error),
        SemanticProposalStoreError::Sqlite(error) => ApiError::internal(error.to_string()),
    }
}

#[cfg(test)]
#[path = "commands_semantic_tests.rs"]
mod tests;

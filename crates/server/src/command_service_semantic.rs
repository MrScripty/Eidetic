use eidetic_core::contracts::{
    AcceptBibleReferenceProposalCommand, BibleReferenceProposalListProjection, CommandEnvelope,
    CreateBibleReferenceProposalCommand, ProjectionEnvelope, RejectBibleReferenceProposalCommand,
};
use serde::Serialize;

use crate::backend_error::BackendError;
use crate::command_service_support::{active_project_path, map_history_error};
use crate::history_store::RecordChangeOutcome;
use crate::semantic_proposal_accept;
use crate::semantic_proposal_store::{self, SemanticProposalStoreError};
use crate::state::{AppState, ServerEvent};

#[derive(Debug, Serialize)]
pub struct BibleReferenceProposalCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<BibleReferenceProposalListProjection>,
}

pub async fn create_bible_reference_proposal(
    state: &AppState,
    command: CommandEnvelope<CreateBibleReferenceProposalCommand>,
) -> Result<BibleReferenceProposalCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response =
        tokio::task::spawn_blocking(move || create_bible_reference_proposal_at_path(path, command))
            .await
            .map_err(|error| {
                BackendError::internal(format!("semantic proposal command task failed: {error}"))
            })??;

    send_semantic_proposals_changed(state, response.outcome);
    Ok(response)
}

pub async fn reject_bible_reference_proposal(
    state: &AppState,
    command: CommandEnvelope<RejectBibleReferenceProposalCommand>,
) -> Result<BibleReferenceProposalCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response =
        tokio::task::spawn_blocking(move || reject_bible_reference_proposal_at_path(path, command))
            .await
            .map_err(|error| {
                BackendError::internal(format!("semantic proposal reject task failed: {error}"))
            })??;

    send_semantic_proposals_changed(state, response.outcome);
    Ok(response)
}

pub async fn accept_bible_reference_proposal(
    state: &AppState,
    command: CommandEnvelope<AcceptBibleReferenceProposalCommand>,
) -> Result<BibleReferenceProposalCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response =
        tokio::task::spawn_blocking(move || accept_bible_reference_proposal_at_path(path, command))
            .await
            .map_err(|error| {
                BackendError::internal(format!("semantic proposal accept task failed: {error}"))
            })??;

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::SemanticProposalsChanged);
        let _ = state.events_tx.send(ServerEvent::BibleChanged);
    }
    Ok(response)
}

fn create_bible_reference_proposal_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<CreateBibleReferenceProposalCommand>,
) -> Result<BibleReferenceProposalCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
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
) -> Result<BibleReferenceProposalCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
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
) -> Result<BibleReferenceProposalCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
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

fn send_semantic_proposals_changed(state: &AppState, outcome: RecordChangeOutcome) {
    if outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::SemanticProposalsChanged);
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

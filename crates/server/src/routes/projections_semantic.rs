use axum::Router;
use axum::extract::Query;
use axum::extract::State;
use axum::routing::get;
use eidetic_core::contracts::{
    BibleReferenceProposalListProjection, ProjectionEnvelope, PropagationProposalListProjection,
    SemanticDependencyProjection,
};
use serde::Deserialize;

use crate::error::{ApiError, ApiJson};
use crate::propagation_proposal_store::{self, PropagationProposalStoreError};
use crate::semantic_dependency_store::{
    self, DependencyDirection, DependencyEndpointFilter, SemanticDependencyFilter,
    SemanticDependencyStoreError,
};
use crate::semantic_proposal_store::{self, SemanticProposalStoreError};
use crate::state::AppState;

use super::support::{active_project_path, map_history_error};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/projections/semantic/bible-reference-proposals",
            get(get_bible_reference_proposal_list),
        )
        .route(
            "/projections/semantic/dependencies",
            get(get_semantic_dependencies),
        )
        .route(
            "/projections/semantic/propagation-proposals",
            get(get_propagation_proposal_list),
        )
}

#[derive(Debug, Deserialize)]
struct SemanticDependencyProjectionQuery {
    source_kind: Option<String>,
    source_id: Option<String>,
    source_part_key: Option<String>,
    source_field_key: Option<String>,
    target_kind: Option<String>,
    target_id: Option<String>,
    target_part_key: Option<String>,
    target_field_key: Option<String>,
}

async fn get_bible_reference_proposal_list(State(state): State<AppState>) -> ApiJson {
    let path = active_project_path(&state)?;
    let projection =
        tokio::task::spawn_blocking(move || load_bible_reference_proposal_list_at_path(path))
            .await
            .map_err(|e| {
                ApiError::internal(format!("semantic proposal projection task failed: {e}"))
            })??;

    crate::error::json_value(projection)
}

async fn get_semantic_dependencies(
    State(state): State<AppState>,
    Query(query): Query<SemanticDependencyProjectionQuery>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let filter = semantic_dependency_filter_from_query(query)?;
    let projection = tokio::task::spawn_blocking(move || {
        load_semantic_dependency_projection_at_path(path, filter)
    })
    .await
    .map_err(|e| {
        ApiError::internal(format!("semantic dependency projection task failed: {e}"))
    })??;

    crate::error::json_value(projection)
}

async fn get_propagation_proposal_list(State(state): State<AppState>) -> ApiJson {
    let path = active_project_path(&state)?;
    let projection =
        tokio::task::spawn_blocking(move || load_propagation_proposal_list_at_path(path))
            .await
            .map_err(|e| {
                ApiError::internal(format!("propagation proposal projection task failed: {e}"))
            })??;

    crate::error::json_value(projection)
}

fn load_bible_reference_proposal_list_at_path(
    path: std::path::PathBuf,
) -> Result<ProjectionEnvelope<BibleReferenceProposalListProjection>, ApiError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    semantic_proposal_store::load_bible_reference_proposal_list_projection(&conn)
        .map_err(map_semantic_proposal_error)
}

fn load_semantic_dependency_projection_at_path(
    path: std::path::PathBuf,
    filter: SemanticDependencyFilter,
) -> Result<ProjectionEnvelope<SemanticDependencyProjection>, ApiError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    semantic_dependency_store::load_semantic_dependency_projection(&conn, &filter)
        .map_err(map_semantic_dependency_error)
}

fn load_propagation_proposal_list_at_path(
    path: std::path::PathBuf,
) -> Result<ProjectionEnvelope<PropagationProposalListProjection>, ApiError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
        .map_err(map_propagation_proposal_error)
}

fn semantic_dependency_filter_from_query(
    query: SemanticDependencyProjectionQuery,
) -> Result<SemanticDependencyFilter, ApiError> {
    let source = endpoint_filter(
        query.source_kind,
        query.source_id,
        query.source_part_key,
        query.source_field_key,
    )?;
    let target = endpoint_filter(
        query.target_kind,
        query.target_id,
        query.target_part_key,
        query.target_field_key,
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
        (None, None) => Err(ApiError::bad_request(
            "semantic dependency projection requires a source or target filter",
        )),
        (Some(_), Some(_)) => Err(ApiError::bad_request(
            "semantic dependency projection accepts only one source or target filter",
        )),
    }
}

fn endpoint_filter(
    kind: Option<String>,
    id: Option<String>,
    part_key: Option<String>,
    field_key: Option<String>,
) -> Result<Option<DependencyEndpointFilter>, ApiError> {
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
        _ => Err(ApiError::bad_request(
            "semantic dependency filter requires kind and id",
        )),
    }
}

fn validate_endpoint_filter(
    kind: &str,
    part_key: Option<&str>,
    field_key: Option<&str>,
) -> Result<(), ApiError> {
    if !matches!(
        kind,
        "timeline_node" | "bible_node" | "bible_field" | "script_segment" | "script_block"
    ) {
        return Err(ApiError::bad_request(format!(
            "unknown semantic dependency endpoint kind: {kind}"
        )));
    }
    if (part_key.is_some() || field_key.is_some()) && kind != "bible_field" {
        return Err(ApiError::bad_request(
            "semantic dependency part and field filters require bible_field kind",
        ));
    }
    Ok(())
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
    }
}

#[cfg(test)]
#[path = "projections_semantic_tests.rs"]
mod tests;

use std::path::PathBuf;

use eidetic_core::contracts::{ProjectionEnvelope, SemanticDependencyProjection};
use serde::Deserialize;

use crate::backend_error::BackendError;
use crate::history_store::HistoryStoreError;
use crate::projection_service::active_project_path;
use crate::semantic_dependency_store::{
    self, DependencyDirection, DependencyEndpointFilter, SemanticDependencyFilter,
    SemanticDependencyStoreError,
};
use crate::state::AppState;

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

fn load_semantic_dependency_projection_at_path(
    path: PathBuf,
    filter: SemanticDependencyFilter,
) -> Result<ProjectionEnvelope<SemanticDependencyProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    semantic_dependency_store::load_semantic_dependency_projection(&conn, &filter)
        .map_err(map_semantic_dependency_error)
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

fn map_history_error(error: HistoryStoreError) -> BackendError {
    match error {
        HistoryStoreError::InvalidValue(message) => BackendError::conflict(message),
        HistoryStoreError::InvalidId(message) => BackendError::bad_request(message),
        HistoryStoreError::MissingColumn(message) => BackendError::internal(message),
        HistoryStoreError::Sqlite(error) => BackendError::internal(error.to_string()),
        HistoryStoreError::Json(error) => BackendError::bad_request(error.to_string()),
    }
}

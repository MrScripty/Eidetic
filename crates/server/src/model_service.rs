use serde::{Deserialize, Serialize};

use crate::backend_error::BackendError;
use crate::state::AppState;

pub const MODEL_LIBRARY_UNCONFIGURED: &str =
    "Model library not configured. Set PUMAS_MODELS_DIR env var.";

#[derive(Debug, Clone, Deserialize)]
pub struct ModelListRequest {
    #[serde(default)]
    pub q: String,
    #[serde(default)]
    pub model_type: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

impl Default for ModelListRequest {
    fn default() -> Self {
        Self {
            q: String::new(),
            model_type: None,
            limit: default_limit(),
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub model_type: String,
    pub size_bytes: Option<u64>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelListResponse {
    pub models: Vec<ModelEntry>,
    pub total_count: usize,
}

pub async fn list_models(
    state: &AppState,
    request: ModelListRequest,
) -> Result<ModelListResponse, BackendError> {
    let library = state
        .model_library
        .as_ref()
        .ok_or_else(|| BackendError::Internal(MODEL_LIBRARY_UNCONFIGURED.to_string()))?;

    let result = library
        .search_models_filtered(
            &request.q,
            request.limit,
            request.offset,
            request.model_type.as_deref(),
            None,
        )
        .await
        .map_err(|error| BackendError::Internal(error.to_string()))?;

    let models = result
        .models
        .into_iter()
        .map(|model| {
            let size_bytes = model
                .metadata
                .get("size_bytes")
                .or_else(|| model.metadata.get("sizeBytes"))
                .and_then(|value| value.as_u64());

            ModelEntry {
                id: model.id,
                name: model.official_name,
                path: model.path,
                model_type: model.model_type,
                size_bytes,
                tags: model.tags,
            }
        })
        .collect();

    Ok(ModelListResponse {
        models,
        total_count: result.total_count,
    })
}

fn default_limit() -> usize {
    100
}

#[cfg(test)]
mod tests {
    use super::{ModelListRequest, list_models};
    use crate::state::AppState;

    #[tokio::test]
    async fn list_models_reports_unconfigured_library() {
        let mut state = AppState::new().await;
        state.model_library = None;

        let error = list_models(&state, ModelListRequest::default())
            .await
            .expect_err("missing model library should fail");

        assert!(error.message().contains("Model library not configured"));
    }
}

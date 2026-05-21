use serde::{Deserialize, Serialize};

use crate::ai_backends::Backend;
use crate::state::{AiConfig, AppState, BackendType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiStatus {
    pub backend: BackendType,
    pub model: String,
    pub connected: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AiConfigUpdate {
    pub backend_type: Option<BackendType>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub base_url: Option<String>,
    pub api_key: Option<Option<String>>,
}

pub async fn get_ai_status(state: &AppState) -> AiStatus {
    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);

    match backend.health_check().await {
        Ok(status) => AiStatus {
            backend: config.backend_type,
            model: display_model(&config, &status.model),
            connected: status.connected,
            message: Some(status.message),
            error: None,
        },
        Err(error) => AiStatus {
            backend: config.backend_type,
            model: config.model,
            connected: false,
            message: None,
            error: Some(error.to_string()),
        },
    }
}

pub fn update_ai_config(state: &AppState, update: AiConfigUpdate) -> AiConfig {
    let mut config = state.ai_config.lock();
    if let Some(backend_type) = update.backend_type {
        config.backend_type = backend_type;
    }
    if let Some(model) = update.model {
        config.model = model;
    }
    if let Some(temperature) = update.temperature {
        config.temperature = temperature;
    }
    if let Some(max_tokens) = update.max_tokens {
        config.max_tokens = max_tokens;
    }
    if let Some(base_url) = update.base_url {
        config.base_url = base_url;
    }
    if let Some(api_key) = update.api_key {
        config.api_key = api_key.filter(|value| !value.is_empty());
    }
    config.clone()
}

fn display_model(config: &AiConfig, detected_model: &str) -> String {
    if config.model.eq_ignore_ascii_case("auto") || config.model.is_empty() {
        if detected_model.is_empty() {
            "auto".to_string()
        } else {
            detected_model.to_string()
        }
    } else {
        config.model.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{AiConfigUpdate, display_model, update_ai_config};
    use crate::state::{AiConfig, AppState, BackendType};

    #[test]
    fn display_model_uses_detected_model_for_auto_config() {
        let config = AiConfig {
            model: "auto".to_string(),
            ..AiConfig::default()
        };

        assert_eq!(display_model(&config, "served-model"), "served-model");
    }

    #[tokio::test]
    async fn update_ai_config_applies_sparse_updates_and_filters_blank_key() {
        let state = AppState::new().await;

        let config = update_ai_config(
            &state,
            AiConfigUpdate {
                backend_type: Some(BackendType::OpenRouter),
                model: Some("open-model".to_string()),
                temperature: Some(0.2),
                max_tokens: Some(1024),
                base_url: Some("https://example.test/v1".to_string()),
                api_key: Some(Some(String::new())),
            },
        );

        assert_eq!(config.backend_type, BackendType::OpenRouter);
        assert_eq!(config.model, "open-model");
        assert_eq!(config.temperature, 0.2);
        assert_eq!(config.max_tokens, 1024);
        assert_eq!(config.base_url, "https://example.test/v1");
        assert_eq!(config.api_key, None);
    }
}

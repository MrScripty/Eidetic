pub(crate) mod ollama;
pub(crate) mod openrouter;

use serde::Serialize;

use crate::prompt_format::ChatPrompt;
use crate::state::{AiConfig, BackendType};
use eidetic_core::ai::backend::GenerateStream;
use eidetic_core::error::Error;

/// Unified backend that dispatches to the configured implementation.
pub(crate) enum Backend {
    Ollama(ollama::OllamaBackend),
    OpenRouter(openrouter::OpenRouterBackend),
}

impl Backend {
    pub fn from_config(config: &AiConfig) -> Self {
        match config.backend_type {
            BackendType::Ollama => Backend::Ollama(ollama::OllamaBackend::new(config)),
            BackendType::OpenRouter => {
                Backend::OpenRouter(openrouter::OpenRouterBackend::new(config))
            }
        }
    }

    pub async fn generate(
        &self,
        prompt: &ChatPrompt,
        config: &AiConfig,
    ) -> Result<GenerateStream, Error> {
        match self {
            Backend::Ollama(b) => b.generate(prompt, config).await,
            Backend::OpenRouter(b) => b.generate(prompt, config).await,
        }
    }

    pub async fn health_check(&self) -> Result<BackendStatus, Error> {
        match self {
            Backend::Ollama(b) => b.health_check().await,
            Backend::OpenRouter(b) => b.health_check().await,
        }
    }
}

/// Status of a backend connection.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct BackendStatus {
    pub connected: bool,
    pub model: String,
    pub backend_type: BackendType,
    pub message: String,
}

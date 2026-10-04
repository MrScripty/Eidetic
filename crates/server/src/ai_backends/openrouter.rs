use reqwest::Client;

use crate::prompt_format::ChatPrompt;
use crate::state::{AiConfig, BackendType};
use eidetic_core::ai::backend::GenerateStream;
use eidetic_core::error::Error;

use super::BackendStatus;

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

pub(crate) struct OpenRouterBackend {
    client: Client,
}

impl OpenRouterBackend {
    pub fn new(_config: &AiConfig) -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn generate(
        &self,
        prompt: &ChatPrompt,
        config: &AiConfig,
    ) -> Result<GenerateStream, Error> {
        self.generate_at_url(prompt, config, OPENROUTER_URL).await
    }

    // The endpoint seam lets loopback tests exercise the actual HTTP adapter.
    // Production generation always uses the fixed OpenRouter endpoint above.
    pub(super) async fn generate_at_url(
        &self,
        prompt: &ChatPrompt,
        config: &AiConfig,
        url: &str,
    ) -> Result<GenerateStream, Error> {
        let api_key = config
            .api_key
            .as_deref()
            .filter(|k| !k.is_empty())
            .ok_or_else(|| Error::AiBackend("OpenRouter API key not configured".into()))?;

        let body = serde_json::json!({
            "model": config.model,
            "messages": [
                { "role": "system", "content": prompt.system },
                { "role": "user", "content": prompt.user }
            ],
            "stream": true,
            "temperature": config.temperature,
            "max_tokens": config.max_tokens,
        });

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("HTTP-Referer", "https://eidetic.app")
            .header("X-Title", "Eidetic")
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::AiBackend(format!("OpenRouter request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "unknown".into());
            return Err(Error::AiBackend(format!(
                "OpenRouter returned {status}: {body}"
            )));
        }

        Ok(super::sse::tokens(response, "OpenRouter"))
    }

    /// Non-streaming generation with JSON mode enabled.
    /// Uses OpenAI-compatible `response_format` parameter.
    pub async fn generate_json(
        &self,
        prompt: &ChatPrompt,
        config: &AiConfig,
    ) -> Result<String, Error> {
        let api_key = config
            .api_key
            .as_deref()
            .filter(|k| !k.is_empty())
            .ok_or_else(|| Error::AiBackend("OpenRouter API key not configured".into()))?;

        let body = serde_json::json!({
            "model": config.model,
            "messages": [
                { "role": "system", "content": prompt.system },
                { "role": "user", "content": prompt.user }
            ],
            "stream": false,
            "temperature": config.temperature,
            "max_tokens": config.max_tokens,
            "response_format": { "type": "json_object" },
        });

        let response = self
            .client
            .post(OPENROUTER_URL)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("HTTP-Referer", "https://eidetic.app")
            .header("X-Title", "Eidetic")
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::AiBackend(format!("OpenRouter request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "unknown".into());
            return Err(Error::AiBackend(format!(
                "OpenRouter returned {status}: {text}"
            )));
        }

        let value: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::AiBackend(format!("OpenRouter response parse error: {e}")))?;

        let content = value
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_owned();

        Ok(content)
    }

    pub async fn health_check(&self) -> Result<BackendStatus, Error> {
        // A lightweight check — just verify we can reach OpenRouter.
        match self
            .client
            .get("https://openrouter.ai/api/v1/models")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => Ok(BackendStatus {
                connected: true,
                model: String::new(),
                backend_type: BackendType::OpenRouter,
                message: "Connected to OpenRouter".into(),
            }),
            Ok(resp) => Ok(BackendStatus {
                connected: false,
                model: String::new(),
                backend_type: BackendType::OpenRouter,
                message: format!("OpenRouter returned {}", resp.status()),
            }),
            Err(e) => Ok(BackendStatus {
                connected: false,
                model: String::new(),
                backend_type: BackendType::OpenRouter,
                message: format!("Cannot reach OpenRouter: {e}"),
            }),
        }
    }
}

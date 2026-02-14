use futures::stream::{self, StreamExt};
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
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".into());
            return Err(Error::AiBackend(format!(
                "OpenRouter returned {status}: {body}"
            )));
        }

        // Parse SSE stream: lines like `data: {"choices":[{"delta":{"content":"token"}}]}`
        let byte_stream = response.bytes_stream();
        let token_stream = byte_stream
            .map(|chunk| match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    let mut tokens = Vec::new();
                    for line in text.lines() {
                        let line = line.trim();
                        if line == "data: [DONE]" {
                            break;
                        }
                        let Some(json_str) = line.strip_prefix("data: ") else {
                            continue;
                        };
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
                            if let Some(content) = value
                                .get("choices")
                                .and_then(|c| c.get(0))
                                .and_then(|c| c.get("delta"))
                                .and_then(|d| d.get("content"))
                                .and_then(|c| c.as_str())
                            {
                                if !content.is_empty() {
                                    tokens.push(content.to_owned());
                                }
                            }
                        }
                    }
                    tokens
                }
                Err(e) => {
                    tracing::warn!("OpenRouter stream chunk error: {e}");
                    vec![]
                }
            })
            .flat_map(stream::iter)
            .map(Ok);

        Ok(Box::pin(token_stream))
    }

    pub async fn health_check(&self) -> Result<BackendStatus, Error> {
        // A lightweight check — just verify we can reach OpenRouter.
        match self.client.get("https://openrouter.ai/api/v1/models").send().await {
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

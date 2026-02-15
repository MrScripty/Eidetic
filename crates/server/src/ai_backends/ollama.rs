use futures::stream::{self, StreamExt};
use reqwest::Client;

use crate::prompt_format::ChatPrompt;
use crate::state::{AiConfig, BackendType};
use eidetic_core::ai::backend::GenerateStream;
use eidetic_core::error::Error;

use super::BackendStatus;

pub(crate) struct OllamaBackend {
    client: Client,
    base_url: String,
}

impl OllamaBackend {
    pub fn new(config: &AiConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: config.base_url.clone(),
        }
    }

    pub async fn generate(
        &self,
        prompt: &ChatPrompt,
        config: &AiConfig,
    ) -> Result<GenerateStream, Error> {
        let url = format!("{}/api/chat", self.base_url);
        let body = serde_json::json!({
            "model": config.model,
            "messages": [
                { "role": "system", "content": prompt.system },
                { "role": "user", "content": prompt.user }
            ],
            "stream": true,
            "options": {
                "temperature": config.temperature,
                "num_predict": config.max_tokens,
            }
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::AiBackend(format!("Ollama request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".into());
            return Err(Error::AiBackend(format!(
                "Ollama returned {status}: {body}"
            )));
        }

        // Stream NDJSON: each line is {"message":{"content":"token"},"done":false}
        let byte_stream = response.bytes_stream();
        let token_stream = byte_stream
            .map(|chunk| match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    let mut tokens = Vec::new();
                    for line in text.lines() {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                            // Check if done.
                            if value.get("done").and_then(|d| d.as_bool()) == Some(true) {
                                break;
                            }
                            // Extract token content.
                            if let Some(content) = value
                                .get("message")
                                .and_then(|m| m.get("content"))
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
                    tracing::warn!("Ollama stream chunk error: {e}");
                    vec![]
                }
            })
            .flat_map(stream::iter)
            .map(Ok);

        Ok(Box::pin(token_stream))
    }

    /// Non-streaming generation with JSON mode enabled.
    /// Ollama will constrain output to valid JSON.
    pub async fn generate_json(&self, prompt: &ChatPrompt, config: &AiConfig) -> Result<String, Error> {
        let url = format!("{}/api/chat", self.base_url);
        let body = serde_json::json!({
            "model": config.model,
            "messages": [
                { "role": "system", "content": prompt.system },
                { "role": "user", "content": prompt.user }
            ],
            "stream": false,
            "format": "json",
            "options": {
                "temperature": config.temperature,
                "num_predict": config.max_tokens,
            }
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::AiBackend(format!("Ollama request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "unknown".into());
            return Err(Error::AiBackend(format!("Ollama returned {status}: {text}")));
        }

        let value: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::AiBackend(format!("Ollama response parse error: {e}")))?;

        let content = value
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_owned();

        Ok(content)
    }

    pub async fn health_check(&self) -> Result<BackendStatus, Error> {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let body: serde_json::Value = resp
                    .json()
                    .await
                    .unwrap_or(serde_json::Value::Null);
                let models: Vec<String> = body
                    .get("models")
                    .and_then(|m| m.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|m| m.get("name").and_then(|n| n.as_str()))
                            .map(|s| s.to_owned())
                            .collect()
                    })
                    .unwrap_or_default();
                let message = if models.is_empty() {
                    "Connected, no models loaded".into()
                } else {
                    format!("Connected, {} models available", models.len())
                };
                Ok(BackendStatus {
                    connected: true,
                    model: models.first().cloned().unwrap_or_default(),
                    backend_type: BackendType::Ollama,
                    message,
                })
            }
            Ok(resp) => Err(Error::AiBackend(format!(
                "Ollama returned {}",
                resp.status()
            ))),
            Err(e) => Ok(BackendStatus {
                connected: false,
                model: String::new(),
                backend_type: BackendType::Ollama,
                message: format!("Cannot reach Ollama: {e}"),
            }),
        }
    }
}

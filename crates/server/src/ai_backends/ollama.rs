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

    /// Query Ollama for the currently loaded/running model via `/api/ps`.
    /// Falls back to the first available model from `/api/tags` if nothing is running.
    /// Returns `Err` if Ollama is unreachable or has no models.
    pub async fn resolve_running_model(&self) -> Result<String, Error> {
        // First try /api/ps — models currently loaded in memory.
        let ps_url = format!("{}/api/ps", self.base_url);
        if let Ok(resp) = self.client.get(&ps_url).send().await {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    if let Some(models) = body.get("models").and_then(|m| m.as_array()) {
                        if let Some(name) = models
                            .first()
                            .and_then(|m| m.get("name"))
                            .and_then(|n| n.as_str())
                        {
                            return Ok(name.to_owned());
                        }
                    }
                }
            }
        }

        // Fallback: first model from /api/tags.
        let tags_url = format!("{}/api/tags", self.base_url);
        let resp = self
            .client
            .get(&tags_url)
            .send()
            .await
            .map_err(|e| Error::AiBackend(format!("Cannot reach Ollama: {e}")))?;
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::AiBackend(format!("Failed to parse Ollama tags: {e}")))?;
        body.get("models")
            .and_then(|m| m.as_array())
            .and_then(|arr| arr.first())
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_owned())
            .ok_or_else(|| Error::AiBackend("No models available in Ollama".into()))
    }

    /// Return the model name to use — resolves "auto" / empty to the running model.
    async fn effective_model(&self, config: &AiConfig) -> Result<String, Error> {
        if config.model.is_empty() || config.model.eq_ignore_ascii_case("auto") {
            self.resolve_running_model().await
        } else {
            Ok(config.model.clone())
        }
    }

    pub async fn generate(
        &self,
        prompt: &ChatPrompt,
        config: &AiConfig,
    ) -> Result<GenerateStream, Error> {
        let model = self.effective_model(config).await?;
        let url = format!("{}/api/chat", self.base_url);
        let body = serde_json::json!({
            "model": model,
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
        let model = self.effective_model(config).await?;
        let url = format!("{}/api/chat", self.base_url);
        let body = serde_json::json!({
            "model": model,
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

                // Check which model is actually loaded/running.
                let running_model = self.resolve_running_model().await.ok();

                let message = match &running_model {
                    Some(m) => format!("Connected, running {m}"),
                    None if models.is_empty() => "Connected, no models available".into(),
                    None => format!("Connected, {} models available (none running)", models.len()),
                };

                Ok(BackendStatus {
                    connected: true,
                    model: running_model.unwrap_or_else(|| {
                        models.first().cloned().unwrap_or_default()
                    }),
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

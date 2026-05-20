use futures::stream::{self, StreamExt};
use reqwest::{Client, RequestBuilder};

use crate::prompt_format::ChatPrompt;
use crate::state::{AiConfig, BackendType};
use eidetic_core::ai::backend::GenerateStream;
use eidetic_core::error::Error;

use super::BackendStatus;

pub(crate) struct LlamaCppBackend {
    client: Client,
    base_url: String,
}

impl LlamaCppBackend {
    pub fn new(config: &AiConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: config.base_url.trim_end_matches('/').to_owned(),
        }
    }

    pub async fn generate(
        &self,
        prompt: &ChatPrompt,
        config: &AiConfig,
    ) -> Result<GenerateStream, Error> {
        let model = self.effective_model(config).await?;
        let body = serde_json::json!({
            "model": model,
            "messages": [
                { "role": "system", "content": prompt.system },
                { "role": "user", "content": prompt.user }
            ],
            "stream": true,
            "temperature": config.temperature,
            "max_tokens": config.max_tokens,
        });

        let response = self
            .authorized(self.client.post(self.chat_completions_url()), config)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::AiBackend(format!("llama.cpp request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "unknown".into());
            return Err(Error::AiBackend(format!(
                "llama.cpp returned {status}: {body}"
            )));
        }

        let byte_stream = response.bytes_stream();
        let token_stream = byte_stream
            .map(|chunk| match chunk {
                Ok(bytes) => parse_sse_tokens(&String::from_utf8_lossy(&bytes)),
                Err(e) => {
                    tracing::warn!("llama.cpp stream chunk error: {e}");
                    vec![]
                }
            })
            .flat_map(stream::iter)
            .map(Ok);

        Ok(Box::pin(token_stream))
    }

    pub async fn generate_json(
        &self,
        prompt: &ChatPrompt,
        config: &AiConfig,
    ) -> Result<String, Error> {
        let model = self.effective_model(config).await?;
        let body = serde_json::json!({
            "model": model,
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
            .authorized(self.client.post(self.chat_completions_url()), config)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::AiBackend(format!("llama.cpp request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "unknown".into());
            return Err(Error::AiBackend(format!(
                "llama.cpp returned {status}: {text}"
            )));
        }

        let value: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::AiBackend(format!("llama.cpp response parse error: {e}")))?;

        Ok(value
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_owned())
    }

    pub async fn health_check(&self) -> Result<BackendStatus, Error> {
        match self.client.get(self.models_url()).send().await {
            Ok(resp) if resp.status().is_success() => {
                let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
                let model = first_model_id(&body).unwrap_or_default();
                let message = if model.is_empty() {
                    "Connected to llama.cpp".to_owned()
                } else {
                    format!("Connected to llama.cpp, model {model}")
                };

                Ok(BackendStatus {
                    connected: true,
                    model,
                    backend_type: BackendType::LlamaCpp,
                    message,
                })
            }
            Ok(resp) => Ok(BackendStatus {
                connected: false,
                model: String::new(),
                backend_type: BackendType::LlamaCpp,
                message: format!("llama.cpp returned {}", resp.status()),
            }),
            Err(e) => Ok(BackendStatus {
                connected: false,
                model: String::new(),
                backend_type: BackendType::LlamaCpp,
                message: format!("Cannot reach llama.cpp: {e}"),
            }),
        }
    }

    async fn effective_model(&self, config: &AiConfig) -> Result<String, Error> {
        if !config.model.is_empty() && !config.model.eq_ignore_ascii_case("auto") {
            return Ok(config.model.clone());
        }

        let response = self
            .client
            .get(self.models_url())
            .send()
            .await
            .map_err(|e| Error::AiBackend(format!("Cannot reach llama.cpp: {e}")))?;

        if !response.status().is_success() {
            return Err(Error::AiBackend(format!(
                "llama.cpp returned {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::AiBackend(format!("Failed to parse llama.cpp models: {e}")))?;

        first_model_id(&body)
            .ok_or_else(|| Error::AiBackend("No models available in llama.cpp".into()))
    }

    fn authorized(&self, request: RequestBuilder, config: &AiConfig) -> RequestBuilder {
        match config.api_key.as_deref().filter(|key| !key.is_empty()) {
            Some(api_key) => request.bearer_auth(api_key),
            None => request,
        }
    }

    fn chat_completions_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    fn models_url(&self) -> String {
        format!("{}/models", self.base_url)
    }
}

fn first_model_id(body: &serde_json::Value) -> Option<String> {
    body.get("data")
        .and_then(|data| data.as_array())
        .and_then(|models| models.first())
        .and_then(|model| model.get("id"))
        .and_then(|id| id.as_str())
        .map(str::to_owned)
}

fn parse_sse_tokens(text: &str) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{first_model_id, parse_sse_tokens};

    #[test]
    fn first_model_id_reads_openai_compatible_model_list() {
        let body = json!({
            "data": [
                { "id": "local-model", "object": "model" }
            ]
        });

        assert_eq!(first_model_id(&body), Some("local-model".to_owned()));
    }

    #[test]
    fn parse_sse_tokens_reads_streaming_delta_content() {
        let tokens = parse_sse_tokens(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\
             data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\
             data: [DONE]\n",
        );

        assert_eq!(tokens, vec!["Hel".to_owned(), "lo".to_owned()]);
    }
}

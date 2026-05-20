use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Client for generating text embeddings via an OpenAI-compatible `/embeddings` endpoint.
pub struct EmbeddingClient {
    client: Client,
    base_url: String,
    model: String,
}

#[derive(Serialize)]
struct EmbedRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Deserialize)]
struct EmbedResponse {
    data: Vec<EmbedResponseData>,
}

#[derive(Deserialize)]
struct EmbedResponseData {
    embedding: Vec<f32>,
}

impl EmbeddingClient {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            model: model.into(),
        }
    }

    /// Generate an embedding vector for the given text.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let url = format!("{}/embeddings", self.base_url.trim_end_matches('/'));
        let body = EmbedRequest {
            model: &self.model,
            input: text,
        };

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("embedding request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("embedding API returned {status}: {text}"));
        }

        let parsed: EmbedResponse = resp
            .json()
            .await
            .map_err(|e| format!("failed to parse embedding response: {e}"))?;

        parsed
            .data
            .into_iter()
            .next()
            .map(|item| item.embedding)
            .ok_or_else(|| "embedding response did not contain an embedding".to_string())
    }
}

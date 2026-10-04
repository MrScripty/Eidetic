use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Configured representation space. This does not attest immutable model weights;
/// Pumas package/revision identity must replace it before that stronger guarantee.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EmbeddingIdentity {
    endpoint: String,
    model: String,
}

/// A non-empty, finite, non-zero vector with its configured provider/model identity.
#[derive(Debug, Clone)]
pub(crate) struct Embedding {
    pub(crate) identity: EmbeddingIdentity,
    pub(crate) values: Vec<f32>,
}

impl Embedding {
    pub(crate) fn new(identity: EmbeddingIdentity, values: Vec<f32>) -> Result<Self, String> {
        if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
            return Err("embedding must be non-empty and finite".into());
        }
        if values.iter().all(|value| *value == 0.0) {
            return Err("embedding must have non-zero magnitude".into());
        }
        Ok(Self { identity, values })
    }
}

/// Transitional OpenAI-compatible adapter. Pumas runtime integration is separate.
pub struct EmbeddingClient {
    client: Client,
    identity: EmbeddingIdentity,
}

#[derive(Serialize)]
struct EmbedRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Deserialize)]
struct EmbedResponse {
    model: String,
    data: Vec<EmbedResponseData>,
}

#[derive(Deserialize)]
struct EmbedResponseData {
    index: usize,
    embedding: Vec<f32>,
}

impl EmbeddingClient {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            identity: EmbeddingIdentity {
                endpoint: base_url.into().trim_end_matches('/').to_string(),
                model: model.into(),
            },
        }
    }

    pub(crate) fn identity(&self) -> EmbeddingIdentity {
        self.identity.clone()
    }

    pub(crate) async fn embed(&self, text: &str) -> Result<Embedding, String> {
        let url = format!("{}/embeddings", self.identity.endpoint);
        let body = EmbedRequest {
            model: &self.identity.model,
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
            return Err(format!("embedding API returned {}", resp.status()));
        }
        let parsed: EmbedResponse = resp
            .json()
            .await
            .map_err(|e| format!("failed to parse embedding response: {e}"))?;
        self.validate_response(parsed)
    }

    fn validate_response(&self, mut response: EmbedResponse) -> Result<Embedding, String> {
        if response.model != self.identity.model {
            return Err("embedding response model does not match requested model".into());
        }
        if response.data.len() != 1 || response.data[0].index != 0 {
            return Err(
                "embedding response must contain exactly the requested input at index zero".into(),
            );
        }
        Embedding::new(self.identity.clone(), response.data.remove(0).embedding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn embedding(values: Vec<f32>) -> Embedding {
        Embedding::new(
            EmbeddingClient::new("http://localhost:18080/v1", "model").identity,
            values,
        )
        .unwrap()
    }

    #[test]
    fn rejects_invalid_vector_values() {
        for values in [vec![], vec![0.0, 0.0], vec![f32::NAN], vec![f32::INFINITY]] {
            assert!(Embedding::new(embedding(vec![1.0]).identity, values).is_err());
        }
    }

    #[test]
    fn validates_response_identity_cardinality_and_input_index() {
        let client = EmbeddingClient::new("http://localhost:18080/v1", "model");
        for value in [
            serde_json::json!({"model":"other", "data":[{"index":0,"embedding":[1.0]}]}),
            serde_json::json!({"model":"model", "data":[]}),
            serde_json::json!({"model":"model", "data":[{"index":1,"embedding":[1.0]}]}),
            serde_json::json!({"model":"model", "data":[{"index":0,"embedding":[1.0]},{"index":0,"embedding":[1.0]}]}),
        ] {
            assert!(
                client
                    .validate_response(serde_json::from_value(value).unwrap())
                    .is_err()
            );
        }
        assert!(
            serde_json::from_value::<EmbedResponse>(
                serde_json::json!({"data":[{"embedding":[1.0]}]})
            )
            .is_err()
        );
        let result = client
            .validate_response(EmbedResponse {
                model: "model".into(),
                data: vec![EmbedResponseData {
                    index: 0,
                    embedding: vec![1.0],
                }],
            })
            .unwrap();
        assert_eq!(result.values, vec![1.0]);
    }

    #[test]
    fn endpoint_and_model_are_both_part_of_identity() {
        let first = EmbeddingClient::new("http://localhost:18080/v1/", "model").identity;
        assert_eq!(first, embedding(vec![1.0]).identity);
        assert_ne!(
            first,
            EmbeddingClient::new("http://localhost:18081/v1", "model").identity
        );
        assert_ne!(
            first,
            EmbeddingClient::new("http://localhost:18080/v1", "other").identity
        );
    }
}

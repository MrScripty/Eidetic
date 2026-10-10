use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingProvider {
    #[default]
    Disabled,
    Pumas,
    OpenAiCompatible,
}

/// Embeddings have their own endpoint, model, profile, revision and credentials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EmbeddingConfig {
    pub provider: EmbeddingProvider,
    pub base_url: String,
    pub model: String,
    pub profile: String,
    /// Required operator revision for a non-Pumas endpoint. Pumas resolves its
    /// current model provenance/load revision directly from the producer.
    pub revision: String,
    pub api_key: Option<String>,
}

/// Exact selected representation and producer load revision. Dimensions are
/// validated on each vector and must also match at retrieval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EmbeddingIdentity {
    provider: EmbeddingProvider,
    endpoint: String,
    model: String,
    profile: String,
    revision: String,
}

/// A non-empty, finite, non-zero vector with its configured provider/model identity.
#[derive(Debug, Clone)]
pub(crate) struct Embedding {
    pub(crate) identity: EmbeddingIdentity,
    pub(crate) values: Vec<f32>,
}

impl Embedding {
    pub(crate) fn model(&self) -> &str {
        &self.identity.model
    }
    pub(crate) fn revision_label(&self) -> String {
        match serde_json::from_str::<serde_json::Value>(&self.identity.revision) {
            Ok(value) => format!(
                "{} / {}",
                value["metadata"]["effective_metadata"]["upstream_revision"]
                    .as_str()
                    .unwrap_or("local package metadata"),
                value["loaded_at"].as_str().unwrap_or("unknown load")
            ),
            Err(_) => self.identity.revision.clone(),
        }
    }
    pub(crate) fn matches_config(&self, config: &EmbeddingConfig) -> bool {
        self.identity.provider == config.provider
            && config.provider != EmbeddingProvider::Disabled
            && self.identity.endpoint == config.base_url.trim_end_matches('/')
            && self.identity.model == config.model
            && self.identity.profile == config.profile
            && (config.provider == EmbeddingProvider::Pumas
                || self.identity.revision == config.revision)
    }
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

/// Real Pumas typed operations or an independently configured HTTP provider.
pub struct EmbeddingClient {
    client: Client,
    identity: EmbeddingIdentity,
    config: EmbeddingConfig,
    pumas: Option<crate::pumas_inference::PumasClient>,
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
    #[cfg(test)]
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            identity: EmbeddingIdentity {
                provider: EmbeddingProvider::OpenAiCompatible,
                endpoint: base_url.into().trim_end_matches('/').to_string(),
                model: model.into(),
                profile: String::new(),
                revision: "fixture".into(),
            },
            config: EmbeddingConfig::default(),
            pumas: None,
        }
    }

    pub(crate) async fn from_config(config: &EmbeddingConfig) -> Result<Self, String> {
        if config.provider == EmbeddingProvider::Disabled {
            return Err(
                "Reference embeddings are disabled; select an embedding provider/model".into(),
            );
        }
        if config.model.trim().is_empty() {
            return Err("Choose an embedding model".into());
        }
        let pumas = if config.provider == EmbeddingProvider::Pumas {
            Some(crate::pumas_inference::PumasClient::connect(&config.base_url).await?)
        } else {
            None
        };
        let revision = if let Some(client) = &pumas {
            client
                .embedding_revision(&config.model, &config.profile)
                .await?
        } else {
            if config.revision.trim().is_empty() {
                return Err("Configure an embedding model revision before indexing".into());
            }
            config.revision.clone()
        };
        Ok(Self {
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(180))
                .build()
                .map_err(|e| e.to_string())?,
            identity: EmbeddingIdentity {
                provider: config.provider,
                endpoint: config.base_url.trim_end_matches('/').to_owned(),
                model: config.model.clone(),
                profile: config.profile.clone(),
                revision,
            },
            config: config.clone(),
            pumas,
        })
    }

    #[cfg(test)]
    pub(crate) fn identity(&self) -> EmbeddingIdentity {
        self.identity.clone()
    }

    pub(crate) async fn embed(&self, text: &str) -> Result<Embedding, String> {
        if let Some(client) = &self.pumas {
            let before = client
                .embedding_revision(&self.identity.model, &self.identity.profile)
                .await?;
            if before != self.identity.revision {
                return Err("Pumas embedding model revision changed; reindex references".into());
            }
            let values = client
                .embed(&self.identity.model, &self.identity.profile, text)
                .await?;
            if client
                .embedding_revision(&self.identity.model, &self.identity.profile)
                .await?
                != before
            {
                return Err(
                    "Pumas embedding model changed during inference; reindex references".into(),
                );
            }
            return Embedding::new(self.identity.clone(), values);
        }
        let url = format!("{}/embeddings", self.identity.endpoint);
        let body = EmbedRequest {
            model: &self.identity.model,
            input: text,
        };
        let request = self.client.post(&url).json(&body);
        let request = match self.config.api_key.as_deref().filter(|key| !key.is_empty()) {
            Some(key) => request.bearer_auth(key),
            None => request,
        };
        let resp = request
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

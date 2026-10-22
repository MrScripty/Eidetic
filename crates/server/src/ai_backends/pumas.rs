use eidetic_core::{Error, ai::backend::GenerateStream};
use serde_json::json;

use super::BackendStatus;
use crate::{
    prompt_format::ChatPrompt,
    pumas_inference::{PumasClient, PumasOperation},
    state::{AiConfig, BackendType},
};

pub(crate) struct PumasBackend {
    config: Box<AiConfig>,
}
impl PumasBackend {
    pub(crate) fn new(config: &AiConfig) -> Self {
        Self {
            config: Box::new(config.clone()),
        }
    }
    pub(crate) async fn generate(
        &self,
        prompt: &ChatPrompt,
        config: &AiConfig,
    ) -> Result<GenerateStream, Error> {
        let client = PumasClient::connect(&config.base_url)
            .await
            .map_err(Error::AiBackend)?;
        let (id, response) = client.operation(PumasOperation {
            model: &config.model,
            profile: &config.pumas_profile,
            capability: "chat_generation",
            input: json!({"kind":"messages","messages":[{"role":"system","content":prompt.system},{"role":"user","content":prompt.user}]}),
            output: "text",
            options: json!({"kind":"text_generation","max_tokens":config.max_tokens,"temperature":config.temperature}),
            stream: true,
        }).await.map_err(Error::AiBackend)?;
        Ok(super::sse::pumas_tokens(
            response,
            id,
            config.model.clone(),
            config.pumas_profile.clone(),
        ))
    }
    pub(crate) async fn generate_json(
        &self,
        prompt: &ChatPrompt,
        config: &AiConfig,
    ) -> Result<String, Error> {
        // Typed v1 has no JSON-mode option. Require JSON in the prompt and validate
        // it here before the existing structured-output parsers see it.
        use futures::StreamExt;
        let prompt = ChatPrompt {
            system: format!(
                "{}\nReturn only valid JSON without Markdown fences.",
                prompt.system
            ),
            user: prompt.user.clone(),
        };
        let mut stream = self.generate(&prompt, config).await?;
        let mut full = String::new();
        while let Some(token) = stream.next().await {
            full.push_str(&token?);
        }
        serde_json::from_str::<serde_json::Value>(&full)
            .map_err(|e| Error::AiBackend(format!("Pumas returned invalid JSON: {e}")))?;
        Ok(full)
    }
    pub(crate) async fn health_check(&self) -> Result<BackendStatus, Error> {
        let client = PumasClient::connect(&self.config.base_url)
            .await
            .map_err(Error::AiBackend)?;
        client
            .selected(
                &self.config.model,
                &self.config.pumas_profile,
                "chat_generation",
            )
            .await
            .map_err(Error::AiBackend)?;
        Ok(BackendStatus {
            connected: true,
            model: self.config.model.clone(),
            backend_type: BackendType::Pumas,
            message: format!(
                "Pumas model {} loaded on {}",
                self.config.model, self.config.pumas_profile
            ),
        })
    }
}

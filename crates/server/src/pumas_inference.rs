//! Consumer of Pumas ad31e391's serving RPC and typed model-operation v1.
use std::time::Duration;

use pumas_library::discovery::{
    HTTP_DISCOVERY_PATH, HTTP_INSTANCE_GENERATION_HEADER, HTTP_SERVICE_GENERATION_HEADER,
    HttpServiceDescription, LoopbackHttpEndpoint,
};
pub use pumas_library::models::{ServeModelRequest, UnserveModelRequest, UnserveModelResponse};
use pumas_library::models::{ServedModelLoadState, ServedModelStatus, ServingStatusResponse};
use reqwest::{Client, RequestBuilder};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct PumasClient {
    client: Client,
    pub(crate) description: HttpServiceDescription,
}

#[derive(serde::Serialize)]
pub(crate) struct PumasOperation<'a> {
    pub(crate) model: &'a str,
    pub(crate) profile: &'a str,
    pub(crate) capability: &'a str,
    pub(crate) input: Value,
    pub(crate) output: &'a str,
    pub(crate) options: Value,
    pub(crate) stream: bool,
}

impl PumasClient {
    pub(crate) async fn connect(endpoint: &str) -> Result<Self, String> {
        let endpoint = LoopbackHttpEndpoint::parse(endpoint).map_err(|e| e.to_string())?;
        let client = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(180))
            .build()
            .map_err(|e| e.to_string())?;
        let response = client
            .get(format!("{}{HTTP_DISCOVERY_PATH}", endpoint.as_str()))
            .send()
            .await
            .map_err(|e| format!("Pumas discovery: {e}"))?;
        let description: HttpServiceDescription =
            serde_json::from_value(read_json(response).await?)
                .map_err(|e| format!("Pumas description: {e}"))?;
        if description.endpoint != endpoint || description.advertisement_schema_version != 1 {
            return Err("Pumas description endpoint/schema mismatch".into());
        }
        description.admission_fence().map_err(|e| e.to_string())?;
        if !description
            .build_info
            .compiled_features
            .iter()
            .any(|f| f == "pumas-rpc/inference-plugins")
        {
            return Err("Pumas was built without inference-plugins".into());
        }
        Ok(Self {
            client,
            description,
        })
    }

    pub(crate) fn for_description(description: HttpServiceDescription) -> Result<Self, String> {
        description.admission_fence().map_err(|e| e.to_string())?;
        let client = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(180))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            client,
            description,
        })
    }

    fn fenced(&self, request: RequestBuilder) -> RequestBuilder {
        request
            .header(
                HTTP_INSTANCE_GENERATION_HEADER,
                &self.description.instance.generation,
            )
            .header(
                HTTP_SERVICE_GENERATION_HEADER,
                &self.description.service_generation,
            )
    }

    pub(crate) async fn rpc(&self, method: &str, params: Value) -> Result<Value, String> {
        let id = Uuid::new_v4().to_string();
        let response = self
            .fenced(
                self.client
                    .post(format!("{}/rpc", self.description.endpoint.as_str())),
            )
            .json(&json!({"jsonrpc":"2.0", "id":id, "method":method, "params":params}))
            .send()
            .await
            .map_err(|e| format!("Pumas {method}: {e}"))?;
        let value = read_json(response).await?;
        if value["jsonrpc"] != "2.0" || value["id"] != id {
            return Err("Pumas RPC response correlation mismatch".into());
        }
        if let Some(error) = value.get("error") {
            return Err(format!("Pumas {method}: {error}"));
        }
        let result = value
            .get("result")
            .cloned()
            .ok_or("Pumas RPC missing result")?;
        if result.get("success") == Some(&Value::Bool(false))
            || result.get("loaded") == Some(&Value::Bool(false))
            || result.get("load_error").is_some_and(|e| !e.is_null())
        {
            return Err(format!(
                "Pumas {method}: {}",
                result
                    .get("load_error")
                    .or_else(|| result.get("error"))
                    .unwrap_or(&result)
            ));
        }
        Ok(result)
    }

    pub(crate) async fn selected(
        &self,
        model: &str,
        profile: &str,
        capability: &str,
    ) -> Result<ServedModelStatus, String> {
        if model.trim().is_empty()
            || model.eq_ignore_ascii_case("auto")
            || profile.trim().is_empty()
        {
            return Err("Choose an explicit Pumas model and runtime profile".into());
        }
        let status: ServingStatusResponse =
            serde_json::from_value(self.rpc("get_serving_status", json!({})).await?)
                .map_err(|e| format!("Pumas serving status: {e}"))?;
        let served = status
            .snapshot
            .served_models
            .into_iter()
            .find(|s| {
                s.model_id == model
                    && s.profile_id.as_str() == profile
                    && s.load_state == ServedModelLoadState::Loaded
            })
            .ok_or("Selected Pumas model/profile is not loaded; use Load in Pumas")?;
        let response = self
            .fenced(self.client.get(format!(
                "{}/v1/capabilities",
                self.description.endpoint.as_str()
            )))
            .query(&[("model", model), ("profile", profile)])
            .send()
            .await
            .map_err(|e| format!("Pumas capabilities: {e}"))?;
        let value = read_json(response).await?;
        if value["model"] != model
            || value["profile"] != profile
            || !value["supported_contract_versions"]
                .as_array()
                .is_some_and(|v| v.contains(&json!(1)))
        {
            return Err("Pumas capability response correlation/version mismatch".into());
        }
        if !value["capabilities"].as_array().is_some_and(|caps| {
            caps.iter()
                .any(|c| c["capability"] == capability && c["availability"]["state"] == "available")
        }) {
            return Err(format!(
                "Pumas {capability} unavailable for selected model/profile: {}",
                value["capabilities"]
            ));
        }
        Ok(served)
    }

    /// Session revision includes actual Pumas provenance and the selected load.
    /// It is deliberately not a claim that a mutable package is weight-attested.
    pub(crate) async fn embedding_revision(
        &self,
        model: &str,
        profile: &str,
    ) -> Result<String, String> {
        let status = self.selected(model, profile, "text_embedding").await?;
        let metadata = self
            .rpc("get_library_model_metadata", json!({"model_id":model}))
            .await?;
        if metadata["model_id"] != model {
            return Err("Pumas metadata model mismatch".into());
        }
        let loaded_at = status
            .loaded_at
            .ok_or("Pumas embedding load is missing revision timestamp")?;
        Ok(json!({"service":self.description.service_generation, "instance":self.description.instance.generation,
            "provider":status.provider, "profile":profile, "loaded_at":loaded_at,
            "metadata": metadata}).to_string())
    }

    pub(crate) async fn operation(
        &self,
        request: PumasOperation<'_>,
    ) -> Result<(String, reqwest::Response), String> {
        self.selected(request.model, request.profile, request.capability)
            .await?;
        let id = Uuid::new_v4().to_string();
        let mut body = serde_json::to_value(request).map_err(|e| e.to_string())?;
        body["contract_version"] = json!(1);
        body["request_id"] = json!(id);
        let response = self
            .fenced(self.client.post(format!(
                "{}/v1/model-operations",
                self.description.endpoint.as_str()
            )))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Pumas operation: {e}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "Pumas operation: {}",
                read_json(response).await.err().unwrap_or_default()
            ));
        }
        Ok((id, response))
    }

    pub(crate) async fn embed(
        &self,
        model: &str,
        profile: &str,
        text: &str,
    ) -> Result<Vec<f32>, String> {
        let (id, response) = self
            .operation(PumasOperation {
                model,
                profile,
                capability: "text_embedding",
                input: json!({"kind":"text", "text":text}),
                output: "embeddings_float32",
                options: json!({"kind":"embeddings"}),
                stream: false,
            })
            .await?;
        let value = read_json(response).await?;
        if value["contract_version"] != 1
            || value["request_id"] != id
            || value["result"]["kind"] != "embeddings"
        {
            return Err("Pumas embedding response contract/correlation mismatch".into());
        }
        let mut vectors: Vec<Vec<f32>> = serde_json::from_value(value["result"]["vectors"].clone())
            .map_err(|e| format!("Pumas embedding vectors: {e}"))?;
        if vectors.len() != 1 {
            return Err("Pumas embedding response must contain one vector".into());
        }
        Ok(vectors.remove(0))
    }
}

pub(crate) async fn read_json(mut response: reqwest::Response) -> Result<Value, String> {
    let status = response.status();
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("Pumas response transport: {e}"))?
    {
        if body.len() + chunk.len() > 32 * 1024 * 1024 {
            return Err("Pumas response exceeds 32 MiB".into());
        }
        body.extend_from_slice(&chunk);
    }
    let value: Value = serde_json::from_slice(&body)
        .map_err(|e| format!("Pumas HTTP {status}: invalid JSON ({e})"))?;
    if !status.is_success() || value.get("error").is_some() {
        return Err(format!(
            "Pumas HTTP {status}: {}",
            value.get("error").unwrap_or(&value)
        ));
    }
    Ok(value)
}

/// Explicit lifecycle controls: Pumas owns provider processes and native sessions.
/// No implicit unload or shutdown is sent to a borrowed Pumas service.
pub async fn load_model(endpoint: &str, request: ServeModelRequest) -> Result<Value, String> {
    let client = PumasClient::connect(endpoint).await?;
    client.rpc("serve_model", json!({"request":request})).await
}
pub async fn unload_model(
    endpoint: &str,
    request: UnserveModelRequest,
) -> Result<UnserveModelResponse, String> {
    let client = PumasClient::connect(endpoint).await?;
    let result = client
        .rpc("unserve_model", json!({"request":request}))
        .await?;
    let response: UnserveModelResponse =
        serde_json::from_value(result).map_err(|e| format!("Pumas unload response: {e}"))?;
    if !response.success || response.error.is_some() {
        return Err(format!(
            "Pumas unserve_model: {}",
            response.error.as_deref().unwrap_or("unload failed")
        ));
    }
    // `unloaded: false` with no error means already unserved, not an unload
    // receipt. Preserve the typed distinction for the operator interface.
    Ok(response)
}

pub async fn catalog(endpoint: &str) -> Result<Value, String> {
    let client = PumasClient::connect(endpoint).await?;
    let models = client.rpc("get_models", json!({})).await?;
    let models = models["models"].as_object().ok_or("Pumas catalog models must be an object")?.values().map(|entry| json!({
        "id":entry["id"], "official_name":entry["displayName"], "model_type":entry["modelType"]
    })).collect::<Vec<_>>();
    let profiles = client
        .rpc("get_runtime_profiles_snapshot", json!({}))
        .await?;
    Ok(json!({"models":{"models":models}, "profiles":profiles}))
}

#[cfg(test)]
#[path = "pumas_inference_tests.rs"]
pub(crate) mod tests;

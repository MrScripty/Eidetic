use std::path::PathBuf;

use pumas_library::ProviderRegistry;
use pumas_library::models::{RuntimeProfileId, RuntimeProviderId};
use pumas_library::runtime_profiles::{RuntimeProfileService, RuntimeProviderAdapters};
use reqwest::Url;

use crate::backend_error::BackendError;

const OPENAI_PATH: &str = "/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlamaCppEndpointSource {
    ExplicitConfig,
    PumasRuntimeProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlamaCppEndpointRequest {
    pub configured_base_url: Option<String>,
    pub pumas_launcher_root: Option<PathBuf>,
    pub pumas_profile_id: Option<String>,
    pub model_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLlamaCppEndpoint {
    pub openai_base_url: String,
    pub source: LlamaCppEndpointSource,
}

pub async fn resolve_llamacpp_openai_endpoint(
    request: LlamaCppEndpointRequest,
) -> Result<ResolvedLlamaCppEndpoint, BackendError> {
    if let Some(configured_base_url) = request
        .configured_base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("auto"))
    {
        return Ok(ResolvedLlamaCppEndpoint {
            openai_base_url: normalize_loopback_openai_base_url(configured_base_url)?,
            source: LlamaCppEndpointSource::ExplicitConfig,
        });
    }

    let launcher_root = request.pumas_launcher_root.ok_or_else(|| {
        BackendError::bad_request(
            "Pumas launcher root is required when llama.cpp base URL is not explicitly configured",
        )
    })?;
    let service = RuntimeProfileService::with_provider_registry_and_adapters(
        launcher_root,
        ProviderRegistry::builtin(),
        RuntimeProviderAdapters::builtin(),
    );
    let profile_id = parse_profile_id(request.pumas_profile_id.as_deref())?;
    let endpoint = match request
        .model_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("auto"))
    {
        Some(model_id) => {
            service
                .resolve_model_endpoint_for_operation(
                    RuntimeProviderId::LlamaCpp,
                    model_id,
                    profile_id,
                )
                .await
        }
        None => {
            service
                .resolve_profile_endpoint_for_operation(RuntimeProviderId::LlamaCpp, profile_id)
                .await
        }
    }
    .map_err(|error| BackendError::internal(format!("Pumas llama.cpp endpoint error: {error}")))?;

    Ok(ResolvedLlamaCppEndpoint {
        openai_base_url: normalize_loopback_openai_base_url(endpoint.as_str())?,
        source: LlamaCppEndpointSource::PumasRuntimeProfile,
    })
}

fn parse_profile_id(value: Option<&str>) -> Result<Option<RuntimeProfileId>, BackendError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(RuntimeProfileId::parse)
        .transpose()
        .map_err(BackendError::bad_request)
}

fn normalize_loopback_openai_base_url(value: &str) -> Result<String, BackendError> {
    let mut url = Url::parse(value)
        .map_err(|error| BackendError::bad_request(format!("invalid llama.cpp URL: {error}")))?;
    if url.scheme() != "http" {
        return Err(BackendError::bad_request(
            "llama.cpp URL must use local http transport",
        ));
    }
    if !is_loopback_host(&url) {
        return Err(BackendError::bad_request(
            "llama.cpp URL must target a loopback host",
        ));
    }
    if url.port().is_none() {
        return Err(BackendError::bad_request(
            "llama.cpp URL must include an explicit port",
        ));
    }

    match url.path().trim_end_matches('/') {
        "" | "/" => url.set_path(OPENAI_PATH),
        OPENAI_PATH => url.set_path(OPENAI_PATH),
        _ => {
            return Err(BackendError::bad_request(
                "llama.cpp URL must be the server root or /v1 OpenAI-compatible base path",
            ));
        }
    }
    url.set_query(None);
    url.set_fragment(None);

    Ok(url.to_string().trim_end_matches('/').to_string())
}

fn is_loopback_host(url: &Url) -> bool {
    url.host_str().is_some_and(|host| {
        host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1" || host == "::1"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn resolver_normalizes_explicit_loopback_server_root_to_openai_base() {
        let resolved = resolve_llamacpp_openai_endpoint(LlamaCppEndpointRequest {
            configured_base_url: Some("http://127.0.0.1:18080".to_string()),
            pumas_launcher_root: None,
            pumas_profile_id: None,
            model_id: None,
        })
        .await
        .unwrap();

        assert_eq!(resolved.openai_base_url, "http://127.0.0.1:18080/v1");
        assert_eq!(resolved.source, LlamaCppEndpointSource::ExplicitConfig);
    }

    #[tokio::test]
    async fn resolver_preserves_explicit_openai_base_path_without_query() {
        let resolved = resolve_llamacpp_openai_endpoint(LlamaCppEndpointRequest {
            configured_base_url: Some("http://localhost:18080/v1/?debug=true".to_string()),
            pumas_launcher_root: None,
            pumas_profile_id: None,
            model_id: None,
        })
        .await
        .unwrap();

        assert_eq!(resolved.openai_base_url, "http://localhost:18080/v1");
    }

    #[tokio::test]
    async fn resolver_rejects_remote_explicit_urls() {
        let error = resolve_llamacpp_openai_endpoint(LlamaCppEndpointRequest {
            configured_base_url: Some("http://192.0.2.10:18080/v1".to_string()),
            pumas_launcher_root: None,
            pumas_profile_id: None,
            model_id: None,
        })
        .await
        .unwrap_err();

        assert!(error.message().contains("loopback"));
    }

    #[tokio::test]
    async fn resolver_requires_pumas_root_for_auto_endpoint_resolution() {
        let error = resolve_llamacpp_openai_endpoint(LlamaCppEndpointRequest {
            configured_base_url: Some("auto".to_string()),
            pumas_launcher_root: None,
            pumas_profile_id: None,
            model_id: None,
        })
        .await
        .unwrap_err();

        assert!(error.message().contains("Pumas launcher root"));
    }
}

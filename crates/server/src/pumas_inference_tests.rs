//! Controlled producer fixtures. Vectors and screenplay text are fixture outputs,
//! never real-model evidence. These exercise the consumer's actual wire path.
use super::*;
use crate::{
    embeddings::{EmbeddingClient, EmbeddingConfig, EmbeddingProvider},
    state::{AiConfig, AppState, BackendType},
};
use parking_lot::Mutex;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

pub(crate) struct Fixture {
    pub(crate) url: String,
    pub(crate) requests: Arc<Mutex<Vec<(String, Value)>>>,
    revision: Arc<AtomicUsize>,
    fail_embedding: Arc<AtomicBool>,
    unavailable: Arc<AtomicBool>,
    loaded: Arc<AtomicBool>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Fixture {
    pub(crate) async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let revision = Arc::new(AtomicUsize::new(1));
        let fail_embedding = Arc::new(AtomicBool::new(false));
        let unavailable = Arc::new(AtomicBool::new(false));
        let loaded = Arc::new(AtomicBool::new(true));
        let (root_url, reqs, rev, fail, unavail, is_loaded) = (
            url.clone(),
            requests.clone(),
            revision.clone(),
            fail_embedding.clone(),
            unavailable.clone(),
            loaded.clone(),
        );
        let task = tokio::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                let (url, reqs, rev, fail, unavail, is_loaded) = (
                    root_url.clone(),
                    reqs.clone(),
                    rev.clone(),
                    fail.clone(),
                    unavail.clone(),
                    is_loaded.clone(),
                );
                tokio::spawn(async move {
                    let mut request = Vec::new();
                    let (header_end, length) = loop {
                        let mut buffer = [0; 4096];
                        let count = socket.read(&mut buffer).await.unwrap();
                        if count == 0 {
                            return;
                        }
                        request.extend_from_slice(&buffer[..count]);
                        if let Some(end) = request.windows(4).position(|v| v == b"\r\n\r\n") {
                            let headers = String::from_utf8_lossy(&request[..end]).to_lowercase();
                            let length = headers
                                .lines()
                                .find_map(|l| l.strip_prefix("content-length: "))
                                .map(|v| v.parse::<usize>().unwrap())
                                .unwrap_or(0);
                            break (end + 4, length);
                        }
                    };
                    while request.len() < header_end + length {
                        let mut buffer = [0; 4096];
                        let count = socket.read(&mut buffer).await.unwrap();
                        if count == 0 {
                            return;
                        }
                        request.extend_from_slice(&buffer[..count]);
                    }
                    let headers = String::from_utf8_lossy(&request[..header_end]);
                    let path = headers
                        .lines()
                        .next()
                        .unwrap()
                        .split_whitespace()
                        .nth(1)
                        .unwrap()
                        .to_owned();
                    let body: Value = if length == 0 {
                        json!({})
                    } else {
                        serde_json::from_slice(&request[header_end..header_end + length]).unwrap()
                    };
                    reqs.lock().push((path.clone(), body.clone()));
                    if path != HTTP_DISCOVERY_PATH {
                        assert!(headers.contains("pumas-instance-generation: instance-fixture"));
                        assert!(headers.contains("pumas-service-generation: service-fixture"));
                    }
                    let response = if path == HTTP_DISCOVERY_PATH {
                        description(&url)
                    } else if path.starts_with("/v1/capabilities?") {
                        let query = reqwest::Url::parse(&format!("{url}{path}")).unwrap();
                        let pairs: std::collections::HashMap<_, _> =
                            query.query_pairs().into_owned().collect();
                        json!({"supported_contract_versions":[1],"model":pairs["model"],"profile":pairs["profile"],"capabilities":[
                            {"capability":"chat_generation","availability":{"state":if unavail.load(Ordering::SeqCst) {"unavailable"} else {"available"}}},
                            {"capability":"text_embedding","availability":{"state":if unavail.load(Ordering::SeqCst) {"unavailable"} else {"available"}}}
                        ]})
                    } else if path == "/rpc" {
                        let result = match body["method"].as_str().unwrap() {
                            "get_serving_status" => {
                                json!({"success":true,"snapshot":{"schema_version":1,"cursor":"serving:1","endpoint":{"endpoint_mode":"pumas_gateway","model_count":2},"last_errors":[],"served_models":if is_loaded.load(Ordering::SeqCst) {vec![served("embed",rev.load(Ordering::SeqCst)),served("chat",1)]} else {vec![]}}})
                            }
                            "get_library_model_metadata" => {
                                json!({"success":true,"model_id":body["params"]["model_id"],"effective_metadata":{"upstream_revision":format!("weights-{}",rev.load(Ordering::SeqCst))}})
                            }
                            "serve_model" if body["params"]["request"]["model_id"] == "missing" => {
                                json!({"success":true,"loaded":false,"load_error":{"code":"model_not_found","message":"fixture model missing"}})
                            }
                            "serve_model" => {
                                is_loaded.store(true, Ordering::SeqCst);
                                json!({"success":true,"loaded":true})
                            }
                            "unserve_model" => {
                                is_loaded.store(false, Ordering::SeqCst);
                                json!({"success":true,"unloaded":true})
                            }
                            "shutdown" => json!({"success":true}),
                            _ => panic!("Unexpected fixture RPC {}", body["method"]),
                        };
                        json!({"jsonrpc":"2.0","id":body["id"],"result":result})
                    } else if path == "/v1/model-operations" {
                        if body["capability"] == "text_embedding" {
                            if fail.load(Ordering::SeqCst) {
                                json!({"contract_version":1,"request_id":body["request_id"],"error":{"code":"provider_failure","outcome":"not_admitted"}})
                            } else {
                                json!({"contract_version":1,"request_id":body["request_id"],"result":{"kind":"embeddings","vectors":[[1.,2.,3.]]}})
                            }
                        } else {
                            let events = [
                                json!({"kind":"started","contract_version":1,"request_id":body["request_id"],"capability":"chat_generation","model":"chat","profile":"fixture"}),
                                json!({"kind":"delta","request_id":body["request_id"],"text":"Fixture screenplay"}),
                                json!({"kind":"completed","request_id":body["request_id"],"finish_reason":"stop"}),
                            ];
                            let text = events
                                .iter()
                                .map(|e| format!("data: {e}\n\n"))
                                .collect::<String>();
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                                text.len()
                            );
                            let _ = socket.write_all(response.as_bytes()).await;
                            return;
                        }
                    } else {
                        panic!("Unexpected fixture path {path}")
                    };
                    let text = response.to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                        text.len()
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                });
            }
        });
        Self {
            url,
            requests,
            revision,
            fail_embedding,
            unavailable,
            loaded,
            task,
        }
    }
    fn embedding_config(&self) -> EmbeddingConfig {
        EmbeddingConfig {
            provider: EmbeddingProvider::Pumas,
            base_url: self.url.clone(),
            model: "embed".into(),
            profile: "fixture".into(),
            ..Default::default()
        }
    }
}
pub(crate) fn description(url: &str) -> Value {
    json!({"advertisement_schema_version":1,"service_generation":"service-fixture","endpoint":url,
        "instance":{"discovery_schema_version":1,"registry_library_id":"fixture","library_root":"/tmp/fixture","generation":"instance-fixture","pumas_version":"0.8.0-rc.1","protocols":[],"capabilities":[],"model_ref_schema_version":1,"selector_schema_version":1},
        "build_info":{"build_info_schema_version":1,"component":"pumas-rpc","package_version":"0.8.0-rc.1","compiled_features":["pumas-rpc/inference-plugins"],"protocols":[],"schemas":[{"name":"pumas.http-admission-fence","version":1}]}})
}
fn served(model: &str, revision: usize) -> Value {
    json!({"model_id":model,"provider":"onnx_runtime","profile_id":"fixture","load_state":"loaded","device_mode":"cpu","keep_loaded":true,"loaded_at":format!("load-{revision}")})
}

#[tokio::test]
async fn pumas_real_wire_path_load_embed_revision_change_unload_and_errors() {
    let fixture = Fixture::start().await;
    let config = fixture.embedding_config();
    let client = EmbeddingClient::from_config(&config).await.unwrap();
    let first = client.embed("Reference text").await.unwrap();
    assert_eq!(first.values, vec![1., 2., 3.]);
    fixture.revision.store(2, Ordering::SeqCst);
    assert!(
        client
            .embed("Query")
            .await
            .unwrap_err()
            .contains("revision changed")
    );
    let replacement = EmbeddingClient::from_config(&config)
        .await
        .unwrap()
        .embed("Query")
        .await
        .unwrap();
    assert_ne!(first.identity, replacement.identity);
    fixture.fail_embedding.store(true, Ordering::SeqCst);
    assert!(
        EmbeddingClient::from_config(&config)
            .await
            .unwrap()
            .embed("Query")
            .await
            .unwrap_err()
            .contains("provider_failure")
    );
    fixture.unavailable.store(true, Ordering::SeqCst);
    assert!(EmbeddingClient::from_config(&config).await.is_err());
    fixture.unavailable.store(false, Ordering::SeqCst);
    let unload: UnserveModelRequest = serde_json::from_value(
        json!({"model_id":"embed","provider":"onnx_runtime","profile_id":"fixture"}),
    )
    .unwrap();
    unload_model(&fixture.url, unload).await.unwrap();
    assert!(!fixture.loaded.load(Ordering::SeqCst));
    assert!(EmbeddingClient::from_config(&config).await.is_err());
    let request = |model| {
        serde_json::from_value::<ServeModelRequest>(json!({"model_id":model,"config":{"provider":"onnx_runtime","profile_id":"fixture","device_mode":"cpu"}})).unwrap()
    };
    assert!(
        load_model(&fixture.url, request("missing"))
            .await
            .unwrap_err()
            .contains("model_not_found")
    );
    load_model(&fixture.url, request("embed")).await.unwrap();
    assert!(EmbeddingClient::from_config(&config).await.is_ok());
}

async fn ready(state: &AppState) {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let status = crate::reference_service::index_status(state).unwrap();
            if status.documents.iter().all(|d| d.state == "ready") {
                return;
            }
            if status.documents.iter().any(|d| d.state == "failed") {
                panic!("Fixture index failed: {:?}", status);
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn upload_reopen_change_model_retrieve_and_chat_generation_use_real_consumer_path() {
    use eidetic_core::{Template, ai::prompt::build_generate_request};
    let fixture = Fixture::start().await;
    let state = AppState::new().await;
    state.ai_config.lock().embedding = fixture.embedding_config();
    let project = Template::MultiCam.build_project("Pumas fixture");
    let path = std::env::temp_dir().join(format!("eidetic-pumas-{}.db", Uuid::new_v4()));
    crate::project_service::replace_active_project(&state, project, path.clone());
    let doc = crate::reference_service::upload_reference(
        &state,
        crate::reference_service::UploadReferenceRequest {
            name: "Secret location".into(),
            content: "The secret station is called Marmalade Harbor.".into(),
            doc_type: "WorldBuilding".into(),
        },
    )
    .unwrap();
    ready(&state).await;
    let saved = state.project.lock().as_ref().unwrap().clone();
    crate::persistence::save_project(&saved, &path, None)
        .await
        .unwrap();
    let (reopened, _) = crate::persistence::load_project(&path).await.unwrap();
    let old_scope = state.vector_store.lock().scope();
    crate::project_service::replace_active_project(&state, reopened.clone(), path.clone());
    ready(&state).await;
    let scope = state.vector_store.lock().scope();
    assert_ne!(old_scope, scope);
    let mut request = build_generate_request(&reopened, reopened.timeline.nodes[0].id).unwrap();
    let config = state.ai_config.lock().clone();
    crate::ai_generation_runtime::attach_rag_context(&state, &config, &path, scope, &mut request)
        .await;
    assert_eq!(request.rag_context[0].content, doc.content);
    let prompt = crate::prompt_format::build_chat_prompt(&request);
    let config = AiConfig {
        backend_type: BackendType::Pumas,
        base_url: fixture.url.clone(),
        model: "chat".into(),
        pumas_profile: "fixture".into(),
        ..Default::default()
    };
    assert_eq!(
        crate::ai_backends::Backend::from_config(&config)
            .generate_full(&prompt, &config)
            .await
            .unwrap(),
        "Fixture screenplay"
    );
    let sent = fixture
        .requests
        .lock()
        .iter()
        .find(|(path, body)| {
            path == "/v1/model-operations" && body["capability"] == "chat_generation"
        })
        .unwrap()
        .1
        .clone();
    assert!(
        sent["input"]["messages"]
            .to_string()
            .contains("Marmalade Harbor")
    );
    fixture.revision.store(2, Ordering::SeqCst);
    let embedding_config = state.ai_config.lock().clone();
    crate::ai_generation_runtime::attach_rag_context(
        &state,
        &embedding_config,
        &path,
        scope,
        &mut request,
    )
    .await;
    assert!(request.rag_context.is_empty());
    crate::reference_service::schedule_reindex(&state);
    ready(&state).await;
    crate::reference_service::delete_reference(&state, doc.id.0).unwrap();
    assert!(state.vector_store.lock().is_empty());
    state.shutdown_tasks_async().await;
    // Borrowing a producer for inference never sends runtime shutdown.
    assert!(
        !fixture
            .requests
            .lock()
            .iter()
            .any(|(_, body)| body["method"] == "shutdown")
    );
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn embedding_failure_is_visible_and_does_not_publish_partial_document() {
    let fixture = Fixture::start().await;
    fixture.fail_embedding.store(true, Ordering::SeqCst);
    let state = AppState::new().await;
    state.ai_config.lock().embedding = fixture.embedding_config();
    *state.project.lock() = Some(eidetic_core::Template::MultiCam.build_project("Failure"));
    crate::reference_service::upload_reference(
        &state,
        crate::reference_service::UploadReferenceRequest {
            name: "Failed reference".into(),
            content: "Test source".into(),
            doc_type: "StyleGuide".into(),
        },
    )
    .unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let status = crate::reference_service::index_status(&state).unwrap();
            if status.documents[0].state == "failed" {
                assert!(
                    status.documents[0]
                        .error
                        .as_ref()
                        .unwrap()
                        .contains("provider_failure")
                );
                assert_eq!(status.documents[0].indexed_chunks, 0);
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(state.vector_store.lock().is_empty());
    state.shutdown_tasks_async().await;
}

#[tokio::test]
async fn remote_or_unsupported_pumas_endpoint_is_refused() {
    assert!(PumasClient::connect("https://example.com").await.is_err());
    assert!(PumasClient::connect("http://localhost:8080").await.is_err());
    assert!(
        EmbeddingClient::from_config(&EmbeddingConfig::default())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn pumas_embedding_selection_change_invalidates_pending_source_tickets() {
    use crate::ai_service::{AiConfigUpdate, update_ai_config};
    let fixture = Fixture::start().await;
    let state = AppState::new().await;
    let mut project = eidetic_core::Template::MultiCam.build_project("Model change");
    project
        .references
        .push(eidetic_core::reference::ReferenceDocument::new(
            "Ref",
            "Source content",
            eidetic_core::reference::ReferenceType::StyleGuide,
        ));
    *state.project.lock() = Some(project);
    update_ai_config(
        &state,
        AiConfigUpdate {
            embedding: Some(fixture.embedding_config()),
            ..Default::default()
        },
    );
    ready(&state).await;
    let scope = state.vector_store.lock().scope();
    let mut replacement = fixture.embedding_config();
    replacement.model = "missing-model".into();
    update_ai_config(
        &state,
        AiConfigUpdate {
            embedding: Some(replacement),
            ..Default::default()
        },
    );
    assert_ne!(scope, state.vector_store.lock().scope());
    assert!(state.vector_store.lock().is_empty());
    tokio::time::timeout(Duration::from_secs(10), async {
        while crate::reference_service::index_status(&state)
            .unwrap()
            .documents[0]
            .state
            != "failed"
        {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    update_ai_config(
        &state,
        AiConfigUpdate {
            embedding: Some(fixture.embedding_config()),
            ..Default::default()
        },
    );
    ready(&state).await;
    state.shutdown_tasks_async().await;
}

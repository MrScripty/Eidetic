use super::router;
use std::path::PathBuf;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use serde_json::json;
use tower::util::ServiceExt;

use crate::state::AppState;

async fn app_with_project_path(path: PathBuf) -> Router {
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Commands Test"));
    *state.project_path.lock() = Some(path);
    router().with_state(state)
}

use eidetic_core::contracts::{
    ScriptBlockId, ScriptBlockKind, ScriptDocumentId, ScriptLockId, ScriptSegmentId,
    ScriptSegmentStatus, ScriptSpanId,
};

#[tokio::test]
async fn script_block_command_returns_script_document_projection() {
    let path = temp_db_path("sets-script-block");
    let app = app_with_project_path(path.clone()).await;
    let body = script_block_command_body("Ada enters with a wet umbrella.");

    let response = app
        .oneshot(script_block_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 5);
    assert_eq!(value["projection"]["payload"]["document"]["title"], "Pilot");
    assert_eq!(
        value["projection"]["payload"]["segments"][0]["blocks"][0]["block"]["text"],
        "Ada enters with a wet umbrella."
    );
    assert_eq!(
        value["projection"]["payload"]["segments"][0]["blocks"][0]["spans"][0]["end_byte"],
        31
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn script_block_command_rejects_invalid_segment_range() {
    let path = temp_db_path("rejects-invalid-script-range");
    let app = app_with_project_path(path.clone()).await;
    let mut body = script_block_command_body("Ada enters with a wet umbrella.");
    body["payload"]["segment_start_ms"] = json!(5_000);
    body["payload"]["segment_end_ms"] = json!(1_000);

    let response = app
        .oneshot(script_block_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn script_block_command_rejects_unexpected_project_session_fields() {
    let path = temp_db_path("rejects-unexpected-script-block-fields");
    let app = app_with_project_path(path.clone()).await;
    let mut body = script_block_command_body("Ada enters with a wet umbrella.");
    body["project_id"] = json!("renderer-project");
    body["session_id"] = json!("renderer-session");

    let response = app
        .oneshot(script_block_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(!path.exists());

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn script_block_command_rejects_unexpected_payload_fields() {
    let path = temp_db_path("rejects-unexpected-script-block-payload-fields");
    let app = app_with_project_path(path.clone()).await;
    let mut body = script_block_command_body("Ada enters with a wet umbrella.");
    body["payload"]["renderer_note"] = json!("not part of the command contract");

    let response = app
        .oneshot(script_block_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(!path.exists());

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn script_lock_command_returns_script_document_projection() {
    let path = temp_db_path("sets-script-lock");
    let app = app_with_project_path(path.clone()).await;
    let block = script_block_command_body("Ada enters with a wet umbrella.");
    let lock = script_lock_command_body("User approved wording.");

    let block_response = app
        .clone()
        .oneshot(script_block_command_request(block))
        .await
        .expect("block route response");
    let lock_response = app
        .oneshot(script_lock_command_request(lock))
        .await
        .expect("lock route response");

    assert_eq!(block_response.status(), StatusCode::OK);
    assert_eq!(lock_response.status(), StatusCode::OK);
    let value = response_json(lock_response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 6);
    assert_eq!(
        value["projection"]["payload"]["segments"][0]["blocks"][0]["locks"][0]["reason"],
        "User approved wording."
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn script_lock_command_rejects_missing_span() {
    let path = temp_db_path("rejects-missing-script-span");
    let app = app_with_project_path(path.clone()).await;
    let body = script_lock_command_body("User approved wording.");

    let response = app
        .oneshot(script_lock_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

fn script_block_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/script/block")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn script_lock_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/script/lock")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn script_block_command_body(text: &str) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "document_id": ScriptDocumentId::new("script.document.main").unwrap(),
            "document_title": "Pilot",
            "document_sort_order": 0,
            "segment_id": ScriptSegmentId::new("script.segment.beat-1").unwrap(),
            "source_node_id": "node.beat.opening",
            "segment_start_ms": 1_000,
            "segment_end_ms": 5_000,
            "segment_status": ScriptSegmentStatus::Current,
            "segment_sort_order": 1,
            "block_id": ScriptBlockId::new("script.block.action-1").unwrap(),
            "block_kind": ScriptBlockKind::Action,
            "text": text,
            "span_provenance": "user_edited",
            "sort_order": 2,
        }
    })
}

fn script_lock_command_body(reason: &str) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "lock_id": ScriptLockId::new("script.lock.action-1").unwrap(),
            "span_id": ScriptSpanId::new("script.block.action-1.span.main").unwrap(),
            "reason": reason,
        }
    })
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json response")
}

fn temp_db_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "eidetic-command-route-{label}-{}.db",
        uuid::Uuid::new_v4()
    ))
}

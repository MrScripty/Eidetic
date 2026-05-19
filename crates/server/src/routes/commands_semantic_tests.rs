use super::router;
use std::path::PathBuf;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{BibleReferenceKind, SemanticProposalId};
use eidetic_core::timeline::node::NodeId;
use serde_json::json;
use tower::util::ServiceExt;

use crate::state::AppState;

async fn app_with_project_path(path: PathBuf) -> Router {
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Semantic Commands Test"));
    *state.project_path.lock() = Some(path);
    router().with_state(state)
}

#[tokio::test]
async fn bible_reference_proposal_command_returns_projection() {
    let path = temp_db_path("creates-bible-reference-proposal");
    let app = app_with_project_path(path.clone()).await;
    let body = bible_reference_proposal_command_body(
        "proposal.child.ada",
        NodeId::new(),
        "Opening Beat",
        BibleReferenceKind::Character,
        "Ada",
    );

    let response = app
        .oneshot(bible_reference_proposal_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 2);
    assert_eq!(
        value["projection"]["payload"]["proposals"][0]["id"],
        "proposal.child.ada"
    );
    assert_eq!(
        value["projection"]["payload"]["proposals"][0]["reference_kind"],
        "character"
    );
    assert_eq!(
        value["projection"]["payload"]["proposals"][0]["proposed_schema_key"],
        "character"
    );
    assert_eq!(
        value["projection"]["payload"]["proposals"][0]["status"],
        "pending"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_reference_proposal_command_replays_duplicate_command() {
    let path = temp_db_path("replays-bible-reference-proposal");
    let app = app_with_project_path(path.clone()).await;
    let body = bible_reference_proposal_command_body(
        "proposal.child.harbor",
        NodeId::new(),
        "Second Beat",
        BibleReferenceKind::Location,
        "Storm Harbor",
    );

    let first = app
        .clone()
        .oneshot(bible_reference_proposal_command_request(body.clone()))
        .await
        .expect("first route response");
    let second = app
        .oneshot(bible_reference_proposal_command_request(body))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    let value = response_json(second).await;
    assert_eq!(value["outcome"], "already_recorded");
    assert_eq!(
        value["projection"]["payload"]["proposals"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_reference_proposal_reject_command_returns_projection() {
    let path = temp_db_path("rejects-bible-reference-proposal");
    let app = app_with_project_path(path.clone()).await;
    let create = bible_reference_proposal_command_body(
        "proposal.child.ring",
        NodeId::new(),
        "Second Beat",
        BibleReferenceKind::Prop,
        "Signal ring",
    );
    let reject = reject_bible_reference_proposal_command_body(
        "proposal.child.ring",
        Some("Not important enough for the bible"),
    );

    let create_response = app
        .clone()
        .oneshot(bible_reference_proposal_command_request(create))
        .await
        .expect("create route response");
    let reject_response = app
        .oneshot(reject_bible_reference_proposal_command_request(reject))
        .await
        .expect("reject route response");

    assert_eq!(create_response.status(), StatusCode::OK);
    assert_eq!(reject_response.status(), StatusCode::OK);
    let value = response_json(reject_response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(
        value["projection"]["payload"]["proposals"][0]["status"],
        "rejected"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_reference_proposal_reject_command_rejects_missing_proposal() {
    let path = temp_db_path("rejects-missing-bible-reference-proposal");
    let app = app_with_project_path(path.clone()).await;
    let reject = reject_bible_reference_proposal_command_body("proposal.child.missing", None);

    let response = app
        .oneshot(reject_bible_reference_proposal_command_request(reject))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_reference_proposal_command_rejects_blank_reference_text() {
    let path = temp_db_path("rejects-blank-bible-reference");
    let app = app_with_project_path(path.clone()).await;
    let body = bible_reference_proposal_command_body(
        "proposal.child.blank",
        NodeId::new(),
        "Third Beat",
        BibleReferenceKind::Prop,
        " ",
    );

    let response = app
        .oneshot(bible_reference_proposal_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

fn bible_reference_proposal_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/semantic/bible-reference-proposal")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn reject_bible_reference_proposal_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/semantic/bible-reference-proposal/reject")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn bible_reference_proposal_command_body(
    proposal_id: &str,
    source_node_id: NodeId,
    child_name: &str,
    reference_kind: BibleReferenceKind,
    reference_text: &str,
) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "proposal_id": SemanticProposalId::new(proposal_id).unwrap(),
            "source_node_id": source_node_id,
            "child_name": child_name,
            "reference_kind": reference_kind,
            "reference_text": reference_text,
            "rationale": "Detected while planning timeline children",
        }
    })
}

fn reject_bible_reference_proposal_command_body(
    proposal_id: &str,
    reason: Option<&str>,
) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "proposal_id": SemanticProposalId::new(proposal_id).unwrap(),
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
        "eidetic-semantic-command-route-{label}-{}.db",
        uuid::Uuid::new_v4()
    ))
}

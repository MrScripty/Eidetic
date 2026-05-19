use super::router;
use std::path::PathBuf;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{
    BibleReferenceKind, CommandEnvelope, CreateBibleReferenceProposalCommand, SemanticProposalId,
};
use eidetic_core::timeline::node::NodeId;
use tower::util::ServiceExt;

use crate::state::AppState;

async fn app_with_project_path(path: PathBuf) -> Router {
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Semantic Projection Test"));
    *state.project_path.lock() = Some(path);
    router().with_state(state)
}

#[tokio::test]
async fn bible_reference_proposal_projection_returns_empty_list_when_absent() {
    let path = temp_db_path("semantic-proposals-empty");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(bible_reference_proposal_projection_request())
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 1);
    assert_eq!(value["payload"]["proposals"], serde_json::json!([]));

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_reference_proposal_projection_returns_persisted_proposals() {
    let path = temp_db_path("semantic-proposals-populated");
    seed_bible_reference_proposal(&path);
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(bible_reference_proposal_projection_request())
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 2);
    assert_eq!(
        value["payload"]["proposals"][0]["id"],
        "proposal.child.ring"
    );
    assert_eq!(value["payload"]["proposals"][0]["reference_kind"], "prop");
    assert_eq!(value["payload"]["proposals"][0]["status"], "pending");

    let _ = std::fs::remove_file(path);
}

fn seed_bible_reference_proposal(path: &PathBuf) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    let command = CommandEnvelope::new(CreateBibleReferenceProposalCommand {
        proposal_id: SemanticProposalId::new("proposal.child.ring").unwrap(),
        source_node_id: NodeId::new(),
        child_name: "Inciting Beat".to_string(),
        reference_kind: BibleReferenceKind::Prop,
        reference_text: "Signal ring".to_string(),
        rationale: None,
    });
    crate::semantic_proposal_store::record_create_bible_reference_proposal(
        &mut conn, &command, 100,
    )
    .unwrap();
}

fn bible_reference_proposal_projection_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/projections/semantic/bible-reference-proposals")
        .body(Body::empty())
        .unwrap()
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json response")
}

fn temp_db_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "eidetic-semantic-projection-route-{label}-{}.db",
        uuid::Uuid::new_v4()
    ))
}

use super::router;
use std::path::PathBuf;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{
    BibleGraphNodeId, BibleReferenceKind, CommandEnvelope, EnsureCanonicalBibleRootsCommand,
    FieldValue, PropagationProposalAction, PropagationProposalId, RejectPropagationProposalCommand,
    ScriptSegmentId, SemanticDependencyId, SemanticDependencyKind, SemanticProposalId,
};
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
async fn bible_reference_proposal_accept_command_returns_projection() {
    let path = temp_db_path("accepts-bible-reference-proposal");
    seed_roots(&path);
    let app = app_with_project_path(path.clone()).await;
    let create = bible_reference_proposal_command_body(
        "proposal.child.harbor",
        NodeId::new(),
        "Second Beat",
        BibleReferenceKind::Location,
        "Storm Harbor",
    );
    let accept = accept_bible_reference_proposal_command_body(
        "proposal.child.harbor",
        "node.location.storm-harbor",
        None,
    );

    let create_response = app
        .clone()
        .oneshot(bible_reference_proposal_command_request(create))
        .await
        .expect("create route response");
    let accept_response = app
        .oneshot(accept_bible_reference_proposal_command_request(accept))
        .await
        .expect("accept route response");

    assert_eq!(create_response.status(), StatusCode::OK);
    assert_eq!(accept_response.status(), StatusCode::OK);
    let value = response_json(accept_response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(
        value["projection"]["payload"]["proposals"][0]["status"],
        "accepted"
    );
    let conn = crate::sqlite::open_write_connection(&path).unwrap();
    let node = crate::bible_graph_store::load_node_detail_projection(
        &conn,
        &BibleGraphNodeId::new("node.location.storm-harbor").unwrap(),
    )
    .unwrap()
    .expect("accepted node");
    assert_eq!(node.node.name, "Storm Harbor");

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_reference_proposal_accept_command_rejects_missing_parent() {
    let path = temp_db_path("accept-rejects-missing-parent");
    let app = app_with_project_path(path.clone()).await;
    let create = bible_reference_proposal_command_body(
        "proposal.child.ada",
        NodeId::new(),
        "Second Beat",
        BibleReferenceKind::Character,
        "Ada",
    );
    let accept = accept_bible_reference_proposal_command_body(
        "proposal.child.ada",
        "node.character.ada",
        None,
    );

    let create_response = app
        .clone()
        .oneshot(bible_reference_proposal_command_request(create))
        .await
        .expect("create route response");
    let accept_response = app
        .oneshot(accept_bible_reference_proposal_command_request(accept))
        .await
        .expect("accept route response");

    assert_eq!(create_response.status(), StatusCode::OK);
    assert_eq!(accept_response.status(), StatusCode::BAD_REQUEST);

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

#[tokio::test]
async fn semantic_dependency_command_returns_source_projection() {
    let path = temp_db_path("records-semantic-dependency");
    let app = app_with_project_path(path.clone()).await;
    let body = semantic_dependency_command_body("dependency.weather.scene");

    let response = app
        .oneshot(semantic_dependency_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 2);
    assert_eq!(
        value["projection"]["payload"]["dependencies"][0]["id"],
        "dependency.weather.scene"
    );
    assert_eq!(
        value["projection"]["payload"]["dependencies"][0]["target"]["kind"],
        "bible_field"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn propagation_proposal_command_returns_projection() {
    let path = temp_db_path("creates-propagation-proposal");
    let app = app_with_project_path(path.clone()).await;
    let body = propagation_proposal_command_body("proposal.propagation.weather");

    let response = app
        .oneshot(propagation_proposal_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 2);
    assert_eq!(
        value["projection"]["payload"]["proposals"][0]["id"],
        "proposal.propagation.weather"
    );
    assert_eq!(
        value["projection"]["payload"]["proposals"][0]["status"],
        "pending"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn propagation_proposal_reject_command_returns_projection() {
    let path = temp_db_path("rejects-propagation-proposal");
    let app = app_with_project_path(path.clone()).await;
    let create = propagation_proposal_command_body("proposal.propagation.reject");
    let reject = reject_propagation_proposal_command_body(
        "proposal.propagation.reject",
        Some("Wrong scope"),
    );

    let create_response = app
        .clone()
        .oneshot(propagation_proposal_command_request(create))
        .await
        .expect("create route response");
    let reject_response = app
        .oneshot(reject_propagation_proposal_command_request(reject))
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

fn accept_bible_reference_proposal_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/semantic/bible-reference-proposal/accept")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn semantic_dependency_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/semantic/dependency")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn propagation_proposal_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/semantic/propagation-proposal")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn reject_propagation_proposal_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/semantic/propagation-proposal/reject")
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

fn semantic_dependency_command_body(dependency_id: &str) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "dependency": {
                "id": SemanticDependencyId::new(dependency_id).unwrap(),
                "source": {
                    "kind": "script_segment",
                    "segment_id": ScriptSegmentId::new("script.segment.scene-1").unwrap(),
                },
                "target": {
                    "kind": "bible_field",
                    "node_id": BibleGraphNodeId::new("node.location.harbor").unwrap(),
                    "part_key": "weather",
                    "field_key": "current",
                },
                "kind": SemanticDependencyKind::UsesFact,
                "rationale": "The generated scene uses the harbor weather.",
                "confidence": 0.84,
                "created_at_ms": 100,
            }
        }
    })
}

fn propagation_proposal_command_body(proposal_id: &str) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "proposal_id": PropagationProposalId::new(proposal_id).unwrap(),
            "action": PropagationProposalAction::SetBibleField,
            "target": {
                "kind": "bible_field",
                "node_id": BibleGraphNodeId::new("node.location.harbor").unwrap(),
                "part_key": "weather",
                "field_key": "current",
            },
            "summary": "Set harbor weather to rainy",
            "proposed_value": FieldValue::Text("rainy".to_string()),
            "source_dependency_id": SemanticDependencyId::new("dependency.weather.scene").unwrap(),
            "rationale": "Manual script edit introduced rainy weather.",
        }
    })
}

fn reject_propagation_proposal_command_body(
    proposal_id: &str,
    reason: Option<&str>,
) -> serde_json::Value {
    let command = RejectPropagationProposalCommand {
        proposal_id: PropagationProposalId::new(proposal_id).unwrap(),
        reason: reason.map(ToString::to_string),
    };
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": command,
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

fn accept_bible_reference_proposal_command_body(
    proposal_id: &str,
    node_id: &str,
    name: Option<&str>,
) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "proposal_id": SemanticProposalId::new(proposal_id).unwrap(),
            "node_id": BibleGraphNodeId::new(node_id).unwrap(),
            "name": name,
            "sort_order": 11,
        }
    })
}

fn seed_roots(path: &PathBuf) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    crate::bible_graph_command::apply_ensure_canonical_bible_roots(
        &mut conn,
        &CommandEnvelope::new(EnsureCanonicalBibleRootsCommand {}),
        1,
    )
    .unwrap();
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

use super::router;
use std::path::PathBuf;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{
    AcceptPropagationProposalCommand, BibleGraphNodeId, BibleGraphSchemaKey, CommandEnvelope,
    CreateBibleGraphNodeCommand, FieldValue, PropagationProposalAction, PropagationProposalId,
    RejectPropagationProposalCommand, SemanticDependencyId,
};
use serde_json::json;
use tower::util::ServiceExt;

use crate::state::AppState;

async fn app_with_project_path(path: PathBuf) -> Router {
    let state = AppState::new().await;
    *state.project.lock() =
        Some(Template::MultiCam.build_project("Semantic Propagation Commands Test"));
    *state.project_path.lock() = Some(path);
    router().with_state(state)
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

#[tokio::test]
async fn propagation_proposal_accept_command_updates_bible_field() {
    let path = temp_db_path("accepts-propagation-proposal");
    seed_location_node(&path);
    let app = app_with_project_path(path.clone()).await;
    let create = propagation_proposal_command_body("proposal.propagation.accept");
    let accept = accept_propagation_proposal_command_body("proposal.propagation.accept");

    let create_response = app
        .clone()
        .oneshot(propagation_proposal_command_request(create))
        .await
        .expect("create route response");
    let accept_response = app
        .oneshot(accept_propagation_proposal_command_request(accept))
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
    let detail = crate::bible_graph_store::load_node_detail_projection(
        &conn,
        &BibleGraphNodeId::new("node.location.harbor").unwrap(),
    )
    .unwrap()
    .expect("node detail");
    let field = detail
        .parts
        .iter()
        .flat_map(|part| part.fields.iter())
        .find(|field| field.field_key.as_str() == "weather")
        .expect("weather field");
    assert_eq!(field.value, Some(FieldValue::Text("rainy".to_string())));

    let _ = std::fs::remove_file(path);
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

fn accept_propagation_proposal_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/semantic/propagation-proposal/accept")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
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
                "part_key": "environment",
                "field_key": "weather",
            },
            "summary": "Set harbor weather to rainy",
            "proposed_value": FieldValue::Text("rainy".to_string()),
            "source_dependency_id": SemanticDependencyId::new("dependency.weather.scene").unwrap(),
            "rationale": "Manual script edit introduced rainy weather.",
        }
    })
}

fn accept_propagation_proposal_command_body(proposal_id: &str) -> serde_json::Value {
    let command = AcceptPropagationProposalCommand {
        proposal_id: PropagationProposalId::new(proposal_id).unwrap(),
    };
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": command,
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

fn seed_location_node(path: &PathBuf) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    crate::bible_graph_command::apply_create_bible_graph_node(
        &mut conn,
        &CommandEnvelope::new(CreateBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new("node.location.harbor").unwrap(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("location").unwrap(),
            name: "Storm Harbor".to_string(),
            sort_order: 1,
        }),
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
        "eidetic-semantic-propagation-command-route-{label}-{}.db",
        uuid::Uuid::new_v4()
    ))
}

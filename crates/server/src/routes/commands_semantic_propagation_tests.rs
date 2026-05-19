use super::router;
use std::path::PathBuf;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{
    AcceptPropagationProposalCommand, BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey,
    BibleGraphSchemaKey, BibleGraphSnapshotFieldId, BibleGraphSnapshotId, CommandEnvelope,
    CreateBibleGraphNodeCommand, FieldValue, PropagationProposalAction, PropagationProposalId,
    RejectPropagationProposalCommand, ScriptBlockId, ScriptBlockKind, ScriptDocumentId,
    ScriptSegmentId, ScriptSegmentStatus, ScriptSpanProvenance, SemanticDependencyId,
    SetBibleGraphSnapshotFieldCommand, SetScriptBlockCommand,
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

#[tokio::test]
async fn propagation_proposal_accept_command_updates_script_block() {
    let path = temp_db_path("accepts-script-block-propagation-proposal");
    seed_script_block(&path, "Ada enters with a wet umbrella.");
    let app = app_with_project_path(path.clone()).await;
    let create = script_block_proposal_command_body(
        "proposal.propagation.script-block",
        "Ada enters with a rain-black umbrella.",
    );
    let accept = accept_propagation_proposal_command_body("proposal.propagation.script-block");

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
    let projection = crate::script_store::load_document_projection(
        &conn,
        &ScriptDocumentId::new("script.document.main").unwrap(),
    )
    .unwrap()
    .expect("script projection");
    assert_eq!(
        projection.segments[0].blocks[0].block.text,
        "Ada enters with a rain-black umbrella."
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn propagation_proposal_accept_command_updates_snapshot_field() {
    let path = temp_db_path("accepts-snapshot-field-propagation-proposal");
    seed_character_snapshot_field(&path, "Rain-soaked");
    let app = app_with_project_path(path.clone()).await;
    let create = snapshot_field_proposal_command_body(
        "proposal.propagation.snapshot",
        FieldValue::Text("Dry and wary".to_string()),
    );
    let accept = accept_propagation_proposal_command_body("proposal.propagation.snapshot");

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
        &BibleGraphNodeId::new("node.character.ada").unwrap(),
    )
    .unwrap()
    .expect("node detail");
    assert_eq!(
        detail.snapshots[0].fields[0].value,
        Some(FieldValue::Text("Dry and wary".to_string()))
    );

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

fn script_block_proposal_command_body(proposal_id: &str, proposed_text: &str) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "proposal_id": PropagationProposalId::new(proposal_id).unwrap(),
            "action": PropagationProposalAction::PatchScriptBlock,
            "target": {
                "kind": "script_block",
                "block_id": ScriptBlockId::new("script.block.action-1").unwrap(),
            },
            "summary": "Patch generated script block",
            "proposed_text": proposed_text,
            "source_dependency_id": SemanticDependencyId::new("dependency.weather.scene").unwrap(),
            "rationale": "Manual script edit requires propagation.",
        }
    })
}

fn snapshot_field_proposal_command_body(
    proposal_id: &str,
    proposed_value: FieldValue,
) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "proposal_id": PropagationProposalId::new(proposal_id).unwrap(),
            "action": PropagationProposalAction::SetBibleSnapshotField,
            "target": {
                "kind": "bible_snapshot_field",
                "node_id": BibleGraphNodeId::new("node.character.ada").unwrap(),
                "snapshot_id": BibleGraphSnapshotId::new("snapshot.character.ada.sequence-1").unwrap(),
                "part_key": BibleGraphPartKey::new("profile").unwrap(),
                "field_key": BibleGraphFieldKey::new("tagline").unwrap(),
                "field_id": BibleGraphSnapshotFieldId::new("snapshot-field.character.status").unwrap(),
            },
            "summary": "Set character snapshot status",
            "proposed_value": proposed_value,
            "source_dependency_id": SemanticDependencyId::new("dependency.weather.scene").unwrap(),
            "rationale": "Manual script edit requires snapshot propagation.",
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

fn seed_script_block(path: &PathBuf, text: &str) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    crate::script_document_command::apply_set_script_block(
        &mut conn,
        &CommandEnvelope::new(SetScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            document_title: "Pilot".to_string(),
            document_sort_order: 0,
            segment_id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
            source_node_id: Some("node.beat.opening".to_string()),
            segment_start_ms: 1_000,
            segment_end_ms: 5_000,
            segment_status: ScriptSegmentStatus::Current,
            segment_sort_order: 1,
            block_id: ScriptBlockId::new("script.block.action-1").unwrap(),
            block_kind: ScriptBlockKind::Action,
            text: text.to_string(),
            span_provenance: ScriptSpanProvenance::AiGenerated,
            sort_order: 2,
        }),
        1,
    )
    .unwrap();
}

fn seed_character_snapshot_field(path: &PathBuf, value: &str) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    crate::bible_graph_command::apply_create_bible_graph_node(
        &mut conn,
        &CommandEnvelope::new(CreateBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            name: "Ada".to_string(),
            sort_order: 1,
        }),
        1,
    )
    .unwrap();
    crate::bible_graph_command::apply_set_bible_graph_snapshot_field(
        &mut conn,
        &CommandEnvelope::new(SetBibleGraphSnapshotFieldCommand {
            snapshot_id: BibleGraphSnapshotId::new("snapshot.character.ada.sequence-1").unwrap(),
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            at_ms: 12_000,
            label: "Sequence 1 state".to_string(),
            snapshot_sort_order: 1,
            field_id: BibleGraphSnapshotFieldId::new("snapshot-field.character.status").unwrap(),
            part_key: BibleGraphPartKey::new("profile").unwrap(),
            part_name: "Profile".to_string(),
            field_key: BibleGraphFieldKey::new("tagline").unwrap(),
            value: Some(FieldValue::Text(value.to_string())),
            field_sort_order: 2,
        }),
        2,
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

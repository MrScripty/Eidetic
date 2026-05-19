use super::router;
use std::path::PathBuf;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::ai::backend::{ChildPlan, ChildPlanId, ChildProposal};
use eidetic_core::contracts::{
    BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey, BibleReferenceKind, CommandEnvelope,
    CreateBibleReferenceProposalCommand, CreatePropagationProposalCommand, FieldValue,
    PropagationProposalAction, PropagationProposalId, PropagationProposalTarget,
    RecordSemanticDependencyCommand, ScriptSegmentId, SemanticDependency,
    SemanticDependencyEndpoint, SemanticDependencyId, SemanticDependencyKind, SemanticProposalId,
};
use eidetic_core::timeline::node::{NodeId, StoryLevel};
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

#[tokio::test]
async fn semantic_dependency_projection_filters_by_target_field() {
    let path = temp_db_path("semantic-dependencies-target-field");
    seed_semantic_dependency(&path);
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(semantic_dependency_projection_request(
            "/projections/semantic/dependencies?target_kind=bible_field&target_id=node.location.harbor&target_part_key=weather&target_field_key=current",
        ))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 2);
    assert_eq!(
        value["payload"]["dependencies"][0]["id"],
        "dependency.weather.scene"
    );
    assert_eq!(
        value["payload"]["dependencies"][0]["source"]["kind"],
        "script_segment"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn semantic_dependency_projection_rejects_missing_filter() {
    let path = temp_db_path("semantic-dependencies-missing-filter");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(semantic_dependency_projection_request(
            "/projections/semantic/dependencies",
        ))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn propagation_proposal_projection_returns_persisted_proposals() {
    let path = temp_db_path("propagation-proposals-populated");
    seed_propagation_proposal(&path);
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(propagation_proposal_projection_request())
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 2);
    assert_eq!(
        value["payload"]["proposals"][0]["id"],
        "proposal.propagation.weather"
    );
    assert_eq!(value["payload"]["proposals"][0]["status"], "pending");
    assert_eq!(
        value["payload"]["proposals"][0]["target"]["kind"],
        "bible_field"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn child_plan_projection_returns_persisted_plans() {
    let path = temp_db_path("child-plans-populated");
    seed_child_plan(&path);
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(child_plan_projection_request())
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 2);
    assert_eq!(
        value["payload"]["plans"][0]["plan"]["id"],
        "child_plan.projection"
    );
    assert_eq!(value["payload"]["plans"][0]["status"], "pending");
    assert_eq!(
        value["payload"]["plans"][0]["plan"]["children"][0]["characters"][0],
        "Ada"
    );

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

fn seed_semantic_dependency(path: &PathBuf) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    let command = CommandEnvelope::new(RecordSemanticDependencyCommand {
        dependency: SemanticDependency {
            id: SemanticDependencyId::new("dependency.weather.scene").unwrap(),
            source: SemanticDependencyEndpoint::ScriptSegment {
                segment_id: ScriptSegmentId::new("script.segment.scene-1").unwrap(),
            },
            target: SemanticDependencyEndpoint::BibleField {
                node_id: BibleGraphNodeId::new("node.location.harbor").unwrap(),
                part_key: BibleGraphPartKey::new("weather").unwrap(),
                field_key: BibleGraphFieldKey::new("current").unwrap(),
                field_id: None,
            },
            kind: SemanticDependencyKind::UsesFact,
            rationale: Some("Scene uses this bible field.".to_string()),
            confidence: Some(0.9),
            created_at_ms: 100,
        },
    });
    crate::semantic_dependency_store::record_semantic_dependency(&mut conn, &command, 100).unwrap();
}

fn seed_propagation_proposal(path: &PathBuf) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    let command = CommandEnvelope::new(CreatePropagationProposalCommand {
        proposal_id: PropagationProposalId::new("proposal.propagation.weather").unwrap(),
        action: PropagationProposalAction::SetBibleField,
        target: PropagationProposalTarget::BibleField {
            node_id: BibleGraphNodeId::new("node.location.harbor").unwrap(),
            part_key: BibleGraphPartKey::new("weather").unwrap(),
            field_key: BibleGraphFieldKey::new("current").unwrap(),
            field_id: None,
        },
        summary: "Set harbor weather to rainy".to_string(),
        proposed_value: Some(FieldValue::Text("rainy".to_string())),
        proposed_text: None,
        proposed_script_patch: None,
        source_dependency_id: Some(SemanticDependencyId::new("dependency.weather.scene").unwrap()),
        source_event_id: None,
        rationale: Some("Manual script edit introduced rainy weather.".to_string()),
    });
    crate::propagation_proposal_store::record_create_propagation_proposal(&mut conn, &command, 100)
        .unwrap();
}

fn seed_child_plan(path: &PathBuf) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    let plan = ChildPlan {
        id: ChildPlanId::new("child_plan.projection").unwrap(),
        parent_node_id: NodeId::new(),
        target_child_level: StoryLevel::Scene,
        children: vec![ChildProposal {
            name: "Arrival".to_string(),
            level: None,
            beat_type: None,
            outline: "Ada arrives at the harbor.".to_string(),
            weight: 1.0,
            characters: vec!["Ada".to_string()],
            location: Some("Harbor".to_string()),
            props: vec!["Signal ring".to_string()],
        }],
    };
    crate::child_plan_store::record_child_plan(&mut conn, &plan, 100).unwrap();
}

fn bible_reference_proposal_projection_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/projections/semantic/bible-reference-proposals")
        .body(Body::empty())
        .unwrap()
}

fn semantic_dependency_projection_request(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

fn propagation_proposal_projection_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/projections/semantic/propagation-proposals")
        .body(Body::empty())
        .unwrap()
}

fn child_plan_projection_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/projections/semantic/child-plans")
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

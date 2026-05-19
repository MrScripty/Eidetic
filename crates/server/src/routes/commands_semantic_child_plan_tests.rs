use super::router;
use std::path::PathBuf;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::ai::backend::{ChildPlan, ChildPlanId, ChildProposal, RejectChildPlanCommand};
use eidetic_core::contracts::CommandEnvelope;
use eidetic_core::timeline::node::{NodeId, StoryLevel};
use serde_json::json;
use tower::util::ServiceExt;

use crate::state::AppState;

#[tokio::test]
async fn child_plan_reject_command_returns_projection() {
    let path = temp_db_path("reject-child-plan");
    seed_child_plan(&path);
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Child Plan Command Test"));
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let plan_id = ChildPlanId::new("child_plan.reject").unwrap();
    let command = CommandEnvelope::new(RejectChildPlanCommand {
        plan_id: plan_id.clone(),
        reason: Some("User discarded this generated structure".to_string()),
    });

    let response = app
        .oneshot(child_plan_reject_request(json!(command)))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(
        value["projection"]["payload"]["plans"][0]["status"],
        "rejected"
    );

    let _ = std::fs::remove_file(path);
}

fn seed_child_plan(path: &PathBuf) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    let plan = ChildPlan {
        id: ChildPlanId::new("child_plan.reject").unwrap(),
        parent_node_id: NodeId::new(),
        target_child_level: StoryLevel::Scene,
        children: vec![ChildProposal {
            name: "Discarded scene".to_string(),
            level: None,
            beat_type: None,
            outline: "This generated scene is not used.".to_string(),
            weight: 1.0,
            characters: Vec::new(),
            location: None,
            props: Vec::new(),
        }],
    };
    crate::child_plan_store::record_child_plan(&mut conn, &plan, 100).unwrap();
}

fn child_plan_reject_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/semantic/child-plan/reject")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
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
        "eidetic-child-plan-command-{label}-{}.db",
        uuid::Uuid::new_v4()
    ))
}

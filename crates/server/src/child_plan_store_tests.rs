use eidetic_core::ai::backend::{ChildPlan, ChildPlanId, ChildProposal};
use eidetic_core::contracts::ObjectKind;
use eidetic_core::timeline::node::{BeatType, NodeId, StoryLevel};
use uuid::Uuid;

use super::{load_child_plan, record_child_plan};
use crate::history_store;

#[test]
fn records_and_loads_child_plan_rows() {
    let mut conn = rusqlite::Connection::open_in_memory().expect("sqlite");
    let plan = sample_plan();

    record_child_plan(&mut conn, &plan, 42).expect("record child plan");

    let loaded = load_child_plan(&conn, &plan.id)
        .expect("load child plan")
        .expect("persisted child plan");
    assert_eq!(loaded.id, plan.id);
    assert_eq!(loaded.parent_node_id, plan.parent_node_id);
    assert_eq!(loaded.target_child_level, StoryLevel::Scene);
    assert_eq!(loaded.children.len(), 2);
    assert_eq!(loaded.children[0].name, "Arrival");
    assert_eq!(loaded.children[0].beat_type, Some(BeatType::Setup));
    assert_eq!(loaded.children[0].characters, vec!["Ada", "Byron"]);
    assert_eq!(loaded.children[0].location.as_deref(), Some("Harbor"));
    assert_eq!(loaded.children[0].props, vec!["Signal ring"]);
}

#[test]
fn records_history_revision_for_child_plan_creation() {
    let mut conn = rusqlite::Connection::open_in_memory().expect("sqlite");
    let plan = sample_plan();

    record_child_plan(&mut conn, &plan, 42).expect("record child plan");

    let revisions =
        history_store::load_revisions_for_object(&conn, ObjectKind::ChildPlan, plan.id.as_str())
            .expect("load revisions");
    assert_eq!(revisions.len(), 1);
    assert_eq!(revisions[0].fields.len(), 4);
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "child_count")
    );
}

#[test]
fn rejects_empty_child_plans_without_writes() {
    let mut conn = rusqlite::Connection::open_in_memory().expect("sqlite");
    let mut plan = sample_plan();
    plan.children.clear();

    let error = record_child_plan(&mut conn, &plan, 42).expect_err("invalid plan");

    assert!(error.to_string().contains("at least one child"));
    assert!(
        load_child_plan(&conn, &plan.id)
            .expect("load absent plan")
            .is_none()
    );
}

fn sample_plan() -> ChildPlan {
    ChildPlan {
        id: ChildPlanId::new("child_plan.test").unwrap(),
        parent_node_id: NodeId(Uuid::new_v4()),
        target_child_level: StoryLevel::Scene,
        children: vec![
            ChildProposal {
                name: "Arrival".to_string(),
                level: None,
                beat_type: Some(BeatType::Setup),
                outline: "Ada arrives at the storm harbor.".to_string(),
                weight: 1.0,
                characters: vec!["Ada".to_string(), "Byron".to_string()],
                location: Some("Harbor".to_string()),
                props: vec!["Signal ring".to_string()],
            },
            ChildProposal {
                name: "Confrontation".to_string(),
                level: None,
                beat_type: None,
                outline: "The crew argues over the signal.".to_string(),
                weight: 1.5,
                characters: vec!["Ada".to_string()],
                location: None,
                props: Vec::new(),
            },
        ],
    }
}

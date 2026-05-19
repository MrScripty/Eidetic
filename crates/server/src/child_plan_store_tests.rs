use eidetic_core::ai::backend::{ChildPlan, ChildPlanId, ChildProposal};
use eidetic_core::contracts::ObjectKind;
use eidetic_core::timeline::node::{BeatType, NodeId, StoryLevel};
use uuid::Uuid;

use super::record_child_plan;
use crate::history_store;

#[test]
fn records_and_loads_child_plan_rows() {
    let mut conn = rusqlite::Connection::open_in_memory().expect("sqlite");
    let plan = sample_plan();

    record_child_plan(&mut conn, &plan, 42).expect("record child plan");

    let parent_node_id: String = conn
        .query_row(
            "SELECT parent_node_id FROM child_plans WHERE id = ?1",
            [plan.id.as_str()],
            |row| row.get(0),
        )
        .expect("stored child plan");
    let child_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM child_plan_children WHERE plan_id = ?1",
            [plan.id.as_str()],
            |row| row.get(0),
        )
        .expect("stored child rows");
    let first_child_name: String = conn
        .query_row(
            "SELECT name FROM child_plan_children WHERE plan_id = ?1 AND child_index = 0",
            [plan.id.as_str()],
            |row| row.get(0),
        )
        .expect("stored first child");
    let first_child_refs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM child_plan_child_references WHERE plan_id = ?1 AND child_index = 0",
            [plan.id.as_str()],
            |row| row.get(0),
        )
        .expect("stored first child references");

    assert_eq!(parent_node_id, plan.parent_node_id.0.to_string());
    assert_eq!(child_count, 2);
    assert_eq!(first_child_name, "Arrival");
    assert_eq!(first_child_refs, 3);
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
    let row_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM child_plans", [], |row| row.get(0))
        .expect("child plan count");
    assert_eq!(row_count, 0);
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

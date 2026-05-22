use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphNodeId, CommandEnvelope, ContextEvaluation,
    ContextEvaluationTaskKind, ContextInfluenceKind, ContextInfluenceProvenance,
    ContextInfluenceRecord, ObjectKind, RecordContextEvaluationCommand,
};
use eidetic_core::timeline::node::{NodeId, StoryLevel};

use super::{
    load_context_influence_projection, load_latest_context_evaluations, record_context_evaluation,
};
use crate::history_store::{self, RecordChangeOutcome};

#[test]
fn records_and_projects_context_influences() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let target_node_id = NodeId::new();
    let command = context_command(target_node_id);

    let outcome = record_context_evaluation(&mut conn, &command, 100).unwrap();
    let projection = load_context_influence_projection(&conn, target_node_id)
        .unwrap()
        .unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(projection.version.0, 3);
    assert_eq!(projection.payload.target_node_id, target_node_id);
    assert_eq!(
        projection.payload.task_kind,
        ContextEvaluationTaskKind::GenerateTimelineContext
    );
    assert_eq!(projection.payload.records.len(), 1);
    assert_eq!(
        projection.payload.records[0]
            .bible_node_id
            .as_ref()
            .unwrap()
            .as_str(),
        "node.place.harbor"
    );
    assert_eq!(
        projection.payload.records[0].influence_kind,
        ContextInfluenceKind::Direct
    );

    let revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::ContextInfluence,
        &command.payload.influences[0].id.0.to_string(),
    )
    .unwrap();
    assert_eq!(revisions.len(), 1);
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "confidence")
    );
}

#[test]
fn replays_duplicate_context_evaluation_command() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let command = context_command(NodeId::new());

    let first = record_context_evaluation(&mut conn, &command, 100).unwrap();
    let second = record_context_evaluation(&mut conn, &command, 100).unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
}

#[test]
fn rejects_out_of_range_context_confidence() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let mut command = context_command(NodeId::new());
    command.payload.influences[0].confidence = 1.4;

    let error = record_context_evaluation(&mut conn, &command, 100).unwrap_err();

    assert!(error.to_string().contains("confidence"));
}

#[test]
fn rejects_influence_for_different_evaluation() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let mut command = context_command(NodeId::new());
    command.payload.influences[0].evaluation_id =
        eidetic_core::contracts::ContextEvaluationId::new();

    let error = record_context_evaluation(&mut conn, &command, 100).unwrap_err();

    assert!(error.to_string().contains("different evaluation"));
}

#[test]
fn loads_latest_context_evaluations_for_context_stack_layers() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let parent_node_id = NodeId::new();
    let child_node_id = NodeId::new();
    let old_parent = context_command_with_distilled_context(
        parent_node_id,
        "Old parent context",
        "old parent context",
        10,
    );
    let new_parent = context_command_with_distilled_context(
        parent_node_id,
        "New parent context",
        "new parent context",
        20,
    );
    let child =
        context_command_with_distilled_context(child_node_id, "Child context", "child context", 30);
    record_context_evaluation(&mut conn, &old_parent, 10).unwrap();
    record_context_evaluation(&mut conn, &new_parent, 20).unwrap();
    record_context_evaluation(&mut conn, &child, 30).unwrap();

    let evaluations =
        load_latest_context_evaluations(&conn, &[parent_node_id, child_node_id]).unwrap();

    assert_eq!(evaluations.len(), 2);
    assert_eq!(
        evaluations
            .iter()
            .find(|evaluation| evaluation.target_node_id == parent_node_id)
            .and_then(|evaluation| evaluation.distilled_context.as_deref()),
        Some("new parent context")
    );
    assert_eq!(
        evaluations
            .iter()
            .find(|evaluation| evaluation.target_node_id == child_node_id)
            .and_then(|evaluation| evaluation.distilled_context.as_deref()),
        Some("child context")
    );
}

fn context_command(target_node_id: NodeId) -> CommandEnvelope<RecordContextEvaluationCommand> {
    context_command_with_distilled_context(
        target_node_id,
        "Scene context evaluation",
        "Harbor weather shapes the scene.",
        100,
    )
}

fn context_command_with_distilled_context(
    target_node_id: NodeId,
    summary: &str,
    distilled_context: &str,
    created_at_ms: u64,
) -> CommandEnvelope<RecordContextEvaluationCommand> {
    let evaluation_id = eidetic_core::contracts::ContextEvaluationId::new();
    CommandEnvelope::new(RecordContextEvaluationCommand {
        evaluation: ContextEvaluation {
            id: evaluation_id,
            target_node_id,
            task_kind: ContextEvaluationTaskKind::GenerateTimelineContext,
            summary: summary.to_string(),
            distilled_context: Some(distilled_context.to_string()),
            created_at_ms,
        },
        influences: vec![ContextInfluenceRecord {
            id: eidetic_core::contracts::ContextInfluenceId::new(),
            evaluation_id,
            timeline_node_id: target_node_id,
            source_layer: StoryLevel::Scene,
            influence_kind: ContextInfluenceKind::Direct,
            confidence: 0.91,
            reason: "Scene takes place at the harbor.".to_string(),
            provenance: ContextInfluenceProvenance::AiSelected,
            bible_node_id: Some(BibleGraphNodeId::new("node.place.harbor").unwrap()),
            bible_edge_id: Some(BibleGraphEdgeId::new("edge.scene.harbor").unwrap()),
            introduced_by_node_id: Some(target_node_id),
            sort_order: 1,
        }],
    })
}

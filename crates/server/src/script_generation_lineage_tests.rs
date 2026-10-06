use eidetic_core::contracts::*;
use eidetic_core::timeline::node::NodeId;
use rusqlite::{Connection, params};

use crate::{history_store, script_block_edit, script_document_command, script_store};

fn seed(conn: &mut Connection, name: &str, start: u64) -> SetScriptBlockCommand {
    let block = SetScriptBlockCommand {
        document_id: ScriptDocumentId::new("script.document.main").unwrap(),
        document_title: "Story".into(),
        document_sort_order: 0,
        segment_id: ScriptSegmentId::new(format!("segment.{name}")).unwrap(),
        source_node_id: Some(NodeId::new().0.to_string()),
        segment_start_ms: start,
        segment_end_ms: start + 1000,
        segment_status: ScriptSegmentStatus::Current,
        segment_sort_order: 0,
        block_id: ScriptBlockId::new(format!("block.{name}")).unwrap(),
        block_kind: ScriptBlockKind::Action,
        text: format!("Original {name}"),
        span_provenance: ScriptSpanProvenance::UserEdited,
        sort_order: 0,
    };
    script_document_command::apply_set_script_block(conn, &CommandEnvelope::new(block.clone()), 10)
        .unwrap();
    block
}

fn fixture() -> (
    Connection,
    SetScriptBlockCommand,
    SetScriptBlockCommand,
    SetScriptBlockCommand,
) {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON").unwrap();
    let a = seed(&mut conn, "A", 0);
    let b = seed(&mut conn, "B", 1000);
    let c = seed(&mut conn, "C", 9000);
    (conn, a, b, c)
}

fn input(conn: &Connection, block: &SetScriptBlockCommand) -> ScriptContextBlock {
    crate::ai_script_context::load_script_context(
        conn,
        NodeId(uuid::Uuid::parse_str(block.source_node_id.as_ref().unwrap()).unwrap()),
        block.segment_start_ms,
        block.segment_end_ms,
    )
    .unwrap()
    .into_iter()
    .find(|input| input.block_id == block.block_id)
    .unwrap()
}

fn generation(
    block: &SetScriptBlockCommand,
    inputs: Option<Vec<ScriptContextBlock>>,
    text: &str,
) -> CommandEnvelope<GenerateScriptBlockCommand> {
    let mut block = block.clone();
    block.text = text.into();
    block.span_provenance = ScriptSpanProvenance::AiGenerated;
    CommandEnvelope::new(GenerateScriptBlockCommand {
        target_binding: None,
        script_context_scope: None,
        bible_inputs: None,
        bible_context_scope: None,
        block,
        script_inputs: inputs,
    })
}

fn edit(conn: &mut Connection, block: &SetScriptBlockCommand, text: &str) {
    let before = input(conn, block);
    script_block_edit::apply_edit_script_block(
        conn,
        &CommandEnvelope::new(EditScriptBlockCommand {
            document_id: block.document_id.clone(),
            block_id: block.block_id.clone(),
            expected_revision_event_id: before.revision_event_id,
            text: text.into(),
        }),
        20,
    )
    .unwrap();
}

fn projection(conn: &Connection) -> ProjectionEnvelope<ScriptDocumentProjection> {
    script_store::load_document_projection_envelope(
        conn,
        &ScriptDocumentId::new("script.document.main").unwrap(),
    )
    .unwrap()
    .unwrap()
}

fn impact(conn: &Connection, block: &SetScriptBlockCommand) -> ScriptImpactProjection {
    projection(conn)
        .payload
        .segments
        .into_iter()
        .find(|segment| segment.segment.id == block.segment_id)
        .unwrap()
        .impact
        .unwrap()
}

fn counts(conn: &Connection) -> (i64, i64, i64, i64) {
    conn.query_row("SELECT (SELECT COUNT(*) FROM commands), (SELECT COUNT(*) FROM object_revisions), (SELECT COUNT(*) FROM script_generations), (SELECT COUNT(*) FROM semantic_dependencies)", [], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).unwrap()
}

#[test]
fn source_edit_marks_dependent_b_and_leaves_unrelated_c_and_authored_text_unchanged() {
    let (mut conn, a, b, c) = fixture();
    let original = input(&conn, &a);
    let command = generation(&b, Some(vec![original.clone()]), "B follows A");
    script_document_command::apply_generated_script_block(&mut conn, &command, 15).unwrap();
    script_document_command::apply_generated_script_block(
        &mut conn,
        &generation(&c, Some(vec![]), "Independent C"),
        16,
    )
    .unwrap();
    let output = input(&conn, &b);
    assert!(!impact(&conn, &b).needs_review);
    edit(&mut conn, &a, "Revised A");
    let review = impact(&conn, &b);
    assert!(review.needs_review);
    assert_eq!(review.causes.len(), 1);
    assert_eq!(
        review.causes[0].consumed_revision_event_id,
        original.revision_event_id
    );
    assert_eq!(
        review.causes[0].current_revision_event_id,
        Some(input(&conn, &a).revision_event_id)
    );
    assert!(!impact(&conn, &c).needs_review);
    assert_eq!(input(&conn, &b), output);
    assert_eq!(input(&conn, &c).text, "Independent C");
    assert_eq!(
        projection(&conn).payload.segments[1].segment.status,
        ScriptSegmentStatus::Current
    );
}

#[test]
fn edit_during_generation_keeps_the_actually_consumed_revision_and_immediately_requires_review() {
    let (mut conn, a, b, _) = fixture();
    let captured = input(&conn, &a);
    let command = generation(
        &b,
        Some(vec![captured.clone()]),
        "Output based on original A",
    );
    edit(&mut conn, &a, "A changed during the model call");
    script_document_command::apply_generated_script_block(&mut conn, &command, 30).unwrap();
    let review = impact(&conn, &b);
    assert!(review.needs_review);
    assert_eq!(
        review.causes[0].consumed_revision_event_id,
        captured.revision_event_id
    );
    assert_eq!(
        review.causes[0].input_excerpt.as_deref(),
        Some("Original A")
    );
    assert_eq!(input(&conn, &b).text, "Output based on original A");
}

#[test]
fn replay_does_not_duplicate_lineage_or_replace_a_later_generation_and_changed_inputs_are_rejected()
{
    let (mut conn, a, b, _) = fixture();
    let first = generation(&b, Some(vec![input(&conn, &a)]), "First output");
    script_document_command::apply_generated_script_block(&mut conn, &first, 15).unwrap();
    edit(&mut conn, &a, "Fresh A");
    let fresh = generation(
        &b,
        Some(vec![input(&conn, &a), input(&conn, &b)]),
        "Fresh output",
    );
    script_document_command::apply_generated_script_block(&mut conn, &fresh, 30).unwrap();
    assert!(
        !impact(&conn, &b).needs_review,
        "own prior draft must not invalidate its intentional replacement"
    );
    let before = counts(&conn);
    let snapshot = projection(&conn);
    assert_eq!(
        script_document_command::apply_generated_script_block(&mut conn, &first, 40)
            .unwrap()
            .0,
        history_store::RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(counts(&conn), before);
    assert_eq!(projection(&conn), snapshot);
    let mut changed_signature = first.clone();
    changed_signature.payload.script_inputs = Some(vec![]);
    assert!(
        script_document_command::apply_generated_script_block(&mut conn, &changed_signature, 50)
            .is_err()
    );
    assert_eq!(counts(&conn), before);
}

#[test]
fn deleted_source_remains_explainable_and_explicit_fresh_generation_replaces_old_bindings() {
    let (mut conn, a, b, _) = fixture();
    let captured = input(&conn, &a);
    let command = generation(&b, Some(vec![captured.clone()]), "Dependent output");
    script_document_command::apply_generated_script_block(&mut conn, &command, 15).unwrap();
    let delete = CommandEnvelope::new(a.block_id.clone());
    let event = ChangeEvent::new(delete.id, ChangeEventKind::UserEdit, "Delete A");
    let revision = ObjectRevision::new(
        ObjectKind::ScriptBlock,
        a.block_id.as_str(),
        event.id,
        RevisionOperation::Delete,
    );
    history_store::record_change_with(
        &mut conn,
        &delete,
        "fixture.script_delete",
        &event,
        &[revision],
        |tx| {
            tx.execute(
                "UPDATE script_blocks SET deleted_event_id = ?1 WHERE id = ?2",
                params![event.id.0.to_string(), a.block_id.as_str()],
            )?;
            Ok(())
        },
    )
    .unwrap();
    let cause = impact(&conn, &b).causes.remove(0);
    assert_eq!(cause.reason, ScriptImpactReason::Deleted);
    assert_eq!(cause.current_revision_event_id, None);
    assert_eq!(cause.input_excerpt.as_deref(), Some("Original A"));
    assert_eq!(cause.consumed_revision_event_id, captured.revision_event_id);
    script_document_command::apply_generated_script_block(
        &mut conn,
        &generation(&b, Some(vec![]), "Explicit regenerated output"),
        30,
    )
    .unwrap();
    assert!(!impact(&conn, &b).needs_review);
    assert_eq!(
        counts(&conn).2,
        2,
        "prior generation lineage remains auditable"
    );
}

#[test]
fn failed_lineage_write_rolls_back_output_history_and_dependencies_together() {
    let (mut conn, a, b, _) = fixture();
    let command = generation(&b, Some(vec![input(&conn, &a)]), "Must not commit");
    let before = projection(&conn);
    let before_counts = counts(&conn);
    conn.execute_batch("CREATE TRIGGER fail_lineage BEFORE INSERT ON semantic_dependency_revisions BEGIN SELECT RAISE(ABORT, 'fixture lineage failure'); END").unwrap();
    assert!(
        script_document_command::apply_generated_script_block(&mut conn, &command, 30)
            .unwrap_err()
            .to_string()
            .contains("fixture lineage failure")
    );
    assert_eq!(counts(&conn), before_counts);
    assert_eq!(projection(&conn), before);
}

#[test]
fn fabricated_revision_or_text_cannot_be_recorded_as_actual_generation_evidence() {
    let (mut conn, a, b, _) = fixture();
    let captured = input(&conn, &a);
    let before = projection(&conn);
    let before_counts = counts(&conn);
    for field in ["text", "start", "end", "source", "revision"] {
        let mut fabricated = captured.clone();
        match field {
            "text" => fabricated.text = "Unobserved text".into(),
            "start" => fabricated.start_ms += 1,
            "end" => fabricated.end_ms += 1,
            "source" => fabricated.source_node_id = Some(NodeId::new().0.to_string()),
            "revision" => fabricated.revision_event_id = ChangeEventId(uuid::Uuid::new_v4()),
            _ => unreachable!(),
        }
        let command = generation(&b, Some(vec![fabricated]), "Must not commit");
        assert!(
            script_document_command::apply_generated_script_block(&mut conn, &command, 30).is_err(),
            "fabricated {field} must be refused"
        );
        assert_eq!(counts(&conn), before_counts);
        assert_eq!(projection(&conn), before);
    }
}

#[test]
fn unavailable_lineage_is_distinct_from_known_empty_inputs() {
    let (mut conn, _, b, _) = fixture();
    script_document_command::apply_generated_script_block(
        &mut conn,
        &generation(&b, None, "Legacy output"),
        15,
    )
    .unwrap();
    assert!(!impact(&conn, &b).lineage_available);
    script_document_command::apply_generated_script_block(
        &mut conn,
        &generation(&b, Some(vec![]), "Known empty input set"),
        20,
    )
    .unwrap();
    assert!(impact(&conn, &b).lineage_available);
    assert!(!impact(&conn, &b).needs_review);
}

#[test]
fn conflicting_revisions_for_the_same_input_are_rejected_without_writes() {
    let (mut conn, a, b, _) = fixture();
    let old = input(&conn, &a);
    edit(&mut conn, &a, "Second observed revision");
    let new = input(&conn, &a);
    let before = projection(&conn);
    let before_counts = counts(&conn);
    let command = generation(&b, Some(vec![old, new]), "Ambiguous evidence");
    assert!(
        script_document_command::apply_generated_script_block(&mut conn, &command, 30)
            .unwrap_err()
            .to_string()
            .contains("conflicting revision bindings")
    );
    assert_eq!(counts(&conn), before_counts);
    assert_eq!(projection(&conn), before);
}

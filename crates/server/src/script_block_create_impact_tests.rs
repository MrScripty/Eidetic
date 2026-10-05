use super::*;
use crate::{ai_script_context, script_impact_projection, script_impact_review};
use eidetic_core::contracts::*;

fn linked_fixture() -> (
    Connection,
    eidetic_core::Project,
    CreateScriptBlockCommand,
    SetScriptBlockCommand,
    SetScriptBlockCommand,
) {
    let (mut conn, project, a) = super::tests::fixture();
    apply_create_script_block(&mut conn, &CommandEnvelope::new(a.clone()), 10).unwrap();
    let input =
        ai_script_context::load_script_context(&conn, a.source_node_id, 1000, 2000).unwrap();
    let station = project
        .timeline
        .nodes
        .iter()
        .find(|node| node.name == "Station")
        .unwrap();
    let b = SetScriptBlockCommand {
        document_id: a.document_id.clone(),
        document_title: project.name.clone(),
        document_sort_order: 0,
        segment_id: ScriptSegmentId::new(format!("script.segment.{}", station.id.0)).unwrap(),
        source_node_id: Some(station.id.0.to_string()),
        segment_start_ms: 4000,
        segment_end_ms: 5000,
        segment_status: ScriptSegmentStatus::Current,
        segment_sort_order: 0,
        block_id: ScriptBlockId::new("block.B.generated").unwrap(),
        block_kind: ScriptBlockKind::Action,
        text: "Original B".into(),
        span_provenance: ScriptSpanProvenance::AiGenerated,
        sort_order: 0,
    };
    script_document_command::apply_generated_script_block(
        &mut conn,
        &CommandEnvelope::new(GenerateScriptBlockCommand {
            block: b.clone(),
            script_inputs: Some(input),
        }),
        20,
    )
    .unwrap();
    let c = SetScriptBlockCommand {
        segment_id: ScriptSegmentId::new("segment.C").unwrap(),
        source_node_id: None,
        segment_start_ms: 9000,
        segment_end_ms: 10000,
        block_id: ScriptBlockId::new("block.C.generated").unwrap(),
        text: "Unrelated C".into(),
        ..b.clone()
    };
    script_document_command::apply_generated_script_block(
        &mut conn,
        &CommandEnvelope::new(GenerateScriptBlockCommand {
            block: c.clone(),
            script_inputs: Some(vec![]),
        }),
        21,
    )
    .unwrap();
    (conn, project, a, b, c)
}

fn append(a: &CreateScriptBlockCommand, text: &str) -> CommandEnvelope<CreateScriptBlockCommand> {
    let mut payload = a.clone();
    payload.text = text.into();
    CommandEnvelope::new(payload)
}

#[test]
fn consumed_source_append_requires_review_and_explicit_update_refreshes_all_members() {
    let (mut conn, _, a, b, c) = linked_fixture();
    assert!(
        !script_impact_projection::load_impact(&conn, &b.segment_id)
            .unwrap()
            .unwrap()
            .needs_review
    );
    let first = ai_script_context::load_script_context(&conn, a.source_node_id, 1000, 2000)
        .unwrap()[0]
        .clone();
    let command = append(&a, "  A2: Mara keeps the blue umbrella — 雨\n\n");
    let (_, document) = apply_create_script_block(&mut conn, &command, 30).unwrap();
    let source = document
        .payload
        .segments
        .iter()
        .find(|segment| segment.segment.id == first.segment_id)
        .unwrap();
    assert_eq!(source.blocks[0].block.text, a.text);
    assert_eq!(
        source.blocks[0].revision_event_id,
        Some(first.revision_event_id)
    );
    assert_eq!(source.blocks[1].block.text, command.payload.text);
    let context =
        ai_script_context::load_script_context(&conn, a.source_node_id, 1000, 2000).unwrap();
    assert!(
        context
            .iter()
            .any(|block| block.text == command.payload.text)
    );
    let impact = script_impact_projection::load_impact(&conn, &b.segment_id)
        .unwrap()
        .unwrap();
    assert!(
        impact.needs_review,
        "B must expose its normal review/update entry point after A gains A2"
    );
    assert_eq!(impact.causes.len(), 1);
    assert_eq!(
        impact.causes[0].input,
        SemanticDependencyEndpoint::ScriptSegment {
            segment_id: first.segment_id.clone()
        }
    );
    assert_eq!(
        impact.causes[0].consumed_revision_event_id,
        first.segment_revision_event_id
    );
    assert_ne!(
        impact.causes[0].current_revision_event_id,
        Some(first.segment_revision_event_id)
    );
    assert!(
        !script_impact_projection::load_impact(&conn, &c.segment_id)
            .unwrap()
            .unwrap()
            .needs_review
    );
    let request = script_impact_review::tests::request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &request.payload).unwrap();
    assert!(
        binding
            .script_inputs
            .iter()
            .any(|input| input.text == a.text)
    );
    assert!(
        binding
            .script_inputs
            .iter()
            .any(|input| input.text == command.payload.text)
    );
    let before = script_store::load_document_projection(&conn, &a.document_id).unwrap();
    script_impact_review::record_proposal(
        &mut conn,
        &request,
        binding,
        "B notices Mara's blue umbrella.\n".into(),
        40,
    )
    .unwrap();
    assert_eq!(
        script_store::load_document_projection(&conn, &a.document_id).unwrap(),
        before,
        "preview cannot change canon"
    );
    assert_eq!(
        script_impact_review::tests::accept(&mut conn, &request).unwrap(),
        RecordChangeOutcome::Recorded
    );
    assert_eq!(block_text(&conn, &b), "B notices Mara's blue umbrella.\n");
    assert_eq!(block_text(&conn, &c), "Unrelated C");
    let after = script_store::load_document_projection(&conn, &a.document_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        after
            .segments
            .iter()
            .find(|segment| segment.segment.id == first.segment_id)
            .unwrap(),
        source
    );
    assert!(
        !script_impact_projection::load_impact(&conn, &b.segment_id)
            .unwrap()
            .unwrap()
            .needs_review
    );
    let edit = CommandEnvelope::new(EditScriptBlockCommand {
        document_id: a.document_id,
        block_id: source.blocks[1].block.id.clone(),
        expected_revision_event_id: source.blocks[1].revision_event_id.unwrap(),
        text: "A2 now carries a red umbrella".into(),
    });
    crate::script_block_edit::apply_edit_script_block(&mut conn, &edit, 50).unwrap();
    let impact = script_impact_projection::load_impact(&conn, &b.segment_id)
        .unwrap()
        .unwrap();
    assert!(
        impact.needs_review,
        "accepted update must bind the newly appended block too"
    );
    assert!(impact.causes.iter().any(|cause| cause.input
        == SemanticDependencyEndpoint::ScriptBlock {
            block_id: edit.payload.block_id.clone()
        }));
}

#[test]
fn later_source_append_refuses_a_pending_update_and_replay_preserves_membership_revision() {
    let (mut conn, _, a, b, _) = linked_fixture();
    let a2 = append(&a, "A2 exact text");
    apply_create_script_block(&mut conn, &a2, 30).unwrap();
    let request = script_impact_review::tests::request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &request.payload).unwrap();
    script_impact_review::record_proposal(
        &mut conn,
        &request,
        binding,
        "Pending update".into(),
        40,
    )
    .unwrap();
    let a3 = append(&a, "A3 intervenes");
    let (_, before) = apply_create_script_block(&mut conn, &a3, 50).unwrap();
    let history = row_counts(&conn);
    assert!(script_impact_review::tests::accept(&mut conn, &request).is_err());
    assert_eq!(row_counts(&conn), history);
    assert_eq!(block_text(&conn, &b), "Original B");
    let (outcome, after) = apply_create_script_block(&mut conn, &a2, 60).unwrap();
    assert_eq!(outcome, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(after, before);
    assert_eq!(row_counts(&conn), history);
    assert!(
        script_impact_projection::load_impact(&conn, &b.segment_id)
            .unwrap()
            .unwrap()
            .needs_review
    );
}

fn row_counts(conn: &Connection) -> Vec<u64> {
    [
        "commands",
        "change_events",
        "object_revisions",
        "object_revision_fields",
    ]
    .into_iter()
    .map(|table| {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
    })
    .collect()
}

#[test]
fn failed_block_insert_rolls_back_segment_membership_revision_and_history() {
    let (mut conn, _, a, b, _) = linked_fixture();
    let before = script_store::load_document_projection_envelope(&conn, &a.document_id).unwrap();
    let context =
        ai_script_context::load_script_context(&conn, a.source_node_id, 1000, 2000).unwrap();
    let history = row_counts(&conn);
    conn.execute_batch("CREATE TRIGGER refuse_append BEFORE INSERT ON script_blocks WHEN NEW.text = 'Refused A2' BEGIN SELECT RAISE(ABORT, 'injected append failure'); END;").unwrap();
    assert!(apply_create_script_block(&mut conn, &append(&a, "Refused A2"), 30).is_err());
    assert_eq!(
        script_store::load_document_projection_envelope(&conn, &a.document_id).unwrap(),
        before
    );
    assert_eq!(
        ai_script_context::load_script_context(&conn, a.source_node_id, 1000, 2000).unwrap(),
        context
    );
    assert_eq!(row_counts(&conn), history);
    assert!(
        !script_impact_projection::load_impact(&conn, &b.segment_id)
            .unwrap()
            .unwrap()
            .needs_review
    );
}

fn block_text(conn: &Connection, block: &SetScriptBlockCommand) -> String {
    conn.query_row(
        "SELECT text FROM script_blocks WHERE id = ?1",
        [block.block_id.as_str()],
        |row| row.get(0),
    )
    .unwrap()
}

#[test]
fn captured_append_membership_remains_valid_after_source_retimes_during_generation() {
    let (mut conn, project, a, b, _) = linked_fixture();
    let (_, before) =
        apply_create_script_block(&mut conn, &append(&a, "A2 captured before move"), 30).unwrap();
    let segment_id = before
        .payload
        .segments
        .iter()
        .find(|segment| {
            segment.segment.source_node_id.as_deref()
                == Some(a.source_node_id.0.to_string().as_str())
        })
        .unwrap()
        .segment
        .id
        .clone();
    let captured: Vec<_> =
        ai_script_context::load_script_context(&conn, a.source_node_id, 1000, 2000)
            .unwrap()
            .into_iter()
            .filter(|input| input.segment_id == segment_id)
            .collect();
    assert_eq!(captured.len(), 2);
    let movement = CommandEnvelope::new(SetTimelineNodeRangeCommand {
        node_id: a.source_node_id,
        start_ms: 6000,
        end_ms: 7000,
    });
    crate::timeline_command_history::record_set_timeline_node_range_history(
        &mut conn, &project, &movement, 40,
    )
    .unwrap();
    // Completion must retain valid consumed historical membership/placement,
    // then expose its later range change instead of rebinding to current input.
    script_document_command::apply_generated_script_block(
        &mut conn,
        &CommandEnvelope::new(GenerateScriptBlockCommand {
            block: b.clone(),
            script_inputs: Some(captured.clone()),
        }),
        50,
    )
    .unwrap();
    let impact = script_impact_projection::load_impact(&conn, &b.segment_id)
        .unwrap()
        .unwrap();
    assert!(impact.needs_review);
    assert_eq!(impact.causes.len(), 1);
    assert_eq!(
        impact.causes[0].consumed_revision_event_id,
        captured[0].segment_revision_event_id
    );
    let after =
        ai_script_context::load_script_context(&conn, a.source_node_id, 6000, 7000).unwrap();
    for original in captured {
        let current = after
            .iter()
            .find(|input| input.block_id == original.block_id)
            .unwrap();
        assert_eq!(current.text, original.text);
        assert_eq!(current.revision_event_id, original.revision_event_id);
        assert_eq!((current.start_ms, current.end_ms), (6000, 7000));
    }
}

#[test]
fn reconciled_manual_edit_preserves_later_exact_text_and_consumed_source_review() {
    let (mut conn, _, a, b, c) = linked_fixture();
    let first = ai_script_context::load_script_context(&conn, a.source_node_id, 1000, 2000)
        .unwrap()[0]
        .clone();
    let original = CommandEnvelope::new(EditScriptBlockCommand {
        document_id: a.document_id.clone(),
        block_id: first.block_id.clone(),
        expected_revision_event_id: first.revision_event_id,
        text: "  First saved edit — 雨\n\n".into(),
    });
    let (_, saved) =
        crate::script_block_edit::apply_edit_script_block(&mut conn, &original, 30).unwrap();
    let after_first = row_counts(&conn);
    // An acknowledgement may be lost. The canonical edit and its real impact
    // already exist; no client receipt is needed to create that review cause.
    assert!(
        script_impact_projection::load_impact(&conn, &b.segment_id)
            .unwrap()
            .unwrap()
            .needs_review
    );
    assert!(
        !script_impact_projection::load_impact(&conn, &c.segment_id)
            .unwrap()
            .unwrap()
            .needs_review
    );
    let current = saved
        .payload
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|block| block.block.id == first.block_id)
        .unwrap();
    let newer = CommandEnvelope::new(EditScriptBlockCommand {
        expected_revision_event_id: current.revision_event_id.unwrap(),
        text: "  Later canonical author text — 雪\n\n  ".into(),
        ..original.payload.clone()
    });
    crate::script_block_edit::apply_edit_script_block(&mut conn, &newer, 40).unwrap();
    let before_retry = row_counts(&conn);
    assert_ne!(before_retry, after_first);
    let before = script_store::load_document_projection_envelope(&conn, &a.document_id)
        .unwrap()
        .unwrap();
    let (outcome, replayed) =
        crate::script_block_edit::apply_edit_script_block(&mut conn, &original, 50).unwrap();
    assert_eq!(outcome, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(row_counts(&conn), before_retry);
    assert_eq!(replayed, before);
    let memory =
        ai_script_context::load_script_context(&conn, a.source_node_id, 1000, 2000).unwrap();
    let latest = memory
        .iter()
        .find(|input| input.block_id == first.block_id)
        .unwrap();
    assert_eq!(latest.text, newer.payload.text);
    assert_ne!(latest.revision_event_id, first.revision_event_id);
    assert_eq!(
        latest.segment_revision_event_id,
        first.segment_revision_event_id
    );
    let impact = script_impact_projection::load_impact(&conn, &b.segment_id)
        .unwrap()
        .unwrap();
    assert!(impact.needs_review);
    assert!(impact.causes.iter().any(|cause| cause.input
        == SemanticDependencyEndpoint::ScriptBlock {
            block_id: first.block_id.clone()
        }));
    assert_eq!(block_text(&conn, &b), "Original B");
    assert_eq!(block_text(&conn, &c), "Unrelated C");
    let request = script_impact_review::tests::request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &request.payload).unwrap();
    assert!(
        binding
            .script_inputs
            .iter()
            .any(|input| input.block_id == first.block_id && input.text == newer.payload.text)
    );
    assert!(
        !binding
            .script_inputs
            .iter()
            .any(|input| input.text == original.payload.text)
    );
}

#[test]
fn retained_draft_from_an_explicit_read_still_refuses_later_edits_then_updates_canonical_memory() {
    let (mut conn, _, a, b, c) = linked_fixture();
    let input = ai_script_context::load_script_context(&conn, a.source_node_id, 1000, 2000)
        .unwrap()[0]
        .clone();
    let draft_text = "  Exact retained draft — 雨\n\n  ";
    let stale = CommandEnvelope::new(EditScriptBlockCommand {
        document_id: a.document_id.clone(),
        block_id: input.block_id.clone(),
        expected_revision_event_id: input.revision_event_id,
        text: draft_text.into(),
    });
    let writer = CommandEnvelope::new(EditScriptBlockCommand {
        text: "Another author's current text".into(),
        ..stale.payload.clone()
    });
    crate::script_block_edit::apply_edit_script_block(&mut conn, &writer, 30).unwrap();
    let before = row_counts(&conn);
    assert!(crate::script_block_edit::apply_edit_script_block(&mut conn, &stale, 40).is_err());
    assert_eq!(row_counts(&conn), before);
    let read = script_store::load_document_projection(&conn, &a.document_id)
        .unwrap()
        .unwrap();
    let current = read
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|block| block.block.id == input.block_id)
        .unwrap();
    assert_eq!(current.block.text, writer.payload.text);
    let continued = CommandEnvelope::new(EditScriptBlockCommand {
        expected_revision_event_id: current.revision_event_id.unwrap(),
        ..stale.payload.clone()
    });
    let later = CommandEnvelope::new(EditScriptBlockCommand {
        text: "Same text, later revision".into(),
        ..continued.payload.clone()
    });
    crate::script_block_edit::apply_edit_script_block(&mut conn, &later, 50).unwrap();
    let before = script_store::load_document_projection_envelope(&conn, &a.document_id)
        .unwrap()
        .unwrap();
    let rows = row_counts(&conn);
    assert!(crate::script_block_edit::apply_edit_script_block(&mut conn, &continued, 60).is_err());
    assert_eq!(row_counts(&conn), rows);
    assert_eq!(
        script_store::load_document_projection_envelope(&conn, &a.document_id)
            .unwrap()
            .unwrap(),
        before
    );
    let reread = before
        .payload
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|block| block.block.id == input.block_id)
        .unwrap();
    let confirmed = CommandEnvelope::new(EditScriptBlockCommand {
        expected_revision_event_id: reread.revision_event_id.unwrap(),
        ..stale.payload.clone()
    });
    let (_, saved) =
        crate::script_block_edit::apply_edit_script_block(&mut conn, &confirmed, 70).unwrap();
    let context =
        ai_script_context::load_script_context(&conn, a.source_node_id, 1000, 2000).unwrap();
    let authored = context
        .iter()
        .find(|item| item.block_id == input.block_id)
        .unwrap();
    assert_eq!(authored.text, draft_text);
    assert_eq!(
        authored.segment_revision_event_id,
        input.segment_revision_event_id
    );
    assert_ne!(authored.revision_event_id, input.revision_event_id);
    assert!(
        script_impact_projection::load_impact(&conn, &b.segment_id)
            .unwrap()
            .unwrap()
            .needs_review
    );
    assert!(
        !script_impact_projection::load_impact(&conn, &c.segment_id)
            .unwrap()
            .unwrap()
            .needs_review
    );
    let request = script_impact_review::tests::request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &request.payload).unwrap();
    assert!(
        binding
            .script_inputs
            .iter()
            .any(|item| item.block_id == input.block_id && item.text == draft_text)
    );
    assert_eq!(block_text(&conn, &b), "Original B");
    assert_eq!(block_text(&conn, &c), "Unrelated C");
    let rows = row_counts(&conn);
    assert_eq!(
        crate::script_block_edit::apply_edit_script_block(&mut conn, &confirmed, 80).unwrap(),
        (RecordChangeOutcome::AlreadyRecorded, saved)
    );
    assert_eq!(row_counts(&conn), rows);
}

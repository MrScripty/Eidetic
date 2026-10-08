use eidetic_core::timeline::{
    node::{NodeId, StoryLevel, StoryNode},
    timing::TimeRange,
};
use eidetic_core::{Project, contracts::*};
use rusqlite::Connection;

use crate::history_store::RecordChangeOutcome;
use crate::{
    ai_script_context, history_store, propagation_proposal_accept, propagation_proposal_review,
    script_document_command, script_impact_review, script_store, timeline_command_history,
    timeline_node_store, timeline_relationship_store,
};

pub(crate) fn fixture() -> (
    Connection,
    Project,
    SetScriptBlockCommand,
    SetScriptBlockCommand,
) {
    let mut project = eidetic_core::Template::MultiCam.build_project("Two scenes");
    project.timeline.total_duration_ms = 10_000;
    project.timeline.nodes.clear();
    project.timeline.node_arcs.clear();
    project.timeline.relationships.clear();
    let mut parents = Vec::new();
    for (index, level) in [StoryLevel::Premise, StoryLevel::Act, StoryLevel::Sequence]
        .into_iter()
        .enumerate()
    {
        let mut node = StoryNode::new(level.label(), level, TimeRange::new(0, 10_000).unwrap());
        node.id = NodeId(uuid::Uuid::from_u128(index as u128 + 1));
        node.parent_id = parents.last().copied();
        parents.push(node.id);
        project.timeline.add_node(node).unwrap();
    }
    let mut blocks = Vec::new();
    for (index, name, start) in [(4, "A", 1000), (5, "B", 4000)] {
        let mut node = StoryNode::new_child(
            name,
            StoryLevel::Scene,
            TimeRange::new(start, start + 1000).unwrap(),
            parents[2],
        );
        node.id = NodeId(uuid::Uuid::from_u128(index));
        project.timeline.add_node(node.clone()).unwrap();
        blocks.push(SetScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            document_title: "Two scenes".into(),
            document_sort_order: 0,
            segment_id: ScriptSegmentId::new(format!("segment.{name}")).unwrap(),
            source_node_id: Some(node.id.0.to_string()),
            segment_start_ms: start,
            segment_end_ms: start + 1000,
            segment_status: ScriptSegmentStatus::Current,
            segment_sort_order: (index - 4) as u32,
            block_id: ScriptBlockId::new(format!("block.{name}")).unwrap(),
            block_kind: ScriptBlockKind::Action,
            text: format!("  Authored {name} — 雨\n\n"),
            span_provenance: ScriptSpanProvenance::UserEdited,
            sort_order: 0,
        });
    }
    let [a, b]: [SetScriptBlockCommand; 2] = blocks.try_into().unwrap();
    let mut conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON; CREATE TABLE project (id INTEGER PRIMARY KEY, name TEXT NOT NULL, premise TEXT NOT NULL DEFAULT '', total_duration_ms INTEGER); INSERT INTO project VALUES (1, 'Two scenes', '', 10000);").unwrap();
    let tx = conn.transaction().unwrap();
    timeline_node_store::upsert_nodes_in_transaction(&tx, &project.timeline.nodes).unwrap();
    timeline_node_store::replace_node_arcs_in_transaction(&tx, &project.timeline.node_arcs)
        .unwrap();
    timeline_relationship_store::upsert_relationships_in_transaction(&tx, &[]).unwrap();
    tx.commit().unwrap();
    for block in [&a, &b] {
        script_document_command::apply_set_script_block(
            &mut conn,
            &CommandEnvelope::new(block.clone()),
            10,
        )
        .unwrap();
    }
    let captured = input(&conn, &a);
    script_document_command::apply_generated_script_block(
        &mut conn,
        &CommandEnvelope::new(GenerateScriptBlockCommand {
            arc_description_applicability: None,
            timeline_title_inputs: None,
            ancestor_notes_inputs: None,
            arc_inputs: None,
            target_binding: None,
            script_context_scope: None,
            bible_node_name_inputs: None,
            bible_relationship_inputs: None,
            bible_inputs: None,
            bible_context_scope: None,
            block: b.clone(),
            script_inputs: Some(vec![captured]),
        }),
        20,
    )
    .unwrap();
    (conn, project, a, b)
}

pub(crate) fn input(conn: &Connection, block: &SetScriptBlockCommand) -> ScriptContextBlock {
    ai_script_context::load_script_context(
        conn,
        node_id(block),
        block.segment_start_ms,
        block.segment_end_ms,
    )
    .unwrap()
    .into_iter()
    .find(|input| input.block_id == block.block_id)
    .unwrap()
}

fn node_id(block: &SetScriptBlockCommand) -> NodeId {
    NodeId(uuid::Uuid::parse_str(block.source_node_id.as_ref().unwrap()).unwrap())
}

pub(crate) fn command(
    a: &SetScriptBlockCommand,
    start: u64,
    end: u64,
) -> CommandEnvelope<SetTimelineNodeRangeCommand> {
    CommandEnvelope::new(SetTimelineNodeRangeCommand {
        node_id: node_id(a),
        start_ms: start,
        end_ms: end,
        expected: None,
    })
}

fn move_a(
    conn: &mut Connection,
    project: &mut Project,
    a: &SetScriptBlockCommand,
    start: u64,
    end: u64,
) {
    timeline_command_history::record_set_timeline_node_range_history(
        conn,
        project,
        &command(a, start, end),
        5,
    )
    .unwrap();
    project.timeline.nodes = timeline_node_store::load_nodes(conn).unwrap();
}

fn document(conn: &Connection, b: &SetScriptBlockCommand) -> ScriptDocumentProjection {
    script_store::load_document_projection(conn, &b.document_id)
        .unwrap()
        .unwrap()
}

fn proposal(
    conn: &mut Connection,
    b: &SetScriptBlockCommand,
) -> CommandEnvelope<RequestScriptImpactProposalCommand> {
    let mut request = script_impact_review::tests::request(conn, b);
    request.payload.story_time_ms = Some(42);
    let binding = script_impact_review::capture(conn, &request.payload).unwrap();
    assert_eq!(binding.bible_context.payload.story_time_ms, Some(42));
    let a = binding
        .script_inputs
        .iter()
        .find(|input| input.segment_id.as_str() == "segment.A")
        .unwrap();
    assert_eq!(a.text, "  Authored A — 雨\n\n");
    script_impact_review::record_proposal(
        conn,
        &request,
        binding,
        "  Proposed B — 雨\n\n".into(),
        30,
    )
    .unwrap();
    request
}

#[test]
fn ordinary_move_reorders_canonical_context_and_marks_only_consumed_placement() {
    let (mut conn, mut project, a, b) = fixture();
    let before_a = input(&conn, &a);
    let before_b = document(&conn, &b)
        .segments
        .into_iter()
        .find(|s| s.segment.id == b.segment_id)
        .unwrap();
    // Regeneration locks do not forbid structural placement edits.
    project.timeline.node_mut(node_id(&a)).unwrap().locked = true;
    let tx = conn.transaction().unwrap();
    timeline_node_store::upsert_nodes_in_transaction(&tx, &project.timeline.nodes).unwrap();
    tx.commit().unwrap();
    move_a(&mut conn, &mut project, &a, 6000, 7000);
    let after_a = input(&conn, &a);
    assert_eq!((after_a.start_ms, after_a.end_ms), (6000, 7000));
    assert_eq!(after_a.text, before_a.text);
    assert_eq!(after_a.revision_event_id, before_a.revision_event_id);
    assert_ne!(
        after_a.segment_revision_event_id,
        before_a.segment_revision_event_id
    );
    let context = ai_script_context::load_script_context(&conn, node_id(&b), 4000, 5000).unwrap();
    assert_eq!(
        context
            .iter()
            .map(|input| input.block_id.as_str())
            .collect::<Vec<_>>(),
        ["block.B", "block.A"]
    );
    let after_b = document(&conn, &b)
        .segments
        .into_iter()
        .find(|s| s.segment.id == b.segment_id)
        .unwrap();
    assert_eq!(after_b.segment, before_b.segment);
    assert_eq!(after_b.blocks, before_b.blocks);
    let impact = after_b.impact.unwrap();
    assert_eq!(impact.causes.len(), 1);
    assert_eq!(
        impact.causes[0].input,
        SemanticDependencyEndpoint::ScriptSegment {
            segment_id: a.segment_id.clone()
        }
    );
    assert_eq!(
        impact.causes[0].consumed_revision_event_id,
        before_a.segment_revision_event_id
    );
    assert_eq!(
        impact.causes[0].current_revision_event_id,
        Some(after_a.segment_revision_event_id)
    );
    let revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::ScriptSegment,
        a.segment_id.as_str(),
    )
    .unwrap();
    let revision = revisions.last().unwrap();
    assert_eq!(revision.change_event_id, after_a.segment_revision_event_id);
    assert_eq!(revision.fields.len(), 2);
    assert_eq!(
        revision.fields[0],
        FieldDelta::new(
            "start_ms",
            Some(FieldValue::Integer(1000)),
            Some(FieldValue::Integer(6000))
        )
    );
    assert_eq!(
        revision.fields[1],
        FieldDelta::new(
            "end_ms",
            Some(FieldValue::Integer(2000)),
            Some(FieldValue::Integer(7000))
        )
    );
    let timeline = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        a.source_node_id.as_ref().unwrap(),
    )
    .unwrap();
    assert_eq!(
        timeline.last().unwrap().change_event_id,
        revision.change_event_id
    );
}

#[test]
fn moved_scene_preview_rejects_or_explicitly_updates_only_b_and_refreshes_lineage() {
    let (mut conn, mut project, a, b) = fixture();
    move_a(&mut conn, &mut project, &a, 6000, 7000);
    let before = document(&conn, &b);
    let rejected = proposal(&mut conn, &b);
    propagation_proposal_review::record_reject_propagation_proposal(
        &mut conn,
        &CommandEnvelope::new(RejectPropagationProposalCommand {
            proposal_id: rejected.payload.proposal_id,
            reason: Some("Keep authored draft".into()),
        }),
        40,
    )
    .unwrap();
    assert_eq!(before, document(&conn, &b));
    let accepted = proposal(&mut conn, &b);
    propagation_proposal_accept::record_accept_propagation_proposal(
        &mut conn,
        &CommandEnvelope::new(AcceptPropagationProposalCommand {
            proposal_id: accepted.payload.proposal_id,
        }),
        50,
    )
    .unwrap();
    let after = document(&conn, &b);
    assert_eq!(after.document, before.document);
    let before_a = before
        .segments
        .iter()
        .find(|s| s.segment.id == a.segment_id)
        .unwrap();
    let after_a = after
        .segments
        .iter()
        .find(|s| s.segment.id == a.segment_id)
        .unwrap();
    assert_eq!(before_a, after_a);
    let before_b = before
        .segments
        .iter()
        .find(|s| s.segment.id == b.segment_id)
        .unwrap();
    let after_b = after
        .segments
        .iter()
        .find(|s| s.segment.id == b.segment_id)
        .unwrap();
    assert_eq!(before_b.segment, after_b.segment);
    assert_eq!(after_b.blocks[0].block.text, "  Proposed B — 雨\n\n");
    assert!(after_b.impact.as_ref().unwrap().causes.is_empty());
}

#[test]
fn retime_and_placement_aba_refuse_old_preview_and_replay_preserves_later_authored_edit() {
    let (mut conn, mut project, a, b) = fixture();
    move_a(&mut conn, &mut project, &a, 2000, 3000);
    let pending = proposal(&mut conn, &b);
    move_a(&mut conn, &mut project, &a, 6000, 7500);
    move_a(&mut conn, &mut project, &a, 2000, 3000);
    let history: i64 = conn
        .query_row("SELECT COUNT(*) FROM change_events", [], |r| r.get(0))
        .unwrap();
    assert!(script_impact_review::tests::accept(&mut conn, &pending).is_err());
    assert_eq!(
        history,
        conn.query_row("SELECT COUNT(*) FROM change_events", [], |r| r
            .get::<_, i64>(0))
            .unwrap()
    );
    let replay = command(&a, 2500, 3500);
    timeline_command_history::record_set_timeline_node_range_history(
        &mut conn, &project, &replay, 5,
    )
    .unwrap();
    crate::script_impact_review::tests::edit(&mut conn, &a, "  Later authored A\n\n");
    let before = document(&conn, &b);
    assert_eq!(
        timeline_command_history::record_set_timeline_node_range_history(
            &mut conn, &project, &replay, 10
        )
        .unwrap(),
        RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(before, document(&conn, &b));
    let mut conflicting = replay;
    conflicting.payload.end_ms += 1;
    assert!(
        timeline_command_history::record_set_timeline_node_range_history(
            &mut conn,
            &project,
            &conflicting,
            10
        )
        .is_err()
    );
}

#[test]
fn parent_resize_updates_both_bound_scenes_and_rollback_preserves_all_state() {
    let (mut conn, project, a, b) = fixture();
    let before = document(&conn, &b);
    let nodes = timeline_node_store::load_nodes(&conn).unwrap();
    let history: i64 = conn
        .query_row("SELECT COUNT(*) FROM change_events", [], |r| r.get(0))
        .unwrap();
    let resize = CommandEnvelope::new(SetTimelineNodeRangeCommand {
        node_id: NodeId(uuid::Uuid::from_u128(3)),
        start_ms: 0,
        end_ms: 8000,
        expected: None,
    });
    conn.execute_batch("CREATE TRIGGER reject_segment BEFORE UPDATE ON script_segments WHEN OLD.id = 'segment.B' BEGIN SELECT RAISE(ABORT, 'injected placement failure'); END;").unwrap();
    assert!(
        timeline_command_history::record_set_timeline_node_range_history(
            &mut conn, &project, &resize, 1
        )
        .is_err()
    );
    assert_eq!(before, document(&conn, &b));
    assert_eq!(
        serde_json::to_string(&nodes).unwrap(),
        serde_json::to_string(&timeline_node_store::load_nodes(&conn).unwrap()).unwrap()
    );
    assert_eq!(
        history,
        conn.query_row("SELECT COUNT(*) FROM change_events", [], |r| r
            .get::<_, i64>(0))
            .unwrap()
    );
    assert!(
        history_store::load_command::<SetTimelineNodeRangeCommand>(&conn, resize.id)
            .unwrap()
            .is_none()
    );
    conn.execute_batch("DROP TRIGGER reject_segment").unwrap();
    timeline_command_history::record_set_timeline_node_range_history(
        &mut conn, &project, &resize, 1,
    )
    .unwrap();
    for block in [&a, &b] {
        let segment = input(&conn, block);
        let node = timeline_node_store::load_nodes(&conn)
            .unwrap()
            .into_iter()
            .find(|node| node.id == node_id(block))
            .unwrap();
        assert_eq!(
            (segment.start_ms, segment.end_ms),
            (node.time_range.start_ms, node.time_range.end_ms)
        );
        assert_eq!(segment.text, block.text);
    }
}

#[test]
fn historical_sparse_placement_validates_captured_input_after_later_move_and_rejects_forgery() {
    let (mut conn, mut project, a, b) = fixture();
    move_a(&mut conn, &mut project, &a, 1000, 3000);
    let revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::ScriptSegment,
        a.segment_id.as_str(),
    )
    .unwrap();
    assert_eq!(revisions.last().unwrap().fields.len(), 1);
    assert_eq!(revisions.last().unwrap().fields[0].field_key, "end_ms");
    let captured = input(&conn, &a);
    move_a(&mut conn, &mut project, &a, 6500, 7500);
    for forged in [true, false] {
        let mut evidence = captured.clone();
        if forged {
            evidence.start_ms += 1;
        }
        let generation = CommandEnvelope::new(GenerateScriptBlockCommand {
            arc_description_applicability: None,
            timeline_title_inputs: None,
            ancestor_notes_inputs: None,
            arc_inputs: None,
            target_binding: None,
            script_context_scope: None,
            bible_node_name_inputs: None,
            bible_relationship_inputs: None,
            bible_inputs: None,
            bible_context_scope: None,
            block: b.clone(),
            script_inputs: Some(vec![evidence]),
        });
        let result =
            script_document_command::apply_generated_script_block(&mut conn, &generation, 30);
        assert_eq!(result.is_err(), forged);
    }
    let after = document(&conn, &b);
    let impact = after
        .segments
        .iter()
        .find(|s| s.segment.id == b.segment_id)
        .unwrap()
        .impact
        .as_ref()
        .unwrap();
    assert_eq!(impact.causes.len(), 1);
    assert_eq!(
        impact.causes[0].consumed_revision_event_id,
        captured.segment_revision_event_id
    );
    assert_eq!(
        impact.causes[0].current_revision_event_id,
        Some(input(&conn, &a).segment_revision_event_id)
    );
}

#[test]
fn unchanged_range_has_no_segment_revision_and_deleted_or_unbound_segments_stay_untouched() {
    let (mut conn, mut project, a, b) = fixture();
    let initial = input(&conn, &a);
    move_a(&mut conn, &mut project, &a, 1000, 2000);
    assert_eq!(initial, input(&conn, &a));
    conn.execute(
        "UPDATE script_segments SET source_node_id = NULL WHERE id = ?1",
        [b.segment_id.as_str()],
    )
    .unwrap();
    conn.execute(
        "UPDATE script_segments SET deleted_event_id = updated_event_id WHERE id = ?1",
        [a.segment_id.as_str()],
    )
    .unwrap();
    let saved: (u64, u64, String) = conn
        .query_row(
            "SELECT start_ms, end_ms, updated_event_id FROM script_segments WHERE id = ?1",
            [a.segment_id.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    move_a(&mut conn, &mut project, &a, 6000, 7000);
    let current: (u64, u64, String) = conn
        .query_row(
            "SELECT start_ms, end_ms, updated_event_id FROM script_segments WHERE id = ?1",
            [a.segment_id.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(saved, current);
    assert_eq!(
        (input(&conn, &b).start_ms, input(&conn, &b).end_ms),
        (4000, 5000)
    );
}

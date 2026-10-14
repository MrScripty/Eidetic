use super::*;
use crate::{
    history_store::RecordChangeOutcome, timeline_node_store,
    timeline_sibling_reorder_history::record_reorder_timeline_sibling_history as save,
};
use eidetic_core::Project;
fn refresh(conn: &Connection, p: &mut Project) {
    p.timeline.nodes = timeline_node_store::load_nodes(conn).unwrap();
}
fn command(
    conn: &Connection,
    a: NodeId,
    b: NodeId,
) -> CommandEnvelope<ReorderTimelineSiblingCommand> {
    CommandEnvelope::new(ReorderTimelineSiblingCommand {
        node_id: a,
        neighbor_id: b,
        expected: capture(conn, a).unwrap(),
    })
}
fn snapshot(conn: &Connection) -> (String, Vec<i64>) {
    let materials =
        serde_json::to_string(&crate::timeline_notes_lineage::tests::logical_rows(conn)).unwrap();
    let counts = [
        "commands",
        "change_events",
        "object_revisions",
        "object_revision_fields",
    ]
    .map(|table| {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    });
    (materials, counts.to_vec())
}
#[test]
fn both_placements_preserve_authored_material_and_original_receipt_after_reorder_aba() {
    let (mut conn, mut p, a, b) = crate::timeline_script_placement::tests::fixture();
    let a = NodeId(uuid::Uuid::parse_str(a.source_node_id.as_ref().unwrap()).unwrap());
    let b = NodeId(uuid::Uuid::parse_str(b.source_node_id.as_ref().unwrap()).unwrap());
    refresh(&conn, &mut p);
    let before = authored(&conn);
    let cmd = command(&conn, a, b);
    assert_eq!(
        save(&mut conn, &p, &cmd, 50).unwrap(),
        RecordChangeOutcome::Recorded
    );
    let receipt = command_receipt(&conn, &cmd).unwrap();
    assert_eq!(authored(&conn), before);
    assert_eq!(receipt.siblings[0].node_id, b);
    assert_eq!(receipt.siblings[0].start_ms, 1000);
    assert_eq!(receipt.siblings[1].start_ms, 4000);
    refresh(&conn, &mut p);
    let reverse = command(&conn, a, b);
    save(&mut conn, &p, &reverse, 60).unwrap();
    refresh(&conn, &mut p);
    let snap = snapshot(&conn);
    assert_eq!(
        save(&mut conn, &p, &cmd, 70).unwrap(),
        RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(command_receipt(&conn, &cmd).unwrap(), receipt);
    assert_eq!(snapshot(&conn), snap);
    let mut forged = cmd.clone();
    forged.payload.neighbor_id = a;
    assert!(save(&mut conn, &p, &forged, 80).is_err());
    assert_eq!(snapshot(&conn), snap);
    let stale = CommandEnvelope::new(cmd.payload.clone());
    assert!(save(&mut conn, &p, &stale, 90).is_err());
    assert_eq!(snapshot(&conn), snap);
}
#[test]
fn structural_moves_allow_content_locks_and_stale_neighbor_notes_refuse() {
    let (mut conn, mut p, a, b) = crate::timeline_script_placement::tests::fixture();
    let a = NodeId(uuid::Uuid::parse_str(a.source_node_id.as_ref().unwrap()).unwrap());
    let b = NodeId(uuid::Uuid::parse_str(b.source_node_id.as_ref().unwrap()).unwrap());
    refresh(&conn, &mut p);
    let stale = command(&conn, a, b);
    crate::timeline_command_history::record_set_timeline_node_notes_history(
        &mut conn,
        &p,
        &CommandEnvelope::new(SetTimelineNodeNotesCommand {
            node_id: b,
            notes: "new notes 雨".into(),
            expected: None,
        }),
        51,
    )
    .unwrap();
    refresh(&conn, &mut p);
    let before = snapshot(&conn);
    assert!(save(&mut conn, &p, &stale, 52).is_err());
    assert_eq!(snapshot(&conn), before);
    crate::timeline_command_history::record_set_timeline_node_lock_history(
        &mut conn,
        &p,
        &CommandEnvelope::new(SetTimelineNodeLockCommand {
            node_id: a,
            locked: true,
        }),
        53,
    )
    .unwrap();
    refresh(&conn, &mut p);
    let fresh = command(&conn, a, b);
    save(&mut conn, &p, &fresh, 54).unwrap();
    refresh(&conn, &mut p);
    assert!(p.timeline.node(a).unwrap().locked);
}
#[test]
fn reordered_scene_context_requires_targeted_review_and_explicit_acceptance() {
    let (mut conn, mut p, blocks) = crate::script_context_scope::tests::fixture();
    refresh(&conn, &mut p);
    let b = crate::script_context_scope::tests::node(&blocks[1]);
    let f = crate::script_context_scope::tests::node(&blocks[2]);
    let before = authored(&conn);
    let cmd = command(&conn, b, f);
    save(&mut conn, &p, &cmd, 51).unwrap();
    refresh(&conn, &mut p);
    assert_eq!(authored(&conn), before);
    let review = crate::script_context_scope::tests::target(&conn, &blocks[1])
        .impact
        .unwrap();
    assert!(review.needs_review);
    assert!(
        review
            .causes
            .iter()
            .any(|c| c.reason == ScriptImpactReason::ContextChanged)
    );
    let req = crate::script_context_scope::tests::request(&conn, &blocks[1]);
    let binding = crate::script_impact_review::capture(&conn, &req.payload).unwrap();
    crate::script_impact_review::record_proposal(
        &mut conn,
        &req,
        binding,
        "Labelled synthetic reordered-scene preview".into(),
        52,
    )
    .unwrap();
    assert_eq!(authored(&conn), before);
    crate::script_impact_review::tests::accept(&mut conn, &req).unwrap();
    assert!(
        !crate::script_context_scope::tests::target(&conn, &blocks[1])
            .impact
            .unwrap()
            .needs_review
    );
}

fn authored(conn: &Connection) -> String {
    let doc = crate::script_store::load_document_projection(
        conn,
        &ScriptDocumentId::new("script.document.main").unwrap(),
    )
    .unwrap()
    .unwrap();
    let mut blocks = doc
        .segments
        .into_iter()
        .flat_map(|s| s.blocks)
        .collect::<Vec<_>>();
    blocks.sort_by_key(|b| b.block.id.clone());
    serde_json::to_string(&blocks).unwrap()
}
#[test]
fn inserted_then_deleted_sibling_invalidates_original_read_without_writing() {
    let (mut conn, mut p, a, b) = crate::timeline_script_placement::tests::fixture();
    refresh(&conn, &mut p);
    let a = NodeId(uuid::Uuid::parse_str(a.source_node_id.as_ref().unwrap()).unwrap());
    let b = NodeId(uuid::Uuid::parse_str(b.source_node_id.as_ref().unwrap()).unwrap());
    let stale = command(&conn, a, b);
    let created = NodeId::new();
    crate::timeline_command_history::record_create_timeline_node_history(
        &mut conn,
        &p,
        &CommandEnvelope::new(CreateTimelineNodeCommand {
            node_id: created,
            parent_id: p.timeline.node(a).unwrap().parent_id,
            level: eidetic_core::timeline::node::StoryLevel::Scene,
            name: "Temporary neighbor".into(),
            start_ms: 2500,
            end_ms: 3000,
            beat_type: None,
        }),
        55,
    )
    .unwrap();
    refresh(&conn, &mut p);
    crate::timeline_node_delete_history::record_delete_timeline_node_history(
        &mut conn,
        &p,
        &CommandEnvelope::new(DeleteTimelineNodeCommand { node_id: created }),
        56,
    )
    .unwrap();
    refresh(&conn, &mut p);
    let now = capture(&conn, a).unwrap();
    assert_eq!(now.siblings, stale.payload.expected.siblings);
    assert_ne!(
        now.membership_revision_event_id,
        stale.payload.expected.membership_revision_event_id
    );
    let before = snapshot(&conn);
    assert!(save(&mut conn, &p, &stale, 57).is_err());
    assert_eq!(snapshot(&conn), before);
}
#[test]
fn nested_source_bound_segments_translate_and_pending_acceptance_refuses_reorder_aba() {
    let (mut conn, mut p, blocks) = crate::script_context_scope::tests::fixture();
    refresh(&conn, &mut p);
    let b = crate::script_context_scope::tests::node(&blocks[1]);
    let f = crate::script_context_scope::tests::node(&blocks[2]);
    let child = NodeId::new();
    crate::timeline_command_history::record_create_timeline_node_history(
        &mut conn,
        &p,
        &CommandEnvelope::new(CreateTimelineNodeCommand {
            node_id: child,
            parent_id: Some(b),
            level: eidetic_core::timeline::node::StoryLevel::Beat,
            name: "Nested authored beat".into(),
            start_ms: 4100,
            end_ms: 4500,
            beat_type: None,
        }),
        50,
    )
    .unwrap();
    refresh(&conn, &mut p);
    let mut block = blocks[1].clone();
    block.source_node_id = Some(child.0.to_string());
    block.segment_id = ScriptSegmentId::new("segment.child").unwrap();
    block.block_id = ScriptBlockId::new("block.child").unwrap();
    block.segment_start_ms = 4100;
    block.segment_end_ms = 4500;
    block.text = "  Nested author text 雨\n\n".into();
    crate::script_document_command::apply_set_script_block(
        &mut conn,
        &CommandEnvelope::new(block),
        51,
    )
    .unwrap();
    let before = authored(&conn);
    let cmd = command(&conn, b, f);
    save(&mut conn, &p, &cmd, 52).unwrap();
    refresh(&conn, &mut p);
    assert_eq!(authored(&conn), before);
    assert_eq!(p.timeline.node(child).unwrap().time_range.start_ms, 2100);
    let (start, end): (i64, i64) = conn
        .query_row(
            "SELECT start_ms,end_ms FROM script_segments WHERE id='segment.child'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((start, end), (2100, 2500));
    let req = crate::script_context_scope::tests::request(&conn, &blocks[1]);
    let binding = crate::script_impact_review::capture(&conn, &req.payload).unwrap();
    crate::script_impact_review::record_proposal(
        &mut conn,
        &req,
        binding,
        "Labelled synthetic pending reorder preview".into(),
        53,
    )
    .unwrap();
    for time in [54, 55] {
        let reverse = command(&conn, b, f);
        save(&mut conn, &p, &reverse, time).unwrap();
        refresh(&conn, &mut p);
    }
    let before = snapshot(&conn);
    assert!(crate::script_impact_review::tests::accept(&mut conn, &req).is_err());
    assert_eq!(snapshot(&conn), before);
}
#[test]
fn pending_generation_refuses_reordered_then_restored_source_without_writes() {
    let (mut conn, mut p, generation) = crate::script_generation_target::tests::fixture();
    refresh(&conn, &mut p);
    let b = generation.payload.target_binding.as_ref().unwrap().node_id;
    let f = NodeId(uuid::Uuid::from_u128(6));
    for time in [65, 66] {
        let cmd = command(&conn, b, f);
        save(&mut conn, &p, &cmd, time).unwrap();
        refresh(&conn, &mut p);
    }
    let before = snapshot(&conn);
    assert!(
        crate::script_document_command::apply_generated_script_block(&mut conn, &generation, 67)
            .is_err()
    );
    assert_eq!(snapshot(&conn), before);
}
#[test]
fn original_receipt_replays_after_selected_clip_deletion() {
    let (mut conn, mut p, a, b) = crate::timeline_script_placement::tests::fixture();
    refresh(&conn, &mut p);
    let a = NodeId(uuid::Uuid::parse_str(a.source_node_id.as_ref().unwrap()).unwrap());
    let b = NodeId(uuid::Uuid::parse_str(b.source_node_id.as_ref().unwrap()).unwrap());
    let cmd = command(&conn, a, b);
    save(&mut conn, &p, &cmd, 65).unwrap();
    let receipt = command_receipt(&conn, &cmd).unwrap();
    refresh(&conn, &mut p);
    crate::timeline_node_delete_history::record_delete_timeline_node_history(
        &mut conn,
        &p,
        &CommandEnvelope::new(DeleteTimelineNodeCommand { node_id: a }),
        66,
    )
    .unwrap();
    refresh(&conn, &mut p);
    let before = snapshot(&conn);
    assert_eq!(
        save(&mut conn, &p, &cmd, 67).unwrap(),
        RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(command_receipt(&conn, &cmd).unwrap(), receipt);
    assert_eq!(snapshot(&conn), before);
}

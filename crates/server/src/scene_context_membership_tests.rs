use super::*;
use crate::{
    ai_script_context, history_store::RecordChangeOutcome, script_document_command,
    script_impact_review, script_store, timeline_command_history, timeline_node_store,
};
use eidetic_core::{
    Project,
    timeline::{
        node::{StoryLevel, StoryNode},
        timing::TimeRange,
    },
};

const MANUAL: &str = "  Retained human B — 雨\n\n";
const PROPOSED: &str = "Synthetic fixture: B reflects its changed context.\n\n";

pub(crate) fn fixture() -> (Connection, Project, Vec<SetScriptBlockCommand>) {
    let (mut conn, mut project, a, b) = crate::timeline_script_placement::tests::fixture();
    let mut blocks = vec![a.clone(), b];
    for (id, name, start) in [
        (6, "F", 2000),
        (7, "C", 6000),
        (8, "D", 8000),
        (9, "E", 9000),
    ] {
        let mut node = StoryNode::new_child(
            name,
            StoryLevel::Scene,
            TimeRange::new(start, start + 500).unwrap(),
            NodeId(uuid::Uuid::from_u128(3)),
        );
        node.id = NodeId(uuid::Uuid::from_u128(id));
        project.timeline.add_node(node.clone()).unwrap();
        let mut block = a.clone();
        block.segment_id = ScriptSegmentId::new(format!("segment.{name}")).unwrap();
        block.block_id = ScriptBlockId::new(format!("block.{name}")).unwrap();
        block.source_node_id = Some(node.id.0.to_string());
        block.segment_start_ms = start;
        block.segment_end_ms = start + 500;
        block.segment_sort_order = id as u32;
        block.text = format!("  Exact manual {name} — 雨\n\n");
        blocks.push(block);
    }
    let tx = conn.transaction().unwrap();
    timeline_node_store::upsert_nodes_in_transaction(&tx, &project.timeline.nodes).unwrap();
    tx.commit().unwrap();
    for block in &blocks[2..] {
        script_document_command::apply_set_script_block(
            &mut conn,
            &CommandEnvelope::new(block.clone()),
            25,
        )
        .unwrap();
    }
    let generated = generation(&conn, &blocks[1]);
    script_document_command::apply_generated_script_block(&mut conn, &generated, 30).unwrap();
    crate::script_impact_review::tests::edit(&mut conn, &blocks[1], MANUAL);
    assert!(!target(&conn, &blocks[1]).impact.unwrap().needs_review);
    (conn, project, blocks)
}

pub(crate) fn node(block: &SetScriptBlockCommand) -> NodeId {
    NodeId(uuid::Uuid::parse_str(block.source_node_id.as_ref().unwrap()).unwrap())
}

fn generation(
    conn: &Connection,
    block: &SetScriptBlockCommand,
) -> CommandEnvelope<GenerateScriptBlockCommand> {
    let inputs = ai_script_context::load_script_context(
        conn,
        node(block),
        block.segment_start_ms,
        block.segment_end_ms,
    )
    .unwrap();
    let scope = capture(
        conn,
        node(block),
        block.segment_start_ms,
        block.segment_end_ms,
        &inputs,
    )
    .unwrap();
    CommandEnvelope::new(GenerateScriptBlockCommand {
        target_binding: None,
        block: block.clone(),
        script_inputs: Some(inputs),
        bible_node_name_inputs: None,
        bible_relationship_inputs: None,
        bible_inputs: None,
        bible_context_scope: None,
        script_context_scope: Some(scope),
    })
}

pub(crate) fn move_scene(
    conn: &mut Connection,
    project: &mut Project,
    block: &SetScriptBlockCommand,
    start: u64,
    end: u64,
) {
    timeline_command_history::record_set_timeline_node_range_history(
        conn,
        project,
        &crate::timeline_script_placement::tests::command(block, start, end),
        50,
    )
    .unwrap();
    project.timeline.nodes = timeline_node_store::load_nodes(conn).unwrap();
}

pub(crate) fn target(conn: &Connection, block: &SetScriptBlockCommand) -> ScriptSegmentProjection {
    script_store::load_document_projection(conn, &block.document_id)
        .unwrap()
        .unwrap()
        .segments
        .into_iter()
        .find(|s| s.segment.id == block.segment_id)
        .unwrap()
}

fn ids(inputs: &[ScriptContextBlock]) -> Vec<&str> {
    inputs.iter().map(|i| i.segment_id.as_str()).collect()
}

pub(crate) fn request(
    conn: &Connection,
    b: &SetScriptBlockCommand,
) -> CommandEnvelope<RequestScriptImpactProposalCommand> {
    let mut request = crate::script_impact_review::tests::request(conn, b);
    let impact = target(conn, b).impact.unwrap();
    if let Some(cause) = impact
        .causes
        .iter()
        .find(|c| c.reason == ScriptImpactReason::ContextChanged)
    {
        request.payload.dependency_id = cause.dependency_id.clone();
    }
    request
}

fn preview(
    conn: &mut Connection,
    b: &SetScriptBlockCommand,
) -> CommandEnvelope<RequestScriptImpactProposalCommand> {
    let request = request(conn, b);
    let binding = script_impact_review::capture(conn, &request.payload).unwrap();
    script_impact_review::record_proposal(conn, &request, binding, PROPOSED.into(), 60).unwrap();
    request
}

#[test]
fn entering_unconsumed_scene_displaces_neighbor_and_requires_explicit_target_acceptance() {
    let (mut conn, mut project, blocks) = fixture();
    let b = &blocks[1];
    let before = target(&conn, b);
    let a_before = crate::timeline_script_placement::tests::input(&conn, &blocks[0]);
    let e_before = crate::timeline_script_placement::tests::input(&conn, &blocks[5]);
    move_scene(&mut conn, &mut project, &blocks[5], 3000, 3500);
    let after = target(&conn, b);
    assert_eq!(before.blocks, after.blocks);
    assert_eq!(before.segment, after.segment);
    let impact = after.impact.unwrap();
    assert_eq!(impact.causes.len(), 1);
    assert_eq!(impact.causes[0].reason, ScriptImpactReason::ContextChanged);
    assert_eq!(
        impact.causes[0].input_excerpt.as_deref(),
        Some("Entered: E. Left: A.")
    );
    assert_eq!(
        a_before,
        crate::timeline_script_placement::tests::input(&conn, &blocks[0])
    );
    assert_eq!(
        e_before.text,
        crate::timeline_script_placement::tests::input(&conn, &blocks[5]).text
    );
    let request = request(&conn, b);
    let binding = script_impact_review::capture(&conn, &request.payload).unwrap();
    assert_eq!(
        ids(&binding.script_inputs),
        [
            "segment.F",
            "segment.E",
            "segment.B",
            "segment.C",
            "segment.D"
        ]
    );
    assert_eq!(binding.script_inputs[2].text, MANUAL);
    script_impact_review::record_proposal(&mut conn, &request, binding, PROPOSED.into(), 60)
        .unwrap();
    assert_eq!(before.blocks, target(&conn, b).blocks);
    crate::script_impact_review::tests::accept(&mut conn, &request).unwrap();
    let accepted = target(&conn, b);
    assert_eq!(accepted.blocks[0].block.text, PROPOSED);
    assert!(!accepted.impact.unwrap().needs_review);
    assert_eq!(
        e_before.text,
        crate::timeline_script_placement::tests::input(&conn, &blocks[5]).text
    );
    crate::script_impact_review::tests::edit(&mut conn, &blocks[5], "Later human E\n\n");
    assert!(
        target(&conn, b).impact.unwrap().needs_review,
        "acceptance must bind newly consumed E rather than old A"
    );
}

#[test]
fn relocating_target_reports_entering_and_leaving_material_without_replacing_manual_text() {
    let (mut conn, mut project, blocks) = fixture();
    let b = &blocks[1];
    let before = target(&conn, b);
    move_scene(&mut conn, &mut project, b, 8500, 9500);
    let after = target(&conn, b);
    assert_eq!(before.blocks, after.blocks);
    let impact = after.impact.unwrap();
    assert_eq!(impact.causes.len(), 1);
    assert_eq!(
        impact.causes[0].input_excerpt.as_deref(),
        Some("Entered: E. Left: A, F.")
    );
    let request = request(&conn, b);
    let binding = script_impact_review::capture(&conn, &request.payload).unwrap();
    assert_eq!(
        ids(&binding.script_inputs),
        ["segment.C", "segment.D", "segment.B", "segment.E"]
    );
    assert_eq!(binding.script_inputs[2].text, MANUAL);
}

#[test]
fn same_external_window_and_ordinary_target_shift_do_not_invent_review_causes() {
    let (mut conn, mut project, blocks) = fixture();
    let b = &blocks[1];
    let before = target(&conn, b).blocks;
    move_scene(&mut conn, &mut project, &blocks[5], 9200, 9700);
    assert!(!target(&conn, b).impact.unwrap().needs_review);
    move_scene(&mut conn, &mut project, b, 4100, 5100);
    assert!(!target(&conn, b).impact.unwrap().needs_review);
    assert_eq!(before, target(&conn, b).blocks);
}

#[test]
fn unconsumed_move_into_and_back_out_of_pending_window_refuses_old_preview_atomically() {
    let (mut conn, mut project, blocks) = fixture();
    let b = &blocks[1];
    crate::script_impact_review::tests::edit(&mut conn, &blocks[0], "New human A\n\n");
    let request = preview(&mut conn, b);
    let old = script_impact_review::capture(&conn, &request.payload).unwrap();
    move_scene(&mut conn, &mut project, &blocks[5], 3000, 3500);
    move_scene(&mut conn, &mut project, &blocks[5], 9000, 9500);
    let mut current = script_impact_review::capture(&conn, &request.payload).unwrap();
    assert_ne!(old.script_context_scope, current.script_context_scope);
    current
        .script_context_scope
        .as_mut()
        .unwrap()
        .revision_event_id = old.script_context_scope.as_ref().unwrap().revision_event_id;
    assert_eq!(
        current, old,
        "unconsumed placement epoch is the only changed guard"
    );
    let before = target(&conn, b);
    let history: i64 = conn
        .query_row("SELECT COUNT(*) FROM change_events", [], |r| r.get(0))
        .unwrap();
    assert!(crate::script_impact_review::tests::accept(&mut conn, &request).is_err());
    assert_eq!(before, target(&conn, b));
    assert_eq!(
        history,
        conn.query_row("SELECT COUNT(*) FROM change_events", [], |r| r
            .get::<_, i64>(0))
            .unwrap()
    );
    let proposal = crate::propagation_proposal_store::load_propagation_proposal(
        &conn,
        &request.payload.proposal_id,
    )
    .unwrap()
    .unwrap();
    assert_eq!(proposal.status, SemanticProposalStatus::Pending);
    crate::propagation_proposal_review::record_reject_propagation_proposal(
        &mut conn,
        &CommandEnvelope::new(RejectPropagationProposalCommand {
            proposal_id: request.payload.proposal_id,
            reason: None,
        }),
        70,
    )
    .unwrap();
    assert_eq!(before, target(&conn, b));
}

#[test]
fn late_generation_retains_captured_window_and_replay_preserves_later_human_edit() {
    let (mut conn, mut project, blocks) = fixture();
    let b = &blocks[1];
    let generated = generation(&conn, b);
    assert_eq!(
        ids(generated.payload.script_inputs.as_ref().unwrap()),
        [
            "segment.A",
            "segment.F",
            "segment.B",
            "segment.C",
            "segment.D"
        ]
    );
    move_scene(&mut conn, &mut project, &blocks[5], 3000, 3500);
    script_document_command::apply_generated_script_block(&mut conn, &generated, 60).unwrap();
    let impact = target(&conn, b).impact.unwrap();
    assert!(
        impact
            .causes
            .iter()
            .any(|c| c.reason == ScriptImpactReason::ContextChanged)
    );
    crate::script_impact_review::tests::edit(&mut conn, b, MANUAL);
    let before = target(&conn, b);
    assert_eq!(
        script_document_command::apply_generated_script_block(&mut conn, &generated, 99)
            .unwrap()
            .0,
        RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(before, target(&conn, b));
}

#[test]
fn scope_identity_and_historical_epoch_forgery_roll_back_canonical_history() {
    let (mut conn, _, blocks) = fixture();
    let before = target(&conn, &blocks[1]);
    for mutation in 0..4 {
        let mut command = generation(&conn, &blocks[1]);
        let scope = command.payload.script_context_scope.as_mut().unwrap();
        match mutation {
            0 => scope.node_id = node(&blocks[0]),
            1 => scope.segment_ids.push(scope.segment_ids[0].clone()),
            2 => scope
                .segment_ids
                .push(ScriptSegmentId::new("missing").unwrap()),
            _ => scope.revision_event_id = Some(ChangeEventId(uuid::Uuid::new_v4())),
        }
        let history: i64 = conn
            .query_row("SELECT COUNT(*) FROM change_events", [], |r| r.get(0))
            .unwrap();
        assert!(
            script_document_command::apply_generated_script_block(&mut conn, &command, 90).is_err()
        );
        assert_eq!(before, target(&conn, &blocks[1]));
        assert_eq!(
            history,
            conn.query_row("SELECT COUNT(*) FROM change_events", [], |r| r
                .get::<_, i64>(0))
                .unwrap()
        );
    }
}

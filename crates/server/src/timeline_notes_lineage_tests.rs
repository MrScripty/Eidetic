use super::*;
use crate::script_impact_review::tests::{accept, request};
use crate::{
    script_document_command, script_impact_review, timeline_command_history, timeline_node_store,
};
use eidetic_core::Project;

const OLD: &str = "  Mara conceals the witness — 雨.\n\n  ";
const NEW: &str = "  Mara reveals the witness — 雨.\n\n  ";
const PREVIEW: &str = "  Synthetic preview: Mara identifies the witness — 雨.\n\n  ";

pub(crate) fn notes(conn: &mut Connection, project: &mut Project, node: NodeId, text: &str) {
    project.timeline.nodes = timeline_node_store::load_nodes(conn).unwrap();
    timeline_command_history::record_set_timeline_node_notes_history(
        conn,
        project,
        &CommandEnvelope::new(SetTimelineNodeNotesCommand {
            expected: None,
            node_id: node,
            notes: text.into(),
        }),
        200,
    )
    .unwrap();
    project.timeline.nodes = timeline_node_store::load_nodes(conn).unwrap();
}

fn setup() -> (
    Connection,
    Project,
    CommandEnvelope<GenerateScriptBlockCommand>,
) {
    let (mut conn, mut project, mut generation) = crate::script_generation_target::tests::fixture();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    notes(&mut conn, &mut project, node, OLD);
    generation.payload.target_binding = Some(
        crate::script_generation_target::capture(&conn, project.timeline.node(node).unwrap())
            .unwrap(),
    );
    script_document_command::apply_generated_script_block(&mut conn, &generation, 210).unwrap();
    (conn, project, generation)
}

fn impact(
    conn: &Connection,
    generation: &CommandEnvelope<GenerateScriptBlockCommand>,
) -> ScriptImpactProjection {
    crate::script_impact_projection::load_impact(conn, &generation.payload.block.segment_id)
        .unwrap()
        .unwrap()
}

pub(crate) fn materials(conn: &Connection) -> String {
    let doc = crate::script_store::load_document_projection(
        conn,
        &ScriptDocumentId::new("script.document.main").unwrap(),
    )
    .unwrap()
    .unwrap();
    // Derived review can change; all canonical text/spans/locks/placement must stay exact.
    serde_json::to_string(
        &doc.segments
            .into_iter()
            .map(|segment| (segment.segment, segment.blocks))
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

pub(crate) fn logical_rows(conn: &Connection) -> Vec<(String, Vec<String>)> {
    let mut tables = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").unwrap();
    let names = tables
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    names
        .into_iter()
        .map(|table| {
            let mut stmt = conn
                .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
                .unwrap();
            let count = stmt.column_count();
            let rows = stmt
                .query_map([], |row| {
                    Ok((0..count)
                        .map(|i| format!("{:?}", row.get_ref(i).unwrap()))
                        .collect::<Vec<_>>()
                        .join("|"))
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            (table, rows)
        })
        .collect()
}

fn preview(
    conn: &mut Connection,
    generation: &CommandEnvelope<GenerateScriptBlockCommand>,
) -> CommandEnvelope<RequestScriptImpactProposalCommand> {
    let command = request(conn, &generation.payload.block);
    let binding = script_impact_review::capture(conn, &command.payload).unwrap();
    script_impact_review::record_proposal(conn, &command, binding, PREVIEW.into(), 220).unwrap();
    command
}

#[test]
fn exact_notes_edit_review_preview_and_acceptance_preserve_unrelated_authored_material() {
    let (mut conn, mut project, generation) = setup();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    assert!(!impact(&conn, &generation).needs_review);
    let before = materials(&conn);
    notes(&mut conn, &mut project, node, NEW);
    assert_eq!(materials(&conn), before);
    let review = impact(&conn, &generation);
    assert_eq!(review.causes.len(), 1);
    assert_eq!(review.causes[0].input_excerpt.as_deref(), Some(OLD));
    assert!(
        review.causes[0]
            .dependency_id
            .as_str()
            .ends_with(".timeline_notes")
    );
    let command = request(&conn, &generation.payload.block);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    assert_eq!(binding.timeline_notes_previous.as_ref().unwrap().notes, OLD);
    assert_eq!(binding.timeline_notes_current.as_ref().unwrap().notes, NEW);
    script_impact_review::record_proposal(&mut conn, &command, binding, PREVIEW.into(), 220)
        .unwrap();
    assert_eq!(materials(&conn), before, "preview is not acceptance");
    let before_doc =
        crate::script_store::load_document_projection(&conn, &generation.payload.block.document_id)
            .unwrap()
            .unwrap();
    accept(&mut conn, &command).unwrap();
    let after_doc =
        crate::script_store::load_document_projection(&conn, &generation.payload.block.document_id)
            .unwrap()
            .unwrap();
    for segment in &before_doc.segments {
        let after = after_doc
            .segments
            .iter()
            .find(|item| item.segment.id == segment.segment.id)
            .unwrap();
        assert_eq!(after.segment, segment.segment);
        for block in &segment.blocks {
            let after = after
                .blocks
                .iter()
                .find(|item| item.block.id == block.block.id)
                .unwrap();
            if block.block.id == generation.payload.block.block_id {
                assert_eq!(after.block.text, PREVIEW);
            } else {
                assert_eq!(after, block);
            }
        }
    }
    assert!(!impact(&conn, &generation).needs_review);
    notes(
        &mut conn,
        &mut project,
        node,
        "Next exact authored decision.",
    );
    assert!(impact(&conn, &generation).needs_review);
}

#[test]
fn unrelated_notes_and_non_notes_node_events_do_not_create_review() {
    let (mut conn, mut project, generation) = setup();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    let other = project
        .timeline
        .nodes
        .iter()
        .find(|item| item.id != node)
        .unwrap()
        .id;
    notes(&mut conn, &mut project, other, NEW);
    for locked in [true, false] {
        timeline_command_history::record_set_timeline_node_lock_history(
            &mut conn,
            &project,
            &CommandEnvelope::new(SetTimelineNodeLockCommand {
                node_id: node,
                locked,
            }),
            230,
        )
        .unwrap();
        project.timeline.nodes = timeline_node_store::load_nodes(&conn).unwrap();
    }
    assert!(!impact(&conn, &generation).needs_review);
    assert_eq!(
        capture(&conn, node).unwrap().revision_event_id,
        recorded(&conn, impact(&conn, &generation).generation_event_id)
            .unwrap()
            .unwrap()
            .revision_event_id
    );
}

#[test]
fn notes_aba_keeps_old_consumption_and_refuses_pending_storage_and_acceptance_atomically() {
    let (mut conn, mut project, generation) = setup();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    notes(&mut conn, &mut project, node, NEW);
    let command = preview(&mut conn, &generation);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    for text in ["Intervening decision", NEW] {
        notes(&mut conn, &mut project, node, text);
    }
    assert_eq!(
        recorded(&conn, impact(&conn, &generation).generation_event_id)
            .unwrap()
            .unwrap()
            .notes,
        OLD
    );
    let before = logical_rows(&conn);
    let mut late = command.clone();
    late.id = CommandId(uuid::Uuid::new_v4());
    late.payload.proposal_id = PropagationProposalId::new("late.notes.preview").unwrap();
    let mut delayed = binding;
    delayed.request = late.payload.clone();
    assert!(
        script_impact_review::record_proposal(&mut conn, &late, delayed, PREVIEW.into(), 240)
            .is_err()
    );
    let accept_command = CommandEnvelope::new(AcceptPropagationProposalCommand {
        proposal_id: command.payload.proposal_id.clone(),
    });
    assert!(
        crate::propagation_proposal_accept::record_accept_propagation_proposal(
            &mut conn,
            &accept_command,
            240
        )
        .is_err()
    );
    assert_eq!(logical_rows(&conn), before);
    assert!(impact(&conn, &generation).needs_review);
    let fresh = preview(&mut conn, &generation);
    accept(&mut conn, &fresh).unwrap();
    assert!(!impact(&conn, &generation).needs_review);
}

#[test]
fn clearing_notes_is_exact_reviewable_input_not_missing_consumption() {
    let (mut conn, mut project, generation) = setup();
    notes(
        &mut conn,
        &mut project,
        generation.payload.target_binding.as_ref().unwrap().node_id,
        "",
    );
    let command = request(&conn, &generation.payload.block);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    assert_eq!(binding.timeline_notes_current.as_ref().unwrap().notes, "");
    assert!(
        binding
            .timeline_notes_current
            .as_ref()
            .unwrap()
            .revision_event_id
            .is_some()
    );
    assert!(impact(&conn, &generation).needs_review);
}

#[test]
fn prior_target_receipt_qualifies_notes_without_backfilling_missing_graph_or_legacy_receipts() {
    let (mut conn, mut project, generation) = setup();
    conn.execute(
        "DELETE FROM semantic_dependency_revisions WHERE dependency_id LIKE '%.timeline_notes'",
        [],
    )
    .unwrap();
    conn.execute(
        "DELETE FROM semantic_dependencies WHERE id LIKE '%.timeline_notes'",
        [],
    )
    .unwrap(); // Explicit older dependency-shape fixture.
    notes(
        &mut conn,
        &mut project,
        generation.payload.target_binding.as_ref().unwrap().node_id,
        NEW,
    );
    assert!(impact(&conn, &generation).needs_review);
    let mut legacy = generation.clone();
    legacy.id = CommandId(uuid::Uuid::new_v4());
    legacy.payload.target_binding = None;
    script_document_command::apply_generated_script_block(&mut conn, &legacy, 250).unwrap();
    assert!(
        recorded(&conn, impact(&conn, &legacy).generation_event_id)
            .unwrap()
            .is_none()
    );
    assert!(!impact(&conn, &legacy).needs_review);
}

#[tokio::test]
async fn provider_preview_receives_exact_current_notes_and_failed_stream_cannot_become_saved_material()
 {
    use eidetic_core::ai::backend::GenerateStream;
    use futures::stream;
    let (mut conn, mut project, generation) = setup();
    notes(
        &mut conn,
        &mut project,
        generation.payload.target_binding.as_ref().unwrap().node_id,
        NEW,
    );
    let binding =
        script_impact_review::capture(&conn, &request(&conn, &generation.payload.block).payload)
            .unwrap();
    let before = materials(&conn);
    let result =
        crate::script_impact_prompt::preview_with_provider(&binding, |prompt| async move {
            assert!(
                prompt
                    .user
                    .contains(&format!("CURRENT SELECTED CLIP NOTES:\n{NEW}"))
            );
            let stream: GenerateStream = Box::pin(stream::iter(vec![
                Ok("Unfinished synthetic prefix".into()),
                Err(eidetic_core::Error::AiBackend("Synthetic failure".into())),
            ]));
            Ok(stream)
        })
        .await;
    assert!(result.is_err());
    assert_eq!(materials(&conn), before);
}

#[test]
fn reject_and_exact_acceptance_replay_never_overwrite_later_authored_work() {
    let (mut conn, mut project, generation) = setup();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    notes(&mut conn, &mut project, node, NEW);
    let rejected = preview(&mut conn, &generation);
    let before = materials(&conn);
    crate::propagation_proposal_review::record_reject_propagation_proposal(
        &mut conn,
        &CommandEnvelope::new(RejectPropagationProposalCommand {
            proposal_id: rejected.payload.proposal_id,
            reason: Some("Keep authored screenplay".into()),
        }),
        230,
    )
    .unwrap();
    assert_eq!(materials(&conn), before);
    assert!(impact(&conn, &generation).needs_review);
    let fresh = preview(&mut conn, &generation);
    let acceptance = CommandEnvelope::new(AcceptPropagationProposalCommand {
        proposal_id: fresh.payload.proposal_id,
    });
    crate::propagation_proposal_accept::record_accept_propagation_proposal(
        &mut conn,
        &acceptance,
        240,
    )
    .unwrap();
    crate::script_impact_review::tests::edit(
        &mut conn,
        &generation.payload.block,
        "  Exact later manual screenplay — 雨.\n\n  ",
    );
    notes(&mut conn, &mut project, node, "Next decision");
    let before = logical_rows(&conn);
    assert_eq!(
        crate::propagation_proposal_accept::record_accept_propagation_proposal(
            &mut conn,
            &acceptance,
            250
        )
        .unwrap(),
        crate::history_store::RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(logical_rows(&conn), before);
    assert!(impact(&conn, &generation).needs_review);
}

#[test]
fn notes_preview_refuses_later_target_edit_and_both_locks_without_partial_writes() {
    for guard in ["manual", "node lock", "span lock"] {
        let (mut conn, mut project, generation) = setup();
        let node = generation.payload.target_binding.as_ref().unwrap().node_id;
        notes(&mut conn, &mut project, node, NEW);
        let pending = preview(&mut conn, &generation);
        match guard {
            "manual" => crate::script_impact_review::tests::edit(
                &mut conn,
                &generation.payload.block,
                "Manual target must survive",
            ),
            "node lock" => {
                timeline_command_history::record_set_timeline_node_lock_history(
                    &mut conn,
                    &project,
                    &CommandEnvelope::new(SetTimelineNodeLockCommand {
                        node_id: node,
                        locked: true,
                    }),
                    240,
                )
                .unwrap();
            }
            _ => {
                let doc = crate::script_store::load_document_projection(
                    &conn,
                    &generation.payload.block.document_id,
                )
                .unwrap()
                .unwrap();
                let block = doc
                    .segments
                    .iter()
                    .flat_map(|segment| &segment.blocks)
                    .find(|block| block.block.id == generation.payload.block.block_id)
                    .unwrap();
                script_document_command::apply_set_script_lock(
                    &mut conn,
                    &CommandEnvelope::new(SetScriptLockCommand {
                        lock_id: ScriptLockId::new("notes.target.lock").unwrap(),
                        span_id: block.spans[0].id.clone(),
                        reason: "Keep exact target".into(),
                    }),
                    240,
                )
                .unwrap();
            }
        }
        let before = logical_rows(&conn);
        assert!(accept(&mut conn, &pending).is_err(), "{guard}");
        assert_eq!(logical_rows(&conn), before, "{guard}");
    }
}

#[test]
fn historical_target_receipt_rejects_forged_notes_and_other_node_clock() {
    let (mut conn, mut project, generation) = setup();
    let mut target = generation.payload.target_binding.unwrap();
    let original = from_target(&conn, &target).unwrap();
    notes(&mut conn, &mut project, target.node_id, NEW);
    assert_eq!(from_target(&conn, &target).unwrap(), original);
    target.notes = NEW.into();
    assert!(from_target(&conn, &target).is_err());
    target.notes = OLD.into();
    target.node_id = project
        .timeline
        .nodes
        .iter()
        .find(|node| node.id != target.node_id)
        .unwrap()
        .id;
    assert!(from_target(&conn, &target).is_err());
}

#[test]
fn explicit_guarded_notes_save_updates_existing_impact_without_replacing_any_saved_screenplay() {
    let (mut conn, mut project, generation) = setup();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    let material = materials(&conn);
    project.timeline.nodes = timeline_node_store::load_nodes(&conn).unwrap();
    let expected = capture(&conn, node).unwrap();
    let command = CommandEnvelope::new(SetTimelineNodeNotesCommand {
        node_id: node,
        notes: NEW.into(),
        expected: Some(expected),
    });
    timeline_command_history::record_set_timeline_node_notes_history(
        &mut conn, &project, &command, 300,
    )
    .unwrap();
    project.timeline.nodes = timeline_node_store::load_nodes(&conn).unwrap();
    assert_eq!(materials(&conn), material);
    let affected = impact(&conn, &generation);
    assert!(!affected.causes.is_empty());
    assert!(
        affected
            .causes
            .iter()
            .any(|cause| cause.input == SemanticDependencyEndpoint::TimelineNode { node_id: node })
    );
    assert_eq!(
        command_receipt(&conn, &command).unwrap(),
        capture(&conn, node).unwrap()
    );
}

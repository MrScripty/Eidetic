use super::*;
use crate::script_impact_review::tests::{accept, edit, request};
use crate::timeline_notes_lineage::tests::{logical_rows, materials, notes};
use crate::{script_document_command, script_impact_review};
use eidetic_core::Project;

const OLD: &str = "  Mara conceals the witness — 雨.\n\n  ";
const NEW: &str = "  Mara reveals the witness — 雨.\n\n  ";
const HUMAN: &str = "  Exact authored scene: Eli keeps his whistle — 雨.\n\n  ";
const PREVIEW: &str = "  Synthetic reviewed scene: Mara reveals the witness — 雨.\n\n  ";

#[test]
fn equal_notes_on_distinct_ancestors_keep_distinct_owned_dependency_identities() {
    let (mut conn, mut project, mut generation, ancestor) = setup(false);
    let outer = project.timeline.node(ancestor).unwrap().parent_id.unwrap();
    notes(&mut conn, &mut project, outer, OLD);
    let target = generation.payload.target_binding.as_ref().unwrap().node_id;
    let expected = project
        .timeline
        .ancestors_of(target)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    generation.payload.ancestor_notes_inputs =
        Some(capture_generation(&conn, target, &expected).unwrap());
    script_document_command::apply_generated_script_block(&mut conn, &generation, 210).unwrap();
    let event = impact(&conn, &generation).generation_event_id;
    assert_ne!(dependency_id(event, ancestor), dependency_id(event, outer));
    for node in [ancestor, outer] {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM semantic_dependencies WHERE id=?1",
                [dependency_id(event, node).as_str()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
    notes(&mut conn, &mut project, outer, NEW);
    let changed = impact(&conn, &generation);
    assert_eq!(changed.causes.len(), 1);
    assert_eq!(
        changed.causes[0].input,
        SemanticDependencyEndpoint::TimelineNode { node_id: outer }
    );
    assert_eq!(changed.causes[0].dependency_id, dependency_id(event, outer));
}

#[test]
fn capture_refuses_a_prompt_chain_changed_since_the_initial_project_read() {
    let (mut conn, mut project, generation, ancestor) = setup(false);
    let target = generation.payload.target_binding.as_ref().unwrap().node_id;
    let expected = project
        .timeline
        .ancestors_of(target)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    notes(&mut conn, &mut project, ancestor, NEW);
    assert!(capture_generation(&conn, target, &expected).is_err());
}

fn setup(
    persist: bool,
) -> (
    Connection,
    Project,
    CommandEnvelope<GenerateScriptBlockCommand>,
    NodeId,
) {
    let (mut conn, mut project, mut generation) = crate::script_generation_target::tests::fixture();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    let ancestor = project.timeline.ancestors_of(node)[0].id;
    notes(&mut conn, &mut project, ancestor, OLD);
    let chain = project
        .timeline
        .ancestors_of(node)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    generation.payload.ancestor_notes_inputs =
        Some(capture_generation(&conn, node, &chain).unwrap());
    if persist {
        script_document_command::apply_generated_script_block(&mut conn, &generation, 210).unwrap();
    }
    (conn, project, generation, ancestor)
}

fn impact(
    conn: &Connection,
    generation: &CommandEnvelope<GenerateScriptBlockCommand>,
) -> ScriptImpactProjection {
    crate::script_impact_projection::load_impact(conn, &generation.payload.block.segment_id)
        .unwrap()
        .unwrap()
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
fn exact_ancestor_identity_review_and_explicit_acceptance_preserve_authored_material() {
    let (mut conn, mut project, generation, ancestor) = setup(true);
    edit(&mut conn, &generation.payload.block, HUMAN);
    let original = recorded(&conn, impact(&conn, &generation).generation_event_id)
        .unwrap()
        .unwrap();
    assert_eq!(original.len(), 1);
    assert_eq!(original[0].node_id, ancestor);
    assert_eq!(original[0].notes, OLD);
    let before = materials(&conn);
    notes(&mut conn, &mut project, ancestor, NEW);
    let changed = impact(&conn, &generation);
    assert_eq!(changed.causes.len(), 1);
    assert_eq!(
        changed.causes[0].dependency_id,
        dependency_id(changed.generation_event_id, ancestor)
    );
    assert_eq!(changed.causes[0].input_excerpt.as_deref(), Some(OLD));
    assert_eq!(materials(&conn), before);
    let command = preview(&mut conn, &generation);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    assert_eq!(
        binding.ancestor_notes_previous.as_ref().unwrap()[0],
        original[0]
    );
    assert_eq!(
        binding.ancestor_notes_current.as_ref().unwrap()[0].notes,
        NEW
    );
    assert_eq!(
        materials(&conn),
        before,
        "preview must preserve the exact manual screenplay"
    );
    let before_doc =
        crate::script_store::load_document_projection(&conn, &generation.payload.block.document_id)
            .unwrap()
            .unwrap();
    accept(&mut conn, &command).unwrap();
    let after_doc =
        crate::script_store::load_document_projection(&conn, &generation.payload.block.document_id)
            .unwrap()
            .unwrap();
    for (before, after) in before_doc.segments.iter().zip(&after_doc.segments) {
        assert_eq!(before.segment, after.segment);
        for (before, after) in before.blocks.iter().zip(&after.blocks) {
            if before.block.id == generation.payload.block.block_id {
                assert_eq!(after.block.text, PREVIEW);
            } else {
                assert_eq!(before, after);
            }
        }
    }
    let accepted = impact(&conn, &generation);
    assert!(!accepted.needs_review);
    assert_eq!(
        recorded(&conn, accepted.generation_event_id).unwrap(),
        binding.ancestor_notes_current
    );
    notes(
        &mut conn,
        &mut project,
        ancestor,
        "Next authored ancestor decision",
    );
    let next = impact(&conn, &generation);
    assert_eq!(next.causes[0].input_excerpt.as_deref(), Some(NEW));
    assert_eq!(
        next.causes[0].dependency_id,
        dependency_id(accepted.generation_event_id, ancestor)
    );
}

#[test]
fn delayed_generation_refuses_ancestor_notes_change_and_restore_atomically() {
    for restored in [false, true] {
        let (mut conn, mut project, generation, ancestor) = setup(false);
        notes(&mut conn, &mut project, ancestor, NEW);
        if restored {
            notes(&mut conn, &mut project, ancestor, OLD);
        }
        let before = logical_rows(&conn);
        assert!(
            script_document_command::apply_generated_script_block(&mut conn, &generation, 230)
                .is_err()
        );
        assert_eq!(logical_rows(&conn), before);
    }
}

#[test]
fn duplicate_forged_notes_wrong_revision_and_unrelated_owned_receipts_are_refused_atomically() {
    let (mut conn, mut project, generation, ancestor) = setup(false);
    let unrelated = project
        .timeline
        .nodes
        .iter()
        .find(|node| {
            node.id != generation.payload.target_binding.as_ref().unwrap().node_id
                && node.parent_id == Some(ancestor)
        })
        .unwrap()
        .id;
    notes(&mut conn, &mut project, unrelated, OLD);
    let unrelated = notes::capture(&conn, unrelated).unwrap();
    let original = generation.payload.ancestor_notes_inputs.as_ref().unwrap()[0].clone();
    for forged in [
        vec![TimelineNotesInput {
            notes: NEW.into(),
            ..original.clone()
        }],
        vec![TimelineNotesInput {
            revision_event_id: unrelated.revision_event_id,
            ..original.clone()
        }],
        vec![TimelineNotesInput {
            revision_event_id: None,
            ..original.clone()
        }],
        vec![original.clone(), original.clone()],
        vec![unrelated],
    ] {
        let mut forged_command = generation.clone();
        forged_command.payload.ancestor_notes_inputs = Some(forged);
        let before = logical_rows(&conn);
        assert!(
            script_document_command::apply_generated_script_block(&mut conn, &forged_command, 230)
                .is_err()
        );
        assert_eq!(logical_rows(&conn), before);
    }
}

#[test]
fn pending_storage_and_acceptance_refuse_ancestor_aba_and_preserve_exact_state() {
    let (mut conn, mut project, generation, ancestor) = setup(true);
    notes(&mut conn, &mut project, ancestor, NEW);
    let command = preview(&mut conn, &generation);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    notes(
        &mut conn,
        &mut project,
        ancestor,
        "Intervening ancestor decision",
    );
    notes(&mut conn, &mut project, ancestor, NEW);
    let before = logical_rows(&conn);
    let mut late = command.clone();
    late.id = CommandId(uuid::Uuid::new_v4());
    late.payload.proposal_id = PropagationProposalId::new("late.ancestor.preview").unwrap();
    let mut delayed = binding;
    delayed.request = late.payload.clone();
    assert!(
        script_impact_review::record_proposal(&mut conn, &late, delayed, PREVIEW.into(), 230)
            .is_err()
    );
    assert!(accept(&mut conn, &command).is_err());
    assert_eq!(logical_rows(&conn), before);
    assert_eq!(
        recorded(&conn, impact(&conn, &generation).generation_event_id)
            .unwrap()
            .unwrap()[0]
            .notes,
        OLD
    );
    let fresh = preview(&mut conn, &generation);
    accept(&mut conn, &fresh).unwrap();
    assert!(!impact(&conn, &generation).needs_review);
}

#[test]
fn clearing_and_accepting_records_an_exact_empty_read_for_later_changes() {
    let (mut conn, mut project, generation, ancestor) = setup(true);
    notes(&mut conn, &mut project, ancestor, "");
    let command = preview(&mut conn, &generation);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    let current = &binding.ancestor_notes_current.as_ref().unwrap()[0];
    assert_eq!(current.notes, "");
    assert!(current.revision_event_id.is_some());
    accept(&mut conn, &command).unwrap();
    assert!(!impact(&conn, &generation).needs_review);
    notes(&mut conn, &mut project, ancestor, NEW);
    let changed = impact(&conn, &generation);
    assert!(changed.needs_review);
    assert_eq!(changed.causes[0].input_excerpt.as_deref(), Some(""));
}

#[test]
fn unrelated_notes_and_non_notes_events_do_not_invalidate_a_pending_ancestor_preview() {
    let (mut conn, mut project, generation, ancestor) = setup(true);
    notes(&mut conn, &mut project, ancestor, NEW);
    let command = preview(&mut conn, &generation);
    let unrelated = project
        .timeline
        .nodes
        .iter()
        .find(|node| node.parent_id == Some(ancestor))
        .unwrap()
        .id;
    notes(
        &mut conn,
        &mut project,
        unrelated,
        "Unconsumed unrelated Notes",
    );
    for locked in [true, false] {
        crate::timeline_command_history::record_set_timeline_node_lock_history(
            &mut conn,
            &project,
            &CommandEnvelope::new(SetTimelineNodeLockCommand {
                node_id: ancestor,
                locked,
            }),
            230,
        )
        .unwrap();
        project.timeline.nodes = crate::timeline_node_store::load_nodes(&conn).unwrap();
    }
    accept(&mut conn, &command).unwrap();
    assert!(!impact(&conn, &generation).needs_review);
}

#[test]
fn missing_legacy_consumption_is_never_backfilled_from_current_ancestor_notes() {
    let (mut conn, mut project, mut generation, ancestor) = setup(false);
    generation.payload.ancestor_notes_inputs = None;
    script_document_command::apply_generated_script_block(&mut conn, &generation, 210).unwrap();
    notes(&mut conn, &mut project, ancestor, NEW);
    let current = impact(&conn, &generation);
    assert!(!current.needs_review);
    assert!(
        recorded(&conn, current.generation_event_id)
            .unwrap()
            .is_none()
    );
}

#[test]
fn ancestor_edit_and_preview_preserve_locked_manual_text_and_refuse_destructive_acceptance() {
    let (mut conn, mut project, generation, ancestor) = setup(true);
    edit(&mut conn, &generation.payload.block, HUMAN);
    let document =
        crate::script_store::load_document_projection(&conn, &generation.payload.block.document_id)
            .unwrap()
            .unwrap();
    let span = document
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|block| block.block.id == generation.payload.block.block_id)
        .unwrap()
        .spans[0]
        .id
        .clone();
    script_document_command::apply_set_script_lock(
        &mut conn,
        &CommandEnvelope::new(SetScriptLockCommand {
            lock_id: ScriptLockId::new("lock.ancestor.manual").unwrap(),
            span_id: span,
            reason: "Keep my exact wording".into(),
        }),
        215,
    )
    .unwrap();
    let saved = materials(&conn);
    notes(&mut conn, &mut project, ancestor, NEW);
    assert_eq!(materials(&conn), saved);
    let command = preview(&mut conn, &generation);
    assert_eq!(materials(&conn), saved);
    let before = logical_rows(&conn);
    assert!(accept(&mut conn, &command).is_err());
    assert_eq!(logical_rows(&conn), before);
}

#[test]
fn deleting_consumed_ancestor_retains_original_review_and_refuses_pending_acceptance() {
    let (mut conn, mut project, generation, ancestor) = setup(true);
    notes(&mut conn, &mut project, ancestor, NEW);
    let command = preview(&mut conn, &generation);
    crate::timeline_node_delete_history::record_delete_timeline_node_history(
        &mut conn,
        &project,
        &CommandEnvelope::new(DeleteTimelineNodeCommand { node_id: ancestor }),
        230,
    )
    .unwrap();
    let current = impact(&conn, &generation);
    assert!(current.needs_review);
    assert!(current.causes.iter().any(|cause| cause.input
        == SemanticDependencyEndpoint::TimelineNode { node_id: ancestor }
        && cause.reason == ScriptImpactReason::Deleted
        && cause.input_excerpt.as_deref() == Some(OLD)));
    let before = logical_rows(&conn);
    assert!(accept(&mut conn, &command).is_err());
    assert_eq!(logical_rows(&conn), before);
}

#[tokio::test]
async fn targeted_provider_receives_exact_current_ancestor_and_preserves_manual_text_on_failure() {
    use eidetic_core::ai::backend::GenerateStream;
    use futures::stream;
    let (mut conn, mut project, generation, ancestor) = setup(true);
    edit(&mut conn, &generation.payload.block, HUMAN);
    notes(&mut conn, &mut project, ancestor, NEW);
    let binding =
        script_impact_review::capture(&conn, &request(&conn, &generation.payload.block).payload)
            .unwrap();
    let before = logical_rows(&conn);
    let result =
        crate::script_impact_prompt::preview_with_provider(&binding, |prompt| async move {
            assert!(prompt.user.contains(&format!(
                "CURRENT CONSUMED ANCESTOR NOTES ({}):\n{NEW}",
                ancestor.0
            )));
            assert!(
                prompt
                    .user
                    .contains(&format!("TARGET BLOCK TO UPDATE:\n{HUMAN}"))
            );
            let output: GenerateStream = Box::pin(stream::iter([
                Ok("Unfinished synthetic prefix".into()),
                Err(eidetic_core::Error::AiBackend("Synthetic failure".into())),
            ]));
            Ok(output)
        })
        .await;
    assert!(result.is_err());
    assert_eq!(logical_rows(&conn), before);
}

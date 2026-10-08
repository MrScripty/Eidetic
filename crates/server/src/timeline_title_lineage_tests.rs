use super::*;
use crate::script_impact_review::tests::{accept, edit, request};
use crate::timeline_notes_lineage::tests::{logical_rows, materials};
use crate::{
    script_document_command, script_impact_review, timeline_command_history, timeline_node_store,
};
use eidetic_core::Project;
const NEW: &str = "  Station departure — 雨.  ";
const HUMAN: &str = "  Authored B: Eli keeps his whistle — 雨.\n\n";
const PREVIEW: &str = "  Synthetic reviewed B: Eli departs — 雨.\n\n";

fn rename(conn: &mut Connection, project: &mut Project, node: NodeId, name: &str) {
    project.timeline.nodes = timeline_node_store::load_nodes(conn).unwrap();
    let original = capture(conn, node).unwrap();
    timeline_command_history::record_set_timeline_node_name_history(
        conn,
        project,
        &CommandEnvelope::new(SetTimelineNodeNameCommand {
            node_id: node,
            name: name.into(),
            expected: TimelineNodeNameRead {
                name: original.name,
                revision_event_id: original.revision_event_id,
            },
        }),
        200,
    )
    .unwrap();
    project.timeline.nodes = timeline_node_store::load_nodes(conn).unwrap();
}
fn setup(
    persist: bool,
) -> (
    Connection,
    Project,
    CommandEnvelope<GenerateScriptBlockCommand>,
) {
    let (mut conn, project, mut generation) = crate::script_generation_target::tests::fixture();
    let target = generation.payload.target_binding.as_ref().unwrap().node_id;
    let prompt = eidetic_core::ai::prompt::build_generate_request(&project, target).unwrap();
    generation.payload.timeline_title_inputs =
        Some(capture_generation(&conn, &supplied_titles(&prompt).unwrap()).unwrap());
    if persist {
        script_document_command::apply_generated_script_block(&mut conn, &generation, 210).unwrap();
    }
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
fn baseline_selected_ancestor_and_sibling_title_changes_have_exact_distinct_causes() {
    for role in ["selected", "ancestor", "sibling"] {
        let (mut conn, mut project, generation) = setup(true);
        let target = generation.payload.target_binding.as_ref().unwrap().node_id;
        let node = match role {
            "selected" => target,
            "ancestor" => project.timeline.ancestors_of(target)[0].id,
            _ => {
                project
                    .timeline
                    .siblings_of(target)
                    .iter()
                    .find(|node| node.id != target)
                    .unwrap()
                    .id
            }
        };
        let old = capture(&conn, node).unwrap();
        assert_eq!(
            old.revision_event_id, None,
            "baseline name must not invent authored history"
        );
        let event = impact(&conn, &generation).generation_event_id;
        let before = materials(&conn);
        rename(&mut conn, &mut project, node, NEW);
        let changed = impact(&conn, &generation);
        assert_eq!(changed.causes.len(), 1, "{role}");
        assert_eq!(changed.causes[0].dependency_id, dependency_id(event, node));
        assert_eq!(
            changed.causes[0].input_excerpt.as_deref(),
            Some(old.name.as_str())
        );
        assert_eq!(
            changed.causes[0].consumed_revision_event_id, event,
            "baseline is a read stamp"
        );
        assert_eq!(
            materials(&conn),
            before,
            "rename must never replace authored screenplay"
        );
    }
}

#[test]
fn canonical_prompt_capture_refuses_changed_input_and_never_resolves_names_by_text() {
    let (mut conn, mut project, mut generation) = setup(false);
    let target = generation.payload.target_binding.as_ref().unwrap().node_id;
    let sibling = project
        .timeline
        .siblings_of(target)
        .iter()
        .find(|node| node.id != target)
        .unwrap()
        .id;
    let target_title = project.timeline.node(target).unwrap().name.clone();
    rename(&mut conn, &mut project, sibling, &target_title);
    let prompt = eidetic_core::ai::prompt::build_generate_request(&project, target).unwrap();
    let supplied = supplied_titles(&prompt).unwrap();
    let receipts = capture_generation(&conn, &supplied).unwrap();
    assert_eq!(
        receipts
            .iter()
            .filter(|input| input.name == target_title)
            .count(),
        2
    );
    generation.payload.timeline_title_inputs = Some(receipts);
    script_document_command::apply_generated_script_block(&mut conn, &generation, 210).unwrap();
    let event = impact(&conn, &generation).generation_event_id;
    assert_ne!(dependency_id(event, target), dependency_id(event, sibling));
    rename(&mut conn, &mut project, sibling, NEW);
    assert!(capture_generation(&conn, &supplied).is_err());
    assert_eq!(impact(&conn, &generation).causes.len(), 1);
}

#[test]
fn preceding_recap_title_has_its_actual_source_identity_and_unconsumed_title_is_out_of_scope() {
    use eidetic_core::timeline::{
        node::{StoryLevel, StoryNode},
        timing::TimeRange,
    };
    let (mut conn, mut project, mut generation) = setup(false);
    let target = generation.payload.target_binding.as_ref().unwrap().node_id;
    let preceding = project
        .timeline
        .siblings_of(target)
        .iter()
        .find(|node| {
            node.id != target
                && node.time_range.end_ms
                    < project.timeline.node(target).unwrap().time_range.start_ms
        })
        .unwrap()
        .id;
    let mut recap = StoryNode::new_child(
        "Recap-only 雨",
        StoryLevel::Beat,
        TimeRange::new(1100, 1200).unwrap(),
        preceding,
    );
    recap.content.scene_recap = Some("Synthetic canonical recap".into());
    let recap_id = recap.id;
    let mut unrelated = recap.clone();
    unrelated.id = NodeId::new();
    unrelated.name = "Unconsumed".into();
    unrelated.content.scene_recap = None;
    let unrelated_id = unrelated.id;
    project.timeline.add_node(recap).unwrap();
    project.timeline.add_node(unrelated).unwrap();
    let tx = conn.transaction().unwrap();
    timeline_node_store::upsert_nodes_in_transaction(&tx, &project.timeline.nodes).unwrap();
    tx.commit().unwrap();
    let prompt = eidetic_core::ai::prompt::build_generate_request(&project, target).unwrap();
    assert!(
        prompt
            .surrounding_context
            .preceding_recaps
            .iter()
            .any(|entry| entry.node_id == Some(recap_id))
    );
    let supplied = supplied_titles(&prompt).unwrap();
    assert!(supplied.iter().any(|(id, _)| *id == recap_id));
    assert!(!supplied.iter().any(|(id, _)| *id == unrelated_id));
    generation.payload.timeline_title_inputs = Some(capture_generation(&conn, &supplied).unwrap());
    script_document_command::apply_generated_script_block(&mut conn, &generation, 210).unwrap();
    rename(&mut conn, &mut project, unrelated_id, "Unrelated edit");
    assert!(!impact(&conn, &generation).needs_review);
    rename(&mut conn, &mut project, recap_id, NEW);
    let changed = impact(&conn, &generation);
    assert_eq!(changed.causes.len(), 1);
    assert_eq!(
        changed.causes[0].input,
        SemanticDependencyEndpoint::TimelineNode { node_id: recap_id }
    );
    let mut legacy = prompt.clone();
    legacy.surrounding_context.preceding_recaps[0].node_id = None;
    assert!(
        supplied_titles(&legacy).is_err(),
        "cannot guess recap source by its title"
    );
}

#[test]
fn delayed_generation_refuses_changed_and_restored_consumed_titles_without_writes() {
    for restored in [false, true] {
        let (mut conn, mut project, generation) = setup(false);
        let target = generation.payload.target_binding.as_ref().unwrap().node_id;
        let sibling = project
            .timeline
            .siblings_of(target)
            .iter()
            .find(|node| node.id != target)
            .unwrap()
            .id;
        let old = capture(&conn, sibling).unwrap().name;
        rename(&mut conn, &mut project, sibling, NEW);
        if restored {
            rename(&mut conn, &mut project, sibling, &old);
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
fn duplicate_forged_wrong_clock_and_missing_selected_receipts_are_refused_atomically() {
    let (mut conn, mut project, generation) = setup(false);
    let target = generation.payload.target_binding.as_ref().unwrap().node_id;
    let original = generation.payload.timeline_title_inputs.clone().unwrap();
    let sibling = original
        .iter()
        .find(|input| input.node_id != target)
        .unwrap()
        .node_id;
    rename(&mut conn, &mut project, sibling, NEW);
    let current = capture(&conn, sibling).unwrap();
    for receipt in [
        vec![original[0].clone(), original[0].clone()],
        vec![TimelineTitleInput {
            name: "Invented".into(),
            ..original[0].clone()
        }],
        vec![TimelineTitleInput {
            revision_event_id: current.revision_event_id,
            ..original[0].clone()
        }],
        vec![current],
    ] {
        let mut forged = generation.clone();
        forged.payload.timeline_title_inputs = Some(receipt);
        let before = logical_rows(&conn);
        assert!(
            script_document_command::apply_generated_script_block(&mut conn, &forged, 230).is_err()
        );
        assert_eq!(logical_rows(&conn), before);
    }
}

#[tokio::test]
async fn targeted_title_prompt_and_acceptance_preserve_manual_text_and_refresh_lineage() {
    let (mut conn, mut project, generation) = setup(true);
    edit(&mut conn, &generation.payload.block, HUMAN);
    let target = generation.payload.target_binding.as_ref().unwrap().node_id;
    let sibling = project
        .timeline
        .siblings_of(target)
        .iter()
        .find(|node| node.id != target)
        .unwrap()
        .id;
    let old = capture(&conn, sibling).unwrap();
    let before = materials(&conn);
    rename(&mut conn, &mut project, sibling, NEW);
    let command = request(&conn, &generation.payload.block);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    assert!(
        binding
            .timeline_title_previous
            .as_ref()
            .unwrap()
            .contains(&old)
    );
    assert!(
        binding
            .timeline_title_current
            .as_ref()
            .unwrap()
            .iter()
            .any(|input| input.node_id == sibling && input.name == NEW)
    );
    let proposed =
        crate::script_impact_prompt::preview_with_provider(&binding, |prompt| async move {
            assert!(prompt.user.contains(HUMAN));
            assert!(prompt.user.contains(&format!(
                "ORIGINAL CONSUMED TIMELINE TITLE ({}):\n{}",
                sibling.0, old.name
            )));
            assert!(prompt.user.contains(&format!(
                "CURRENT CONSUMED TIMELINE TITLE ({}):\n{}",
                sibling.0, NEW
            )));
            let result: eidetic_core::ai::backend::GenerateStream =
                Box::pin(futures::stream::iter(vec![Ok(PREVIEW.into())]));
            Ok(result)
        })
        .await
        .unwrap();
    script_impact_review::record_proposal(&mut conn, &command, binding.clone(), proposed, 220)
        .unwrap();
    assert_eq!(
        materials(&conn),
        before,
        "preview must retain exact manual screenplay"
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
        binding.timeline_title_current
    );
    rename(&mut conn, &mut project, sibling, "Next title");
    let next = impact(&conn, &generation);
    assert_eq!(next.causes[0].input_excerpt.as_deref(), Some(NEW));
    assert_eq!(
        next.causes[0].dependency_id,
        dependency_id(accepted.generation_event_id, sibling)
    );
}

#[test]
fn pending_preview_refuses_consumed_title_change_and_restore_without_any_write() {
    for restored in [false, true] {
        let (mut conn, mut project, generation) = setup(true);
        let target = generation.payload.target_binding.as_ref().unwrap().node_id;
        let sibling = project
            .timeline
            .siblings_of(target)
            .iter()
            .find(|node| node.id != target)
            .unwrap()
            .id;
        rename(&mut conn, &mut project, sibling, NEW);
        let command = preview(&mut conn, &generation);
        rename(&mut conn, &mut project, sibling, "Later title");
        if restored {
            rename(&mut conn, &mut project, sibling, NEW);
        }
        let before = logical_rows(&conn);
        assert!(accept(&mut conn, &command).is_err());
        assert_eq!(logical_rows(&conn), before);
    }
}

#[test]
fn legacy_unknown_receipts_are_not_backfilled_after_title_edits() {
    let (mut conn, mut project, mut generation) = setup(false);
    generation.payload.timeline_title_inputs = None;
    script_document_command::apply_generated_script_block(&mut conn, &generation, 210).unwrap();
    let event = impact(&conn, &generation).generation_event_id;
    let target = generation.payload.target_binding.as_ref().unwrap().node_id;
    let sibling = project
        .timeline
        .siblings_of(target)
        .iter()
        .find(|node| node.id != target)
        .unwrap()
        .id;
    rename(&mut conn, &mut project, sibling, NEW);
    assert_eq!(recorded(&conn, event).unwrap(), None);
    assert!(impact(&conn, &generation).causes.is_empty());
    let inputs = preview_inputs(&conn, event).unwrap();
    assert!(inputs.previous.is_none() && inputs.current.is_none() && inputs.absent.is_none());
}

#[test]
fn deleted_title_sources_bind_owned_absence_and_acceptance_tracks_only_remaining_titles() {
    let (mut conn, mut project, generation) = setup(true);
    let target = generation.payload.target_binding.as_ref().unwrap().node_id;
    let sibling = project
        .timeline
        .siblings_of(target)
        .iter()
        .find(|node| node.id != target)
        .unwrap()
        .id;
    let old = capture(&conn, sibling).unwrap();
    project.timeline.nodes = timeline_node_store::load_nodes(&conn).unwrap();
    crate::timeline_node_delete_history::record_delete_timeline_node_history(
        &mut conn,
        &project,
        &CommandEnvelope::new(DeleteTimelineNodeCommand { node_id: sibling }),
        220,
    )
    .unwrap();
    let changed = impact(&conn, &generation);
    let cause = changed
        .causes
        .iter()
        .find(|cause| cause.dependency_id == dependency_id(changed.generation_event_id, sibling))
        .unwrap();
    assert_eq!(cause.reason, ScriptImpactReason::Deleted);
    assert_eq!(cause.input_excerpt.as_deref(), Some(old.name.as_str()));
    let mut command = request(&conn, &generation.payload.block);
    command.payload.dependency_id = cause.dependency_id.clone();
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    assert!(
        !binding
            .timeline_title_current
            .as_ref()
            .unwrap()
            .iter()
            .any(|input| input.node_id == sibling)
    );
    let deletion = timeline_node_store::latest_name_event(&conn, sibling, None)
        .unwrap()
        .unwrap();
    assert_eq!(
        binding.timeline_title_absence_revisions.as_ref().unwrap(),
        &vec![(sibling, deletion)]
    );
    let before = materials(&conn);
    script_impact_review::record_proposal(
        &mut conn,
        &command,
        binding.clone(),
        PREVIEW.into(),
        230,
    )
    .unwrap();
    assert_eq!(materials(&conn), before);
    accept(&mut conn, &command).unwrap();
    let accepted = impact(&conn, &generation);
    assert!(!accepted.needs_review);
    assert_eq!(
        recorded(&conn, accepted.generation_event_id).unwrap(),
        binding.timeline_title_current
    );
}

#[test]
fn deleted_title_absence_edit_restore_delete_aba_refuses_pending_acceptance() {
    let (mut conn, mut project, generation) = setup(true);
    let target = generation.payload.target_binding.as_ref().unwrap().node_id;
    let sibling = (**project
        .timeline
        .siblings_of(target)
        .iter()
        .find(|node| node.id != target)
        .unwrap())
    .clone();
    let delete = |conn: &mut Connection, project: &mut Project| {
        project.timeline.nodes = timeline_node_store::load_nodes(conn).unwrap();
        crate::timeline_node_delete_history::record_delete_timeline_node_history(
            conn,
            project,
            &CommandEnvelope::new(DeleteTimelineNodeCommand {
                node_id: sibling.id,
            }),
            220,
        )
        .unwrap();
        project.timeline.nodes = timeline_node_store::load_nodes(conn).unwrap();
    };
    delete(&mut conn, &mut project);
    let command = preview(&mut conn, &generation);
    timeline_command_history::record_create_timeline_node_history(
        &mut conn,
        &project,
        &CommandEnvelope::new(CreateTimelineNodeCommand {
            node_id: sibling.id,
            parent_id: sibling.parent_id,
            level: sibling.level,
            name: sibling.name.clone(),
            start_ms: sibling.time_range.start_ms,
            end_ms: sibling.time_range.end_ms,
            beat_type: sibling.beat_type.clone(),
        }),
        230,
    )
    .unwrap();
    delete(&mut conn, &mut project);
    let before = logical_rows(&conn);
    assert!(accept(&mut conn, &command).is_err());
    assert_eq!(logical_rows(&conn), before);
}

use super::*;
use crate::{script_document_command, timeline_command_history};
use eidetic_core::Project;

pub(crate) fn fixture() -> (
    Connection,
    Project,
    CommandEnvelope<GenerateScriptBlockCommand>,
) {
    let (conn, project, blocks) = crate::script_context_scope::tests::fixture();
    let node_id = crate::script_context_scope::tests::node(&blocks[1]);
    let node = project.timeline.node(node_id).unwrap();
    let mut block = blocks[1].clone();
    block.segment_id = ScriptSegmentId::new(format!("script.segment.{}", node_id.0)).unwrap();
    block.block_id = ScriptBlockId::new(format!("script.block.{}.generated", node_id.0)).unwrap();
    block.span_provenance = ScriptSpanProvenance::AiGenerated;
    block.text = "Synthetic new canonical scene.\n\n".into();
    let command = CommandEnvelope::new(GenerateScriptBlockCommand {
        arc_description_applicability: None,
        timeline_title_inputs: None,
        ancestor_notes_inputs: None,
        arc_inputs: None,
        block,
        script_inputs: None,
        bible_node_name_inputs: None,
        bible_relationship_inputs: None,
        bible_inputs: None,
        bible_context_scope: None,
        script_context_scope: None,
        target_binding: Some(capture(&conn, node).unwrap()),
    });
    (conn, project, command)
}

fn snapshot(conn: &Connection) -> (String, Vec<i64>) {
    let doc = script_store::load_document_projection(
        conn,
        &ScriptDocumentId::new("script.document.main").unwrap(),
    )
    .unwrap();
    let counts = [
        "commands",
        "change_events",
        "object_revisions",
        "semantic_dependencies",
    ]
    .map(|table| {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
    });
    (serde_json::to_string(&doc).unwrap(), counts.into())
}

fn apply(conn: &mut Connection, command: &CommandEnvelope<GenerateScriptBlockCommand>) {
    script_document_command::apply_generated_script_block(conn, command, 100).unwrap();
}

fn refuse(conn: &mut Connection, command: &CommandEnvelope<GenerateScriptBlockCommand>) {
    let before = snapshot(conn);
    assert!(script_document_command::apply_generated_script_block(conn, command, 100).is_err());
    assert_eq!(
        snapshot(conn),
        before,
        "refusal cannot write screenplay or history"
    );
}

#[test]
fn manual_timeline_notes_edit_marks_its_saved_generated_screenplay_for_review() {
    let (mut conn, mut project, mut command) = fixture();
    let node_id = command.payload.target_binding.as_ref().unwrap().node_id;
    for (index, notes) in [
        "Mara conceals the witness.",
        "  Mara reveals the witness — 雨.\n\n  ",
    ]
    .into_iter()
    .enumerate()
    {
        project.timeline.nodes = timeline_node_store::load_nodes(&conn).unwrap();
        timeline_command_history::record_set_timeline_node_notes_history(
            &mut conn,
            &project,
            &CommandEnvelope::new(SetTimelineNodeNotesCommand {
                expected: None,
                node_id,
                notes: notes.into(),
            }),
            110 + index as u64,
        )
        .unwrap();
        if index == 0 {
            project.timeline.nodes = timeline_node_store::load_nodes(&conn).unwrap();
            command.payload.target_binding =
                Some(capture(&conn, project.timeline.node(node_id).unwrap()).unwrap());
            apply(&mut conn, &command);
        }
    }
    let review =
        crate::script_impact_projection::load_impact(&conn, &command.payload.block.segment_id)
            .unwrap()
            .unwrap();
    assert!(
        review.needs_review,
        "authored timeline Notes changed after generation, but saved screenplay has no review cause"
    );
}

#[test]
fn canonical_generation_commits_status_and_preserves_every_prior_authored_block() {
    let (mut conn, project, command) = fixture();
    let before = script_store::load_document_projection(&conn, &command.payload.block.document_id)
        .unwrap()
        .unwrap();
    apply(&mut conn, &command);
    let after = script_store::load_document_projection(&conn, &command.payload.block.document_id)
        .unwrap()
        .unwrap();
    for segment in &before.segments {
        let retained = after
            .segments
            .iter()
            .find(|s| s.segment.id == segment.segment.id)
            .unwrap();
        assert_eq!(retained.segment, segment.segment);
        assert_eq!(retained.blocks, segment.blocks);
    }
    let node_id = command.payload.target_binding.as_ref().unwrap().node_id;
    let node = timeline_node_store::load_node_ancestor_stack(&conn, node_id)
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(
        node.content.notes,
        project.timeline.node(node_id).unwrap().content.notes
    );
    assert_eq!(
        node.content.status,
        eidetic_core::timeline::node::ContentStatus::HasContent
    );
    assert_eq!(
        snapshot(&conn),
        {
            let before = snapshot(&conn);
            apply(&mut conn, &command);
            before
        },
        "replay ignores now changed revisions without another write"
    );
}

#[test]
fn delayed_generation_refuses_intervening_human_output_even_after_text_aba() {
    let (mut conn, project, first) = fixture();
    apply(&mut conn, &first);
    let mut delayed = first.clone();
    delayed.id = CommandId(uuid::Uuid::new_v4());
    let node_id = first.payload.target_binding.as_ref().unwrap().node_id;
    delayed.payload.target_binding =
        Some(capture(&conn, project.timeline.node(node_id).unwrap()).unwrap());
    delayed.payload.block.text = "Delayed synthetic output".into();
    for text in [
        "Exact newer human output\n\n",
        first.payload.block.text.as_str(),
    ] {
        let document =
            script_store::load_document_projection(&conn, &first.payload.block.document_id)
                .unwrap()
                .unwrap();
        let block = document
            .segments
            .iter()
            .flat_map(|s| &s.blocks)
            .find(|b| b.block.id == first.payload.block.block_id)
            .unwrap();
        crate::script_block_edit::apply_edit_script_block(
            &mut conn,
            &CommandEnvelope::new(EditScriptBlockCommand {
                document_id: first.payload.block.document_id.clone(),
                block_id: block.block.id.clone(),
                expected_revision_event_id: block.revision_event_id.unwrap(),
                text: text.into(),
            }),
            110,
        )
        .unwrap();
        refuse(&mut conn, &delayed);
    }
    assert!(
        validate_admission(
            &conn,
            &capture(&conn, project.timeline.node(node_id).unwrap()).unwrap()
        )
        .is_err(),
        "manual output needs reviewed replacement"
    );
}

#[test]
fn notes_edit_and_restore_advances_target_revision_and_refuses_delayed_output() {
    let (mut conn, mut project, command) = fixture();
    let node_id = command.payload.target_binding.as_ref().unwrap().node_id;
    let old = project
        .timeline
        .node(node_id)
        .unwrap()
        .content
        .notes
        .clone();
    for notes in ["Exact later notes — 雨".to_string(), old] {
        timeline_command_history::record_set_timeline_node_notes_history(
            &mut conn,
            &project,
            &CommandEnvelope::new(SetTimelineNodeNotesCommand {
                expected: None,
                node_id,
                notes: notes.clone(),
            }),
            110,
        )
        .unwrap();
        project.timeline.nodes = timeline_node_store::load_nodes(&conn).unwrap();
        refuse(&mut conn, &command);
    }
}

#[test]
fn retime_restore_lock_restore_and_delete_all_refuse_delayed_generation() {
    for operation in ["retime", "lock", "delete"] {
        let (mut conn, mut project, command) = fixture();
        let node_id = command.payload.target_binding.as_ref().unwrap().node_id;
        match operation {
            "retime" => {
                for start in [4500, 4000] {
                    timeline_command_history::record_set_timeline_node_range_history(
                        &mut conn,
                        &project,
                        &CommandEnvelope::new(SetTimelineNodeRangeCommand {
                            node_id,
                            start_ms: start,
                            end_ms: start + 500,
                            expected: None,
                        }),
                        110,
                    )
                    .unwrap();
                    project.timeline.node_mut(node_id).unwrap().time_range =
                        eidetic_core::timeline::timing::TimeRange::new(start, start + 500).unwrap();
                    refuse(&mut conn, &command);
                }
            }
            "lock" => {
                for locked in [true, false] {
                    timeline_command_history::record_set_timeline_node_lock_history(
                        &mut conn,
                        &project,
                        &CommandEnvelope::new(SetTimelineNodeLockCommand { node_id, locked }),
                        110,
                    )
                    .unwrap();
                    project.timeline.node_mut(node_id).unwrap().locked = locked;
                    refuse(&mut conn, &command);
                }
            }
            _ => {
                crate::timeline_node_delete_history::record_delete_timeline_node_history(
                    &mut conn,
                    &project,
                    &CommandEnvelope::new(DeleteTimelineNodeCommand { node_id }),
                    110,
                )
                .unwrap();
                refuse(&mut conn, &command);
            }
        }
    }
}

#[test]
fn manual_append_after_admission_refuses_output_without_touching_new_authored_text() {
    let (mut conn, project, first) = fixture();
    apply(&mut conn, &first);
    let node_id = first.payload.target_binding.as_ref().unwrap().node_id;
    let mut delayed = first.clone();
    delayed.id = CommandId(uuid::Uuid::new_v4());
    delayed.payload.target_binding =
        Some(capture(&conn, project.timeline.node(node_id).unwrap()).unwrap());
    crate::script_block_create::apply_create_script_block(
        &mut conn,
        &CommandEnvelope::new(CreateScriptBlockCommand {
            document_id: first.payload.block.document_id.clone(),
            source_node_id: node_id,
            expected_start_ms: first.payload.block.segment_start_ms,
            expected_end_ms: first.payload.block.segment_end_ms,
            block_kind: ScriptBlockKind::Action,
            text: "Exact authored append survives.\n\n".into(),
        }),
        110,
    )
    .unwrap();
    refuse(&mut conn, &delayed);
}

#[test]
fn forged_target_or_completion_placement_is_refused_transactionally() {
    for field in ["node", "placement", "block"] {
        let (mut conn, _, mut command) = fixture();
        match field {
            "node" => {
                command.payload.target_binding.as_mut().unwrap().node_id =
                    NodeId(uuid::Uuid::new_v4())
            }
            "placement" => command.payload.block.segment_start_ms += 1,
            _ => {
                command.payload.block.block_id = ScriptBlockId::new("another.human.block").unwrap()
            }
        }
        refuse(&mut conn, &command);
    }
}

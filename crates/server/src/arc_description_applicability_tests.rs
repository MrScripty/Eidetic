use super::*;
use crate::script_impact_review::tests::{accept, edit, fixture, request, text};
use crate::{script_document_command, script_impact_review, story_arc_command};
use eidetic_core::story::arc::{ArcId, ArcType, Color};

const NEW: &str = "  Mara chooses exile — 雨.\n\n  ";
const HUMAN: &str = "  Exact authored screenplay: Mara keeps the letter — 雨.\n\n  ";
const PREVIEW: &str = "Synthetic review: Mara leaves with the letter.\n\n";

fn create(conn: &mut Connection) -> ArcId {
    let arc_id = ArcId::new();
    story_arc_command::record_create_story_arc_history(
        conn,
        &CommandEnvelope::new(CreateStoryArcCommand {
            arc_id,
            parent_arc_id: None,
            name: "Exile".into(),
            description: String::new(),
            arc_type: ArcType::APlot,
            color: Color::new(1, 2, 3),
        }),
        20,
    )
    .unwrap();
    arc_id
}

fn metadata(conn: &mut Connection, arc_id: ArcId, description: Option<&str>, color: Option<Color>) {
    story_arc_command::record_set_story_arc_metadata_history(
        conn,
        &CommandEnvelope::new(SetStoryArcMetadataCommand {
            arc_id,
            name: None,
            description: description.map(str::to_owned),
            arc_type: None,
            color,
        }),
        30,
    )
    .unwrap();
}

fn setup(
    persist: bool,
) -> (
    Connection,
    ArcId,
    GenerateScriptBlockCommand,
    SetScriptBlockCommand,
) {
    let (mut conn, _, _, b, c) = fixture();
    let arc = create(&mut conn);
    conn.execute(
        "INSERT INTO node_arcs(node_id,arc_id) VALUES (?1,?2)",
        params![b.source_node_id, arc.0.to_string()],
    )
    .unwrap();
    let node = NodeId(uuid::Uuid::parse_str(b.source_node_id.as_ref().unwrap()).unwrap());
    let (_, fields) = arcs::capture(&conn, node, &[]).unwrap();
    let command = GenerateScriptBlockCommand {
        arc_description_applicability: Some(capture(&conn, node, &fields).unwrap()),
        arc_inputs: Some(fields),
        ancestor_notes_inputs: None,
        block: b,
        script_inputs: Some(vec![]),
        bible_inputs: Some(vec![]),
        bible_node_name_inputs: None,
        bible_relationship_inputs: None,
        bible_context_scope: None,
        script_context_scope: None,
        target_binding: None,
    };
    if persist {
        script_document_command::apply_generated_script_block(
            &mut conn,
            &CommandEnvelope::new(command.clone()),
            25,
        )
        .unwrap();
    }
    (conn, arc, command, c)
}

fn impact(conn: &Connection, command: &GenerateScriptBlockCommand) -> ScriptImpactProjection {
    crate::script_impact_projection::load_impact(conn, &command.block.segment_id)
        .unwrap()
        .unwrap()
}

fn rows(conn: &Connection) -> Vec<(String, Vec<String>)> {
    let mut result = Vec::new();
    for table in [
        "commands",
        "change_events",
        "object_revisions",
        "object_revision_fields",
        "semantic_dependencies",
        "semantic_dependency_revisions",
        "script_generations",
        "script_documents",
        "script_segments",
        "script_blocks",
        "script_spans",
        "script_locks",
        "propagation_proposals",
        "script_impact_proposal_bindings",
    ] {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
                [table],
                |row| row.get(0),
            )
            .unwrap();
        if exists {
            let mut stmt = conn
                .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
                .unwrap();
            let count = stmt.column_count();
            let values = stmt
                .query_map([], |row| {
                    Ok((0..count)
                        .map(|index| format!("{:?}", row.get_ref(index).unwrap()))
                        .collect::<Vec<_>>()
                        .join("|"))
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            result.push((table.into(), values));
        }
    }
    result
}

fn preview(
    conn: &mut Connection,
    command: &GenerateScriptBlockCommand,
) -> CommandEnvelope<RequestScriptImpactProposalCommand> {
    let request = request(conn, &command.block);
    let binding = script_impact_review::capture(conn, &request.payload).unwrap();
    script_impact_review::record_proposal(conn, &request, binding, PREVIEW.into(), 40).unwrap();
    request
}

#[test]
fn owned_omission_entry_reviews_manual_material_and_acceptance_installs_actual_description() {
    let (mut conn, arc, command, c) = setup(true);
    edit(&mut conn, &command.block, HUMAN);
    let before = crate::script_store::load_document_projection(&conn, &command.block.document_id)
        .unwrap()
        .unwrap();
    let omission = command.arc_description_applicability.as_ref().unwrap()[0].clone();
    assert!(omission.value.is_empty() && omission.revision_event_id.is_some());
    assert!(
        !command
            .arc_inputs
            .as_ref()
            .unwrap()
            .iter()
            .any(|value| value.field == StoryArcPromptField::Description)
    );
    metadata(&mut conn, arc, Some(NEW), None);
    let changed = impact(&conn, &command);
    assert_eq!(changed.causes.len(), 1);
    assert_eq!(
        changed.causes[0].dependency_id,
        dependency_id(changed.generation_event_id, &omission)
    );
    assert_eq!(changed.causes[0].input, arcs::endpoint(&omission));
    let stored: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM semantic_dependencies WHERE id=?1",
            [changed.causes[0].dependency_id.as_str()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        stored, 1,
        "Applicability must have an actual precise graph dependency"
    );
    assert_eq!(
        changed.causes[0].input_excerpt.as_deref(),
        Some("Description was not supplied (known empty).")
    );
    let request = preview(&mut conn, &command);
    let binding = script_impact_review::capture(&conn, &request.payload).unwrap();
    assert_eq!(
        binding.arc_description_applicability_previous,
        command.arc_description_applicability
    );
    assert!(
        binding
            .arc_description_applicability_current
            .as_ref()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        binding
            .arc_inputs
            .as_ref()
            .unwrap()
            .iter()
            .find(|value| value.field == StoryArcPromptField::Description)
            .unwrap()
            .value,
        NEW
    );
    assert_eq!(text(&conn, &command.block), HUMAN);
    let after = crate::script_store::load_document_projection(&conn, &command.block.document_id)
        .unwrap()
        .unwrap();
    for (a, b) in before.segments.iter().zip(&after.segments) {
        assert_eq!(a.segment, b.segment);
        assert_eq!(a.blocks, b.blocks);
    }
    accept(&mut conn, &request).unwrap();
    assert_eq!(text(&conn, &command.block), PREVIEW);
    assert_eq!(text(&conn, &c), c.text);
    assert!(!impact(&conn, &command).needs_review);
    metadata(&mut conn, arc, Some("Later authored direction"), None);
    let next = impact(&conn, &command);
    assert_eq!(next.causes.len(), 1);
    assert!(
        !next.causes[0]
            .dependency_id
            .as_str()
            .contains(".arc_description_applicability.")
    );
    assert_eq!(next.causes[0].input_excerpt.as_deref(), Some(NEW));
}

#[test]
fn clear_withdraws_entry_and_restore_aba_refuses_old_preview_without_writes() {
    let (mut conn, arc, command, _) = setup(true);
    edit(&mut conn, &command.block, HUMAN);
    metadata(&mut conn, arc, Some(NEW), None);
    let request = preview(&mut conn, &command);
    let binding = script_impact_review::capture(&conn, &request.payload).unwrap();
    metadata(&mut conn, arc, Some(""), None);
    assert!(!impact(&conn, &command).needs_review);
    let before = rows(&conn);
    assert!(accept(&mut conn, &request).is_err());
    assert_eq!(rows(&conn), before);
    metadata(&mut conn, arc, Some(NEW), None);
    let before = rows(&conn);
    assert!(accept(&mut conn, &request).is_err());
    let mut late = request.clone();
    late.id = CommandId(uuid::Uuid::new_v4());
    late.payload.proposal_id = PropagationProposalId::new("late.arc.description").unwrap();
    let mut delayed = binding;
    delayed.request = late.payload.clone();
    assert!(
        script_impact_review::record_proposal(&mut conn, &late, delayed, PREVIEW.into(), 50)
            .is_err()
    );
    assert_eq!(rows(&conn), before);
    assert_eq!(text(&conn, &command.block), HUMAN);
    let fresh = preview(&mut conn, &command);
    accept(&mut conn, &fresh).unwrap();
    assert!(!impact(&conn, &command).needs_review);
}

#[test]
fn delayed_generation_and_forged_omission_receipts_refuse_atomically() {
    for restore in [false, true] {
        let (mut conn, arc, command, _) = setup(false);
        metadata(&mut conn, arc, Some(NEW), None);
        if restore {
            metadata(&mut conn, arc, Some(""), None);
        }
        let before = rows(&conn);
        assert!(
            script_document_command::apply_generated_script_block(
                &mut conn,
                &CommandEnvelope::new(command),
                50
            )
            .is_err()
        );
        assert_eq!(rows(&conn), before);
    }
    let (mut conn, _, command, _) = setup(false);
    let original = command.arc_description_applicability.as_ref().unwrap()[0].clone();
    let other = create(&mut conn);
    let wrong = StoryArcFieldInput {
        arc_id: other,
        ..original.clone()
    };
    for inputs in [
        vec![],
        vec![original.clone(), original.clone()],
        vec![StoryArcFieldInput {
            value: NEW.into(),
            ..original.clone()
        }],
        vec![StoryArcFieldInput {
            revision_event_id: None,
            ..original
        }],
        vec![wrong],
    ] {
        let mut forged = command.clone();
        forged.arc_description_applicability = Some(inputs);
        let before = rows(&conn);
        assert!(
            script_document_command::apply_generated_script_block(
                &mut conn,
                &CommandEnvelope::new(forged),
                50
            )
            .is_err()
        );
        assert_eq!(rows(&conn), before);
    }
}

#[test]
fn unrelated_arc_color_and_repeated_empty_edits_do_not_create_review_or_stale_preview() {
    let (mut conn, arc, command, _) = setup(true);
    let other = create(&mut conn);
    metadata(&mut conn, arc, Some(""), None);
    metadata(&mut conn, other, Some(NEW), None);
    assert!(!impact(&conn, &command).needs_review);
    metadata(&mut conn, arc, Some(NEW), None);
    let request = preview(&mut conn, &command);
    metadata(&mut conn, arc, None, Some(Color::new(9, 8, 7)));
    metadata(&mut conn, other, Some("Other direction"), None);
    accept(&mut conn, &request).unwrap();
    assert!(!impact(&conn, &command).needs_review);
}

#[test]
fn legacy_missing_applicability_and_unowned_empty_history_remain_unknown() {
    for legacy in [true, false] {
        let (mut conn, arc, mut command, _) = setup(false);
        if legacy {
            command.arc_description_applicability = None;
        } else {
            conn.execute("DELETE FROM object_revision_fields WHERE revision_id IN (SELECT id FROM object_revisions WHERE object_kind='story_arc' AND object_id=?1)",[arc.0.to_string()]).unwrap();
            let node = NodeId(
                uuid::Uuid::parse_str(command.block.source_node_id.as_ref().unwrap()).unwrap(),
            );
            let (_, fields) = arcs::capture(&conn, node, &[]).unwrap();
            command.arc_description_applicability = Some(capture(&conn, node, &fields).unwrap());
            command.arc_inputs = Some(fields);
            assert!(
                command.arc_description_applicability.as_ref().unwrap()[0]
                    .revision_event_id
                    .is_none()
            );
        }
        script_document_command::apply_generated_script_block(
            &mut conn,
            &CommandEnvelope::new(command.clone()),
            50,
        )
        .unwrap();
        metadata(&mut conn, arc, Some(NEW), None);
        assert!(!impact(&conn, &command).needs_review);
    }
}

#[test]
fn accepting_one_consumer_preserves_the_other_consumer_and_its_precise_cause() {
    let (mut conn, arc, command, c) = setup(true);
    conn.execute(
        "INSERT INTO node_arcs(node_id,arc_id) VALUES (?1,?2)",
        params![c.source_node_id, arc.0.to_string()],
    )
    .unwrap();
    let mut second = command.clone();
    second.block = c.clone();
    script_document_command::apply_generated_script_block(
        &mut conn,
        &CommandEnvelope::new(second.clone()),
        26,
    )
    .unwrap();
    edit(&mut conn, &command.block, HUMAN);
    metadata(&mut conn, arc, Some(NEW), None);
    let first_cause = impact(&conn, &command).causes[0].clone();
    let second_cause = impact(&conn, &second).causes[0].clone();
    assert_ne!(first_cause.dependency_id, second_cause.dependency_id);
    for cause in [&first_cause, &second_cause] {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM semantic_dependencies WHERE id=?1",
                [cause.dependency_id.as_str()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
    let request = preview(&mut conn, &command);
    accept(&mut conn, &request).unwrap();
    assert!(!impact(&conn, &command).needs_review);
    assert_eq!(impact(&conn, &second).causes, [second_cause]);
    assert_eq!(text(&conn, &c), c.text);
}

#[test]
fn applicability_preview_preserves_locked_manual_text_and_refuses_replacement() {
    let (mut conn, arc, command, _) = setup(true);
    edit(&mut conn, &command.block, HUMAN);
    let doc = crate::script_store::load_document_projection(&conn, &command.block.document_id)
        .unwrap()
        .unwrap();
    let span = doc
        .segments
        .iter()
        .flat_map(|row| &row.blocks)
        .find(|block| block.block.id == command.block.block_id)
        .unwrap()
        .spans[0]
        .id
        .clone();
    script_document_command::apply_set_script_lock(
        &mut conn,
        &CommandEnvelope::new(SetScriptLockCommand {
            lock_id: ScriptLockId::new("lock.arc.applicability").unwrap(),
            span_id: span,
            reason: "Keep authored wording".into(),
        }),
        27,
    )
    .unwrap();
    metadata(&mut conn, arc, Some(NEW), None);
    let request = preview(&mut conn, &command);
    let before = rows(&conn);
    assert!(accept(&mut conn, &request).is_err());
    assert_eq!(rows(&conn), before);
    assert_eq!(text(&conn, &command.block), HUMAN);
}

#[tokio::test]
async fn exact_new_description_and_manual_target_reach_provider_failure_preserves_material() {
    use eidetic_core::ai::backend::GenerateStream;
    let (mut conn, arc, command, _) = setup(true);
    edit(&mut conn, &command.block, HUMAN);
    metadata(&mut conn, arc, Some(NEW), None);
    let binding =
        script_impact_review::capture(&conn, &request(&conn, &command.block).payload).unwrap();
    let before = rows(&conn);
    let result =
        crate::script_impact_prompt::preview_with_provider(&binding, |prompt| async move {
            assert!(prompt.user.contains(NEW));
            assert!(prompt.user.contains(HUMAN));
            assert!(
                prompt
                    .user
                    .contains("ORIGINAL KNOWN-EMPTY ARC DESCRIPTION APPLICABILITY")
            );
            Ok(Box::pin(futures::stream::iter([
                Ok("Synthetic partial prefix".into()),
                Err(eidetic_core::Error::AiBackend("Synthetic failure".into())),
            ])) as GenerateStream)
        })
        .await;
    assert!(result.is_err());
    assert_eq!(rows(&conn), before);
    assert_eq!(text(&conn, &command.block), HUMAN);
}

use super::*;
use crate::project_service::replace_active_project;
use eidetic_core::{Error, Template};
use futures::stream;

const CONSUMED_ANCESTOR_NOTES: &str = "  Mara conceals the witness — 雨.\n\n  ";
const UNCONSUMED_ACT_NOTES: &str = "Eli keeps a brass whistle in the unrelated Act.";

const ENTERED_ARC_DESCRIPTION: &str = "  Mara chooses exile — 雨.\n\n  ";

async fn empty_arc_description_fixture() -> (
    Fixture,
    eidetic_core::story::arc::ArcId,
    eidetic_core::story::arc::ArcId,
) {
    use eidetic_core::timeline::node::StoryLevel;
    let mut fixture = fixture().await;
    let (project, _) = active_sqlite_project(&fixture.state).await.unwrap();
    fixture.node_id = project.timeline.nodes_at_level(StoryLevel::Scene)[0].id;
    let arc = project.timeline.arcs_for_node(fixture.node_id)[0];
    let unrelated = project
        .arcs
        .iter()
        .find(|candidate| candidate.id != arc)
        .unwrap()
        .id;
    // Publicly commit a known-empty field; imported/unowned absence is distinct.
    set_fixture_arc_description(&fixture, arc, "").await;
    let (project, _) = active_sqlite_project(&fixture.state).await.unwrap();
    let mut request =
        eidetic_core::ai::prompt::build_generate_request(&project, fixture.node_id).unwrap();
    crate::ai_service::attach_ai_generation_context(
        &mut request,
        fixture.path.clone(),
        fixture.node_id,
    )
    .await
    .unwrap();
    assert!(
        request
            .tagged_arcs
            .iter()
            .any(|value| value.id == arc && value.description.is_empty())
    );
    assert!(
        !request
            .arc_inputs
            .as_ref()
            .unwrap()
            .iter()
            .any(|input| input.arc_id == arc
                && input.field == eidetic_core::contracts::StoryArcPromptField::Description)
    );
    assert!(
        !crate::prompt_format::build_chat_prompt(&request)
            .user
            .contains(ENTERED_ARC_DESCRIPTION)
    );
    persist_generated_script_block(
        fixture.path.clone(),
        fixture.node_id.0,
        "Synthetic saved screenplay without arc description.".into(),
        GenerationInputs {
            arc_description_applicability: request.arc_description_applicability,
            ancestor_notes_inputs: request.ancestor_notes_inputs,
            arc_inputs: request.arc_inputs,
            script_inputs: request.script_context,
            bible_inputs: request.bible_inputs,
            bible_node_name_inputs: request.bible_node_name_inputs,
            bible_relationship_inputs: request.bible_relationship_inputs,
            bible_context_scope: request.bible_context_scope,
            script_context_scope: request.script_context_scope,
            target_binding: request.generation_target,
            ..GenerationInputs::default()
        },
    )
    .await
    .unwrap();
    (fixture, arc, unrelated)
}

async fn set_fixture_arc_description(
    fixture: &Fixture,
    arc_id: eidetic_core::story::arc::ArcId,
    text: &str,
) {
    use eidetic_core::contracts::*;
    crate::command_service::update_story_arc(
        &fixture.state,
        CommandEnvelope::new(SetStoryArcMetadataCommand {
            arc_id,
            name: None,
            description: Some(text.into()),
            arc_type: None,
            color: None,
        }),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn public_known_empty_arc_description_entry_marks_saved_scene_for_review() {
    use eidetic_core::contracts::*;
    let (fixture, arc, _) = empty_arc_description_fixture().await;
    let before = script(&fixture);
    set_fixture_arc_description(&fixture, arc, ENTERED_ARC_DESCRIPTION).await;
    let after = script(&fixture);
    assert_saved_material_unchanged(&before.payload, &after.payload);
    let (project, _) = active_sqlite_project(&fixture.state).await.unwrap();
    let mut fresh =
        eidetic_core::ai::prompt::build_generate_request(&project, fixture.node_id).unwrap();
    crate::ai_service::attach_ai_generation_context(
        &mut fresh,
        fixture.path.clone(),
        fixture.node_id,
    )
    .await
    .unwrap();
    assert!(
        crate::prompt_format::build_chat_prompt(&fresh)
            .user
            .contains(ENTERED_ARC_DESCRIPTION)
    );
    let changed = after
        .payload
        .segments
        .iter()
        .find(|row| {
            row.segment.source_node_id.as_deref() == Some(fixture.node_id.0.to_string().as_str())
        })
        .unwrap()
        .impact
        .as_ref()
        .unwrap();
    fixture.state.shutdown_tasks_async().await;
    assert!(
        changed.causes.iter().any(|cause| cause.input
            == SemanticDependencyEndpoint::StoryArcField {
                arc_id: arc,
                field: StoryArcPromptField::Description
            }
            && cause
                .dependency_id
                .as_str()
                .ends_with(&format!(".arc_description_applicability.{}", arc.0))),
        "Known-empty tagged arc description became supplied without precise downstream Scene review"
    );
}

#[tokio::test]
async fn public_known_empty_arc_clear_and_untagged_description_preserve_saved_scene() {
    let (fixture, arc, unrelated) = empty_arc_description_fixture().await;
    let before = script(&fixture);
    set_fixture_arc_description(&fixture, arc, "").await;
    set_fixture_arc_description(&fixture, unrelated, ENTERED_ARC_DESCRIPTION).await;
    let after = script(&fixture);
    assert_saved_material_unchanged(&before.payload, &after.payload);
    assert_eq!(
        before
            .payload
            .segments
            .iter()
            .map(|row| &row.impact)
            .collect::<Vec<_>>(),
        after
            .payload
            .segments
            .iter()
            .map(|row| &row.impact)
            .collect::<Vec<_>>()
    );
    fixture.state.shutdown_tasks_async().await;
}

async fn ancestor_notes_fixture() -> (Fixture, NodeId, NodeId) {
    use eidetic_core::timeline::node::StoryLevel;
    let mut fixture = fixture().await;
    let (project, _) = active_sqlite_project(&fixture.state).await.unwrap();
    let scene = project.timeline.nodes_at_level(StoryLevel::Scene)[0];
    fixture.node_id = scene.id;
    let ancestor = scene.parent_id.unwrap();
    let unrelated = project
        .timeline
        .nodes_at_level(StoryLevel::Act)
        .into_iter()
        .find(|node| node.id != ancestor)
        .unwrap()
        .id;
    set_fixture_notes(&fixture, ancestor, CONSUMED_ANCESTOR_NOTES).await;
    set_fixture_notes(&fixture, unrelated, UNCONSUMED_ACT_NOTES).await;
    let (project, _) = active_sqlite_project(&fixture.state).await.unwrap();
    let mut request =
        eidetic_core::ai::prompt::build_generate_request(&project, fixture.node_id).unwrap();
    crate::ai_service::attach_ai_generation_context(
        &mut request,
        fixture.path.clone(),
        fixture.node_id,
    )
    .await
    .unwrap();
    assert!(
        request
            .ancestor_chain
            .iter()
            .any(|node| { node.id == ancestor && node.content.notes == CONSUMED_ANCESTOR_NOTES })
    );
    let prompt = crate::prompt_format::build_chat_prompt(&request).user;
    assert!(prompt.contains(CONSUMED_ANCESTOR_NOTES));
    assert!(!prompt.contains(UNCONSUMED_ACT_NOTES));
    // Explicitly synthetic output; use the existing canonical generation writer.
    // This qualifies prompt consumption and lineage, not model narrative quality.
    persist_generated_script_block(
        fixture.path.clone(),
        fixture.node_id.0,
        "Synthetic screenplay: Mara conceals the witness.".into(),
        GenerationInputs {
            arc_description_applicability: request.arc_description_applicability,
            ancestor_notes_inputs: request.ancestor_notes_inputs,
            arc_inputs: request.arc_inputs,
            script_inputs: request.script_context,
            bible_inputs: request.bible_inputs,
            bible_node_name_inputs: request.bible_node_name_inputs,
            bible_relationship_inputs: request.bible_relationship_inputs,
            bible_context_scope: request.bible_context_scope,
            script_context_scope: request.script_context_scope,
            target_binding: request.generation_target,
            ..GenerationInputs::default()
        },
    )
    .await
    .unwrap();
    (fixture, ancestor, unrelated)
}

async fn set_fixture_notes(fixture: &Fixture, node_id: NodeId, notes: &str) {
    use eidetic_core::contracts::{CommandEnvelope, SetTimelineNodeNotesCommand};
    crate::command_service::set_timeline_node_notes(
        &fixture.state,
        CommandEnvelope::new(SetTimelineNodeNotesCommand {
            node_id,
            notes: notes.into(),
        }),
    )
    .await
    .unwrap();
}

fn assert_saved_material_unchanged(
    before: &eidetic_core::contracts::ScriptDocumentProjection,
    after: &eidetic_core::contracts::ScriptDocumentProjection,
) {
    assert_eq!(before.document, after.document);
    assert_eq!(before.segments.len(), after.segments.len());
    for (before, after) in before.segments.iter().zip(&after.segments) {
        assert_eq!(before.segment, after.segment);
        assert_eq!(before.blocks, after.blocks);
    }
}

#[tokio::test]
async fn public_consumed_ancestor_notes_edit_marks_saved_scene_for_review() {
    use eidetic_core::contracts::SemanticDependencyEndpoint;
    let (fixture, ancestor, _) = ancestor_notes_fixture().await;
    let before = script(&fixture);
    assert!(
        before
            .payload
            .segments
            .iter()
            .all(|segment| { !segment.impact.as_ref().unwrap().needs_review })
    );
    set_fixture_notes(&fixture, ancestor, "  Mara reveals the witness — 雨.\n\n  ").await;
    let after = script(&fixture);
    assert_saved_material_unchanged(&before.payload, &after.payload);
    let segment = after
        .payload
        .segments
        .iter()
        .find(|segment| {
            segment.segment.source_node_id.as_deref()
                == Some(fixture.node_id.0.to_string().as_str())
        })
        .unwrap();
    let impact = segment.impact.as_ref().unwrap().clone();
    fixture.state.shutdown_tasks_async().await;
    assert!(
        impact.needs_review,
        "Exact ancestor Notes supplied to generation changed without downstream scene review"
    );
    assert!(impact.causes.iter().any(|cause| {
        cause.input == SemanticDependencyEndpoint::TimelineNode { node_id: ancestor }
            && cause.input_excerpt.as_deref() == Some(CONSUMED_ANCESTOR_NOTES)
    }));
}

#[tokio::test]
async fn public_unconsumed_act_notes_edit_preserves_saved_scene_review_state() {
    let (fixture, _, unrelated) = ancestor_notes_fixture().await;
    let before = script(&fixture);
    set_fixture_notes(
        &fixture,
        unrelated,
        "Eli discards the unrelated brass whistle.",
    )
    .await;
    let after = script(&fixture);
    assert_saved_material_unchanged(&before.payload, &after.payload);
    assert_eq!(
        before
            .payload
            .segments
            .iter()
            .map(|segment| &segment.impact)
            .collect::<Vec<_>>(),
        after
            .payload
            .segments
            .iter()
            .map(|segment| &segment.impact)
            .collect::<Vec<_>>()
    );
    fixture.state.shutdown_tasks_async().await;
}

#[tokio::test]
async fn public_consumed_bible_name_edit_marks_saved_screenplay_for_review() {
    use eidetic_core::contracts::*;
    let fixture = fixture().await;
    crate::command_service::create_bible_graph_node(
        &fixture.state,
        serde_json::from_value(serde_json::json!({
            "id": Uuid::new_v4(), "payload": {
                "node_id": "Mara", "schema_key": "character", "name": "Mara", "sort_order": 0
            }
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    let project = fixture.state.project.lock().as_ref().unwrap().clone();
    let mut request =
        eidetic_core::ai::prompt::build_generate_request(&project, fixture.node_id).unwrap();
    crate::ai_service::attach_ai_generation_context(
        &mut request,
        fixture.path.clone(),
        fixture.node_id,
    )
    .await
    .unwrap();
    assert!(
        crate::prompt_format::build_chat_prompt(&request)
            .user
            .contains("- Mara [character] (Mara)")
    );
    persist_generated_script_block(
        fixture.path.clone(),
        fixture.node_id.0,
        "Synthetic screenplay starring Mara.".into(),
        GenerationInputs {
            arc_description_applicability: request.arc_description_applicability,
            ancestor_notes_inputs: request.ancestor_notes_inputs,
            arc_inputs: request.arc_inputs,
            script_inputs: request.script_context,
            bible_inputs: request.bible_inputs,
            bible_node_name_inputs: request.bible_node_name_inputs,
            bible_relationship_inputs: request.bible_relationship_inputs,
            bible_context_scope: request.bible_context_scope,
            script_context_scope: request.script_context_scope,
            target_binding: request.generation_target,
            ..GenerationInputs::default()
        },
    )
    .await
    .unwrap();
    let before = script(&fixture);
    crate::command_service::set_bible_graph_node_name(
        &fixture.state,
        CommandEnvelope::new(SetBibleGraphNodeNameCommand {
            node_id: BibleGraphNodeId::new("Mara").unwrap(),
            name: "Marisol".into(),
        }),
    )
    .await
    .unwrap();
    let after = script(&fixture);
    assert_eq!(
        after.payload.segments[0].blocks,
        before.payload.segments[0].blocks
    );
    let needs_review = after.payload.segments[0]
        .impact
        .as_ref()
        .unwrap()
        .needs_review;
    fixture.state.shutdown_tasks_async().await;
    std::fs::remove_file(&fixture.path).unwrap();
    assert!(
        needs_review,
        "A name actually supplied to generation changed without downstream review"
    );
}

#[tokio::test]
async fn public_relationship_edit_publishes_affected_review_from_original_generation_read() {
    use eidetic_core::contracts::*;
    let fixture = fixture().await;
    for id in ["Mara", "Eli"] {
        crate::command_service::create_bible_graph_node(
            &fixture.state,
            serde_json::from_value(
                serde_json::to_value(CommandEnvelope::new(CreateBibleGraphNodeCommand {
                    node_id: BibleGraphNodeId::new(id).unwrap(),
                    parent_id: None,
                    schema_key: BibleGraphSchemaKey::new("character").unwrap(),
                    name: id.into(),
                    sort_order: 0,
                }))
                .unwrap(),
            )
            .unwrap(),
        )
        .await
        .unwrap();
    }
    let mut edge = SetBibleGraphEdgeCommand {
        edge_id: BibleGraphEdgeId::new("Mara.Eli").unwrap(),
        from_node_id: BibleGraphNodeId::new("Mara").unwrap(),
        to_node_id: BibleGraphNodeId::new("Eli").unwrap(),
        edge_kind: BibleGraphEdgeKind::References,
        label: "Mara trusts Eli".into(),
        directed: true,
        sort_order: 0,
    };
    crate::command_service::set_bible_graph_edge(
        &fixture.state,
        serde_json::from_value(serde_json::to_value(CommandEnvelope::new(edge.clone())).unwrap())
            .unwrap(),
    )
    .await
    .unwrap();
    let project = fixture.state.project.lock().as_ref().unwrap().clone();
    let mut request =
        eidetic_core::ai::prompt::build_generate_request(&project, fixture.node_id).unwrap();
    crate::ai_service::attach_ai_generation_context(
        &mut request,
        fixture.path.clone(),
        fixture.node_id,
    )
    .await
    .unwrap();
    let inputs = request.bible_relationship_inputs.as_ref().unwrap();
    assert_eq!(inputs.len(), 1);
    let original = inputs[0].clone();
    assert!(
        crate::prompt_format::build_chat_prompt(&request)
            .user
            .contains("Mara trusts Eli")
    );
    // Predefined synthetic output exercises the real runtime and command service.
    finish_generation_stream(
        fixture.state.clone(),
        fixture.path.clone(),
        fixture.node_id.0,
        Box::pin(stream::iter([Ok(
            "Synthetic screenplay: Mara trusts Eli.".into()
        )])),
        GenerationInputs {
            arc_description_applicability: request.arc_description_applicability,
            ancestor_notes_inputs: request.ancestor_notes_inputs,
            arc_inputs: request.arc_inputs,
            script_inputs: request.script_context,
            bible_inputs: request.bible_inputs,
            bible_node_name_inputs: request.bible_node_name_inputs,
            bible_relationship_inputs: request.bible_relationship_inputs,
            bible_context_scope: request.bible_context_scope,
            script_context_scope: request.script_context_scope,
            target_binding: request.generation_target,
            ..GenerationInputs::default()
        },
    )
    .await;
    let before = script(&fixture);
    assert!(
        !before.payload.segments[0]
            .impact
            .as_ref()
            .unwrap()
            .needs_review
    );
    let mut events = fixture.state.events_tx.subscribe();
    edge.label = "Mara doubts Eli".into();
    crate::command_service::set_bible_graph_edge(
        &fixture.state,
        serde_json::from_value(serde_json::to_value(CommandEnvelope::new(edge)).unwrap()).unwrap(),
    )
    .await
    .unwrap();
    assert!(matches!(
        events.recv().await.unwrap(),
        ServerEvent::BibleChanged
    ));
    let after = script(&fixture);
    assert!(after.version.0 > before.version.0);
    assert_eq!(
        after.payload.segments[0].blocks,
        before.payload.segments[0].blocks
    );
    let impact = after.payload.segments[0].impact.as_ref().unwrap();
    assert!(impact.needs_review);
    assert_eq!(
        impact.causes[0].input,
        crate::bible_relationship_lineage::endpoint(&original)
    );
    assert_eq!(
        impact.causes[0].consumed_revision_event_id,
        original.revision_event_id
    );
    assert_eq!(
        impact.causes[0].input_excerpt.as_deref(),
        Some("Mara trusts Eli")
    );
}

struct Fixture {
    state: AppState,
    path: PathBuf,
    node_id: NodeId,
}

#[tokio::test]
async fn independent_real_service_bible_fact_capture_output_and_manual_change_publish_review() {
    use eidetic_core::contracts::*;
    let fixture = fixture().await;
    let create = serde_json::from_value(serde_json::json!({
        "id": Uuid::new_v4(), "payload": {
            "node_id": "Mara", "schema_key": "character", "name": "Mara", "sort_order": 0
        }
    }))
    .unwrap();
    crate::command_service::create_bible_graph_node(&fixture.state, create)
        .await
        .unwrap();
    let mut field = SetBibleGraphFieldCommand {
        node_id: BibleGraphNodeId::new("Mara").unwrap(),
        part_id: BibleGraphPartId::new("Mara.profile").unwrap(),
        part_key: BibleGraphPartKey::new("profile").unwrap(),
        part_name: "Profile".into(),
        part_sort_order: 0,
        field_id: BibleGraphFieldId::new("Mara.tagline").unwrap(),
        field_key: BibleGraphFieldKey::new("tagline").unwrap(),
        value: Some(FieldValue::Text("Mara carries red.".into())),
        field_sort_order: 0,
    };
    crate::command_service::set_bible_graph_field(
        &fixture.state,
        CommandEnvelope::new(field.clone()),
    )
    .await
    .unwrap();
    let project = fixture.state.project.lock().as_ref().unwrap().clone();
    let mut request =
        eidetic_core::ai::prompt::build_generate_request(&project, fixture.node_id).unwrap();
    crate::ai_service::attach_ai_generation_context(
        &mut request,
        fixture.path.clone(),
        fixture.node_id,
    )
    .await
    .unwrap();
    assert_eq!(request.bible_inputs.as_ref().unwrap().len(), 1);
    assert!(request.bible_context_scope.is_some());
    let captured = request.bible_inputs.as_ref().unwrap()[0].clone();
    assert!(
        crate::prompt_format::build_chat_prompt(&request)
            .user
            .contains("Mara carries red.")
    );
    // Predefined stream exercises canonical runtime persistence, not model quality.
    finish_generation_stream(
        fixture.state.clone(),
        fixture.path.clone(),
        fixture.node_id.0,
        Box::pin(stream::iter([Ok(
            "Synthetic fixture screenplay: Mara carries red.".into(),
        )])),
        GenerationInputs {
            arc_description_applicability: request.arc_description_applicability,
            ancestor_notes_inputs: request.ancestor_notes_inputs,
            arc_inputs: request.arc_inputs,
            script_inputs: request.script_context,
            bible_node_name_inputs: request.bible_node_name_inputs,
            bible_relationship_inputs: request.bible_relationship_inputs,
            bible_inputs: request.bible_inputs,
            bible_context_scope: request.bible_context_scope,
            script_context_scope: request.script_context_scope,
            target_binding: request.generation_target,
            ..GenerationInputs::default()
        },
    )
    .await;
    assert!(
        !script(&fixture).payload.segments[0]
            .impact
            .as_ref()
            .unwrap()
            .needs_review
    );
    let before_projection = script(&fixture);
    let before = before_projection.payload.segments[0].blocks[0].clone();
    let mut events = fixture.state.events_tx.subscribe();
    field.value = Some(FieldValue::Text("Mara carries blue.".into()));
    crate::command_service::set_bible_graph_field(&fixture.state, CommandEnvelope::new(field))
        .await
        .unwrap();
    assert!(matches!(
        events.recv().await.unwrap(),
        ServerEvent::BibleChanged
    ));
    let after = script(&fixture);
    assert!(after.version.0 > before_projection.version.0);
    assert_eq!(after.payload.segments[0].blocks[0], before);
    let impact = after.payload.segments[0].impact.as_ref().unwrap();
    assert!(impact.needs_review);
    assert_eq!(
        impact.causes[0].input,
        crate::bible_field_lineage::endpoint(&captured)
    );
    assert_eq!(
        impact.causes[0].consumed_revision_event_id,
        captured.revision_event_id
    );
    assert_eq!(
        impact.causes[0].input_excerpt.as_deref(),
        Some("Mara carries red.")
    );
    fixture.state.shutdown_tasks_async().await;
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.state.shutdown_tasks();
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", self.path.display()));
        }
    }
}

async fn fixture() -> Fixture {
    let state = AppState::new().await;
    let mut project = Template::MultiCam.build_project("Stream failure");
    let node_id = project.timeline.nodes[0].id;
    project.timeline.node_mut(node_id).unwrap().content.status = ContentStatus::HasContent;
    let path =
        std::env::temp_dir().join(format!("eidetic-generation-stream-{}.db", Uuid::new_v4()));
    crate::persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    persist_generated_script_block(
        path.clone(),
        node_id.0,
        "Previously approved screenplay".to_string(),
        GenerationInputs::default(),
    )
    .await
    .unwrap();
    replace_active_project(&state, project, path.clone());
    Fixture {
        state,
        path,
        node_id,
    }
}

fn script(
    fixture: &Fixture,
) -> eidetic_core::contracts::ProjectionEnvelope<eidetic_core::contracts::ScriptDocumentProjection>
{
    let conn = crate::sqlite::open_write_connection(&fixture.path).unwrap();
    crate::script_store::load_document_projection_envelope(
        &conn,
        &ScriptDocumentId::new("script.document.main").unwrap(),
    )
    .unwrap()
    .unwrap()
}

async fn assert_failed_stream(
    items: Vec<Result<String, Error>>,
    expected_progress: Vec<(String, usize)>,
    expected_error: &str,
) {
    let fixture = fixture().await;
    let before = script(&fixture);
    let conn = crate::sqlite::open_write_connection(&fixture.path).unwrap();
    let inputs =
        crate::ai_script_context::load_script_context(&conn, fixture.node_id, 0, u64::MAX / 2)
            .unwrap();
    assert!(!inputs.is_empty());
    let lineage_before = lineage_counts(&conn);
    drop(conn);
    fixture.state.generating.lock().insert(fixture.node_id.0);
    mark_node_generating(
        &fixture.state,
        fixture.path.clone(),
        fixture.node_id,
        fixture.node_id.0,
    )
    .await;
    let mut events = fixture.state.events_tx.subscribe();
    finish_generation_stream(
        fixture.state.clone(),
        fixture.path.clone(),
        fixture.node_id.0,
        Box::pin(stream::iter(items)),
        GenerationInputs {
            script_inputs: Some(inputs),
            ..GenerationInputs::default()
        },
    )
    .await;

    assert_eq!(
        script(&fixture),
        before,
        "prior script and script revision history must remain unchanged"
    );
    let conn = crate::sqlite::open_write_connection(&fixture.path).unwrap();
    assert_eq!(lineage_counts(&conn), lineage_before);
    drop(conn);
    assert!(!fixture.state.generating.lock().contains(&fixture.node_id.0));
    assert_ne!(
        fixture
            .state
            .project
            .lock()
            .as_ref()
            .unwrap()
            .timeline
            .node(fixture.node_id)
            .unwrap()
            .content
            .status,
        ContentStatus::Generating
    );
    let (persisted, _) = crate::persistence::load_project(&fixture.path)
        .await
        .unwrap();
    assert_ne!(
        persisted
            .timeline
            .node(fixture.node_id)
            .unwrap()
            .content
            .status,
        ContentStatus::Generating
    );
    let mut progress = Vec::new();
    let mut errors = Vec::new();
    while let Ok(event) = events.try_recv() {
        match event {
            ServerEvent::GenerationProgress {
                node_id,
                token,
                tokens_generated,
            } => {
                assert_eq!(node_id, fixture.node_id.0);
                progress.push((token, tokens_generated));
            }
            ServerEvent::GenerationError { node_id, error } => {
                assert_eq!(node_id, fixture.node_id.0);
                errors.push(error);
            }
            ServerEvent::GenerationComplete { .. } | ServerEvent::ScriptChanged => {
                panic!("failed or empty stream cannot publish success: {event:?}");
            }
            _ => {}
        }
    }
    assert_eq!(progress, expected_progress);
    assert_eq!(errors, vec![expected_error]);
}

fn lineage_counts(conn: &rusqlite::Connection) -> (i64, i64, i64) {
    conn.query_row(
        "SELECT (SELECT COUNT(*) FROM script_generations),
                (SELECT COUNT(*) FROM semantic_dependencies),
                (SELECT COUNT(*) FROM semantic_dependency_revisions)",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
    .unwrap()
}

#[tokio::test]
async fn partial_stream_error_preserves_prior_script_and_releases_generation() {
    assert_failed_stream(
        vec![
            Ok("unfinished prefix".into()),
            Err(Error::AiBackend("broken stream".into())),
        ],
        vec![("unfinished prefix".to_string(), 1)],
        "AI backend error: broken stream",
    )
    .await;
}

#[tokio::test]
async fn immediate_stream_error_preserves_prior_script_and_releases_generation() {
    assert_failed_stream(
        vec![Err(Error::AiBackend("unavailable".into()))],
        vec![],
        "AI backend error: unavailable",
    )
    .await;
}

#[tokio::test]
async fn empty_eof_remains_no_output_and_releases_generation() {
    assert_failed_stream(vec![], vec![], "AI produced no output").await;
}

#[tokio::test]
async fn successful_persistence_keeps_captured_input_lineage_after_an_intervening_edit() {
    let fixture = fixture().await;
    let source_node = Uuid::new_v4();
    let source_command = generated_script_block_command(
        Uuid::new_v4(),
        source_node,
        GeneratedScriptMetadata {
            project_name: "Source evidence".into(),
            start_ms: 0,
            end_ms: 1000,
        },
        "  Consumed A — 雨\n\n".into(),
    )
    .unwrap();
    let mut conn = crate::sqlite::open_write_connection(&fixture.path).unwrap();
    script_document_command::apply_set_script_block(&mut conn, &source_command, 10).unwrap();
    let captured =
        crate::ai_script_context::load_script_context(&conn, NodeId(source_node), 0, 1000)
            .unwrap()
            .into_iter()
            .find(|input| input.block_id == source_command.payload.block_id)
            .unwrap();
    crate::script_block_edit::apply_edit_script_block(
        &mut conn,
        &CommandEnvelope::new(eidetic_core::contracts::EditScriptBlockCommand {
            document_id: captured.document_id.clone(),
            block_id: captured.block_id.clone(),
            expected_revision_event_id: captured.revision_event_id,
            text: "A changed while generation ran".into(),
        }),
        20,
    )
    .unwrap();
    drop(conn);
    persist_generated_script_block(
        fixture.path.clone(),
        fixture.node_id.0,
        "  B from captured A\n\n".into(),
        GenerationInputs {
            script_inputs: Some(vec![captured.clone()]),
            ..GenerationInputs::default()
        },
    )
    .await
    .unwrap();
    let document = script(&fixture);
    let output = document
        .payload
        .segments
        .iter()
        .find(|segment| {
            segment.segment.source_node_id.as_deref()
                == Some(fixture.node_id.0.to_string().as_str())
        })
        .unwrap();
    assert_eq!(output.blocks[0].block.text, "  B from captured A\n\n");
    let impact = output.impact.as_ref().unwrap();
    assert!(impact.needs_review);
    assert_eq!(impact.causes.len(), 1);
    assert_eq!(
        impact.causes[0].consumed_revision_event_id,
        captured.revision_event_id
    );
    assert_eq!(
        impact.causes[0].input_excerpt.as_deref(),
        Some(captured.text.as_str())
    );
    let conn = crate::sqlite::open_write_connection(&fixture.path).unwrap();
    assert_eq!(lineage_counts(&conn), (2, 2, 2));
}

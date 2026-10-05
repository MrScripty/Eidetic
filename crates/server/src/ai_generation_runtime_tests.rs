use super::*;
use crate::project_service::replace_active_project;
use eidetic_core::{Error, Template};
use futures::stream;

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
        request.script_context,
        request.bible_inputs,
    )
    .await;
    assert!(
        !script(&fixture).payload.segments[0]
            .impact
            .as_ref()
            .unwrap()
            .needs_review
    );
    let before = script(&fixture).payload.segments[0].blocks[0].clone();
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
    let node = project.timeline.node(node_id).unwrap();
    persist_generated_script_block(
        path.clone(),
        node_id.0,
        GeneratedScriptMetadata {
            project_name: project.name.clone(),
            start_ms: node.time_range.start_ms,
            end_ms: node.time_range.end_ms,
        },
        "Previously approved screenplay".to_string(),
        None,
        None,
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
        Some(inputs),
        None,
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
        GeneratedScriptMetadata {
            project_name: "Stream failure".into(),
            start_ms: 1000,
            end_ms: 2000,
        },
        "  B from captured A\n\n".into(),
        Some(vec![captured.clone()]),
        None,
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

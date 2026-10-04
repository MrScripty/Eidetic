use super::*;
use crate::project_service::replace_active_project;
use eidetic_core::{Error, Template};
use futures::stream;

struct Fixture {
    state: AppState,
    path: PathBuf,
    node_id: NodeId,
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
    )
    .await;

    assert_eq!(
        script(&fixture),
        before,
        "prior script and script revision history must remain unchanged"
    );
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

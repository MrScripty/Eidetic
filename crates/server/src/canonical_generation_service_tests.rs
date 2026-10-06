//! Real AppState/public command/HTTP boundary tests; responses are synthetic.
use crate::ai_generation_service::{AiGenerateRequest, start_generation};
use crate::state::{AiConfig, AppState, ServerEvent};
use crate::{command_service, persistence, projection_service, sqlite};
use eidetic_core::{
    Template,
    contracts::*,
    timeline::node::{NodeId, StoryLevel},
};
use std::{
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    thread,
    time::Duration,
};
use uuid::Uuid;

const NOTES: &str = "Exact newly created scene notes — 雨";
const A: &str = "Exact preceding authored scene A.\n\n";
const B: &str = "Exact target human anchor B.\n\n";
const OUTPUT: &str = "Synthetic immediate canonical scene B.\n\n";

struct Fixture {
    state: AppState,
    path: PathBuf,
    node: NodeId,
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
    let project = Template::MultiCam.build_project("Immediate canonical generation");
    let act = project
        .timeline
        .nodes
        .iter()
        .find(|node| node.level == StoryLevel::Act)
        .unwrap()
        .clone();
    let path = std::env::temp_dir().join(format!("eidetic-new-scene-{}.db", Uuid::new_v4()));
    persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    crate::project_service::replace_active_project(&state, project, path.clone());
    let sequence = NodeId(Uuid::new_v4());
    command_service::create_timeline_node_from_core_command(
        &state,
        CommandEnvelope::new(CreateTimelineNodeCommand {
            node_id: sequence,
            parent_id: Some(act.id),
            level: StoryLevel::Sequence,
            name: "New canonical sequence".into(),
            start_ms: act.time_range.start_ms,
            end_ms: act.time_range.end_ms,
            beat_type: None,
        }),
    )
    .await
    .unwrap();
    let a = NodeId(Uuid::new_v4());
    let b = NodeId(Uuid::new_v4());
    for (id, name, offset, text) in [
        (a, "New canonical A", 1000, A),
        (b, "New canonical B", 3000, B),
    ] {
        let start = act.time_range.start_ms + offset;
        command_service::create_timeline_node_from_core_command(
            &state,
            CommandEnvelope::new(CreateTimelineNodeCommand {
                node_id: id,
                parent_id: Some(sequence),
                level: StoryLevel::Scene,
                name: name.into(),
                start_ms: start,
                end_ms: start + 1000,
                beat_type: None,
            }),
        )
        .await
        .unwrap();
        command_service::create_script_block(
            &state,
            CommandEnvelope::new(CreateScriptBlockCommand {
                document_id: ScriptDocumentId::new("script.document.main").unwrap(),
                source_node_id: id,
                expected_start_ms: start,
                expected_end_ms: start + 1000,
                block_kind: ScriptBlockKind::Action,
                text: text.into(),
            }),
        )
        .await
        .unwrap();
    }
    command_service::set_timeline_node_notes(
        &state,
        CommandEnvelope::new(SetTimelineNodeNotesCommand {
            node_id: b,
            notes: NOTES.into(),
        }),
    )
    .await
    .unwrap();
    assert!(
        state
            .project
            .lock()
            .as_ref()
            .unwrap()
            .timeline
            .node(b)
            .is_err(),
        "the actual mirror is stale; no reopen or synchronization workaround"
    );
    Fixture {
        state,
        path,
        node: b,
    }
}

// Pause after reading the actual production request, before any response bytes.
// The native test can mutate canonical state while external I/O is pending.
fn delayed_provider() -> (
    String,
    tokio::sync::oneshot::Receiver<serde_json::Value>,
    std::sync::mpsc::Sender<()>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/v1", listener.local_addr().unwrap());
    let (observed_tx, observed_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let worker = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0; 4096];
        let body = loop {
            let count = socket.read(&mut buffer).unwrap();
            assert_ne!(count, 0);
            request.extend_from_slice(&buffer[..count]);
            if let Some(end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                let header = String::from_utf8_lossy(&request[..end]).to_lowercase();
                assert!(header.starts_with("post /v1/chat/completions http/1.1"));
                let length: usize = header
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length: "))
                    .unwrap()
                    .parse()
                    .unwrap();
                if request.len() >= end + 4 + length {
                    break serde_json::from_slice(&request[end + 4..end + 4 + length]).unwrap();
                }
            }
        };
        observed_tx.send(body).unwrap();
        release_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let frame = format!(
            "data: {}\n\ndata: [DONE]\n\n",
            serde_json::json!({"choices":[{"delta":{"content":OUTPUT}}]})
        );
        write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",frame.len(),frame).unwrap();
    });
    (url, observed_rx, release_tx, worker)
}

async fn start(
    f: &Fixture,
) -> (
    std::sync::mpsc::Sender<()>,
    thread::JoinHandle<()>,
    tokio::sync::broadcast::Receiver<ServerEvent>,
) {
    let (url, observed, release, worker) = delayed_provider();
    *f.state.ai_config.lock() = AiConfig {
        base_url: url,
        model: "synthetic-fixture-model".into(),
        ..AiConfig::default()
    };
    let events = f.state.events_tx.subscribe();
    start_generation(
        &f.state,
        AiGenerateRequest {
            node_id: f.node.0,
            story_time_ms: None,
        },
    )
    .await
    .unwrap();
    let request = tokio::time::timeout(Duration::from_secs(10), observed)
        .await
        .unwrap()
        .unwrap();
    let user = request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|message| message["role"] == "user")
        .unwrap()["content"]
        .as_str()
        .unwrap();
    assert!(
        user.contains(NOTES) && user.contains(A) && user.contains(B),
        "actual request must use canonical selected notes and exact saved anchors"
    );
    assert_eq!(request["stream"], true);
    (release, worker, events)
}

async fn terminal(
    f: &Fixture,
    events: &mut tokio::sync::broadcast::Receiver<ServerEvent>,
) -> Result<(), String> {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match events.recv().await.unwrap() {
                ServerEvent::GenerationComplete { node_id } if node_id == f.node.0 => return Ok(()),
                ServerEvent::GenerationError { node_id, error } if node_id == f.node.0 => {
                    return Err(error);
                }
                _ => {}
            }
        }
    })
    .await
    .unwrap()
}

fn document(f: &Fixture) -> ScriptDocumentProjection {
    crate::script_store::load_document_projection(
        &sqlite::open_write_connection(&f.path).unwrap(),
        &ScriptDocumentId::new("script.document.main").unwrap(),
    )
    .unwrap()
    .unwrap()
}

fn history_count(f: &Fixture) -> i64 {
    sqlite::open_write_connection(&f.path)
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM commands WHERE payload_type='script.generate_block'",
            [],
            |row| row.get(0),
        )
        .unwrap()
}

#[tokio::test]
async fn paused_public_http_generation_retains_original_bible_membership_and_publishes_added_field_review()
 {
    let f = fixture().await;
    command_service::create_bible_graph_node(&f.state,serde_json::from_value(serde_json::json!({
        "id":Uuid::new_v4(),"payload":{"node_id":"Mara","schema_key":"character","name":"Mara","sort_order":0}
    })).unwrap()).await.unwrap();
    let field = |key: &str, value: Option<&str>| SetBibleGraphFieldCommand {
        node_id: BibleGraphNodeId::new("Mara").unwrap(),
        part_id: BibleGraphPartId::new("Mara.profile").unwrap(),
        part_key: BibleGraphPartKey::new("profile").unwrap(),
        part_name: "Profile".into(),
        part_sort_order: 0,
        field_id: BibleGraphFieldId::new(format!("Mara.{key}")).unwrap(),
        field_key: BibleGraphFieldKey::new(key).unwrap(),
        value: value.map(|value| FieldValue::Text(value.into())),
        field_sort_order: 0,
    };
    command_service::set_bible_graph_field(
        &f.state,
        CommandEnvelope::new(field("tagline", Some("Blue umbrella"))),
    )
    .await
    .unwrap();
    command_service::set_bible_graph_field(
        &f.state,
        CommandEnvelope::new(field("motivation", None)),
    )
    .await
    .unwrap();
    let before = document(&f);
    let (release, worker, mut events) = start(&f).await;
    command_service::set_bible_graph_field(
        &f.state,
        CommandEnvelope::new(field("motivation", Some("Keep the station key"))),
    )
    .await
    .unwrap();
    release.send(()).unwrap();
    terminal(&f, &mut events).await.unwrap();
    worker.join().unwrap();
    let after = document(&f);
    for old in before.segments.iter().flat_map(|segment| &segment.blocks) {
        assert!(
            after
                .segments
                .iter()
                .flat_map(|segment| &segment.blocks)
                .any(|block| block == old)
        );
    }
    let impact = after
        .segments
        .iter()
        .find(|segment| {
            segment.segment.source_node_id.as_deref() == Some(f.node.0.to_string().as_str())
        })
        .unwrap()
        .impact
        .as_ref()
        .unwrap();
    assert!(impact.needs_review);
    assert!(impact.causes.iter().any(|cause| {
        cause.reason == ScriptImpactReason::ContextChanged
            && cause
                .input_excerpt
                .as_deref()
                .is_some_and(|text| text.contains("Mara.profile.motivation"))
    }));
    let conn = sqlite::open_write_connection(&f.path).unwrap();
    let json:String=conn.query_row("SELECT payload_json FROM commands WHERE payload_type='script.generate_block' ORDER BY rowid DESC LIMIT 1",[],|row|row.get(0)).unwrap();
    let command: GenerateScriptBlockCommand = serde_json::from_str(&json).unwrap();
    let scope = command.bible_context_scope.unwrap();
    assert_eq!(scope.node_ids, vec![BibleGraphNodeId::new("Mara").unwrap()]);
    assert_eq!(
        scope.field_ids,
        vec![BibleGraphFieldId::new("Mara.tagline").unwrap()]
    );
    assert_eq!(command.bible_inputs.unwrap().len(), 1);
}

#[tokio::test]
async fn create_select_and_generate_without_reopening_reads_canonical_metadata_and_retains_anchors()
{
    let f = fixture().await;
    let selected = projection_service::selected_node_editor_projection(
        &f.state,
        projection_service::SelectedNodeEditorProjectionRequest {
            node_id: Some(f.node),
        },
    )
    .await
    .unwrap();
    assert_eq!(selected.payload.node.unwrap().notes, NOTES);
    let before = document(&f);
    let (release, worker, mut events) = start(&f).await;
    release.send(()).unwrap();
    terminal(&f, &mut events).await.unwrap();
    worker.join().unwrap();
    let after = document(&f);
    for block in before.segments.iter().flat_map(|segment| &segment.blocks) {
        assert_eq!(
            after
                .segments
                .iter()
                .flat_map(|segment| &segment.blocks)
                .find(|saved| saved.block.id == block.block.id)
                .unwrap(),
            block
        );
    }
    let target = after
        .segments
        .iter()
        .find(|segment| {
            segment.segment.source_node_id.as_deref() == Some(f.node.0.to_string().as_str())
        })
        .unwrap();
    assert!(target.blocks.iter().any(|block| block.block.text == OUTPUT));
    let canonical = crate::ai_service::active_sqlite_project(&f.state)
        .await
        .unwrap()
        .0;
    assert_eq!(
        (target.segment.start_ms, target.segment.end_ms),
        (
            canonical.timeline.node(f.node).unwrap().time_range.start_ms,
            canonical.timeline.node(f.node).unwrap().time_range.end_ms
        )
    );
    assert_eq!(
        canonical.timeline.node(f.node).unwrap().content.status,
        eidetic_core::timeline::node::ContentStatus::HasContent
    );
    assert!(
        f.state
            .project
            .lock()
            .as_ref()
            .unwrap()
            .timeline
            .node(f.node)
            .is_err(),
        "completion did not replace canonical ownership with mirror refresh"
    );
    f.state.shutdown_tasks_async().await;
}

#[tokio::test]
async fn delayed_http_after_new_scene_notes_aba_preserves_canonical_manual_script_and_history() {
    let f = fixture().await;
    let (release, worker, mut events) = start(&f).await;
    for notes in ["Newer exact manual scene notes", NOTES] {
        command_service::set_timeline_node_notes(
            &f.state,
            CommandEnvelope::new(SetTimelineNodeNotesCommand {
                node_id: f.node,
                notes: notes.into(),
            }),
        )
        .await
        .unwrap();
    }
    let before = document(&f);
    let count = history_count(&f);
    release.send(()).unwrap();
    assert!(
        terminal(&f, &mut events)
            .await
            .unwrap_err()
            .contains("generation target changed")
    );
    worker.join().unwrap();
    assert_eq!(document(&f), before);
    assert_eq!(history_count(&f), count);
    assert_eq!(
        crate::ai_service::active_sqlite_project(&f.state)
            .await
            .unwrap()
            .0
            .timeline
            .node(f.node)
            .unwrap()
            .content
            .notes,
        NOTES
    );
    f.state.shutdown_tasks_async().await;
}

#[tokio::test]
async fn delayed_http_after_target_retime_lock_or_delete_never_writes_a_scene_from_stale_selection()
{
    for operation in ["retime", "lock", "delete"] {
        let f = fixture().await;
        let (release, worker, mut events) = start(&f).await;
        match operation {
            "retime" => {
                let current = crate::ai_service::active_sqlite_project(&f.state)
                    .await
                    .unwrap()
                    .0;
                let node = current.timeline.node(f.node).unwrap();
                command_service::set_timeline_node_range(
                    &f.state,
                    CommandEnvelope::new(SetTimelineNodeRangeCommand {
                        node_id: f.node,
                        start_ms: node.time_range.start_ms + 200,
                        end_ms: node.time_range.end_ms + 200,
                    }),
                )
                .await
                .unwrap();
            }
            "lock" => {
                command_service::set_timeline_node_lock(
                    &f.state,
                    CommandEnvelope::new(SetTimelineNodeLockCommand {
                        node_id: f.node,
                        locked: true,
                    }),
                )
                .await
                .unwrap();
            }
            _ => {
                command_service::delete_timeline_node(
                    &f.state,
                    CommandEnvelope::new(DeleteTimelineNodeCommand { node_id: f.node }),
                )
                .await
                .unwrap();
            }
        }
        let before = document(&f);
        let count = history_count(&f);
        release.send(()).unwrap();
        assert!(terminal(&f, &mut events).await.is_err());
        worker.join().unwrap();
        assert_eq!(document(&f), before);
        assert_eq!(history_count(&f), count);
        f.state.shutdown_tasks_async().await;
    }
}

#[tokio::test]
async fn delayed_regeneration_refuses_a_saved_human_edit_and_canonical_preview_still_reads_it() {
    let f = fixture().await;
    let (release, worker, mut events) = start(&f).await;
    release.send(()).unwrap();
    terminal(&f, &mut events).await.unwrap();
    worker.join().unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        while f.state.generating.lock().contains(&f.node.0) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let (release, worker, mut events) = start(&f).await;
    let doc = document(&f);
    let block = doc
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|block| block.block.text == OUTPUT)
        .unwrap();
    command_service::edit_script_block(
        &f.state,
        CommandEnvelope::new(EditScriptBlockCommand {
            document_id: doc.document.id,
            block_id: block.block.id.clone(),
            expected_revision_event_id: block.revision_event_id.unwrap(),
            text: "Exact human screenplay saved during delayed generation.\n\n".into(),
        }),
    )
    .await
    .unwrap();
    let before = document(&f);
    let count = history_count(&f);
    release.send(()).unwrap();
    assert!(
        terminal(&f, &mut events)
            .await
            .unwrap_err()
            .contains("generation target changed")
    );
    worker.join().unwrap();
    assert_eq!(document(&f), before);
    assert_eq!(history_count(&f), count);
    let preview = crate::ai_service::preview_ai_context(&f.state, f.node.0)
        .await
        .unwrap();
    assert!(
        preview
            .user
            .contains("Exact human screenplay saved during delayed generation.\n\n")
    );
    let refused = start_generation(
        &f.state,
        AiGenerateRequest {
            node_id: f.node.0,
            story_time_ms: None,
        },
    )
    .await
    .unwrap_err();
    assert!(refused.to_string().contains("manually edited"));
    f.state.shutdown_tasks_async().await;
}

#[tokio::test]
async fn autosave_interleaving_after_public_edit_signature_read_preserves_manual_and_generation_custody()
 {
    use crate::write_concurrency_probe::{
        self as probe, AutosaveStage, CommandProbe, CommandStage,
    };
    let f = fixture().await;
    let (release, worker, mut events) = start(&f).await;
    release.send(()).unwrap();
    terminal(&f, &mut events).await.unwrap();
    worker.join().unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        while f.state.generating.lock().contains(&f.node.0) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let (generation_release, worker, mut events) = start(&f).await;
    // Hold the actual autosave admission gate, not a timing approximation.
    // The existing public edit is not a session-gate participant.
    let autosave_admission = f.state.project_session_gate.clone().lock_owned().await;
    let doc = document(&f);
    let block = doc
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|block| block.block.text == OUTPUT)
        .unwrap();
    let manual = "Exact human edit across an autosave commit.\n\n";
    let command = CommandEnvelope::new(EditScriptBlockCommand {
        document_id: doc.document.id,
        block_id: block.block.id.clone(),
        expected_revision_event_id: block.revision_event_id.unwrap(),
        text: manual.into(),
    });
    let (stages, mut command_stages) = tokio::sync::mpsc::unbounded_channel();
    let (edit_release, release) = std::sync::mpsc::channel();
    let _command_probe = probe::command(command.id, CommandProbe { stages, release });
    let (autosaves, mut autosave_stages) = tokio::sync::mpsc::unbounded_channel();
    let _autosave_probe = probe::autosave(f.path.clone(), autosaves);
    let state = f.state.clone();
    let edit =
        tokio::spawn(async move { command_service::edit_script_block(&state, command).await });
    assert!(matches!(
        tokio::time::timeout(Duration::from_secs(10), command_stages.recv())
            .await
            .unwrap()
            .unwrap(),
        CommandStage::SignatureRead
    ));
    // A zero-wait independent observer identifies writer ownership. This never
    // changes the application connection's existing busy timeout.
    let observer = rusqlite::Connection::open(&f.path).unwrap();
    observer.busy_timeout(Duration::ZERO).unwrap();
    let writer_reserved = match observer.execute_batch("BEGIN IMMEDIATE") {
        Ok(()) => {
            observer.execute_batch("ROLLBACK").unwrap();
            false
        }
        Err(rusqlite::Error::SqliteFailure(error, _))
            if error.code == rusqlite::ErrorCode::DatabaseBusy =>
        {
            true
        }
        other => panic!("unexpected writer ownership probe: {other:?}"),
    };
    drop(observer);
    // Force the real 2-second autosave loop to persist while the public edit is
    // paused immediately after its signature SELECT. No injected database write.
    f.state.trigger_save();
    drop(autosave_admission);
    let before = tokio::time::timeout(Duration::from_secs(10), autosave_stages.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(before, AutosaveStage::BeforePersistence));
    if !writer_reserved {
        // Baseline DEFERRED transaction: autosave commits before command INSERT,
        // deterministically producing SQLITE_BUSY_SNAPSHOT at that exact INSERT.
        let saved = tokio::time::timeout(Duration::from_secs(10), autosave_stages.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(saved, AutosaveStage::Persisted(Ok(()))));
    }
    edit_release.send(()).unwrap();
    let outcome = edit.await.unwrap();
    let insertion = tokio::time::timeout(Duration::from_secs(10), command_stages.recv())
        .await
        .unwrap()
        .unwrap();
    eprintln!(
        "Public edit/autosave ordering: writer_reserved={writer_reserved}; first_insert={insertion:?}; public_result={outcome:?}"
    );
    if writer_reserved {
        let saved = tokio::time::timeout(Duration::from_secs(10), autosave_stages.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(saved, AutosaveStage::Persisted(Ok(()))));
    }
    let command_count = history_count(&f);
    generation_release.send(()).unwrap();
    let generation = terminal(&f, &mut events).await;
    worker.join().unwrap();
    let canonical = document(&f);
    assert_eq!(
        history_count(&f),
        command_count,
        "Refused late output must roll back history"
    );
    let preview = crate::ai_service::preview_ai_context(&f.state, f.node.0)
        .await
        .unwrap();
    f.state.shutdown_tasks_async().await;
    assert!(
        matches!(
            insertion,
            CommandStage::FirstInsert {
                extended_error_code: None
            }
        ),
        "actual INSERT must succeed: {insertion:?}"
    );
    outcome.unwrap();
    assert!(
        generation
            .unwrap_err()
            .contains("generation target changed")
    );
    assert!(
        canonical
            .segments
            .iter()
            .flat_map(|segment| &segment.blocks)
            .any(|block| block.block.text == manual)
    );
    assert!(preview.user.contains(manual));
}

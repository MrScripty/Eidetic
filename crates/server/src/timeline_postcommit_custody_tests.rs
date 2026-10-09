use std::future::Future;
use std::path::{Path, PathBuf};
use std::task::Context;

use eidetic_core::Template;
use eidetic_core::contracts::{CommandEnvelope, SetTimelineNodeNotesCommand};
use eidetic_core::timeline::node::NodeId;

use crate::backend_error::BackendError;
use crate::command_service::set_timeline_node_notes;
use crate::history_store::RecordChangeOutcome;
use crate::project_service::{
    LoadProjectRequest, SaveProjectRequest, load_project, replace_active_project, save_project,
};
use crate::state::AppState;
use crate::state::ServerEvent;
use crate::ydoc::{self, ContentField, DocCommand};

pub(crate) fn history(path: &Path) -> Vec<Vec<Vec<String>>> {
    let conn = crate::sqlite::open_write_connection(path).unwrap();
    [
        "commands",
        "change_events",
        "object_revisions",
        "object_revision_fields",
    ]
    .iter()
    .map(|table| {
        let mut statement = conn
            .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
            .unwrap();
        let count = statement.column_count();
        statement
            .query_map([], |row| {
                (0..count)
                    .map(|column| Ok(format!("{:?}", row.get_ref(column)?)))
                    .collect::<rusqlite::Result<Vec<_>>>()
            })
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
    })
    .collect()
}

struct Fixture {
    state: AppState,
    directory: PathBuf,
    path_a: PathBuf,
    path_b: PathBuf,
    node: NodeId,
}

impl Fixture {
    async fn new() -> Self {
        let state = AppState::new().await;
        let directory = crate::persistence::default_project_dir()
            .join(format!("custody-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        // Match the canonical paths that public lifecycle validation publishes.
        // Keep every custody/history/session assertion below exact and intact.
        let directory = directory.canonicalize().unwrap();
        let path_a = directory.join("A.db");
        let path_b = directory.join("B.db");
        let project_a = Template::MultiCam.build_project("A");
        let node = project_a.timeline.nodes[0].id;
        let mut project_b = project_a.clone();
        project_b.name = "B".into();
        for (project, path, notes) in [
            (&project_a, &path_a, "A existing revision"),
            (&project_b, &path_b, "B existing revision"),
        ] {
            crate::persistence::save_project(project, path, None)
                .await
                .unwrap();
            let mut conn = crate::sqlite::open_write_connection(path).unwrap();
            let command = CommandEnvelope::new(SetTimelineNodeNotesCommand {
                expected: None,
                node_id: node,
                notes: notes.into(),
            });
            crate::timeline_command::record_set_timeline_node_notes_history(
                &mut conn, project, &command, 0,
            )
            .unwrap();
        }
        let (project_a, _) = crate::persistence::load_project(&path_a).await.unwrap();
        replace_active_project(&state, project_a, path_a.clone());
        state
            .doc_tx
            .send(DocCommand::WriteNodeContent {
                node_id: node,
                field: ContentField::Notes,
                text: "A existing revision".into(),
                author: "test:A".into(),
            })
            .await
            .unwrap();
        assert_eq!(
            ydoc::read_content(&state.doc_tx, node).await.unwrap().notes,
            "A existing revision"
        );
        Self {
            state,
            directory,
            path_a,
            path_b,
            node,
        }
    }

    async fn finish(self) {
        self.state.task_supervisor.shutdown_all().await;
        std::fs::remove_dir_all(&self.directory).unwrap();
    }
}

fn assert_pending(future: std::pin::Pin<&mut impl Future>) {
    assert!(
        future
            .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
            .is_pending()
    );
}

#[tokio::test]
async fn postcommit_events_wait_for_document_enqueue_before_project_transition() {
    let mut fixture = Fixture::new().await;
    let real_doc = fixture.state.doc_tx.clone();
    let (proxy, mut receiver) = tokio::sync::mpsc::channel(1);
    fixture.state.doc_tx = proxy.clone();
    let (reply, serialized) = tokio::sync::oneshot::channel();
    proxy.send(DocCommand::Serialize { reply }).await.unwrap();
    let mut events = fixture.state.events_tx.subscribe();
    let command = CommandEnvelope::new(SetTimelineNodeNotesCommand {
        expected: None,
        node_id: fixture.node,
        notes: "A queued document edit".into(),
    });
    let command_id = command.id.0.to_string();
    let worker = fixture.state.clone();
    let edit = tokio::spawn(async move { set_timeline_node_notes(&worker, command).await });
    let path = fixture.path_a.clone();
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            let path = path.clone();
            let id = command_id.clone();
            let recorded = tokio::task::spawn_blocking(move || {
                let conn = crate::sqlite::open_write_connection(&path).unwrap();
                conn.query_row(
                    "SELECT EXISTS(SELECT 1 FROM commands WHERE id = ?1)",
                    [id],
                    |row| row.get::<_, bool>(0),
                )
                .unwrap()
            })
            .await
            .unwrap();
            if recorded {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    // SQLite has committed, but the full document channel still owns the
    // continuation. It must neither drop that write nor publish its events yet.
    assert!(matches!(
        events.try_recv(),
        Err(tokio::sync::broadcast::error::TryRecvError::Empty)
    ));
    let mut load = Box::pin(load_project(
        &fixture.state,
        LoadProjectRequest {
            path: fixture.path_b.display().to_string(),
        },
    ));
    assert_pending(load.as_mut());
    assert_eq!(fixture.state.project.lock().as_ref().unwrap().name, "A");
    let forwarding = tokio::spawn(async move {
        while let Some(command) = receiver.recv().await {
            real_doc.send(command).await.unwrap();
        }
    });
    serialized.await.unwrap();
    edit.await.unwrap().unwrap();
    assert_eq!(fixture.state.project.lock().as_ref().unwrap().name, "A");
    assert!(matches!(
        events.try_recv().unwrap(),
        ServerEvent::TimelineChanged
    ));
    assert!(
        matches!(events.try_recv().unwrap(), ServerEvent::NodeUpdated { node_id } if node_id == fixture.node.0)
    );
    load.await.unwrap();
    assert_eq!(
        ydoc::read_content(&fixture.state.doc_tx, fixture.node)
            .await
            .unwrap()
            .notes,
        "B existing revision"
    );
    assert!(matches!(
        events.try_recv(),
        Err(tokio::sync::broadcast::error::TryRecvError::Empty)
    ));
    fixture.finish().await;
    forwarding.abort();
    let _ = forwarding.await;
}

#[tokio::test]
async fn postcommit_notes_finish_before_project_load_and_preserve_existing_histories() {
    let fixture = Fixture::new().await;
    let history_a = history(&fixture.path_a);
    let history_b = history(&fixture.path_b);
    let held = fixture.state.project_session_gate.lock().await;
    let command = CommandEnvelope::new(SetTimelineNodeNotesCommand {
        expected: None,
        node_id: fixture.node,
        notes: "A committed edit".into(),
    });
    let mut edit = Box::pin(set_timeline_node_notes(&fixture.state, command));
    assert_pending(edit.as_mut());
    let mut load = Box::pin(load_project(
        &fixture.state,
        LoadProjectRequest {
            path: fixture.path_b.display().to_string(),
        },
    ));
    assert_pending(load.as_mut());
    drop(held);
    let (edit, load) = futures::join!(edit, load);
    edit.unwrap();
    load.unwrap();

    assert_eq!(fixture.state.project.lock().as_ref().unwrap().name, "B");
    assert_eq!(
        ydoc::read_content(&fixture.state.doc_tx, fixture.node)
            .await
            .unwrap()
            .notes,
        "B existing revision"
    );
    let (saved_a, _) = crate::persistence::load_project(&fixture.path_a)
        .await
        .unwrap();
    assert_eq!(
        saved_a.timeline.node(fixture.node).unwrap().content.notes,
        "A committed edit"
    );
    let after_a = history(&fixture.path_a);
    for (before, after) in history_a.iter().zip(&after_a) {
        assert!(
            after.starts_with(before),
            "existing A history must remain intact"
        );
    }
    assert_eq!(after_a[0].len(), history_a[0].len() + 1);
    assert_eq!(history(&fixture.path_b), history_b);
    save_project(&fixture.state, SaveProjectRequest { path: None })
        .await
        .unwrap();
    assert_eq!(history(&fixture.path_b), history_b);
    let (_, blob_b) = crate::persistence::load_project(&fixture.path_b)
        .await
        .unwrap();
    let supervisor = crate::backend_task::BackendTaskSupervisor::default();
    let (doc_b, _) = ydoc::spawn_doc_manager(&supervisor);
    ydoc::load_doc(&doc_b, blob_b.unwrap()).await.unwrap();
    assert_eq!(
        ydoc::read_content(&doc_b, fixture.node)
            .await
            .unwrap()
            .notes,
        "B existing revision"
    );
    supervisor.shutdown_all().await;
    fixture.finish().await;
}

#[tokio::test]
async fn queued_command_rejects_same_path_reopen_without_events_or_history() {
    let fixture = Fixture::new().await;
    let before = history(&fixture.path_a);
    let mut events = fixture.state.events_tx.subscribe();
    let held = fixture.state.project_session_gate.lock().await;
    let command = CommandEnvelope::new(SetTimelineNodeNotesCommand {
        expected: None,
        node_id: fixture.node,
        notes: "obsolete".into(),
    });
    let mut edit = Box::pin(set_timeline_node_notes(&fixture.state, command));
    assert_pending(edit.as_mut());
    // Model the publication phase of a same-path reopen while its gate is held.
    let (reopened, _) = crate::persistence::load_project(&fixture.path_a)
        .await
        .unwrap();
    replace_active_project(&fixture.state, reopened, fixture.path_a.clone());
    drop(held);
    let error = edit.await.unwrap_err();
    assert!(matches!(error, BackendError::Conflict(_)));
    assert_eq!(history(&fixture.path_a), before);
    assert!(matches!(
        events.try_recv(),
        Err(tokio::sync::broadcast::error::TryRecvError::Empty)
    ));
    assert_eq!(
        ydoc::read_content(&fixture.state.doc_tx, fixture.node)
            .await
            .unwrap()
            .notes,
        "A existing revision"
    );
    fixture.finish().await;
}

#[tokio::test]
async fn queued_command_rejects_save_as_without_writing_either_database() {
    let fixture = Fixture::new().await;
    let before = history(&fixture.path_a);
    let path_c = fixture.directory.join("save-as.db");
    let held = fixture.state.project_session_gate.lock().await;
    let mut save = Box::pin(save_project(
        &fixture.state,
        SaveProjectRequest {
            path: Some(path_c.display().to_string()),
        },
    ));
    assert_pending(save.as_mut());
    let command = CommandEnvelope::new(SetTimelineNodeNotesCommand {
        expected: None,
        node_id: fixture.node,
        notes: "obsolete".into(),
    });
    let mut edit = Box::pin(set_timeline_node_notes(&fixture.state, command));
    assert_pending(edit.as_mut());
    drop(held);
    let (save, edit) = futures::join!(save, edit);
    save.unwrap();
    assert!(matches!(edit.unwrap_err(), BackendError::Conflict(_)));
    assert_eq!(history(&fixture.path_a), before);
    assert!(history(&path_c)[0].is_empty());
    assert_eq!(fixture.state.project_database.active_path(), Some(path_c));
    fixture.finish().await;
}

#[tokio::test]
async fn project_load_sqlite_with_json_suffix_keeps_selected_save_target() {
    let fixture = Fixture::new().await;
    let selected = fixture.directory.join("selected.json");
    let sibling = fixture.directory.join("project.db");
    std::fs::copy(&fixture.path_b, &selected).unwrap();
    std::fs::copy(&fixture.path_a, &sibling).unwrap();
    let sibling_before = std::fs::read(&sibling).unwrap();
    let selected_history = history(&selected);
    let (autosave_tx, mut autosave_rx) = tokio::sync::mpsc::unbounded_channel();
    let _autosave = crate::write_concurrency_probe::autosave(selected.clone(), autosave_tx);

    let loaded = load_project(
        &fixture.state,
        LoadProjectRequest {
            path: selected.display().to_string(),
        },
    )
    .await
    .unwrap();
    assert_eq!(loaded["name"], "B");
    assert_eq!(
        fixture.state.project_database.active_path(),
        Some(selected.clone())
    );
    crate::project_service::update_project(
        &fixture.state,
        crate::project_service::UpdateProjectRequest {
            name: None,
            premise: Some("Autosaved to the selected SQLite file".into()),
        },
    )
    .unwrap();
    assert_eq!(
        ydoc::read_content(&fixture.state.doc_tx, fixture.node)
            .await
            .unwrap()
            .notes,
        "B existing revision"
    );
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            match autosave_rx
                .recv()
                .await
                .expect("autosave probe remains open")
            {
                crate::write_concurrency_probe::AutosaveStage::BeforePersistence => {}
                crate::write_concurrency_probe::AutosaveStage::Persisted(result) => {
                    result.unwrap();
                    break;
                }
            }
        }
    })
    .await
    .expect("selected SQLite file is autosaved");
    let (saved, blob) = crate::persistence::load_project(&selected).await.unwrap();
    assert_eq!(saved.name, "B");
    assert_eq!(saved.premise, "Autosaved to the selected SQLite file");
    assert!(blob.is_some());
    assert_eq!(history(&selected), selected_history);

    let saved = save_project(&fixture.state, SaveProjectRequest { path: None })
        .await
        .unwrap();
    assert_eq!(saved["saved"], selected.display().to_string());
    assert_eq!(std::fs::read(&sibling).unwrap(), sibling_before);
    fixture.finish().await;
}

#[tokio::test]
async fn project_load_rejects_json_without_creating_or_changing_sibling() {
    let fixture = Fixture::new().await;
    let selected = fixture.directory.join("selected.json");
    let sibling = fixture.directory.join("project.db");
    let session_id = *fixture.state.project_session_id.lock();
    let before_history = history(&fixture.path_a);
    let mut events = fixture.state.events_tx.subscribe();

    for sibling_exists in [false, true] {
        if sibling_exists {
            std::fs::copy(&fixture.path_b, &sibling).unwrap();
        }
        let sibling_before = std::fs::read(&sibling).ok();
        for json in [r#"{"name":"Legacy JSON"}"#, "{ malformed JSON"] {
            std::fs::write(&selected, json).unwrap();
            let error = load_project(
                &fixture.state,
                LoadProjectRequest {
                    path: selected.display().to_string(),
                },
            )
            .await
            .unwrap_err();
            assert!(matches!(error, BackendError::BadRequest(_)));
            assert!(error.message().contains("unsupported project schema"));
            assert_eq!(std::fs::read_to_string(&selected).unwrap(), json);
            assert_eq!(std::fs::read(&sibling).ok(), sibling_before);
            assert_eq!(*fixture.state.project_session_id.lock(), session_id);
            assert_eq!(fixture.state.project.lock().as_ref().unwrap().name, "A");
            assert_eq!(
                fixture.state.project_database.active_path(),
                Some(fixture.path_a.clone())
            );
            assert_eq!(history(&fixture.path_a), before_history);
            assert_eq!(
                ydoc::read_content(&fixture.state.doc_tx, fixture.node)
                    .await
                    .unwrap()
                    .notes,
                "A existing revision"
            );
            assert!(matches!(
                events.try_recv(),
                Err(tokio::sync::broadcast::error::TryRecvError::Empty)
            ));
        }
    }
    fixture.finish().await;
}

#[tokio::test]
async fn failed_document_population_returns_error_without_publishing_project() {
    let fixture = Fixture::new().await;
    fixture.state.task_supervisor.shutdown_all().await;
    let error = load_project(
        &fixture.state,
        LoadProjectRequest {
            path: fixture.path_b.display().to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(error, BackendError::Internal(_)));
    assert_eq!(fixture.state.project.lock().as_ref().unwrap().name, "A");
    assert_eq!(
        fixture.state.project_database.active_path(),
        Some(fixture.path_a.clone())
    );
    fixture.finish().await;
}

#[tokio::test]
async fn failed_postcommit_document_send_reports_committed_failure() {
    let fixture = Fixture::new().await;
    let command = CommandEnvelope::new(SetTimelineNodeNotesCommand {
        expected: None,
        node_id: fixture.node,
        notes: "committed before doc failure".into(),
    });
    fixture.state.task_supervisor.shutdown_all().await;
    let error = set_timeline_node_notes(&fixture.state, command.clone())
        .await
        .unwrap_err();
    assert!(matches!(error, BackendError::Internal(_)));
    assert!(error.to_string().contains("committed"));
    let mut conn = crate::sqlite::open_write_connection(&fixture.path_a).unwrap();
    let (project, _) = crate::persistence::load_project(&fixture.path_a)
        .await
        .unwrap();
    assert_eq!(
        project.timeline.node(fixture.node).unwrap().content.notes,
        "committed before doc failure"
    );
    let outcome = crate::timeline_command::record_set_timeline_node_notes_history(
        &mut conn, &project, &command, 0,
    )
    .unwrap();
    assert_eq!(outcome, RecordChangeOutcome::AlreadyRecorded);
    // Windows keeps an open SQLite file handle from being removed at teardown.
    drop(conn);
    fixture.finish().await;
}

#[tokio::test]
async fn transition_reopen_keeps_committed_notes_in_sqlite_and_document() {
    let fixture = Fixture::new().await;
    // A real earlier blob is necessary: a blob-less fixture takes a fallback
    // path and cannot reproduce the stale saved-document failure.
    save_project(&fixture.state, SaveProjectRequest { path: None })
        .await
        .unwrap();
    set_timeline_node_notes(
        &fixture.state,
        CommandEnvelope::new(SetTimelineNodeNotesCommand {
            expected: None,
            node_id: fixture.node,
            notes: "A edit after earlier blob".into(),
        }),
    )
    .await
    .unwrap();
    let history_a = history(&fixture.path_a);
    load_project(
        &fixture.state,
        LoadProjectRequest {
            path: fixture.path_b.display().to_string(),
        },
    )
    .await
    .unwrap();
    assert_eq!(history(&fixture.path_a), history_a);
    let (saved_a, blob_a) = crate::persistence::load_project(&fixture.path_a)
        .await
        .unwrap();
    assert!(blob_a.is_some());
    assert_eq!(
        saved_a.timeline.node(fixture.node).unwrap().content.notes,
        "A edit after earlier blob"
    );
    load_project(
        &fixture.state,
        LoadProjectRequest {
            path: fixture.path_a.display().to_string(),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        ydoc::read_content(&fixture.state.doc_tx, fixture.node)
            .await
            .unwrap()
            .notes,
        "A edit after earlier blob"
    );
    assert_eq!(history(&fixture.path_a), history_a);
    fixture.finish().await;
}

#[tokio::test]
async fn transition_save_as_copies_committed_source_timeline_and_document() {
    let fixture = Fixture::new().await;
    save_project(&fixture.state, SaveProjectRequest { path: None })
        .await
        .unwrap();
    set_timeline_node_notes(
        &fixture.state,
        CommandEnvelope::new(SetTimelineNodeNotesCommand {
            expected: None,
            node_id: fixture.node,
            notes: "committed source notes".into(),
        }),
    )
    .await
    .unwrap();
    assert_eq!(
        fixture
            .state
            .project
            .lock()
            .as_ref()
            .unwrap()
            .timeline
            .node(fixture.node)
            .unwrap()
            .content
            .notes,
        "A existing revision"
    );
    let history_a = history(&fixture.path_a);
    let path_c = fixture.directory.join("C.db");
    save_project(
        &fixture.state,
        SaveProjectRequest {
            path: Some(path_c.display().to_string()),
        },
    )
    .await
    .unwrap();
    let (saved_c, blob_c) = crate::persistence::load_project(&path_c).await.unwrap();
    assert_eq!(
        saved_c.timeline.node(fixture.node).unwrap().content.notes,
        "committed source notes"
    );
    assert_eq!(
        fixture
            .state
            .project
            .lock()
            .as_ref()
            .unwrap()
            .timeline
            .node(fixture.node)
            .unwrap()
            .content
            .notes,
        "committed source notes"
    );
    assert_eq!(history(&fixture.path_a), history_a);
    // Save As copies the current project snapshot, not its command history.
    assert!(history(&path_c)[0].is_empty());
    ydoc::load_doc(&fixture.state.doc_tx, blob_c.unwrap())
        .await
        .unwrap();
    assert_eq!(
        ydoc::read_content(&fixture.state.doc_tx, fixture.node)
            .await
            .unwrap()
            .notes,
        "committed source notes"
    );
    fixture.finish().await;
}

#[tokio::test]
async fn transition_serialization_failure_keeps_active_state_and_saved_blob() {
    let fixture = Fixture::new().await;
    save_project(&fixture.state, SaveProjectRequest { path: None })
        .await
        .unwrap();
    let (_, before_blob) = crate::persistence::load_project(&fixture.path_a)
        .await
        .unwrap();
    let before_history = history(&fixture.path_a);
    let path_c = fixture.directory.join("C.db");
    fixture.state.task_supervisor.shutdown_all().await;
    let load_error = load_project(
        &fixture.state,
        LoadProjectRequest {
            path: fixture.path_b.display().to_string(),
        },
    )
    .await
    .unwrap_err();
    let save_error = save_project(
        &fixture.state,
        SaveProjectRequest {
            path: Some(path_c.display().to_string()),
        },
    )
    .await
    .unwrap_err();
    assert!(load_error.to_string().contains("serialization"));
    assert!(save_error.to_string().contains("serialization"));
    assert!(!path_c.exists());
    assert_eq!(fixture.state.project.lock().as_ref().unwrap().name, "A");
    assert_eq!(
        fixture.state.project_database.active_path(),
        Some(fixture.path_a.clone())
    );
    assert_eq!(
        crate::persistence::load_project(&fixture.path_a)
            .await
            .unwrap()
            .1,
        before_blob
    );
    assert_eq!(history(&fixture.path_a), before_history);
    fixture.finish().await;
}

#[tokio::test]
async fn transition_source_persistence_failure_does_not_replace_document() {
    let fixture = Fixture::new().await;
    // A directory cannot be opened as SQLite. Serialization succeeds, but the
    // failed outgoing write must leave the live document and session in place.
    fixture
        .state
        .project_database
        .set_active_path(fixture.directory.clone());
    let session_id = *fixture.state.project_session_id.lock();
    let error = load_project(
        &fixture.state,
        LoadProjectRequest {
            path: fixture.path_b.display().to_string(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(error, BackendError::Internal(_)));
    assert_eq!(*fixture.state.project_session_id.lock(), session_id);
    assert_eq!(fixture.state.project.lock().as_ref().unwrap().name, "A");
    assert_eq!(
        ydoc::read_content(&fixture.state.doc_tx, fixture.node)
            .await
            .unwrap()
            .notes,
        "A existing revision"
    );
    fixture.finish().await;
}

#[tokio::test]
async fn transition_cancelled_caller_keeps_flush_and_replacement_in_its_gate() {
    let mut fixture = Fixture::new().await;
    let real_doc = fixture.state.doc_tx.clone();
    let blob_a = ydoc::serialize_doc(&real_doc).await.unwrap();
    let (proxy, mut receiver) = tokio::sync::mpsc::channel(1);
    fixture.state.doc_tx = proxy;
    let worker = fixture.state.clone();
    let path_b = fixture.path_b.display().to_string();
    let request =
        tokio::spawn(
            async move { load_project(&worker, LoadProjectRequest { path: path_b }).await },
        );
    let command = tokio::time::timeout(std::time::Duration::from_secs(10), receiver.recv())
        .await
        .unwrap()
        .unwrap();
    let DocCommand::Serialize { reply } = command else {
        panic!("expected outgoing serialization");
    };
    request.abort();
    assert!(request.await.unwrap_err().is_cancelled());
    assert_eq!(fixture.state.project.lock().as_ref().unwrap().name, "A");
    assert!(fixture.state.project_session_gate.try_lock().is_err());
    reply.send(blob_a).unwrap();
    let forwarding = tokio::spawn(async move {
        while let Some(command) = receiver.recv().await {
            real_doc.send(command).await.unwrap();
        }
    });
    let gate = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        fixture.state.project_session_gate.lock(),
    )
    .await
    .unwrap();
    assert_eq!(fixture.state.project.lock().as_ref().unwrap().name, "B");
    assert_eq!(
        ydoc::read_content(&fixture.state.doc_tx, fixture.node)
            .await
            .unwrap()
            .notes,
        "B existing revision"
    );
    drop(gate);
    fixture.finish().await;
    forwarding.abort();
    let _ = forwarding.await;
}

#[tokio::test]
async fn transition_save_as_existing_destination_returns_conflict_without_mixing_histories() {
    let fixture = Fixture::new().await;
    let history_a = history(&fixture.path_a);
    let history_b = history(&fixture.path_b);
    let error = save_project(
        &fixture.state,
        SaveProjectRequest {
            path: Some(fixture.path_b.display().to_string()),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(error, BackendError::Conflict(_)));
    assert_eq!(
        fixture.state.project_database.active_path(),
        Some(fixture.path_a.clone())
    );
    assert_eq!(history(&fixture.path_a), history_a);
    assert_eq!(history(&fixture.path_b), history_b);
    fixture.finish().await;
}

#[tokio::test]
async fn transition_cancelled_notes_caller_still_publishes_and_flushes_source_document() {
    let mut fixture = Fixture::new().await;
    let real_doc = fixture.state.doc_tx.clone();
    let (proxy, mut receiver) = tokio::sync::mpsc::channel(1);
    fixture.state.doc_tx = proxy.clone();
    let (reply, serialized) = tokio::sync::oneshot::channel();
    proxy.send(DocCommand::Serialize { reply }).await.unwrap();
    let command = CommandEnvelope::new(SetTimelineNodeNotesCommand {
        expected: None,
        node_id: fixture.node,
        notes: "A edit survives cancelled response".into(),
    });
    let command_id = command.id.0.to_string();
    let worker = fixture.state.clone();
    let request = tokio::spawn(async move { set_timeline_node_notes(&worker, command).await });
    let path = fixture.path_a.clone();
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            let path = path.clone();
            let id = command_id.clone();
            let recorded = tokio::task::spawn_blocking(move || {
                let conn = crate::sqlite::open_write_connection(&path).unwrap();
                conn.query_row(
                    "SELECT EXISTS(SELECT 1 FROM commands WHERE id = ?1)",
                    [id],
                    |row| row.get::<_, bool>(0),
                )
                .unwrap()
            })
            .await
            .unwrap();
            if recorded {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    request.abort();
    assert!(request.await.unwrap_err().is_cancelled());
    assert!(fixture.state.project_session_gate.try_lock().is_err());
    let mut load = Box::pin(load_project(
        &fixture.state,
        LoadProjectRequest {
            path: fixture.path_b.display().to_string(),
        },
    ));
    assert_pending(load.as_mut());
    let forwarding = tokio::spawn(async move {
        while let Some(command) = receiver.recv().await {
            real_doc.send(command).await.unwrap();
        }
    });
    serialized.await.unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(10), load)
        .await
        .unwrap()
        .unwrap();
    let (saved_a, blob_a) = crate::persistence::load_project(&fixture.path_a)
        .await
        .unwrap();
    assert_eq!(
        saved_a.timeline.node(fixture.node).unwrap().content.notes,
        "A edit survives cancelled response"
    );
    let supervisor = crate::backend_task::BackendTaskSupervisor::default();
    let (check_doc, _) = ydoc::spawn_doc_manager(&supervisor);
    ydoc::load_doc(&check_doc, blob_a.unwrap()).await.unwrap();
    assert_eq!(
        ydoc::read_content(&check_doc, fixture.node)
            .await
            .unwrap()
            .notes,
        "A edit survives cancelled response"
    );
    assert_eq!(
        ydoc::read_content(&fixture.state.doc_tx, fixture.node)
            .await
            .unwrap()
            .notes,
        "B existing revision"
    );
    supervisor.shutdown_all().await;
    fixture.finish().await;
    forwarding.abort();
    let _ = forwarding.await;
}

#[tokio::test]
async fn normal_runtime_repeated_commands_bound_completed_supervisor_records() {
    let fixture = Fixture::new().await;
    let baseline = fixture.state.task_supervisor.retained_task_count();
    // This observer only reads the registry; it never invokes the desktop smoke
    // counter or drives cleanup. Each real command must reap earlier completions.
    for index in 0..64 {
        set_timeline_node_notes(
            &fixture.state,
            CommandEnvelope::new(SetTimelineNodeNotesCommand {
                expected: None,
                node_id: fixture.node,
                notes: format!("normal command {index}"),
            }),
        )
        .await
        .unwrap();
        tokio::task::yield_now().await;
        assert!(
            fixture.state.task_supervisor.retained_task_count() <= baseline + 2,
            "normal command completion records accumulated at iteration {index}"
        );
    }
    let (saved, _) = crate::persistence::load_project(&fixture.path_a)
        .await
        .unwrap();
    assert_eq!(
        saved.timeline.node(fixture.node).unwrap().content.notes,
        "normal command 63"
    );
    assert_eq!(
        ydoc::read_content(&fixture.state.doc_tx, fixture.node)
            .await
            .unwrap()
            .notes,
        "normal command 63"
    );
    assert_eq!(history(&fixture.path_a)[0].len(), 65);
    fixture.finish().await;
}

// Exercise the actual public services with a legacy stored spelling, rather than
// only comparing path strings. All writes still go through the validated target.
async fn assert_same_project_aliases_preserve_custody(fixture: Fixture, spellings: Vec<PathBuf>) {
    let before = history(&fixture.path_a);
    let session_id = *fixture.state.project_session_id.lock();
    for source in &spellings {
        for destination in &spellings {
            fixture
                .state
                .project_database
                .set_active_path(source.clone());
            let saved = save_project(
                &fixture.state,
                SaveProjectRequest {
                    path: Some(destination.display().to_string()),
                },
            )
            .await
            .unwrap();
            assert_eq!(saved["saved"], fixture.path_a.display().to_string());
            assert_eq!(
                fixture.state.project_database.active_path(),
                Some(fixture.path_a.clone())
            );
            assert_eq!(*fixture.state.project_session_id.lock(), session_id);
            assert_eq!(history(&fixture.path_a), before);
            assert_eq!(
                ydoc::read_content(&fixture.state.doc_tx, fixture.node)
                    .await
                    .unwrap()
                    .notes,
                "A existing revision"
            );
        }
    }
    // A spelling-only save must not invalidate a command already queued for the
    // same session. A real Save As/reopen still changes the session below.
    fixture
        .state
        .project_database
        .set_active_path(spellings[0].clone());
    let held = fixture.state.project_session_gate.lock().await;
    let mut save = Box::pin(save_project(
        &fixture.state,
        SaveProjectRequest {
            path: Some(fixture.path_a.display().to_string()),
        },
    ));
    assert_pending(save.as_mut());
    let mut edit = Box::pin(set_timeline_node_notes(
        &fixture.state,
        CommandEnvelope::new(SetTimelineNodeNotesCommand {
            expected: None,
            node_id: fixture.node,
            notes: "alias-admitted edit".into(),
        }),
    ));
    assert_pending(edit.as_mut());
    drop(held);
    let (saved, edited) = futures::join!(save, edit);
    saved.unwrap();
    edited.unwrap();
    assert_eq!(*fixture.state.project_session_id.lock(), session_id);
    let edited_history = history(&fixture.path_a);
    assert_eq!(edited_history[0].len(), before[0].len() + 1);
    assert_eq!(
        ydoc::read_content(&fixture.state.doc_tx, fixture.node)
            .await
            .unwrap()
            .notes,
        "alias-admitted edit"
    );

    let destination = fixture.directory.join("identity-save-as.db");
    save_project(
        &fixture.state,
        SaveProjectRequest {
            path: Some(destination.display().to_string()),
        },
    )
    .await
    .unwrap();
    assert_ne!(*fixture.state.project_session_id.lock(), session_id);
    assert_eq!(history(&fixture.path_a), edited_history);
    assert!(history(&destination)[0].is_empty());
    let (saved_project, saved_blob) = crate::persistence::load_project(&destination)
        .await
        .unwrap();
    assert_eq!(
        saved_project
            .timeline
            .node(fixture.node)
            .unwrap()
            .content
            .notes,
        "alias-admitted edit"
    );
    assert!(saved_blob.is_some());
    let save_as_session = *fixture.state.project_session_id.lock();
    load_project(
        &fixture.state,
        LoadProjectRequest {
            path: spellings[0].display().to_string(),
        },
    )
    .await
    .unwrap();
    assert_ne!(*fixture.state.project_session_id.lock(), save_as_session);
    assert_eq!(
        fixture.state.project_database.active_path(),
        Some(fixture.path_a.clone())
    );
    assert_eq!(history(&fixture.path_a), edited_history);
    assert_eq!(
        ydoc::read_content(&fixture.state.doc_tx, fixture.node)
            .await
            .unwrap()
            .notes,
        "alias-admitted edit"
    );
    fixture.finish().await;
}

#[cfg(unix)]
#[tokio::test]
async fn same_project_symlink_save_preserves_session_queued_edits_and_reopen_history() {
    let fixture = Fixture::new().await;
    let alias = fixture.directory.join("A-alias.db");
    std::os::unix::fs::symlink(&fixture.path_a, &alias).unwrap();
    let spellings = vec![alias, fixture.path_a.clone()];
    assert_same_project_aliases_preserve_custody(fixture, spellings).await;
}

#[cfg(windows)]
#[tokio::test]
async fn same_project_windows_alias_save_preserves_session_queued_edits_and_reopen_history() {
    let fixture = Fixture::new().await;
    let spellings = crate::validation::tests::windows_path_spellings(&fixture.path_a);
    assert_same_project_aliases_preserve_custody(fixture, spellings).await;
}

#[tokio::test]
async fn save_as_hardlink_destination_keeps_existing_database_and_history_untouched() {
    let fixture = Fixture::new().await;
    let before = history(&fixture.path_a);
    let session_id = *fixture.state.project_session_id.lock();
    let hardlink = fixture.directory.join("A-hardlink.db");
    std::fs::hard_link(&fixture.path_a, &hardlink).unwrap();
    // A hard link has a different canonical name and therefore a different WAL
    // namespace. It is an existing destination, not a spelling-only save.
    let error = save_project(
        &fixture.state,
        SaveProjectRequest {
            path: Some(hardlink.display().to_string()),
        },
    )
    .await
    .unwrap_err();
    assert_eq!(error.message(), "Save As destination already exists");
    assert!(matches!(error, BackendError::Conflict(_)));
    assert_eq!(*fixture.state.project_session_id.lock(), session_id);
    assert_eq!(
        fixture.state.project_database.active_path(),
        Some(fixture.path_a.clone())
    );
    assert_eq!(history(&fixture.path_a), before);
    assert_eq!(
        ydoc::read_content(&fixture.state.doc_tx, fixture.node)
            .await
            .unwrap()
            .notes,
        "A existing revision"
    );
    fixture.finish().await;
}

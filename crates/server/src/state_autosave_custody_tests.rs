use std::future::Future;
use std::sync::Arc;
use std::task::Context;
use std::time::Duration;

use eidetic_core::Template;
use eidetic_core::contracts::{CommandEnvelope, SetTimelineNodeNotesCommand};
use parking_lot::Mutex;

use super::auto_save_task;
use crate::backend_task::BackendTaskSupervisor;
use crate::timeline_postcommit_custody_tests::history;
use crate::ydoc::{self, ContentField, DocCommand};

#[tokio::test]
async fn autosave_custody_holds_snapshot_and_document_until_persistence_finishes() {
    let directory =
        std::env::temp_dir().join(format!("eidetic-autosave-custody-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&directory).unwrap();
    let path_a = directory.join("A.db");
    let path_b = directory.join("B.db");
    let project_a = Template::MultiCam.build_project("A");
    let node = project_a.timeline.nodes[0].id;
    let mut project_b = project_a.clone();
    project_b.name = "B".into();
    for (project, path, notes) in [
        (&project_a, &path_a, "A prior history"),
        (&project_b, &path_b, "B prior history"),
    ] {
        crate::persistence::save_project(project, path, None)
            .await
            .unwrap();
        let mut conn = crate::sqlite::open_write_connection(path).unwrap();
        crate::timeline_command::record_set_timeline_node_notes_history(
            &mut conn,
            project,
            &CommandEnvelope::new(SetTimelineNodeNotesCommand {
                node_id: node,
                notes: notes.into(),
            }),
            0,
        )
        .unwrap();
    }
    let (project_a, _) = crate::persistence::load_project(&path_a).await.unwrap();
    let (project_b, _) = crate::persistence::load_project(&path_b).await.unwrap();
    let history_a = history(&path_a);
    let history_b = history(&path_b);
    let supervisor = BackendTaskSupervisor::default();
    let (real_doc, _) = ydoc::spawn_doc_manager(&supervisor);
    real_doc
        .send(DocCommand::WriteNodeContent {
            node_id: node,
            field: ContentField::Notes,
            text: "A prior history".into(),
            author: "test:A".into(),
        })
        .await
        .unwrap();
    let blob_a = ydoc::serialize_doc(&real_doc).await.unwrap();

    let project = Arc::new(Mutex::new(Some(project_a)));
    let path = Arc::new(Mutex::new(Some(path_a.clone())));
    let gate = Arc::new(tokio::sync::Mutex::new(()));
    let (save_tx, save_rx) = tokio::sync::mpsc::channel(1);
    // Intercept the real Serialize request to pause at the precise memento
    // boundary after mirror/path capture and before document bytes arrive.
    let (doc_tx, mut doc_rx) = tokio::sync::mpsc::channel(1);
    let job = tokio::spawn(auto_save_task(
        save_rx,
        project.clone(),
        path.clone(),
        doc_tx,
        gate.clone(),
    ));
    save_tx.send(()).await.unwrap();
    drop(save_tx);
    let request = tokio::time::timeout(Duration::from_secs(10), doc_rx.recv())
        .await
        .unwrap()
        .unwrap();
    let DocCommand::Serialize { reply } = request else {
        panic!("expected serialize request")
    };
    let mut transition = Box::pin(async {
        let _session = gate.lock().await;
        *project.lock() = Some(project_b);
        *path.lock() = Some(path_b.clone());
    });
    assert!(
        transition
            .as_mut()
            .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
            .is_pending()
    );
    assert_eq!(project.lock().as_ref().unwrap().name, "A");
    reply.send(blob_a).unwrap();
    job.await.unwrap();
    transition.await;

    assert_eq!(project.lock().as_ref().unwrap().name, "B");
    assert_eq!(history(&path_a), history_a);
    assert_eq!(history(&path_b), history_b);
    let (saved_a, blob) = crate::persistence::load_project(&path_a).await.unwrap();
    assert_eq!(saved_a.name, "A");
    ydoc::load_doc(&real_doc, blob.unwrap()).await.unwrap();
    assert_eq!(
        ydoc::read_content(&real_doc, node).await.unwrap().notes,
        "A prior history"
    );
    supervisor.shutdown_all().await;
    std::fs::remove_dir_all(directory).unwrap();
}

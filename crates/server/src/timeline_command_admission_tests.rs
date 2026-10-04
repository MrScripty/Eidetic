use std::future::Future;
use std::path::PathBuf;
use std::sync::mpsc;
use std::task::Context;
use std::time::Duration;

use eidetic_core::Template;
use eidetic_core::contracts::{CommandEnvelope, CreateTimelineChildFromParentCommand};
use eidetic_core::timeline::node::{NodeId, StoryLevel};

use super::{create_timeline_child_from_parent_core_command, timeline_command_project};
use crate::history_store::RecordChangeOutcome;
use crate::project_service::replace_active_project;
use crate::state::AppState;

/// Occupy the only blocking worker so a polled admission is deterministically
/// paused inside persistence loading, without sleeps or production test hooks.
struct BlockingPause {
    release: Option<mpsc::Sender<()>>,
    job: Option<tokio::task::JoinHandle<()>>,
}

impl BlockingPause {
    fn start() -> Self {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let job = tokio::task::spawn_blocking(move || {
            started_tx.send(()).unwrap();
            let _ = release_rx.recv();
        });
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        Self {
            release: Some(release_tx),
            job: Some(job),
        }
    }

    async fn resume(mut self) {
        self.release.take().unwrap().send(()).unwrap();
        self.job.take().unwrap().await.unwrap();
    }
}

impl Drop for BlockingPause {
    fn drop(&mut self) {
        // Release even after a failed assertion so runtime shutdown cannot hang.
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
    }
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .max_blocking_threads(1)
        .enable_all()
        .build()
        .unwrap()
}

fn test_directory() -> PathBuf {
    let path = std::env::temp_dir().join(format!("eidetic-admission-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn failed_database_load_uses_admitted_mirror_after_project_replacement() {
    runtime().block_on(async {
        let state = AppState::new().await;
        let directory = test_directory();
        let path_a = directory.join("missing-A.db");
        let project_a = Template::MultiCam.build_project("A");
        replace_active_project(&state, project_a.clone(), path_a.clone());

        let pause = BlockingPause::start();
        let mut admission = Box::pin(timeline_command_project(&state));
        assert!(
            admission
                .as_mut()
                .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
                .is_pending()
        );
        replace_active_project(
            &state,
            Template::MultiCam.build_project("B"),
            directory.join("B.db"),
        );
        pause.resume().await;

        let (_session, path, admitted) = admission.await.unwrap();
        assert_eq!(path, path_a);
        assert_eq!(admitted.name, "A");
        assert_eq!(
            admitted.timeline.nodes[0].id,
            project_a.timeline.nodes[0].id
        );
        assert_eq!(state.project.lock().as_ref().unwrap().name, "B");
        state.task_supervisor.shutdown_all().await;
        std::fs::remove_dir_all(directory).unwrap();
    });
}

#[test]
fn queued_create_child_keeps_its_admitted_database_after_project_replacement() {
    runtime().block_on(async {
        let state = AppState::new().await;
        let directory = test_directory();
        let path_a = directory.join("A.db");
        let path_b = directory.join("B.db");
        let project_a = Template::MultiCam.build_project("A");
        // Reopening/copying a project can preserve node IDs; identity overlap must
        // not make a command from A apply successfully to the new active B.
        let mut project_b = project_a.clone();
        project_b.name = "B".into();
        crate::persistence::save_project(&project_a, &path_a, None)
            .await
            .unwrap();
        crate::persistence::save_project(&project_b, &path_b, None)
            .await
            .unwrap();
        replace_active_project(&state, project_a.clone(), path_a.clone());
        let parent_id = project_a
            .timeline
            .nodes
            .iter()
            .find(|node| node.level == StoryLevel::Premise)
            .unwrap()
            .id;
        let child_id = NodeId::new();
        let command = CommandEnvelope::new(CreateTimelineChildFromParentCommand {
            node_id: child_id,
            parent_id,
        });

        let pause = BlockingPause::start();
        let mut request = Box::pin(create_timeline_child_from_parent_core_command(
            &state, command,
        ));
        assert!(
            request
                .as_mut()
                .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
                .is_pending()
        );
        replace_active_project(&state, project_b, path_b.clone());
        pause.resume().await;
        let response = request.await.unwrap();
        assert_eq!(response.outcome, RecordChangeOutcome::Recorded);

        let (saved_a, _) = crate::persistence::load_project(&path_a).await.unwrap();
        let (saved_b, _) = crate::persistence::load_project(&path_b).await.unwrap();
        assert!(saved_a.timeline.node(child_id).is_ok());
        assert!(saved_b.timeline.node(child_id).is_err());
        assert_eq!(state.project_database.active_path(), Some(path_b));
        assert_eq!(state.project.lock().as_ref().unwrap().name, "B");
        state.task_supervisor.shutdown_all().await;
        std::fs::remove_dir_all(directory).unwrap();
    });
}

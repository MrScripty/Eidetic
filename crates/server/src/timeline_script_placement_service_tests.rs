use crate::state::{AppState, ServerEvent};
use crate::timeline_script_placement::tests::{command, fixture, input};
use crate::{command_service_timeline, persistence, script_store, sqlite};

#[tokio::test]
async fn native_range_edit_publishes_script_change_only_after_atomic_success_and_not_replay() {
    for rollback in [false, true] {
        let (source, project, a, b) = fixture();
        let directory =
            std::env::temp_dir().join(format!("eidetic-range-script-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("project.db");
        // Existing node rows make this a canonical persisted timeline. Include
        // its required metadata before copying the source fixture to disk.
        source
            .execute_batch(
                "CREATE TABLE episode_structure (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                template_name TEXT NOT NULL,
                segments_json TEXT NOT NULL
            );",
            )
            .unwrap();
        source.execute(
            "INSERT INTO episode_structure (id, template_name, segments_json) VALUES (1, ?1, ?2)",
            rusqlite::params![project.timeline.structure.template_name,
                serde_json::to_string(&project.timeline.structure.segments).unwrap()],
        ).unwrap();
        source
            .execute("VACUUM INTO ?1", [path.to_str().unwrap()])
            .unwrap();
        persistence::save_project(&project, &path, None)
            .await
            .unwrap();
        let conn = sqlite::open_write_connection(&path).unwrap();
        let before_a = input(&conn, &a);
        let before = script_store::load_document_projection(&conn, &b.document_id).unwrap();
        if rollback {
            conn.execute_batch("CREATE TRIGGER reject_placement BEFORE UPDATE ON script_segments BEGIN SELECT RAISE(ABORT, 'injected placement failure'); END;").unwrap();
        }
        let state = AppState::new().await;
        *state.project.lock() = Some(project);
        state.project_database.set_active_path(path);
        let mut events = state.events_tx.subscribe();
        let edit = command(&a, 6000, 7000);
        let result = command_service_timeline::set_timeline_node_range(&state, edit.clone()).await;
        assert_eq!(result.is_err(), rollback);
        if rollback {
            assert!(events.try_recv().is_err());
            assert_eq!(
                before,
                script_store::load_document_projection(&conn, &b.document_id).unwrap()
            );
        } else {
            assert!(matches!(
                events.try_recv().unwrap(),
                ServerEvent::TimelineChanged
            ));
            assert!(matches!(
                events.try_recv().unwrap(),
                ServerEvent::ScriptChanged
            ));
            let after_a = input(&conn, &a);
            assert_eq!((after_a.start_ms, after_a.end_ms), (6000, 7000));
            assert_eq!(after_a.text, before_a.text);
            assert_eq!(after_a.revision_event_id, before_a.revision_event_id);
            assert_ne!(
                after_a.segment_revision_event_id,
                before_a.segment_revision_event_id
            );
            let after = script_store::load_document_projection(&conn, &b.document_id)
                .unwrap()
                .unwrap();
            assert!(
                after
                    .segments
                    .iter()
                    .find(|s| s.segment.id == b.segment_id)
                    .unwrap()
                    .impact
                    .as_ref()
                    .unwrap()
                    .needs_review
            );
            assert!(events.try_recv().is_err());
            command_service_timeline::set_timeline_node_range(&state, edit)
                .await
                .unwrap();
            assert!(
                events.try_recv().is_err(),
                "idempotent replay must not republish"
            );
        }
        state.shutdown_tasks_async().await;
        drop(conn);
        std::fs::remove_dir_all(directory).unwrap();
    }
}

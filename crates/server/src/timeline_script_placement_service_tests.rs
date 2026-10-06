use crate::state::{AppState, ServerEvent};
use crate::timeline_script_placement::tests::{command, fixture, input};
use crate::{command_service_timeline, persistence, script_store, sqlite};

#[tokio::test]
async fn exact_placement_read_refuses_aba_and_replay_preserves_later_range_and_manual_text() {
    use crate::projection_service::{self, SelectedNodeEditorProjectionRequest};
    use eidetic_core::contracts::ObjectKind;

    let (source, project, a, b) = fixture();
    source.execute_batch("CREATE TABLE episode_structure (id INTEGER PRIMARY KEY CHECK(id = 1), template_name TEXT NOT NULL, segments_json TEXT NOT NULL);").unwrap();
    source
        .execute(
            "INSERT INTO episode_structure VALUES (1, ?1, ?2)",
            rusqlite::params![
                project.timeline.structure.template_name,
                serde_json::to_string(&project.timeline.structure.segments).unwrap()
            ],
        )
        .unwrap();
    let directory =
        std::env::temp_dir().join(format!("eidetic-exact-placement-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("project.db");
    source
        .execute("VACUUM INTO ?1", [path.to_str().unwrap()])
        .unwrap();
    persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    let conn = sqlite::open_write_connection(&path).unwrap();
    let original = input(&conn, &a);
    let node_id = eidetic_core::timeline::node::NodeId(
        uuid::Uuid::parse_str(a.source_node_id.as_deref().unwrap()).unwrap(),
    );
    let state = AppState::new().await;
    *state.project.lock() = Some(project);
    state.project_database.set_active_path(path);
    let read = async {
        projection_service::selected_node_editor_projection(
            &state,
            SelectedNodeEditorProjectionRequest {
                node_id: Some(node_id),
            },
        )
        .await
        .unwrap()
        .payload
        .node
        .unwrap()
        .range_read
        .unwrap()
    };
    let base = read.await;
    assert_eq!((base.start_ms, base.end_ms), (1000, 2000));
    assert_eq!(
        base.node_revision_event_id, None,
        "persisted history-free nodes are known reads"
    );
    for (start, end) in [(6000, 7000), (1000, 2000)] {
        command_service_timeline::set_timeline_node_range(&state, command(&a, start, end))
            .await
            .unwrap();
    }
    let before =
        crate::history_store::load_revision_summary_for_kind(&conn, ObjectKind::TimelineNode)
            .unwrap();
    let mut edit = command(&a, 8000, 9000);
    edit.payload.expected = Some(base);
    let error = command_service_timeline::set_timeline_node_range(&state, edit)
        .await
        .unwrap_err();
    assert_eq!(
        error,
        crate::backend_error::BackendError::conflict(
            "Placement edit refused: placement changed; read current placement before applying"
        )
    );
    assert_eq!(
        before,
        crate::history_store::load_revision_summary_for_kind(&conn, ObjectKind::TimelineNode)
            .unwrap()
    );
    assert_eq!(input(&conn, &a).text, original.text);
    assert_eq!(
        (input(&conn, &a).start_ms, input(&conn, &a).end_ms),
        (1000, 2000)
    );

    let current = projection_service::selected_node_editor_projection(
        &state,
        SelectedNodeEditorProjectionRequest {
            node_id: Some(node_id),
        },
    )
    .await
    .unwrap()
    .payload
    .node
    .unwrap()
    .range_read
    .unwrap();
    assert!(current.node_revision_event_id.is_some());
    let mut fresh = command(&a, 6000, 7000);
    fresh.payload.expected = Some(current);
    command_service_timeline::set_timeline_node_range(&state, fresh.clone())
        .await
        .unwrap();
    let moved = input(&conn, &a);
    assert_eq!((moved.start_ms, moved.end_ms), (6000, 7000));
    assert_eq!(
        (moved.text, moved.revision_event_id),
        (original.text.clone(), original.revision_event_id)
    );
    let document = script_store::load_document_projection(&conn, &b.document_id)
        .unwrap()
        .unwrap();
    assert!(
        document
            .segments
            .iter()
            .find(|segment| segment.segment.id == b.segment_id)
            .unwrap()
            .impact
            .as_ref()
            .unwrap()
            .needs_review
    );
    command_service_timeline::set_timeline_node_range(&state, command(&a, 7000, 8000))
        .await
        .unwrap();
    let replay = command_service_timeline::set_timeline_node_range(&state, fresh)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(replay).unwrap()["outcome"],
        "already_recorded"
    );
    assert_eq!(
        (input(&conn, &a).start_ms, input(&conn, &a).end_ms),
        (7000, 8000)
    );
    assert_eq!(input(&conn, &a).text, original.text);
    state.shutdown_tasks_async().await;
    drop(conn);
    std::fs::remove_dir_all(directory).unwrap();
}

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

use crate::state::{AppState, ServerEvent};
use crate::{command_service, project_service, projection_service, script_store, sqlite};
use eidetic_core::contracts::*;

#[tokio::test]
async fn native_manual_creation_edit_save_reopen_and_retime_reach_exact_prompt_memory() {
    let (project, first, second) = crate::script_block_create::tests::story_project();
    let directory = crate::persistence::default_project_dir()
        .join(format!("manual-authoring-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&directory).unwrap();
    let directory = directory.canonicalize().unwrap();
    let path = directory.join("project.db");
    crate::persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    let state = AppState::new().await;
    project_service::replace_active_project(&state, project, path.clone());
    let command = CommandEnvelope::new(CreateScriptBlockCommand {
        document_id: ScriptDocumentId::new("script.document.main").unwrap(),
        source_node_id: first,
        expected_start_ms: 1000,
        expected_end_ms: 2000,
        block_kind: ScriptBlockKind::Action,
        text: "  INT. CAFE — NIGHT\nMara sees 雨.\n\n".into(),
    });
    let mut events = state.events_tx.subscribe();
    let response = command_service::create_script_block(&state, command.clone())
        .await
        .unwrap();
    assert!(matches!(
        events.try_recv().unwrap(),
        ServerEvent::ScriptChanged
    ));
    command_service::create_script_block(&state, command.clone())
        .await
        .unwrap();
    assert!(events.try_recv().is_err(), "replay must not publish again");
    let block = &response.projection.payload.segments[0].blocks[0];
    let preview = crate::ai_service::preview_ai_context(&state, second.0)
        .await
        .unwrap();
    assert!(preview.user.contains(&command.payload.text));
    command_service::edit_script_block(
        &state,
        CommandEnvelope::new(EditScriptBlockCommand {
            document_id: command.payload.document_id.clone(),
            block_id: block.block.id.clone(),
            expected_revision_event_id: block.revision_event_id.unwrap(),
            text: "Mara catches the last train.\nEli waves goodbye.".into(),
        }),
    )
    .await
    .unwrap();
    let conn = sqlite::open_write_connection(&path).unwrap();
    let history = crate::timeline_postcommit_custody_tests::history(&path);
    project_service::save_project(&state, project_service::SaveProjectRequest { path: None })
        .await
        .unwrap();
    project_service::load_project(
        &state,
        project_service::LoadProjectRequest {
            path: path.display().to_string(),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        crate::timeline_postcommit_custody_tests::history(&path),
        history
    );
    let preview = crate::ai_service::preview_ai_context(&state, second.0)
        .await
        .unwrap();
    assert!(
        preview
            .user
            .contains("Mara catches the last train.\nEli waves goodbye.")
    );
    assert!(!preview.user.contains("Mara sees 雨."));
    crate::command_service_timeline::set_timeline_node_range(
        &state,
        CommandEnvelope::new(SetTimelineNodeRangeCommand {
            node_id: first,
            start_ms: 6000,
            end_ms: 7000,
        }),
    )
    .await
    .unwrap();
    let projected = projection_service::script_document_projection(
        &state,
        projection_service::ScriptDocumentProjectionRequest {
            document_id: command.payload.document_id.clone(),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        (
            projected.payload.segments[0].segment.start_ms,
            projected.payload.segments[0].segment.end_ms
        ),
        (6000, 7000)
    );
    assert_eq!(
        projected.payload.segments[0].blocks[0].block.text,
        "Mara catches the last train.\nEli waves goodbye."
    );
    assert_eq!(
        script_store::load_document_projection_envelope(&conn, &command.payload.document_id)
            .unwrap()
            .unwrap(),
        projected
    );
    state.shutdown_tasks_async().await;
    drop(conn);
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn queued_manual_creation_refuses_project_replacement_without_writing_script_or_history() {
    let (project, first, _) = crate::script_block_create::tests::story_project();
    let directory =
        std::env::temp_dir().join(format!("eidetic-queued-authoring-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("project.db");
    crate::persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    let state = AppState::new().await;
    project_service::replace_active_project(&state, project.clone(), path.clone());
    let held = state.project_session_gate.lock().await;
    let mut creation = Box::pin(command_service::create_script_block(
        &state,
        CommandEnvelope::new(CreateScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            source_node_id: first,
            expected_start_ms: 1000,
            expected_end_ms: 2000,
            block_kind: ScriptBlockKind::Action,
            text: "Admitted to original session".into(),
        }),
    ));
    use std::future::Future;
    assert!(
        creation
            .as_mut()
            .poll(&mut std::task::Context::from_waker(
                futures::task::noop_waker_ref()
            ))
            .is_pending()
    );
    project_service::replace_active_project(&state, project, path.clone());
    drop(held);
    assert!(matches!(
        creation.await.unwrap_err(),
        crate::backend_error::BackendError::Conflict(_)
    ));
    let conn = sqlite::open_write_connection(&path).unwrap();
    assert!(
        script_store::load_document_projection(
            &conn,
            &ScriptDocumentId::new("script.document.main").unwrap()
        )
        .unwrap()
        .is_none()
    );
    crate::history_store::create_schema(&conn).unwrap();
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM commands", [], |row| row
            .get::<_, u64>(0))
            .unwrap(),
        0
    );
    state.shutdown_tasks_async().await;
    drop(conn);
    std::fs::remove_dir_all(directory).unwrap();
}

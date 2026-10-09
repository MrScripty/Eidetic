use super::*;
use eidetic_core::contracts::*;

pub(super) fn removal(
    conn: &Connection,
    block: &SetScriptBlockCommand,
) -> CommandEnvelope<RemoveScriptBlockCommand> {
    let projection = script_store::load_document_projection(conn, &block.document_id)
        .unwrap()
        .unwrap();
    let current = projection
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|candidate| candidate.block.id == block.block_id)
        .unwrap();
    CommandEnvelope::new(RemoveScriptBlockCommand {
        document_id: block.document_id.clone(),
        block_id: block.block_id.clone(),
        expected_revision_event_id: current.revision_event_id.unwrap(),
    })
}

#[test]
fn removal_retains_exact_history_other_blocks_and_placement_on_reopen_and_replay() {
    let (mut conn, _, a, _, _) = crate::script_impact_review::tests::fixture();
    let before = script_store::load_document_projection(&conn, &a.document_id)
        .unwrap()
        .unwrap();
    let command = removal(&conn, &a);
    let (outcome, after) = apply_remove_script_block(&mut conn, &command, 30).unwrap();
    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(after.payload.document, before.document);
    assert_eq!(
        after.payload.segments[0].segment,
        before.segments[0].segment
    );
    assert!(after.payload.segments[0].blocks.is_empty());
    for (after, before) in after.payload.segments[1..]
        .iter()
        .zip(&before.segments[1..])
    {
        assert_eq!(after.segment, before.segment);
        assert_eq!(after.blocks, before.blocks);
    }
    let revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::ScriptBlock,
        a.block_id.as_str(),
    )
    .unwrap();
    let deleted = revisions.last().unwrap();
    assert_eq!(deleted.operation, RevisionOperation::Delete);
    assert_eq!(
        deleted.fields[0].old_value,
        Some(FieldValue::Text(
            "  A now carries a blue umbrella — 雨\n\n".into()
        ))
    );
    assert_eq!(deleted.fields[0].new_value, None);
    let retained: String = conn
        .query_row(
            "SELECT text FROM script_blocks WHERE id=?1 AND deleted_event_id IS NOT NULL",
            [a.block_id.as_str()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(retained, "  A now carries a blue umbrella — 雨\n\n");
    let spans: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM script_spans WHERE block_id=?1 AND deleted_event_id IS NULL",
            [a.block_id.as_str()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(spans, 0);
    let path = std::env::temp_dir().join(format!("eidetic-remove-{}.db", uuid::Uuid::new_v4()));
    conn.execute("VACUUM INTO ?1", [path.to_str().unwrap()])
        .unwrap();
    let mut reopened = Connection::open(&path).unwrap();
    assert_eq!(
        apply_remove_script_block(&mut reopened, &command, 40).unwrap(),
        (RecordChangeOutcome::AlreadyRecorded, after)
    );
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn stale_aba_and_later_lock_refuse_without_any_history_write() {
    let (mut conn, _, a, _, _) = crate::script_impact_review::tests::fixture();
    let command = removal(&conn, &a);
    crate::script_impact_review::tests::edit(&mut conn, &a, "Changed");
    crate::script_impact_review::tests::edit(
        &mut conn,
        &a,
        "  A now carries a blue umbrella — 雨\n\n",
    );
    let before = projection(&conn, &command).unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM commands", [], |r| r.get(0))
        .unwrap();
    assert!(
        apply_remove_script_block(&mut conn, &command, 40)
            .unwrap_err()
            .to_string()
            .contains("changed")
    );
    assert_eq!(projection(&conn, &command).unwrap(), before);
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM commands", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        count
    );
    let current = removal(&conn, &a);
    crate::script_document_command::apply_set_script_lock(
        &mut conn,
        &CommandEnvelope::new(SetScriptLockCommand {
            lock_id: ScriptLockId::new("remove.lock").unwrap(),
            span_id: ScriptSpanId::new(format!("{}.span.main", a.block_id.as_str())).unwrap(),
            reason: "Keep exact words".into(),
        }),
        45,
    )
    .unwrap();
    let locked = projection(&conn, &current).unwrap();
    assert!(
        apply_remove_script_block(&mut conn, &current, 50)
            .unwrap_err()
            .to_string()
            .contains("locked")
    );
    assert_eq!(projection(&conn, &current).unwrap(), locked);
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM commands", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        count + 1
    );
}

#[tokio::test]
async fn public_removal_replay_save_reopen_and_queued_session_retirement_preserve_canon() {
    use crate::{
        command_service, project_service, sqlite,
        state::{AppState, ServerEvent},
    };
    let (project, first, _) = crate::script_block_create::tests::story_project();
    let directory = crate::persistence::default_project_dir()
        .join(format!("eidetic-remove-service-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&directory).unwrap();
    let directory = directory.canonicalize().unwrap();
    let path = directory.join("project.db");
    crate::persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    let state = AppState::new().await;
    project_service::replace_active_project(&state, project.clone(), path.clone());
    let created = command_service::create_script_block(
        &state,
        CommandEnvelope::new(CreateScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            source_node_id: first,
            expected_start_ms: 1000,
            expected_end_ms: 2000,
            block_kind: ScriptBlockKind::Action,
            text: "  Exact saved words — 雨\n\n".into(),
        }),
    )
    .await
    .unwrap();
    let created_projection: ProjectionEnvelope<ScriptDocumentProjection> =
        serde_json::from_value(serde_json::to_value(&created).unwrap()["projection"].clone())
            .unwrap();
    let block = &created_projection.payload.segments[0].blocks[0];
    let command = CommandEnvelope::new(RemoveScriptBlockCommand {
        document_id: created_projection.payload.document.id.clone(),
        block_id: block.block.id.clone(),
        expected_revision_event_id: block.revision_event_id.unwrap(),
    });
    let held = state.project_session_gate.lock().await;
    let mut queued = Box::pin(command_service::remove_script_block(
        &state,
        command.clone(),
    ));
    use std::future::Future;
    assert!(
        queued
            .as_mut()
            .poll(&mut std::task::Context::from_waker(
                futures::task::noop_waker_ref()
            ))
            .is_pending()
    );
    project_service::replace_active_project(&state, project, path.clone());
    drop(held);
    assert!(matches!(
        queued.await.unwrap_err(),
        crate::backend_error::BackendError::Conflict(_)
    ));
    let conn = sqlite::open_write_connection(&path).unwrap();
    assert_eq!(
        script_store::load_document_projection_envelope(&conn, &command.payload.document_id)
            .unwrap()
            .unwrap(),
        created_projection
    );
    let mut events = state.events_tx.subscribe();
    let removed = command_service::remove_script_block(&state, command.clone())
        .await
        .unwrap();
    assert!(matches!(
        events.try_recv().unwrap(),
        ServerEvent::ScriptChanged
    ));
    let replay = command_service::remove_script_block(&state, command.clone())
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(replay).unwrap()["outcome"],
        "already_recorded"
    );
    assert!(events.try_recv().is_err());
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
    let replay = command_service::remove_script_block(&state, command)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(replay).unwrap()["projection"],
        serde_json::to_value(removed).unwrap()["projection"]
    );
    state.shutdown_tasks_async().await;
    drop(conn);
    std::fs::remove_dir_all(directory).unwrap();
}

use super::*;
use eidetic_core::contracts::{
    ProjectionVersion, ScriptBlockId, ScriptBlockKind, ScriptDocumentId, ScriptLockId,
    ScriptSegmentId, ScriptSegmentStatus, ScriptSpanId, SetScriptLockCommand,
};

fn memory_connection() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    script_store::create_schema(&conn).unwrap();
    conn
}

fn block_command(text: &str) -> CommandEnvelope<SetScriptBlockCommand> {
    CommandEnvelope::new(SetScriptBlockCommand {
        document_id: ScriptDocumentId::new("script.document.main").unwrap(),
        document_title: "Pilot".to_string(),
        document_sort_order: 0,
        segment_id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
        source_node_id: Some("node.beat.opening".to_string()),
        segment_start_ms: 1_000,
        segment_end_ms: 5_000,
        segment_status: ScriptSegmentStatus::Current,
        segment_sort_order: 1,
        block_id: ScriptBlockId::new("script.block.action-1").unwrap(),
        block_kind: ScriptBlockKind::Action,
        text: text.to_string(),
        sort_order: 2,
    })
}

fn lock_command(reason: &str) -> CommandEnvelope<SetScriptLockCommand> {
    CommandEnvelope::new(SetScriptLockCommand {
        lock_id: ScriptLockId::new("script.lock.action-1").unwrap(),
        span_id: ScriptSpanId::new("script.block.action-1.span.main").unwrap(),
        reason: reason.to_string(),
    })
}

#[test]
fn set_script_block_records_history_and_projection() {
    let mut conn = memory_connection();
    let command = block_command("Ada enters with a wet umbrella.");

    let (outcome, projection) = apply_set_script_block(&mut conn, &command, 100).unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(projection.version, ProjectionVersion(5));
    assert_eq!(projection.payload.document.title, "Pilot");
    assert_eq!(projection.payload.segments.len(), 1);
    assert_eq!(projection.payload.segments[0].blocks.len(), 1);
    assert_eq!(
        projection.payload.segments[0].blocks[0].block.text,
        "Ada enters with a wet umbrella."
    );
    assert_eq!(projection.payload.segments[0].blocks[0].spans.len(), 1);
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 4);
    assert_eq!(table_count(&conn, "script_documents"), 1);
    assert_eq!(table_count(&conn, "script_segments"), 1);
    assert_eq!(table_count(&conn, "script_blocks"), 1);
    assert_eq!(table_count(&conn, "script_spans"), 1);
}

#[test]
fn duplicate_set_script_block_command_is_idempotent() {
    let mut conn = memory_connection();
    let command = block_command("Ada enters with a wet umbrella.");

    let (first, _) = apply_set_script_block(&mut conn, &command, 100).unwrap();
    let (second, projection) = apply_set_script_block(&mut conn, &command, 100).unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(projection.payload.segments[0].blocks.len(), 1);
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 4);
    assert_eq!(table_count(&conn, "script_blocks"), 1);
}

#[test]
fn set_script_block_rejects_invalid_segment_range_without_writes() {
    let mut conn = memory_connection();
    let mut command = block_command("Ada enters with a wet umbrella.");
    command.payload.segment_start_ms = 5_000;
    command.payload.segment_end_ms = 1_000;

    let error = apply_set_script_block(&mut conn, &command, 100).unwrap_err();

    assert!(matches!(
        error,
        ScriptDocumentCommandError::InvalidCommand(_)
    ));
    assert_eq!(table_count(&conn, "commands"), 0);
    assert_eq!(table_count(&conn, "object_revisions"), 0);
    assert_eq!(table_count(&conn, "script_documents"), 0);
}

#[test]
fn set_script_lock_records_history_and_projection() {
    let mut conn = memory_connection();
    let block = block_command("Ada enters with a wet umbrella.");
    apply_set_script_block(&mut conn, &block, 100).unwrap();
    let command = lock_command("User approved wording.");

    let (outcome, projection) = apply_set_script_lock(&mut conn, &command, 200).unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(projection.version, ProjectionVersion(6));
    assert_eq!(
        projection.payload.segments[0].blocks[0].locks[0].reason,
        "User approved wording."
    );
    assert_eq!(table_count(&conn, "commands"), 2);
    assert_eq!(table_count(&conn, "object_revisions"), 5);
    assert_eq!(table_count(&conn, "script_locks"), 1);
}

#[test]
fn duplicate_set_script_lock_command_is_idempotent() {
    let mut conn = memory_connection();
    let block = block_command("Ada enters with a wet umbrella.");
    apply_set_script_block(&mut conn, &block, 100).unwrap();
    let command = lock_command("User approved wording.");

    let (first, _) = apply_set_script_lock(&mut conn, &command, 200).unwrap();
    let (second, projection) = apply_set_script_lock(&mut conn, &command, 200).unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(projection.payload.segments[0].blocks[0].locks.len(), 1);
    assert_eq!(table_count(&conn, "commands"), 2);
    assert_eq!(table_count(&conn, "object_revisions"), 5);
    assert_eq!(table_count(&conn, "script_locks"), 1);
}

#[test]
fn set_script_lock_rejects_missing_span_without_writes() {
    let mut conn = memory_connection();
    let command = lock_command("User approved wording.");

    let error = apply_set_script_lock(&mut conn, &command, 200).unwrap_err();

    assert!(matches!(
        error,
        ScriptDocumentCommandError::InvalidCommand(_)
    ));
    assert_eq!(table_count(&conn, "commands"), 0);
    assert_eq!(table_count(&conn, "object_revisions"), 0);
    assert_eq!(table_count(&conn, "script_locks"), 0);
}

fn table_count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}

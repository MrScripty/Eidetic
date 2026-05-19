use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, ScriptBlock, ScriptBlockId, ScriptBlockKind,
    ScriptDocument, ScriptDocumentId, ScriptLock, ScriptLockId, ScriptSegment, ScriptSegmentId,
    ScriptSegmentStatus, ScriptSpan, ScriptSpanId, ScriptSpanProvenance,
};

use super::delete_omitted_segment_blocks_in_transaction;
use crate::history_store;
use crate::script_store;

#[derive(Debug, serde::Serialize)]
struct TestCommand;

#[test]
fn deletes_omitted_segment_blocks_spans_and_locks() {
    let mut conn = memory_connection();
    let event = seed_segment_with_two_blocks(&mut conn);
    let command = CommandEnvelope::new(TestCommand);
    let replacement_event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        "replace segment blocks",
    );

    history_store::record_change_with(
        &mut conn,
        &command,
        "test.replace_segment_blocks",
        &replacement_event,
        &[],
        |tx| {
            delete_omitted_segment_blocks_in_transaction(
                tx,
                &ScriptSegmentId::new("script.segment.beat-1").unwrap(),
                &[ScriptBlockId::new("script.block.keep").unwrap()],
                replacement_event.id,
            )
        },
    )
    .unwrap();

    let projection = script_store::load_document_projection(
        &conn,
        &ScriptDocumentId::new("script.document.main").unwrap(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(event.summary, "seed script blocks");
    assert_eq!(projection.segments[0].blocks.len(), 1);
    assert_eq!(
        projection.segments[0].blocks[0].block.id.as_str(),
        "script.block.keep"
    );
    assert_eq!(active_row_count(&conn, "script_blocks"), 1);
    assert_eq!(active_row_count(&conn, "script_spans"), 1);
    assert_eq!(active_row_count(&conn, "script_locks"), 0);
}

#[test]
fn deletes_all_segment_blocks_when_none_are_retained() {
    let mut conn = memory_connection();
    seed_segment_with_two_blocks(&mut conn);
    let command = CommandEnvelope::new(TestCommand);
    let replacement_event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        "clear segment blocks",
    );

    history_store::record_change_with(
        &mut conn,
        &command,
        "test.clear_segment_blocks",
        &replacement_event,
        &[],
        |tx| {
            delete_omitted_segment_blocks_in_transaction(
                tx,
                &ScriptSegmentId::new("script.segment.beat-1").unwrap(),
                &[],
                replacement_event.id,
            )
        },
    )
    .unwrap();

    let projection = script_store::load_document_projection(
        &conn,
        &ScriptDocumentId::new("script.document.main").unwrap(),
    )
    .unwrap()
    .unwrap();

    assert!(projection.segments[0].blocks.is_empty());
    assert_eq!(active_row_count(&conn, "script_blocks"), 0);
    assert_eq!(active_row_count(&conn, "script_spans"), 0);
    assert_eq!(active_row_count(&conn, "script_locks"), 0);
}

fn memory_connection() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    script_store::create_schema(&conn).unwrap();
    conn
}

fn seed_segment_with_two_blocks(conn: &mut rusqlite::Connection) -> ChangeEvent {
    let command = CommandEnvelope::new(TestCommand);
    let event = ChangeEvent::new(command.id, ChangeEventKind::UserEdit, "seed script blocks");
    let document = ScriptDocument {
        id: ScriptDocumentId::new("script.document.main").unwrap(),
        title: "Pilot".to_string(),
        sort_order: 0,
    };
    let segment = ScriptSegment {
        id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
        document_id: document.id.clone(),
        source_node_id: Some("node.beat.opening".to_string()),
        start_ms: 1_000,
        end_ms: 5_000,
        status: ScriptSegmentStatus::Current,
        sort_order: 1,
    };
    let keep_block = test_block("script.block.keep", &segment);
    let delete_block = test_block("script.block.delete", &segment);
    let keep_span = test_span("script.span.keep", &keep_block);
    let delete_span = test_span("script.span.delete", &delete_block);
    let lock = ScriptLock {
        id: ScriptLockId::new("script.lock.delete").unwrap(),
        span_id: delete_span.id.clone(),
        reason: "Approved old wording".to_string(),
    };

    history_store::record_change_with(
        conn,
        &command,
        "test.seed_script_blocks",
        &event,
        &[],
        |tx| {
            script_store::upsert_document_in_transaction(tx, &document, event.id)?;
            script_store::upsert_segment_in_transaction(tx, &segment, event.id)?;
            script_store::upsert_block_in_transaction(tx, &keep_block, event.id)?;
            script_store::upsert_block_in_transaction(tx, &delete_block, event.id)?;
            script_store::upsert_span_in_transaction(tx, &keep_span, event.id)?;
            script_store::upsert_span_in_transaction(tx, &delete_span, event.id)?;
            script_store::upsert_lock_in_transaction(tx, &lock, event.id)?;
            Ok(())
        },
    )
    .unwrap();
    event
}

fn test_block(id: &str, segment: &ScriptSegment) -> ScriptBlock {
    ScriptBlock {
        id: ScriptBlockId::new(id).unwrap(),
        segment_id: segment.id.clone(),
        block_kind: ScriptBlockKind::Action,
        text: "Ada enters with an umbrella.".to_string(),
        sort_order: 1,
    }
}

fn test_span(id: &str, block: &ScriptBlock) -> ScriptSpan {
    ScriptSpan {
        id: ScriptSpanId::new(id).unwrap(),
        block_id: block.id.clone(),
        start_byte: 0,
        end_byte: 28,
        provenance: ScriptSpanProvenance::AiGenerated,
    }
}

fn active_row_count(conn: &rusqlite::Connection, table: &str) -> i64 {
    conn.query_row(
        &format!("SELECT COUNT(*) FROM {table} WHERE deleted_event_id IS NULL"),
        [],
        |row| row.get(0),
    )
    .unwrap()
}

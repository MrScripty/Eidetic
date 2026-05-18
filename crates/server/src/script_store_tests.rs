use super::*;
use eidetic_core::contracts::{
    ChangeEvent, ChangeEventId, ChangeEventKind, CommandEnvelope, ObjectKind, ObjectRevision,
    ProjectionVersion, RevisionOperation, ScriptBlockKind, ScriptLockId, ScriptSegmentStatus,
    ScriptSpanId, ScriptSpanProvenance,
};

use crate::history_store::{self, HistoryStoreError};

#[derive(Debug, serde::Serialize)]
struct TestCommand;

fn memory_connection() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    create_schema(&conn).unwrap();
    conn
}

#[test]
fn missing_script_document_projection_returns_none() {
    let conn = memory_connection();
    let document_id = ScriptDocumentId::new("script.document.missing").unwrap();

    let projection = load_document_projection_envelope(&conn, &document_id).unwrap();

    assert!(projection.is_none());
}

#[test]
fn script_document_projection_includes_segments_blocks_spans_and_locks() {
    let mut conn = memory_connection();
    let document = test_document();
    let segment = test_segment(&document);
    let block = test_block(&segment);
    let span = test_span(&block);
    let lock = test_lock(&span);
    let command = CommandEnvelope::new(TestCommand);
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        "seed script document",
    );
    let revisions = vec![
        revision(ObjectKind::ScriptDocument, document.id.as_str(), event.id),
        revision(ObjectKind::ScriptSegment, segment.id.as_str(), event.id),
        revision(ObjectKind::ScriptBlock, block.id.as_str(), event.id),
        revision(ObjectKind::ScriptSpan, span.id.as_str(), event.id),
        revision(ObjectKind::ScriptLock, lock.id.as_str(), event.id),
    ];

    history_store::record_change_with(
        &mut conn,
        &command,
        "test.seed_script",
        &event,
        &revisions,
        |tx| {
            upsert_document_in_transaction(tx, &document, event.id)?;
            upsert_segment_in_transaction(tx, &segment, event.id)?;
            upsert_block_in_transaction(tx, &block, event.id)?;
            upsert_span_in_transaction(tx, &span, event.id)?;
            upsert_lock_in_transaction(tx, &lock, event.id)?;
            Ok(())
        },
    )
    .unwrap();

    let projection = load_document_projection_envelope(&conn, &document.id)
        .unwrap()
        .unwrap();

    assert_eq!(projection.version, ProjectionVersion(6));
    assert_eq!(projection.change_event_id, Some(event.id));
    assert_eq!(projection.payload.document.title, "Pilot");
    assert_eq!(projection.payload.segments.len(), 1);
    assert_eq!(
        projection.payload.segments[0]
            .segment
            .source_node_id
            .as_deref(),
        Some("node.beat.opening")
    );
    assert_eq!(projection.payload.segments[0].blocks.len(), 1);
    assert_eq!(
        projection.payload.segments[0].blocks[0].block.text,
        "INT. KITCHEN - MORNING"
    );
    assert_eq!(projection.payload.segments[0].blocks[0].spans.len(), 1);
    assert_eq!(projection.payload.segments[0].blocks[0].locks.len(), 1);
}

#[test]
fn invalid_script_span_range_rejects_write() {
    let conn = memory_connection();
    let block = ScriptBlock {
        id: ScriptBlockId::new("script.block.heading-1").unwrap(),
        segment_id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
        block_kind: ScriptBlockKind::SceneHeading,
        text: "INT. KITCHEN - MORNING".to_string(),
        sort_order: 1,
    };
    let span = ScriptSpan {
        id: ScriptSpanId::new("script.span.heading-1").unwrap(),
        block_id: block.id,
        start_byte: 12,
        end_byte: 3,
        provenance: ScriptSpanProvenance::UserEdited,
    };
    let tx = conn.unchecked_transaction().unwrap();

    let error = upsert_span_in_transaction(&tx, &span, ChangeEventId::new()).unwrap_err();

    assert!(matches!(error, HistoryStoreError::InvalidValue(_)));
}

fn test_document() -> ScriptDocument {
    ScriptDocument {
        id: ScriptDocumentId::new("script.document.main").unwrap(),
        title: "Pilot".to_string(),
        sort_order: 0,
    }
}

fn test_segment(document: &ScriptDocument) -> ScriptSegment {
    ScriptSegment {
        id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
        document_id: document.id.clone(),
        source_node_id: Some("node.beat.opening".to_string()),
        start_ms: 1_000,
        end_ms: 5_000,
        status: ScriptSegmentStatus::Current,
        sort_order: 1,
    }
}

fn test_block(segment: &ScriptSegment) -> ScriptBlock {
    ScriptBlock {
        id: ScriptBlockId::new("script.block.heading-1").unwrap(),
        segment_id: segment.id.clone(),
        block_kind: ScriptBlockKind::SceneHeading,
        text: "INT. KITCHEN - MORNING".to_string(),
        sort_order: 1,
    }
}

fn test_span(block: &ScriptBlock) -> ScriptSpan {
    ScriptSpan {
        id: ScriptSpanId::new("script.span.heading-1").unwrap(),
        block_id: block.id.clone(),
        start_byte: 0,
        end_byte: 22,
        provenance: ScriptSpanProvenance::AiGenerated,
    }
}

fn test_lock(span: &ScriptSpan) -> ScriptLock {
    ScriptLock {
        id: ScriptLockId::new("script.lock.heading-1").unwrap(),
        span_id: span.id.clone(),
        reason: "User approved location wording".to_string(),
    }
}

fn revision(object_kind: ObjectKind, object_id: &str, event_id: ChangeEventId) -> ObjectRevision {
    ObjectRevision::new(object_kind, object_id, event_id, RevisionOperation::Create)
}

use eidetic_core::contracts::{
    ChangeEventId, ObjectKind, ProjectionEnvelope, ProjectionVersion, ScriptBlock, ScriptBlockId,
    ScriptBlockProjection, ScriptDocument, ScriptDocumentId, ScriptDocumentProjection, ScriptLock,
    ScriptSegment, ScriptSegmentId, ScriptSegmentProjection, ScriptSpan,
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::history_store::{self, HistoryStoreError};
use crate::script_store_codec::{
    encode_block_kind, encode_object_kind, encode_segment_status, encode_span_provenance,
    parse_uuid, row_to_block, row_to_document, row_to_lock, row_to_segment, row_to_span, to_i64,
};
use crate::script_store_schema;

pub(crate) fn create_schema(conn: &Connection) -> Result<(), HistoryStoreError> {
    script_store_schema::create_schema(conn)
}

pub(crate) fn upsert_document_in_transaction(
    tx: &Transaction<'_>,
    document: &ScriptDocument,
    event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    tx.execute(
        "INSERT INTO script_documents (
            id, title, sort_order, created_event_id, updated_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?4)
         ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            sort_order = excluded.sort_order,
            updated_event_id = excluded.updated_event_id,
            deleted_event_id = NULL",
        params![
            document.id.as_str(),
            document.title,
            document.sort_order as i64,
            event_id.0.to_string(),
        ],
    )?;
    Ok(())
}

pub(crate) fn upsert_segment_in_transaction(
    tx: &Transaction<'_>,
    segment: &ScriptSegment,
    event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    tx.execute(
        "INSERT INTO script_segments (
            id, document_id, source_node_id, start_ms, end_ms, status, sort_order,
            created_event_id, updated_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
         ON CONFLICT(id) DO UPDATE SET
            document_id = excluded.document_id,
            source_node_id = excluded.source_node_id,
            start_ms = excluded.start_ms,
            end_ms = excluded.end_ms,
            status = excluded.status,
            sort_order = excluded.sort_order,
            updated_event_id = excluded.updated_event_id,
            deleted_event_id = NULL",
        params![
            segment.id.as_str(),
            segment.document_id.as_str(),
            segment.source_node_id.as_deref(),
            to_i64(segment.start_ms, "start_ms")?,
            to_i64(segment.end_ms, "end_ms")?,
            encode_segment_status(&segment.status),
            segment.sort_order as i64,
            event_id.0.to_string(),
        ],
    )?;
    Ok(())
}

pub(crate) fn upsert_block_in_transaction(
    tx: &Transaction<'_>,
    block: &ScriptBlock,
    event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    tx.execute(
        "INSERT INTO script_blocks (
            id, segment_id, block_kind, text, sort_order, updated_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
            segment_id = excluded.segment_id,
            block_kind = excluded.block_kind,
            text = excluded.text,
            sort_order = excluded.sort_order,
            updated_event_id = excluded.updated_event_id,
            deleted_event_id = NULL",
        params![
            block.id.as_str(),
            block.segment_id.as_str(),
            encode_block_kind(&block.block_kind),
            block.text,
            block.sort_order as i64,
            event_id.0.to_string(),
        ],
    )?;
    Ok(())
}

pub(crate) fn upsert_span_in_transaction(
    tx: &Transaction<'_>,
    span: &ScriptSpan,
    event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    if span.start_byte > span.end_byte {
        return Err(HistoryStoreError::InvalidValue(
            "script span start_byte must be <= end_byte".to_string(),
        ));
    }

    tx.execute(
        "INSERT INTO script_spans (
            id, block_id, start_byte, end_byte, provenance, updated_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
            block_id = excluded.block_id,
            start_byte = excluded.start_byte,
            end_byte = excluded.end_byte,
            provenance = excluded.provenance,
            updated_event_id = excluded.updated_event_id,
            deleted_event_id = NULL",
        params![
            span.id.as_str(),
            span.block_id.as_str(),
            span.start_byte as i64,
            span.end_byte as i64,
            encode_span_provenance(&span.provenance),
            event_id.0.to_string(),
        ],
    )?;
    Ok(())
}

pub(crate) fn load_document_projection(
    conn: &Connection,
    document_id: &ScriptDocumentId,
) -> Result<Option<ScriptDocumentProjection>, HistoryStoreError> {
    let document = conn
        .query_row(
            "SELECT id, title, sort_order
             FROM script_documents
             WHERE id = ?1 AND deleted_event_id IS NULL",
            [document_id.as_str()],
            row_to_document,
        )
        .optional()?;

    let Some(document) = document else {
        return Ok(None);
    };
    let segments = load_segments(conn, document_id)?;
    Ok(Some(ScriptDocumentProjection { document, segments }))
}

pub(crate) fn load_document_projection_envelope(
    conn: &Connection,
    document_id: &ScriptDocumentId,
) -> Result<Option<ProjectionEnvelope<ScriptDocumentProjection>>, HistoryStoreError> {
    let Some(projection) = load_document_projection(conn, document_id)? else {
        return Ok(None);
    };
    let summary = load_document_revision_summary(conn, document_id)?;

    match summary.latest_change_event_id {
        Some(change_event_id) => Ok(Some(ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            change_event_id,
            projection,
        ))),
        None => Ok(Some(ProjectionEnvelope::initial(projection))),
    }
}

fn load_segments(
    conn: &Connection,
    document_id: &ScriptDocumentId,
) -> Result<Vec<ScriptSegmentProjection>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT id, document_id, source_node_id, start_ms, end_ms, status, sort_order
         FROM script_segments
         WHERE document_id = ?1 AND deleted_event_id IS NULL
         ORDER BY sort_order ASC, start_ms ASC, id ASC",
    )?;
    let rows = statement.query_map([document_id.as_str()], row_to_segment)?;

    let mut segments = Vec::new();
    for row in rows {
        let segment = row?;
        let blocks = load_blocks(conn, &segment.id)?;
        segments.push(ScriptSegmentProjection { segment, blocks });
    }
    Ok(segments)
}

fn load_blocks(
    conn: &Connection,
    segment_id: &ScriptSegmentId,
) -> Result<Vec<ScriptBlockProjection>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT id, segment_id, block_kind, text, sort_order
         FROM script_blocks
         WHERE segment_id = ?1 AND deleted_event_id IS NULL
         ORDER BY sort_order ASC, id ASC",
    )?;
    let rows = statement.query_map([segment_id.as_str()], row_to_block)?;

    let mut blocks = Vec::new();
    for row in rows {
        let block = row?;
        let spans = load_spans(conn, &block.id)?;
        let locks = load_locks_for_block(conn, &block.id)?;
        blocks.push(ScriptBlockProjection {
            block,
            spans,
            locks,
        });
    }
    Ok(blocks)
}

fn load_spans(
    conn: &Connection,
    block_id: &ScriptBlockId,
) -> Result<Vec<ScriptSpan>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT id, block_id, start_byte, end_byte, provenance
         FROM script_spans
         WHERE block_id = ?1 AND deleted_event_id IS NULL
         ORDER BY start_byte ASC, end_byte ASC, id ASC",
    )?;
    let rows = statement.query_map([block_id.as_str()], row_to_span)?;

    let mut spans = Vec::new();
    for row in rows {
        spans.push(row?);
    }
    Ok(spans)
}

fn load_locks_for_block(
    conn: &Connection,
    block_id: &ScriptBlockId,
) -> Result<Vec<ScriptLock>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT locks.id, locks.span_id, locks.reason
         FROM script_locks locks
         INNER JOIN script_spans spans ON spans.id = locks.span_id
         WHERE spans.block_id = ?1
            AND spans.deleted_event_id IS NULL
            AND locks.deleted_event_id IS NULL
         ORDER BY spans.start_byte ASC, locks.id ASC",
    )?;
    let rows = statement.query_map([block_id.as_str()], row_to_lock)?;

    let mut locks = Vec::new();
    for row in rows {
        locks.push(row?);
    }
    Ok(locks)
}

fn load_document_revision_summary(
    conn: &Connection,
    document_id: &ScriptDocumentId,
) -> Result<history_store::RevisionSummary, HistoryStoreError> {
    let script_document = encode_object_kind(&ObjectKind::ScriptDocument)?;
    let script_segment = encode_object_kind(&ObjectKind::ScriptSegment)?;
    let script_block = encode_object_kind(&ObjectKind::ScriptBlock)?;
    let script_span = encode_object_kind(&ObjectKind::ScriptSpan)?;
    let script_lock = encode_object_kind(&ObjectKind::ScriptLock)?;
    let revision_count = conn.query_row(
        "SELECT COUNT(*)
         FROM object_revisions
         WHERE (object_kind = ?1 AND object_id = ?2)
            OR (
                object_kind = ?3
                AND object_id IN (
                    SELECT id FROM script_segments WHERE document_id = ?2
                )
            )
            OR (
                object_kind = ?4
                AND object_id IN (
                    SELECT blocks.id
                    FROM script_blocks blocks
                    INNER JOIN script_segments segments ON segments.id = blocks.segment_id
                    WHERE segments.document_id = ?2
                )
            )
            OR (
                object_kind = ?5
                AND object_id IN (
                    SELECT spans.id
                    FROM script_spans spans
                    INNER JOIN script_blocks blocks ON blocks.id = spans.block_id
                    INNER JOIN script_segments segments ON segments.id = blocks.segment_id
                    WHERE segments.document_id = ?2
                )
            )
            OR (
                object_kind = ?6
                AND object_id IN (
                    SELECT locks.id
                    FROM script_locks locks
                    INNER JOIN script_spans spans ON spans.id = locks.span_id
                    INNER JOIN script_blocks blocks ON blocks.id = spans.block_id
                    INNER JOIN script_segments segments ON segments.id = blocks.segment_id
                    WHERE segments.document_id = ?2
                )
            )",
        params![
            script_document.as_str(),
            document_id.as_str(),
            script_segment.as_str(),
            script_block.as_str(),
            script_span.as_str(),
            script_lock.as_str(),
        ],
        |row| row.get::<_, i64>(0),
    )?;
    let latest_change_event_id = conn
        .query_row(
            "SELECT change_event_id
             FROM object_revisions
             WHERE (object_kind = ?1 AND object_id = ?2)
                OR (
                    object_kind = ?3
                    AND object_id IN (
                        SELECT id FROM script_segments WHERE document_id = ?2
                    )
                )
                OR (
                    object_kind = ?4
                    AND object_id IN (
                        SELECT blocks.id
                        FROM script_blocks blocks
                        INNER JOIN script_segments segments ON segments.id = blocks.segment_id
                        WHERE segments.document_id = ?2
                    )
                )
                OR (
                    object_kind = ?5
                    AND object_id IN (
                        SELECT spans.id
                        FROM script_spans spans
                        INNER JOIN script_blocks blocks ON blocks.id = spans.block_id
                        INNER JOIN script_segments segments ON segments.id = blocks.segment_id
                        WHERE segments.document_id = ?2
                    )
                )
                OR (
                    object_kind = ?6
                    AND object_id IN (
                        SELECT locks.id
                        FROM script_locks locks
                        INNER JOIN script_spans spans ON spans.id = locks.span_id
                        INNER JOIN script_blocks blocks ON blocks.id = spans.block_id
                        INNER JOIN script_segments segments ON segments.id = blocks.segment_id
                        WHERE segments.document_id = ?2
                    )
                )
             ORDER BY rowid DESC
             LIMIT 1",
            params![
                script_document,
                document_id.as_str(),
                script_segment,
                script_block,
                script_span,
                script_lock,
            ],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .map(|id| parse_uuid(&id).map(ChangeEventId))
        .transpose()?;

    Ok(history_store::RevisionSummary {
        revision_count: u64::try_from(revision_count).unwrap_or_default(),
        latest_change_event_id,
    })
}

#[cfg(test)]
#[path = "script_store_tests.rs"]
mod tests;

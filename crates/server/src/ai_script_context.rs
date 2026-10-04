use eidetic_core::contracts::{
    ChangeEventId, ScriptBlockId, ScriptContextBlock, ScriptDocumentId, ScriptSegmentId,
};
use eidetic_core::timeline::node::NodeId;
use rusqlite::{Connection, params};

use crate::history_store::HistoryStoreError;
use crate::script_store;

/// Exact screenplay evidence in the main document: the target's segments and
/// at most two preceding/following segments in presentation order. These ranges
/// never substitute for fictional valid time.
pub(crate) fn load_script_context(
    conn: &Connection,
    node_id: NodeId,
    start_ms: u64,
    end_ms: u64,
) -> Result<Vec<ScriptContextBlock>, HistoryStoreError> {
    script_store::create_schema(conn)?;
    let mut statement = conn.prepare(
        "WITH eligible AS (
            SELECT * FROM script_segments
            WHERE document_id = 'script.document.main' AND deleted_event_id IS NULL
        ), selected AS (
            SELECT id FROM eligible WHERE source_node_id = ?1 OR (start_ms < ?3 AND end_ms > ?2)
            UNION SELECT id FROM (SELECT id FROM eligible WHERE end_ms <= ?2 AND source_node_id IS NOT ?1
                ORDER BY end_ms DESC, start_ms DESC, sort_order DESC, id DESC LIMIT 2)
            UNION SELECT id FROM (SELECT id FROM eligible WHERE start_ms >= ?3 AND source_node_id IS NOT ?1
                ORDER BY start_ms, sort_order, id LIMIT 2)
        )
        SELECT s.document_id, s.id, b.id, s.source_node_id, b.updated_event_id,
               s.updated_event_id, s.start_ms, s.end_ms, b.text
        FROM script_segments s JOIN script_blocks b ON b.segment_id = s.id
        JOIN script_documents d ON d.id = s.document_id
        WHERE s.id IN (SELECT id FROM selected) AND b.deleted_event_id IS NULL AND d.deleted_event_id IS NULL
        ORDER BY s.start_ms, s.sort_order, s.id, b.sort_order, b.id",
    )?;
    let rows = statement.query_map(
        params![
            node_id.0.to_string(),
            i64::try_from(start_ms).map_err(|_| HistoryStoreError::InvalidValue(
                "script context start overflow".into()
            ))?,
            i64::try_from(end_ms).map_err(|_| HistoryStoreError::InvalidValue(
                "script context end overflow".into()
            ))?
        ],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, u64>(6)?,
                row.get::<_, u64>(7)?,
                row.get::<_, String>(8)?,
            ))
        },
    )?;
    rows.map(|row| {
        let (
            document,
            segment,
            block,
            source_node_id,
            revision,
            segment_revision,
            start_ms,
            end_ms,
            text,
        ) = row?;
        Ok(ScriptContextBlock {
            document_id: ScriptDocumentId::new(document)
                .map_err(|e| HistoryStoreError::InvalidValue(e.to_string()))?,
            segment_id: ScriptSegmentId::new(segment)
                .map_err(|e| HistoryStoreError::InvalidValue(e.to_string()))?,
            block_id: ScriptBlockId::new(block)
                .map_err(|e| HistoryStoreError::InvalidValue(e.to_string()))?,
            source_node_id,
            revision_event_id: ChangeEventId(
                uuid::Uuid::parse_str(&revision)
                    .map_err(|e| HistoryStoreError::InvalidId(e.to_string()))?,
            ),
            segment_revision_event_id: ChangeEventId(
                uuid::Uuid::parse_str(&segment_revision)
                    .map_err(|e| HistoryStoreError::InvalidId(e.to_string()))?,
            ),
            start_ms,
            end_ms,
            text,
        })
    })
    .collect()
}

pub(crate) fn attach_script_context(
    request: &mut eidetic_core::ai::backend::GenerateRequest,
    blocks: Vec<ScriptContextBlock>,
) {
    request.script_context = Some(blocks);
    // Legacy node text and recaps have no canonical source-version binding.
    request.surrounding_context = Default::default();
    for sibling in &mut request.siblings {
        sibling.content.content.clear();
    }
}

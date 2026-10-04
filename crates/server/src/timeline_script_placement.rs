use eidetic_core::contracts::{
    ChangeEventId, FieldDelta, FieldValue, ObjectKind, ObjectRevision, RevisionOperation,
};
use eidetic_core::timeline::{node::NodeId, timing::TimeRange};
use rusqlite::{Transaction, params};

use crate::history_store::{self, HistoryStoreError};

/// Source-bound screenplay placement follows the resized source node. Read
/// segments only after the command transaction has obtained the writer lock;
/// unrelated authoring committed before this edit must remain intact.
pub(crate) fn sync_in_transaction(
    tx: &Transaction<'_>,
    ranges: &[(NodeId, TimeRange)],
    event: ChangeEventId,
    mut revision_index: usize,
) -> Result<(), HistoryStoreError> {
    for (node_id, range) in ranges {
        let mut statement = tx.prepare(
            "SELECT s.id, s.start_ms, s.end_ms FROM script_segments s
             JOIN script_documents d ON d.id = s.document_id
             WHERE s.source_node_id = ?1 AND s.deleted_event_id IS NULL
                 AND d.deleted_event_id IS NULL ORDER BY s.id",
        )?;
        let segments = statement
            .query_map([node_id.0.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, u64>(1)?,
                    row.get::<_, u64>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (id, start, end) in segments {
            let mut revision = ObjectRevision::new(
                ObjectKind::ScriptSegment,
                &id,
                event,
                RevisionOperation::Update,
            );
            for (key, old, new) in [
                ("start_ms", start, range.start_ms),
                ("end_ms", end, range.end_ms),
            ] {
                if old != new {
                    revision = revision.with_field(FieldDelta::new(
                        key,
                        Some(FieldValue::Integer(crate::script_store_codec::to_i64(
                            old, key,
                        )?)),
                        Some(FieldValue::Integer(crate::script_store_codec::to_i64(
                            new, key,
                        )?)),
                    ));
                }
            }
            if revision.fields.is_empty() {
                continue;
            }
            history_store::insert_revision_in_transaction(tx, &revision, revision_index)?;
            revision_index += 1;
            // Update only placement and its write identity; leave authored text,
            // spans/locks, block revisions, status and document metadata alone.
            tx.execute("UPDATE script_segments SET start_ms = ?1, end_ms = ?2, updated_event_id = ?3 WHERE id = ?4",
                params![range.start_ms, range.end_ms, event.0.to_string(), id])?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "timeline_script_placement_tests.rs"]
pub(crate) mod tests;

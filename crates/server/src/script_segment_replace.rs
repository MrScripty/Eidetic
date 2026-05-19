use eidetic_core::contracts::{ChangeEventId, ScriptBlockId, ScriptSegmentId, ScriptSpanId};
use rusqlite::{Transaction, params_from_iter};

use crate::history_store::HistoryStoreError;

pub(crate) fn delete_omitted_segment_blocks_in_transaction(
    tx: &Transaction<'_>,
    segment_id: &ScriptSegmentId,
    retained_block_ids: &[ScriptBlockId],
    event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let filter = OmittedBlockFilter::new(segment_id, retained_block_ids);
    let params = filter.params_with_event(event_id);

    tx.execute(
        &format!(
            "UPDATE script_locks
             SET deleted_event_id = ?1
             WHERE deleted_event_id IS NULL
                AND span_id IN (
                    SELECT spans.id
                    FROM script_spans spans
                    INNER JOIN script_blocks blocks ON blocks.id = spans.block_id
                    WHERE {}
                )",
            filter.where_sql("blocks")
        ),
        params_from_iter(params.iter()),
    )?;
    tx.execute(
        &format!(
            "UPDATE script_spans
             SET deleted_event_id = ?1
             WHERE deleted_event_id IS NULL
                AND block_id IN (
                    SELECT blocks.id FROM script_blocks blocks WHERE {}
                )",
            filter.where_sql("blocks")
        ),
        params_from_iter(params.iter()),
    )?;
    tx.execute(
        &format!(
            "UPDATE script_blocks
             SET deleted_event_id = ?1
             WHERE deleted_event_id IS NULL
                AND {}",
            filter.where_sql("script_blocks")
        ),
        params_from_iter(params.iter()),
    )?;

    Ok(())
}

pub(crate) fn delete_omitted_block_spans_in_transaction(
    tx: &Transaction<'_>,
    block_id: &ScriptBlockId,
    retained_span_ids: &[ScriptSpanId],
    event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let filter = OmittedSpanFilter::new(block_id, retained_span_ids);
    let params = filter.params_with_event(event_id);

    tx.execute(
        &format!(
            "UPDATE script_locks
             SET deleted_event_id = ?1
             WHERE deleted_event_id IS NULL
                AND span_id IN (
                    SELECT spans.id FROM script_spans spans WHERE {}
                )",
            filter.where_sql("spans")
        ),
        params_from_iter(params.iter()),
    )?;
    tx.execute(
        &format!(
            "UPDATE script_spans
             SET deleted_event_id = ?1
             WHERE deleted_event_id IS NULL
                AND {}",
            filter.where_sql("script_spans")
        ),
        params_from_iter(params.iter()),
    )?;

    Ok(())
}

struct OmittedBlockFilter {
    retained_placeholders: Option<String>,
    values: Vec<String>,
}

impl OmittedBlockFilter {
    fn new(segment_id: &ScriptSegmentId, retained_block_ids: &[ScriptBlockId]) -> Self {
        let mut values = vec![segment_id.as_str().to_string()];
        if retained_block_ids.is_empty() {
            return Self {
                retained_placeholders: None,
                values,
            };
        }

        values.extend(
            retained_block_ids
                .iter()
                .map(|block_id| block_id.as_str().to_string()),
        );
        let placeholders = (0..retained_block_ids.len())
            .map(|index| format!("?{}", index + 3))
            .collect::<Vec<_>>()
            .join(", ");
        Self {
            retained_placeholders: Some(placeholders),
            values,
        }
    }

    fn where_sql(&self, block_table: &str) -> String {
        let base = format!("{block_table}.segment_id = ?2");
        match self.retained_placeholders.as_ref() {
            Some(placeholders) => format!("{base} AND {block_table}.id NOT IN ({placeholders})"),
            None => base,
        }
    }

    fn params_with_event(&self, event_id: ChangeEventId) -> Vec<String> {
        let mut params = vec![event_id.0.to_string()];
        params.extend(self.values.iter().cloned());
        params
    }
}

struct OmittedSpanFilter {
    retained_placeholders: Option<String>,
    values: Vec<String>,
}

impl OmittedSpanFilter {
    fn new(block_id: &ScriptBlockId, retained_span_ids: &[ScriptSpanId]) -> Self {
        let mut values = vec![block_id.as_str().to_string()];
        if retained_span_ids.is_empty() {
            return Self {
                retained_placeholders: None,
                values,
            };
        }

        values.extend(
            retained_span_ids
                .iter()
                .map(|span_id| span_id.as_str().to_string()),
        );
        let placeholders = (0..retained_span_ids.len())
            .map(|index| format!("?{}", index + 3))
            .collect::<Vec<_>>()
            .join(", ");
        Self {
            retained_placeholders: Some(placeholders),
            values,
        }
    }

    fn where_sql(&self, span_table: &str) -> String {
        let base = format!("{span_table}.block_id = ?2");
        match self.retained_placeholders.as_ref() {
            Some(placeholders) => format!("{base} AND {span_table}.id NOT IN ({placeholders})"),
            None => base,
        }
    }

    fn params_with_event(&self, event_id: ChangeEventId) -> Vec<String> {
        let mut params = vec![event_id.0.to_string()];
        params.extend(self.values.iter().cloned());
        params
    }
}

#[cfg(test)]
#[path = "script_segment_replace_tests.rs"]
mod tests;

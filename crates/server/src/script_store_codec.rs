use eidetic_core::contracts::{
    ObjectKind, ScriptBlock, ScriptBlockId, ScriptBlockKind, ScriptDocument, ScriptDocumentId,
    ScriptLock, ScriptLockId, ScriptSegment, ScriptSegmentId, ScriptSegmentStatus, ScriptSpan,
    ScriptSpanId, ScriptSpanProvenance,
};
use rusqlite::Row;

use crate::history_store::HistoryStoreError;

pub(crate) fn row_to_document(row: &Row<'_>) -> Result<ScriptDocument, rusqlite::Error> {
    let id: String = row.get(0)?;
    let sort_order: i64 = row.get(2)?;

    Ok(ScriptDocument {
        id: ScriptDocumentId::new(id).map_err(|e| conversion_failure(row, 0, e))?,
        title: row.get(1)?,
        sort_order: u32::try_from(sort_order).map_err(|e| conversion_failure(row, 2, e))?,
    })
}

pub(crate) fn row_to_segment(row: &Row<'_>) -> Result<ScriptSegment, rusqlite::Error> {
    let id: String = row.get(0)?;
    let document_id: String = row.get(1)?;
    let start_ms: i64 = row.get(3)?;
    let end_ms: i64 = row.get(4)?;
    let status: String = row.get(5)?;
    let sort_order: i64 = row.get(6)?;

    Ok(ScriptSegment {
        id: ScriptSegmentId::new(id).map_err(|e| conversion_failure(row, 0, e))?,
        document_id: ScriptDocumentId::new(document_id)
            .map_err(|e| conversion_failure(row, 1, e))?,
        source_node_id: row.get(2)?,
        start_ms: u64::try_from(start_ms).map_err(|e| conversion_failure(row, 3, e))?,
        end_ms: u64::try_from(end_ms).map_err(|e| conversion_failure(row, 4, e))?,
        status: decode_segment_status(&status).map_err(|e| conversion_failure(row, 5, e))?,
        sort_order: u32::try_from(sort_order).map_err(|e| conversion_failure(row, 6, e))?,
    })
}

pub(crate) fn row_to_block(row: &Row<'_>) -> Result<ScriptBlock, rusqlite::Error> {
    let id: String = row.get(0)?;
    let segment_id: String = row.get(1)?;
    let block_kind: String = row.get(2)?;
    let sort_order: i64 = row.get(4)?;

    Ok(ScriptBlock {
        id: ScriptBlockId::new(id).map_err(|e| conversion_failure(row, 0, e))?,
        segment_id: ScriptSegmentId::new(segment_id).map_err(|e| conversion_failure(row, 1, e))?,
        block_kind: decode_block_kind(&block_kind).map_err(|e| conversion_failure(row, 2, e))?,
        text: row.get(3)?,
        sort_order: u32::try_from(sort_order).map_err(|e| conversion_failure(row, 4, e))?,
    })
}

pub(crate) fn row_to_span(row: &Row<'_>) -> Result<ScriptSpan, rusqlite::Error> {
    let id: String = row.get(0)?;
    let block_id: String = row.get(1)?;
    let start_byte: i64 = row.get(2)?;
    let end_byte: i64 = row.get(3)?;
    let provenance: String = row.get(4)?;

    Ok(ScriptSpan {
        id: ScriptSpanId::new(id).map_err(|e| conversion_failure(row, 0, e))?,
        block_id: ScriptBlockId::new(block_id).map_err(|e| conversion_failure(row, 1, e))?,
        start_byte: u32::try_from(start_byte).map_err(|e| conversion_failure(row, 2, e))?,
        end_byte: u32::try_from(end_byte).map_err(|e| conversion_failure(row, 3, e))?,
        provenance: decode_span_provenance(&provenance)
            .map_err(|e| conversion_failure(row, 4, e))?,
    })
}

pub(crate) fn row_to_lock(row: &Row<'_>) -> Result<ScriptLock, rusqlite::Error> {
    let id: String = row.get(0)?;
    let span_id: String = row.get(1)?;

    Ok(ScriptLock {
        id: ScriptLockId::new(id).map_err(|e| conversion_failure(row, 0, e))?,
        span_id: ScriptSpanId::new(span_id).map_err(|e| conversion_failure(row, 1, e))?,
        reason: row.get(2)?,
    })
}

pub(crate) fn encode_segment_status(value: &ScriptSegmentStatus) -> &'static str {
    match value {
        ScriptSegmentStatus::Current => "current",
        ScriptSegmentStatus::Stale => "stale",
        ScriptSegmentStatus::Regenerating => "regenerating",
    }
}

pub(crate) fn encode_block_kind(value: &ScriptBlockKind) -> &'static str {
    match value {
        ScriptBlockKind::SceneHeading => "scene_heading",
        ScriptBlockKind::Action => "action",
        ScriptBlockKind::Character => "character",
        ScriptBlockKind::Parenthetical => "parenthetical",
        ScriptBlockKind::Dialogue => "dialogue",
        ScriptBlockKind::Transition => "transition",
        ScriptBlockKind::Shot => "shot",
        ScriptBlockKind::Note => "note",
    }
}

pub(crate) fn encode_span_provenance(value: &ScriptSpanProvenance) -> &'static str {
    match value {
        ScriptSpanProvenance::AiGenerated => "ai_generated",
        ScriptSpanProvenance::UserEdited => "user_edited",
        ScriptSpanProvenance::Imported => "imported",
        ScriptSpanProvenance::System => "system",
    }
}

pub(crate) fn encode_object_kind(value: &ObjectKind) -> Result<String, HistoryStoreError> {
    match serde_json::to_value(value)? {
        serde_json::Value::String(value) => Ok(value),
        _ => Err(HistoryStoreError::InvalidValue(
            "expected object kind to serialize as string".to_string(),
        )),
    }
}

pub(crate) fn parse_uuid(value: &str) -> Result<uuid::Uuid, HistoryStoreError> {
    uuid::Uuid::parse_str(value).map_err(|e| HistoryStoreError::InvalidId(e.to_string()))
}

pub(crate) fn to_i64(value: u64, field_name: &'static str) -> Result<i64, HistoryStoreError> {
    i64::try_from(value).map_err(|_| {
        HistoryStoreError::InvalidValue(format!("{field_name} is too large for sqlite integer"))
    })
}

fn decode_segment_status(value: &str) -> Result<ScriptSegmentStatus, HistoryStoreError> {
    match value {
        "current" => Ok(ScriptSegmentStatus::Current),
        "stale" => Ok(ScriptSegmentStatus::Stale),
        "regenerating" => Ok(ScriptSegmentStatus::Regenerating),
        other => Err(HistoryStoreError::InvalidValue(format!(
            "unknown script segment status: {other}"
        ))),
    }
}

fn decode_block_kind(value: &str) -> Result<ScriptBlockKind, HistoryStoreError> {
    match value {
        "scene_heading" => Ok(ScriptBlockKind::SceneHeading),
        "action" => Ok(ScriptBlockKind::Action),
        "character" => Ok(ScriptBlockKind::Character),
        "parenthetical" => Ok(ScriptBlockKind::Parenthetical),
        "dialogue" => Ok(ScriptBlockKind::Dialogue),
        "transition" => Ok(ScriptBlockKind::Transition),
        "shot" => Ok(ScriptBlockKind::Shot),
        "note" => Ok(ScriptBlockKind::Note),
        other => Err(HistoryStoreError::InvalidValue(format!(
            "unknown script block kind: {other}"
        ))),
    }
}

fn decode_span_provenance(value: &str) -> Result<ScriptSpanProvenance, HistoryStoreError> {
    match value {
        "ai_generated" => Ok(ScriptSpanProvenance::AiGenerated),
        "user_edited" => Ok(ScriptSpanProvenance::UserEdited),
        "imported" => Ok(ScriptSpanProvenance::Imported),
        "system" => Ok(ScriptSpanProvenance::System),
        other => Err(HistoryStoreError::InvalidValue(format!(
            "unknown script span provenance: {other}"
        ))),
    }
}

fn conversion_failure<E>(row: &Row<'_>, index: usize, error: E) -> rusqlite::Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    let value_type = row
        .get_ref(index)
        .map(|value| value.data_type())
        .unwrap_or(rusqlite::types::Type::Null);
    rusqlite::Error::FromSqlConversionFailure(index, value_type, Box::new(error))
}

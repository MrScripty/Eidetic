use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, FieldDelta, FieldValue, ObjectKind,
    ObjectRevision, ProjectionEnvelope, RevisionOperation, ScriptBlock, ScriptDocument,
    ScriptDocumentProjection, ScriptLock, ScriptSegment, ScriptSegmentStatus, ScriptSpan,
    ScriptSpanId, ScriptSpanProvenance, SetScriptBlockCommand, SetScriptLockCommand,
};
use rusqlite::Connection;

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::script_store;

pub(crate) fn apply_set_script_block(
    conn: &mut Connection,
    command: &CommandEnvelope<SetScriptBlockCommand>,
    created_at_ms: u64,
) -> Result<
    (
        RecordChangeOutcome,
        ProjectionEnvelope<ScriptDocumentProjection>,
    ),
    ScriptDocumentCommandError,
> {
    validate_block_command(&command.payload)?;
    script_store::create_schema(conn)?;

    let before = script_store::load_document_projection(conn, &command.payload.document_id)?;
    validate_locked_spans(before.as_ref(), &command.payload)?;
    let old_text = before
        .as_ref()
        .and_then(|projection| find_block_text(projection, &command.payload.block_id));
    let document = command_document(&command.payload);
    let segment = command_segment(&command.payload);
    let block = command_block(&command.payload);
    let span = generated_span_for_block(&block, command.payload.span_provenance.clone())?;
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("set script block {}", command.payload.block_id.as_str()),
    )
    .with_created_at_ms(created_at_ms);
    let revisions = vec![
        document_revision(&document, before.is_some(), event.id),
        segment_revision(&segment, before.as_ref(), event.id),
        block_revision(&block, old_text, event.id),
        span_revision(&span, event.id),
    ];

    let outcome = history_store::record_change_with(
        conn,
        command,
        "script.set_block",
        &event,
        &revisions,
        |tx| {
            script_store::upsert_document_in_transaction(tx, &document, event.id)?;
            script_store::upsert_segment_in_transaction(tx, &segment, event.id)?;
            script_store::upsert_block_in_transaction(tx, &block, event.id)?;
            script_store::upsert_span_in_transaction(tx, &span, event.id)?;
            Ok(())
        },
    )?;
    let projection = script_store::load_document_projection_envelope(conn, &document.id)?
        .ok_or_else(|| {
            ScriptDocumentCommandError::Store(HistoryStoreError::InvalidValue(format!(
                "script document projection missing after block update: {}",
                document.id.as_str()
            )))
        })?;

    Ok((outcome, projection))
}

pub(crate) fn apply_set_script_lock(
    conn: &mut Connection,
    command: &CommandEnvelope<SetScriptLockCommand>,
    created_at_ms: u64,
) -> Result<
    (
        RecordChangeOutcome,
        ProjectionEnvelope<ScriptDocumentProjection>,
    ),
    ScriptDocumentCommandError,
> {
    validate_lock_command(&command.payload)?;
    script_store::create_schema(conn)?;

    if !script_store::span_exists(conn, &command.payload.span_id)? {
        return Err(ScriptDocumentCommandError::InvalidCommand(
            "span_id does not reference an existing script span".to_string(),
        ));
    }
    let document_id = script_store::document_id_for_span(conn, &command.payload.span_id)?
        .ok_or_else(|| {
            ScriptDocumentCommandError::InvalidCommand(
                "span_id does not reference a script document".to_string(),
            )
        })?;
    let lock = command_lock(&command.payload);
    let lock_exists = script_store::lock_exists(conn, &lock.id)?;
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("set script lock {}", command.payload.lock_id.as_str()),
    )
    .with_created_at_ms(created_at_ms);
    let revisions = vec![lock_revision(&lock, lock_exists, event.id)];

    let outcome = history_store::record_change_with(
        conn,
        command,
        "script.set_lock",
        &event,
        &revisions,
        |tx| {
            script_store::upsert_lock_in_transaction(tx, &lock, event.id)?;
            Ok(())
        },
    )?;
    let projection = script_store::load_document_projection_envelope(conn, &document_id)?
        .ok_or_else(|| {
            ScriptDocumentCommandError::Store(HistoryStoreError::InvalidValue(format!(
                "script document projection missing after lock update: {}",
                document_id.as_str()
            )))
        })?;

    Ok((outcome, projection))
}

fn validate_block_command(
    command: &SetScriptBlockCommand,
) -> Result<(), ScriptDocumentCommandError> {
    if command.document_title.trim().is_empty() {
        return Err(ScriptDocumentCommandError::InvalidCommand(
            "document_title is required".to_string(),
        ));
    }
    if command.segment_start_ms > command.segment_end_ms {
        return Err(ScriptDocumentCommandError::InvalidCommand(
            "segment_start_ms must be <= segment_end_ms".to_string(),
        ));
    }
    if i64::try_from(command.segment_start_ms).is_err()
        || i64::try_from(command.segment_end_ms).is_err()
    {
        return Err(ScriptDocumentCommandError::InvalidCommand(
            "segment time is too large".to_string(),
        ));
    }
    Ok(())
}

fn validate_lock_command(command: &SetScriptLockCommand) -> Result<(), ScriptDocumentCommandError> {
    if command.reason.trim().is_empty() {
        return Err(ScriptDocumentCommandError::InvalidCommand(
            "reason is required".to_string(),
        ));
    }
    Ok(())
}

fn validate_locked_spans(
    before: Option<&ScriptDocumentProjection>,
    command: &SetScriptBlockCommand,
) -> Result<(), ScriptDocumentCommandError> {
    let Some(projection) = before else {
        return Ok(());
    };
    let Some(block) = projection
        .segments
        .iter()
        .flat_map(|segment| segment.blocks.iter())
        .find(|block| block.block.id == command.block_id)
    else {
        return Ok(());
    };

    for lock in &block.locks {
        let span = block
            .spans
            .iter()
            .find(|span| span.id == lock.span_id)
            .ok_or_else(|| {
                ScriptDocumentCommandError::Store(HistoryStoreError::InvalidValue(format!(
                    "script lock {} references missing span {}",
                    lock.id.as_str(),
                    lock.span_id.as_str()
                )))
            })?;
        let start = usize::try_from(span.start_byte).map_err(|_| {
            ScriptDocumentCommandError::Store(HistoryStoreError::InvalidValue(
                "script span start_byte is too large".to_string(),
            ))
        })?;
        let end = usize::try_from(span.end_byte).map_err(|_| {
            ScriptDocumentCommandError::Store(HistoryStoreError::InvalidValue(
                "script span end_byte is too large".to_string(),
            ))
        })?;
        let before_text = block.block.text.get(start..end).ok_or_else(|| {
            ScriptDocumentCommandError::Store(HistoryStoreError::InvalidValue(
                "script locked span range is invalid for existing block text".to_string(),
            ))
        })?;
        let after_text = command.text.get(start..end).ok_or_else(|| {
            ScriptDocumentCommandError::InvalidCommand(
                "script block update would remove locked span text".to_string(),
            )
        })?;
        if before_text != after_text {
            return Err(ScriptDocumentCommandError::InvalidCommand(
                "script block update would modify locked span text".to_string(),
            ));
        }
    }

    Ok(())
}

fn command_document(command: &SetScriptBlockCommand) -> ScriptDocument {
    ScriptDocument {
        id: command.document_id.clone(),
        title: command.document_title.clone(),
        sort_order: command.document_sort_order,
    }
}

fn command_segment(command: &SetScriptBlockCommand) -> ScriptSegment {
    ScriptSegment {
        id: command.segment_id.clone(),
        document_id: command.document_id.clone(),
        source_node_id: command.source_node_id.clone(),
        start_ms: command.segment_start_ms,
        end_ms: command.segment_end_ms,
        status: command.segment_status.clone(),
        sort_order: command.segment_sort_order,
    }
}

fn command_block(command: &SetScriptBlockCommand) -> ScriptBlock {
    ScriptBlock {
        id: command.block_id.clone(),
        segment_id: command.segment_id.clone(),
        block_kind: command.block_kind.clone(),
        text: command.text.clone(),
        sort_order: command.sort_order,
    }
}

fn command_lock(command: &SetScriptLockCommand) -> ScriptLock {
    ScriptLock {
        id: command.lock_id.clone(),
        span_id: command.span_id.clone(),
        reason: command.reason.clone(),
    }
}

fn generated_span_for_block(
    block: &ScriptBlock,
    provenance: ScriptSpanProvenance,
) -> Result<ScriptSpan, ScriptDocumentCommandError> {
    let end_byte = u32::try_from(block.text.len()).map_err(|_| {
        ScriptDocumentCommandError::InvalidCommand("script block text is too large".to_string())
    })?;
    Ok(ScriptSpan {
        id: ScriptSpanId::new(format!("{}.span.main", block.id.as_str()))
            .map_err(|e| ScriptDocumentCommandError::InvalidCommand(e.to_string()))?,
        block_id: block.id.clone(),
        start_byte: 0,
        end_byte,
        provenance,
    })
}

fn find_block_text(
    projection: &ScriptDocumentProjection,
    block_id: &eidetic_core::contracts::ScriptBlockId,
) -> Option<FieldValue> {
    projection
        .segments
        .iter()
        .flat_map(|segment| segment.blocks.iter())
        .find(|block| block.block.id == *block_id)
        .map(|block| FieldValue::Text(block.block.text.clone()))
}

fn document_revision(
    document: &ScriptDocument,
    exists: bool,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> ObjectRevision {
    let operation = if exists {
        RevisionOperation::Update
    } else {
        RevisionOperation::Create
    };
    ObjectRevision::new(
        ObjectKind::ScriptDocument,
        document.id.as_str(),
        event_id,
        operation,
    )
    .with_field(FieldDelta::new(
        "title",
        None,
        Some(FieldValue::Text(document.title.clone())),
    ))
    .with_field(FieldDelta::new(
        "sort_order",
        None,
        Some(FieldValue::Integer(i64::from(document.sort_order))),
    ))
}

fn segment_revision(
    segment: &ScriptSegment,
    before: Option<&ScriptDocumentProjection>,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> ObjectRevision {
    let exists = before
        .iter()
        .flat_map(|projection| projection.segments.iter())
        .any(|projection| projection.segment.id == segment.id);
    let operation = if exists {
        RevisionOperation::Update
    } else {
        RevisionOperation::Create
    };
    ObjectRevision::new(
        ObjectKind::ScriptSegment,
        segment.id.as_str(),
        event_id,
        operation,
    )
    .with_field(FieldDelta::new(
        "document_id",
        None,
        Some(FieldValue::ObjectRef {
            kind: ObjectKind::ScriptDocument,
            id: segment.document_id.as_str().to_string(),
        }),
    ))
    .with_field(FieldDelta::new(
        "source_node_id",
        None,
        segment
            .source_node_id
            .as_ref()
            .map(|id| FieldValue::ObjectRef {
                kind: ObjectKind::TimelineNode,
                id: id.clone(),
            }),
    ))
    .with_field(FieldDelta::new(
        "start_ms",
        None,
        Some(FieldValue::Integer(
            i64::try_from(segment.start_ms).unwrap_or(i64::MAX),
        )),
    ))
    .with_field(FieldDelta::new(
        "end_ms",
        None,
        Some(FieldValue::Integer(
            i64::try_from(segment.end_ms).unwrap_or(i64::MAX),
        )),
    ))
    .with_field(FieldDelta::new(
        "status",
        None,
        Some(FieldValue::Text(
            segment_status_label(&segment.status).to_string(),
        )),
    ))
}

fn block_revision(
    block: &ScriptBlock,
    old_text: Option<FieldValue>,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> ObjectRevision {
    let operation = if old_text.is_some() {
        RevisionOperation::Update
    } else {
        RevisionOperation::Create
    };
    ObjectRevision::new(
        ObjectKind::ScriptBlock,
        block.id.as_str(),
        event_id,
        operation,
    )
    .with_field(FieldDelta::new(
        "segment_id",
        None,
        Some(FieldValue::ObjectRef {
            kind: ObjectKind::ScriptSegment,
            id: block.segment_id.as_str().to_string(),
        }),
    ))
    .with_field(FieldDelta::new(
        "text",
        old_text,
        Some(FieldValue::Text(block.text.clone())),
    ))
}

fn span_revision(
    span: &ScriptSpan,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> ObjectRevision {
    ObjectRevision::new(
        ObjectKind::ScriptSpan,
        span.id.as_str(),
        event_id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "block_id",
        None,
        Some(FieldValue::ObjectRef {
            kind: ObjectKind::ScriptBlock,
            id: span.block_id.as_str().to_string(),
        }),
    ))
    .with_field(FieldDelta::new(
        "end_byte",
        None,
        Some(FieldValue::Integer(i64::from(span.end_byte))),
    ))
}

fn lock_revision(
    lock: &ScriptLock,
    exists: bool,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> ObjectRevision {
    let operation = if exists {
        RevisionOperation::Update
    } else {
        RevisionOperation::Create
    };
    ObjectRevision::new(
        ObjectKind::ScriptLock,
        lock.id.as_str(),
        event_id,
        operation,
    )
    .with_field(FieldDelta::new(
        "span_id",
        None,
        Some(FieldValue::ObjectRef {
            kind: ObjectKind::ScriptSpan,
            id: lock.span_id.as_str().to_string(),
        }),
    ))
    .with_field(FieldDelta::new(
        "reason",
        None,
        Some(FieldValue::Text(lock.reason.clone())),
    ))
}

fn segment_status_label(value: &ScriptSegmentStatus) -> &'static str {
    match value {
        ScriptSegmentStatus::Current => "current",
        ScriptSegmentStatus::Stale => "stale",
        ScriptSegmentStatus::Regenerating => "regenerating",
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ScriptDocumentCommandError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),
    #[error(transparent)]
    Store(#[from] HistoryStoreError),
}

#[cfg(test)]
#[path = "script_document_command_tests.rs"]
mod tests;

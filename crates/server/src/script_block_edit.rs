use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, EditScriptBlockCommand, FieldValue,
    ProjectionEnvelope, ScriptDocumentProjection, ScriptSpanProvenance, SetScriptBlockCommand,
};
use rusqlite::Connection;

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::script_document_command::{self, ScriptDocumentCommandError};
use crate::script_store;

pub(crate) fn apply_edit_script_block(
    conn: &mut Connection,
    command: &CommandEnvelope<EditScriptBlockCommand>,
    created_at_ms: u64,
) -> Result<
    (
        RecordChangeOutcome,
        ProjectionEnvelope<ScriptDocumentProjection>,
    ),
    ScriptDocumentCommandError,
> {
    script_store::create_schema(conn)?;
    // Replay precedes expected-version checking: the original edit is already committed.
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "script.edit_block")?
    {
        return Ok((outcome, projection(conn, command)?));
    }
    let before = projection(conn, command)?;
    let segment = before
        .payload
        .segments
        .iter()
        .find(|segment| {
            segment
                .blocks
                .iter()
                .any(|block| block.block.id == command.payload.block_id)
        })
        .ok_or_else(|| {
            ScriptDocumentCommandError::InvalidCommand("script block not found".into())
        })?;
    let existing = segment
        .blocks
        .iter()
        .find(|block| block.block.id == command.payload.block_id)
        .expect("segment selected by block");
    if existing.revision_event_id != Some(command.payload.expected_revision_event_id) {
        return Err(ScriptDocumentCommandError::InvalidCommand(
            "script block changed; reload before saving".into(),
        ));
    }
    let mut block = existing.block.clone();
    block.text = command.payload.text.clone();
    if let Some(kind) = &command.payload.block_kind {
        block.block_kind = kind.clone();
    }
    let type_changed = block.block_kind != existing.block.block_kind;
    let text_changed = block.text != existing.block.text;
    let span = script_document_command::generated_span_for_block(
        &block,
        ScriptSpanProvenance::UserEdited,
    )?;
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("edit script block {}", block.id.as_str()),
    )
    .with_created_at_ms(created_at_ms);
    let mut revision = script_document_command::block_revision(
        &block,
        Some(FieldValue::Text(existing.block.text.clone())),
        event.id,
    );
    revision
        .fields
        .iter_mut()
        .find(|field| field.field_key == "block_kind")
        .expect("block snapshot owns type")
        .old_value = Some(FieldValue::Text(
        crate::script_store_codec::encode_block_kind(&existing.block.block_kind).into(),
    ));
    let mut revisions = vec![revision];
    if text_changed {
        revisions.push(script_document_command::span_revision(&span, event.id));
    }
    let outcome = history_store::record_change_with(
        conn,
        command,
        "script.edit_block",
        &event,
        &revisions,
        |tx| {
            // The command/history insert has acquired the SQLite writer lock. Read the
            // canonical rows again here, so concurrent edits and locks cannot slip
            // between validation and the text write. Rejection rolls back history too.
            let current = script_store::load_document_projection(tx, &command.payload.document_id)?
                .ok_or_else(|| {
                    HistoryStoreError::InvalidValue("script document not found".into())
                })?;
            let current_block = current
                .segments
                .iter()
                .flat_map(|segment| &segment.blocks)
                .find(|candidate| candidate.block.id == block.id)
                .ok_or_else(|| HistoryStoreError::InvalidValue("script block not found".into()))?;
            if current_block.revision_event_id != Some(command.payload.expected_revision_event_id) {
                return Err(HistoryStoreError::InvalidValue(
                    "script block changed; reload before saving".into(),
                ));
            }
            if type_changed && !current_block.locks.is_empty() {
                return Err(HistoryStoreError::InvalidValue(
                    "cannot change the type of a locked script block".into(),
                ));
            }
            // Reuse the existing UTF-8 locked-span validation with server-owned metadata.
            let payload = SetScriptBlockCommand {
                document_id: current.document.id.clone(),
                document_title: current.document.title.clone(),
                document_sort_order: current.document.sort_order,
                segment_id: segment.segment.id.clone(),
                source_node_id: segment.segment.source_node_id.clone(),
                segment_start_ms: segment.segment.start_ms,
                segment_end_ms: segment.segment.end_ms,
                segment_status: segment.segment.status.clone(),
                segment_sort_order: segment.segment.sort_order,
                block_id: block.id.clone(),
                block_kind: block.block_kind.clone(),
                text: block.text.clone(),
                span_provenance: ScriptSpanProvenance::UserEdited,
                sort_order: block.sort_order,
            };
            script_document_command::validate_locked_spans(Some(&current), &payload)
                .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?;
            script_store::upsert_block_in_transaction(tx, &block, event.id)?;
            if text_changed {
                script_store::upsert_span_in_transaction(tx, &span, event.id)?;
            }
            Ok(())
        },
    )?;
    Ok((outcome, projection(conn, command)?))
}

fn projection(
    conn: &Connection,
    command: &CommandEnvelope<EditScriptBlockCommand>,
) -> Result<ProjectionEnvelope<ScriptDocumentProjection>, ScriptDocumentCommandError> {
    script_store::load_document_projection_envelope(conn, &command.payload.document_id)?.ok_or_else(
        || ScriptDocumentCommandError::InvalidCommand("script document not found".into()),
    )
}

#[cfg(test)]
#[path = "script_block_edit_tests.rs"]
mod tests;

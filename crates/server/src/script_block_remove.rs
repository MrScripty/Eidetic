use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, FieldDelta, FieldValue, ObjectKind,
    ObjectRevision, ProjectionEnvelope, RemoveScriptBlockCommand, RevisionOperation,
    ScriptDocumentProjection,
};
use rusqlite::Connection;

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::script_document_command::ScriptDocumentCommandError;
use crate::{script_segment_replace, script_store};

const COMMAND_TYPE: &str = "script.remove_block";

pub(crate) fn apply_remove_script_block(
    conn: &mut Connection,
    command: &CommandEnvelope<RemoveScriptBlockCommand>,
    created_at_ms: u64,
) -> Result<
    (
        RecordChangeOutcome,
        ProjectionEnvelope<ScriptDocumentProjection>,
    ),
    ScriptDocumentCommandError,
> {
    script_store::create_schema(conn)?;
    if let Some(outcome) = history_store::check_recorded_command(conn, command, COMMAND_TYPE)? {
        return Ok((outcome, projection(conn, command)?));
    }
    let before = projection(conn, command)?;
    let existing = before
        .payload
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|block| block.block.id == command.payload.block_id)
        .ok_or_else(|| invalid("script block not found"))?;
    if existing.revision_event_id != Some(command.payload.expected_revision_event_id) {
        return Err(invalid("script block changed; reload before removing"));
    }
    if !existing.locks.is_empty() {
        return Err(invalid("cannot remove a locked script block"));
    }
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("remove script block {}", existing.block.id.as_str()),
    )
    .with_created_at_ms(created_at_ms);
    let mut revisions = vec![
        ObjectRevision::new(
            ObjectKind::ScriptBlock,
            existing.block.id.as_str(),
            event.id,
            RevisionOperation::Delete,
        )
        .with_field(FieldDelta::new(
            "text",
            Some(FieldValue::Text(existing.block.text.clone())),
            None,
        )),
        ObjectRevision::new(
            ObjectKind::ScriptSegment,
            existing.block.segment_id.as_str(),
            event.id,
            RevisionOperation::Update,
        )
        .with_field(FieldDelta::new(
            format!("block.{}", existing.block.id.as_str()),
            Some(FieldValue::ObjectRef {
                kind: ObjectKind::ScriptBlock,
                id: existing.block.id.as_str().into(),
            }),
            None,
        )),
    ];
    revisions.extend(existing.spans.iter().map(|span| {
        ObjectRevision::new(
            ObjectKind::ScriptSpan,
            span.id.as_str(),
            event.id,
            RevisionOperation::Delete,
        )
    }));
    let outcome =
        history_store::record_change_with(conn, command, COMMAND_TYPE, &event, &revisions, |tx| {
            let current = script_store::load_document_projection(tx, &command.payload.document_id)?
                .ok_or_else(|| {
                    HistoryStoreError::InvalidValue("script document not found".into())
                })?;
            let segment = current
                .segments
                .iter()
                .find(|segment| segment.segment.id == existing.block.segment_id)
                .ok_or_else(|| HistoryStoreError::InvalidValue("script block not found".into()))?;
            let block = segment
                .blocks
                .iter()
                .find(|block| block.block.id == existing.block.id)
                .ok_or_else(|| HistoryStoreError::InvalidValue("script block not found".into()))?;
            // Writer-lock recapture protects exact text, spans and locks. Membership
            // is recaptured separately so concurrently appended blocks survive.
            if !block.locks.is_empty() {
                return Err(HistoryStoreError::InvalidValue(
                    "cannot remove a locked script block".into(),
                ));
            }
            if block != existing {
                return Err(HistoryStoreError::InvalidValue(
                    "script block changed; reload before removing".into(),
                ));
            }
            let retained = segment
                .blocks
                .iter()
                .filter(|block| block.block.id != existing.block.id)
                .map(|block| block.block.id.clone())
                .collect::<Vec<_>>();
            script_segment_replace::delete_omitted_segment_blocks_in_transaction(
                tx,
                &segment.segment.id,
                &retained,
                event.id,
            )?;
            tx.execute(
                "UPDATE script_segments SET updated_event_id = ?1 WHERE id = ?2",
                rusqlite::params![event.id.0.to_string(), segment.segment.id.as_str()],
            )?;
            Ok(())
        })?;
    Ok((outcome, projection(conn, command)?))
}

fn projection(
    conn: &Connection,
    command: &CommandEnvelope<RemoveScriptBlockCommand>,
) -> Result<ProjectionEnvelope<ScriptDocumentProjection>, ScriptDocumentCommandError> {
    script_store::load_document_projection_envelope(conn, &command.payload.document_id)?
        .ok_or_else(|| invalid("script document not found"))
}

fn invalid(message: &str) -> ScriptDocumentCommandError {
    ScriptDocumentCommandError::InvalidCommand(message.into())
}

#[cfg(test)]
#[path = "script_block_remove_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "script_block_remove_impact_tests.rs"]
mod impact_tests;

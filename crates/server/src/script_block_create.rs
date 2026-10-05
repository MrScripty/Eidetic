use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, CreateScriptBlockCommand, FieldDelta,
    FieldValue, ObjectKind, ObjectRevision, ProjectionEnvelope, RevisionOperation, ScriptBlock,
    ScriptBlockId, ScriptDocument, ScriptDocumentProjection, ScriptSegment, ScriptSegmentId,
    ScriptSegmentStatus, ScriptSpanProvenance,
};
use rusqlite::Connection;

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::script_document_command::{self, ScriptDocumentCommandError};
use crate::{script_store, script_store_codec, timeline_node_store};

const COMMAND_TYPE: &str = "script.create_block";

#[derive(PartialEq, Eq)]
struct Placement {
    document: ScriptDocument,
    segment: ScriptSegment,
    create_document: bool,
    create_segment: bool,
    next_order: u32,
}

pub(crate) fn apply_create_script_block(
    conn: &mut Connection,
    command: &CommandEnvelope<CreateScriptBlockCommand>,
    created_at_ms: u64,
) -> Result<
    (
        RecordChangeOutcome,
        ProjectionEnvelope<ScriptDocumentProjection>,
    ),
    ScriptDocumentCommandError,
> {
    script_store::create_schema(conn)?;
    // Replays remain valid after later appends/edits or timeline movement.
    if let Some(outcome) = history_store::check_recorded_command(conn, command, COMMAND_TYPE)? {
        return Ok((outcome, projection(conn, command)?));
    }
    if command.payload.text.trim().is_empty() {
        return Err(invalid("screenplay text is required"));
    }
    if command.payload.document_id.as_str() != "script.document.main" {
        return Err(invalid(
            "manual writing requires the main screenplay document",
        ));
    }
    let planned = placement(conn, command)?;
    let block = ScriptBlock {
        id: ScriptBlockId::new(format!("script.block.{}.user", command.id.0))
            .map_err(|error| invalid(&error.to_string()))?,
        segment_id: planned.segment.id.clone(),
        block_kind: command.payload.block_kind.clone(),
        text: command.payload.text.clone(),
        sort_order: planned.next_order,
    };
    let span = script_document_command::generated_span_for_block(
        &block,
        ScriptSpanProvenance::UserEdited,
    )?;
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        "write screenplay block",
    )
    .with_created_at_ms(created_at_ms);
    let mut revisions = Vec::new();
    if planned.create_document {
        revisions.push(script_document_command::document_revision(
            &planned.document,
            false,
            event.id,
        ));
    }
    // A segment dependency covers its consumed block set as well as placement.
    // Record this new member sparsely so downstream generations can see an
    // append without rewriting any existing block or placement field.
    let segment_revision = if planned.create_segment {
        script_document_command::segment_revision(&planned.segment, None, event.id)
    } else {
        ObjectRevision::new(
            ObjectKind::ScriptSegment,
            planned.segment.id.as_str(),
            event.id,
            RevisionOperation::Update,
        )
    };
    revisions.push(segment_revision.with_field(FieldDelta::new(
        format!("block.{}", block.id.as_str()),
        None,
        Some(FieldValue::ObjectRef {
            kind: ObjectKind::ScriptBlock,
            id: block.id.as_str().into(),
        }),
    )));
    revisions.push(
        script_document_command::block_revision(&block, None, event.id)
            .with_field(FieldDelta::new(
                "block_kind",
                None,
                Some(FieldValue::Text(
                    script_store_codec::encode_block_kind(&block.block_kind).into(),
                )),
            ))
            .with_field(FieldDelta::new(
                "sort_order",
                None,
                Some(FieldValue::Integer(i64::from(block.sort_order))),
            )),
    );
    let mut span_revision = script_document_command::span_revision(&span, event.id)
        .with_field(FieldDelta::new(
            "start_byte",
            None,
            Some(FieldValue::Integer(0)),
        ))
        .with_field(FieldDelta::new(
            "provenance",
            None,
            Some(FieldValue::Text("user_edited".into())),
        ));
    span_revision.operation = RevisionOperation::Create;
    revisions.push(span_revision);
    let outcome =
        history_store::record_change_with(conn, command, COMMAND_TYPE, &event, &revisions, |tx| {
            // The history insert owns the writer lock. Recheck placement and append
            // ordering here so another writer cannot retarget or overwrite a draft.
            if placement(tx, command)
                .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?
                != planned
            {
                return Err(HistoryStoreError::InvalidValue(
                    "screenplay changed; refresh and try adding again".into(),
                ));
            }
            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM script_blocks WHERE id = ?1)",
                [block.id.as_str()],
                |row| row.get(0),
            )?;
            if exists {
                return Err(HistoryStoreError::InvalidValue(
                    "new screenplay block already exists".into(),
                ));
            }
            if planned.create_document {
                script_store::upsert_document_in_transaction(tx, &planned.document, event.id)?;
            }
            if planned.create_segment {
                script_store::upsert_segment_in_transaction(tx, &planned.segment, event.id)?;
            } else {
                // Advance only the dependency identity for changed membership.
                tx.execute(
                    "UPDATE script_segments SET updated_event_id = ?1 WHERE id = ?2",
                    rusqlite::params![event.id.0.to_string(), planned.segment.id.as_str()],
                )?;
            }
            script_store::upsert_block_in_transaction(tx, &block, event.id)?;
            script_store::upsert_span_in_transaction(tx, &span, event.id)
        })?;
    Ok((outcome, projection(conn, command)?))
}

fn placement(
    conn: &Connection,
    command: &CommandEnvelope<CreateScriptBlockCommand>,
) -> Result<Placement, ScriptDocumentCommandError> {
    let request = &command.payload;
    let node = timeline_node_store::load_node_ancestor_stack(conn, request.source_node_id)?
        .into_iter()
        .find(|node| node.id == request.source_node_id)
        .ok_or_else(|| invalid("selected timeline context no longer exists"))?;
    if (node.time_range.start_ms, node.time_range.end_ms)
        != (request.expected_start_ms, request.expected_end_ms)
    {
        return Err(invalid(
            "timeline placement changed; use its current placement and try again",
        ));
    }
    let before = script_store::load_document_projection(conn, &request.document_id)?;
    let document = match &before {
        Some(before) => before.document.clone(),
        None => ScriptDocument {
            id: request.document_id.clone(),
            title: conn
                .query_row("SELECT name FROM project WHERE id = 1", [], |row| {
                    row.get(0)
                })
                .map_err(HistoryStoreError::from)?,
            sort_order: 0,
        },
    };
    let segment_id = ScriptSegmentId::new(format!("script.segment.{}", request.source_node_id.0))
        .map_err(|error| invalid(&error.to_string()))?;
    let existing = before.as_ref().and_then(|before| {
        before
            .segments
            .iter()
            .find(|segment| segment.segment.id == segment_id)
    });
    let segment = existing
        .map(|existing| existing.segment.clone())
        .unwrap_or_else(|| ScriptSegment {
            id: segment_id,
            document_id: request.document_id.clone(),
            source_node_id: Some(request.source_node_id.0.to_string()),
            start_ms: node.time_range.start_ms,
            end_ms: node.time_range.end_ms,
            status: ScriptSegmentStatus::Current,
            sort_order: 0,
        });
    if segment.source_node_id.as_deref() != Some(request.source_node_id.0.to_string().as_str())
        || (segment.start_ms, segment.end_ms) != (node.time_range.start_ms, node.time_range.end_ms)
    {
        return Err(invalid(
            "screenplay placement does not match the selected timeline context",
        ));
    }
    // Ordinal zero remains available to the existing generated block. Manual
    // appends are ordered afterward without rewriting prior block ordinals.
    let next_order = existing
        .into_iter()
        .flat_map(|segment| &segment.blocks)
        .map(|block| block.block.sort_order)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| invalid("screenplay block ordering is full"))?;
    Ok(Placement {
        document,
        segment,
        create_document: before.is_none(),
        create_segment: existing.is_none(),
        next_order,
    })
}

fn projection(
    conn: &Connection,
    command: &CommandEnvelope<CreateScriptBlockCommand>,
) -> Result<ProjectionEnvelope<ScriptDocumentProjection>, ScriptDocumentCommandError> {
    script_store::load_document_projection_envelope(conn, &command.payload.document_id)?
        .ok_or_else(|| invalid("screenplay document is unavailable"))
}

fn invalid(message: &str) -> ScriptDocumentCommandError {
    ScriptDocumentCommandError::InvalidCommand(message.into())
}

#[cfg(test)]
#[path = "script_block_create_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "script_block_create_impact_tests.rs"]
mod impact_tests;

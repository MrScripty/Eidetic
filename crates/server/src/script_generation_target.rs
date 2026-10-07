//! Canonical generation target custody in existing command/revision history.
use eidetic_core::contracts::*;
use eidetic_core::timeline::node::{NodeId, StoryNode};
use rusqlite::{Connection, OptionalExtension};

use crate::{history_store::HistoryStoreError, script_store, timeline_node_store};

pub(crate) fn capture(
    conn: &Connection,
    expected: &StoryNode,
) -> Result<ScriptGenerationTarget, HistoryStoreError> {
    let node = timeline_node_store::load_node_ancestor_stack(conn, expected.id)?
        .into_iter()
        .find(|node| node.id == expected.id)
        .ok_or_else(stale)?;
    if node.locked
        || node.id != expected.id
        || node.parent_id != expected.parent_id
        || node.level != expected.level
        || node.name != expected.name
        || node.time_range != expected.time_range
        || node.content.notes != expected.content.notes
    {
        return Err(stale());
    }
    let target = current(conn, &node)?;
    Ok(target)
}

fn current(
    conn: &Connection,
    node: &StoryNode,
) -> Result<ScriptGenerationTarget, HistoryStoreError> {
    let segment_id = format!("script.segment.{}", node.id.0);
    let block_id = format!("script.block.{}.generated", node.id.0);
    Ok(ScriptGenerationTarget {
        node_id: node.id,
        start_ms: node.time_range.start_ms,
        end_ms: node.time_range.end_ms,
        notes: node.content.notes.clone(),
        node_revision_event_id: timeline_node_store::latest_node_event(conn, node.id, None)?,
        segment_revision_event_id: row_event(conn, "script_segments", &segment_id)?,
        block_revision_event_id: row_event(conn, "script_blocks", &block_id)?,
    })
}

fn row_event(
    conn: &Connection,
    table: &str,
    id: &str,
) -> Result<Option<ChangeEventId>, HistoryStoreError> {
    // Table names are internal constants, never caller/model input.
    let value = conn
        .query_row(
            &format!("SELECT updated_event_id FROM {table} WHERE id = ?1"),
            [id],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    event(value)
}

fn event(value: Option<String>) -> Result<Option<ChangeEventId>, HistoryStoreError> {
    value
        .map(|value| {
            uuid::Uuid::parse_str(&value)
                .map(ChangeEventId)
                .map_err(|error| HistoryStoreError::InvalidId(error.to_string()))
        })
        .transpose()
}

fn validate_human_output(conn: &Connection, node: NodeId) -> Result<(), HistoryStoreError> {
    if let Some(document) = script_store::load_document_projection(
        conn,
        &ScriptDocumentId::new("script.document.main").expect("main document id"),
    )? {
        let id = format!("script.block.{}.generated", node.0);
        if document
            .segments
            .iter()
            .flat_map(|segment| &segment.blocks)
            .any(|block| {
                block.block.id.as_str() == id
                    && block
                        .spans
                        .iter()
                        .any(|span| span.provenance == ScriptSpanProvenance::UserEdited)
            })
        {
            return Err(HistoryStoreError::InvalidValue(
                "generated screenplay was manually edited; use a reviewed update".into(),
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_admission(
    conn: &Connection,
    binding: &ScriptGenerationTarget,
) -> Result<(), HistoryStoreError> {
    let node = timeline_node_store::load_node_ancestor_stack(conn, binding.node_id)?
        .into_iter()
        .find(|node| node.id == binding.node_id)
        .ok_or_else(stale)?;
    if node.locked || current(conn, &node)? != *binding {
        return Err(stale());
    }
    validate_human_output(conn, binding.node_id)
}

/// Called after history inserts have acquired the writer lock, before any current
/// state changes. Refusal rolls back command/event/revisions as one transaction.
pub(crate) fn validate(
    conn: &Connection,
    command: &GenerateScriptBlockCommand,
) -> Result<(), HistoryStoreError> {
    let Some(binding) = &command.target_binding else {
        return Ok(());
    };
    let node = timeline_node_store::load_node_ancestor_stack(conn, binding.node_id)?
        .into_iter()
        .find(|node| node.id == binding.node_id)
        .ok_or_else(stale)?;
    let block = &command.block;
    if node.locked
        || current(conn, &node)? != *binding
        || block.document_id.as_str() != "script.document.main"
        || block.source_node_id.as_deref() != Some(binding.node_id.0.to_string().as_str())
        || block.segment_id.as_str() != format!("script.segment.{}", binding.node_id.0)
        || block.block_id.as_str() != format!("script.block.{}.generated", binding.node_id.0)
        || (block.segment_start_ms, block.segment_end_ms) != (binding.start_ms, binding.end_ms)
    {
        return Err(stale());
    }
    validate_human_output(conn, binding.node_id)
}

fn stale() -> HistoryStoreError {
    HistoryStoreError::InvalidValue(
        "generation target changed; saved screenplay was preserved, generate again from current context".into(),
    )
}

#[cfg(test)]
#[path = "script_generation_target_tests.rs"]
pub(crate) mod tests;

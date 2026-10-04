//! Optimistic state validation for commands planned from a timeline snapshot.
//!
//! History writers replace whole node/relationship collections. Validate every
//! collection they read inside the SAME write transaction, before changing any
//! current-state rows. A conflict rolls back the command, event and revisions too.

use eidetic_core::timeline::Timeline;
use rusqlite::{OptionalExtension, Transaction};
use serde::Serialize;

use crate::history_store::HistoryStoreError;
use crate::{timeline_node_store, timeline_relationship_store};

pub(crate) fn validate_current_timeline(
    tx: &Transaction<'_>,
    expected: &Timeline,
) -> Result<(), HistoryStoreError> {
    let has_project_table: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'project')",
        [],
        |row| row.get(0),
    )?;
    let duration: Option<u64> = if has_project_table {
        tx.query_row(
            "SELECT total_duration_ms FROM project WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .optional()?
    } else {
        None
    };
    let duration = duration.ok_or_else(|| HistoryStoreError::InvalidValue(
        "project is not durably initialized; wait for save and reload before editing the timeline".to_string(),
    ))?;
    let nodes = timeline_node_store::load_nodes(tx)?;
    let node_arcs = timeline_node_store::load_node_arcs(tx)?;
    let relationships = timeline_relationship_store::load_relationships(tx)?;
    if duration != expected.total_duration_ms
        || canonical_rows(&nodes)? != canonical_rows(&expected.nodes)?
        || canonical_rows(&node_arcs)? != canonical_rows(&expected.node_arcs)?
        || canonical_rows(&relationships)? != canonical_rows(&expected.relationships)?
    {
        return Err(HistoryStoreError::InvalidValue(
            "timeline changed while preparing this command; reload and review the edit before retrying"
                .to_string(),
        ));
    }
    Ok(())
}

// Store SELECT order and in-memory display order are not timeline identity.
// Compare all serialized row fields (including text/status and arc membership),
// retaining multiplicity while ignoring collection order only.
fn canonical_rows<T: Serialize>(rows: &[T]) -> Result<Vec<String>, HistoryStoreError> {
    let mut rows = rows
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?;
    rows.sort_unstable();
    Ok(rows)
}

#[cfg(test)]
#[path = "timeline_command_guard_tests.rs"]
mod tests;

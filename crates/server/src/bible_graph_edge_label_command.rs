//! Label-only compare-and-set through existing command and revision history.
use crate::history_store::{HistoryStoreError, RecordChangeOutcome};
use crate::{bible_graph_edge_store, bible_graph_store, history_store};
use eidetic_core::contracts::*;
use rusqlite::Connection;

pub(crate) fn apply(
    conn: &mut Connection,
    command: &CommandEnvelope<SetBibleGraphEdgeLabelCommand>,
    created_at_ms: u64,
) -> Result<
    (
        RecordChangeOutcome,
        ProjectionEnvelope<BibleNodeDetailProjection>,
    ),
    HistoryStoreError,
> {
    if command.payload.label.trim().is_empty() {
        return Err(HistoryStoreError::InvalidValue("label is required".into()));
    }
    bible_graph_store::create_schema(conn)?;
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        "edit Bible relationship label",
    )
    .with_created_at_ms(created_at_ms);
    let outcome = history_store::record_change_with(
        conn,
        command,
        "bible_graph.set_edge_label",
        &event,
        &[],
        |tx| {
            let before = bible_graph_edge_store::load_edge(tx, &command.payload.edge_id)?
                .ok_or_else(|| {
                    HistoryStoreError::InvalidValue(
                        "Relationship changed while editing; reopen its label editor.".into(),
                    )
                })?;
            bible_graph_edge_store::set_edge_label_in_transaction(tx, &command.payload, event.id)?;
            let revision = ObjectRevision::new(
                ObjectKind::BibleEdge,
                before.id.as_str(),
                event.id,
                RevisionOperation::Update,
            )
            .with_field(FieldDelta::new(
                "label",
                Some(FieldValue::Text(before.label)),
                Some(FieldValue::Text(command.payload.label.trim().into())),
            ));
            history_store::insert_revision_in_transaction(tx, &revision, 0)?;
            Ok(())
        },
    )?;
    // A replay stays read-only even after another edit or deletion.
    let source: String = conn.query_row(
        "SELECT from_node_id FROM bible_graph_edges WHERE id=?1",
        [command.payload.edge_id.as_str()],
        |row| row.get(0),
    )?;
    let source = BibleGraphNodeId::new(source)
        .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?;
    let projection = bible_graph_store::load_node_detail_projection_envelope(conn, &source)?
        .ok_or_else(|| {
            HistoryStoreError::InvalidValue("Bible relationship source no longer exists".into())
        })?;
    Ok((outcome, projection))
}

#[cfg(test)]
#[path = "bible_graph_edge_label_tests.rs"]
mod tests;

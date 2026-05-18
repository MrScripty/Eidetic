use eidetic_core::contracts::{
    BibleGraphNode, BibleNodeDetailProjection, ChangeEvent, ChangeEventKind, CommandEnvelope,
    CreateBibleGraphNodeCommand, FieldDelta, FieldValue, ObjectKind, ObjectRevision,
    ProjectionEnvelope, RevisionOperation,
};
use rusqlite::Connection;

use crate::bible_graph_store;
use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};

pub(crate) fn apply_create_bible_graph_node(
    conn: &mut Connection,
    command: &CommandEnvelope<CreateBibleGraphNodeCommand>,
    created_at_ms: u64,
) -> Result<
    (
        RecordChangeOutcome,
        ProjectionEnvelope<BibleNodeDetailProjection>,
    ),
    BibleGraphCommandError,
> {
    validate_command(&command.payload)?;
    bible_graph_store::create_schema(conn)?;

    let node = command.payload.clone().into_node();
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("create bible graph node {}", node.name),
    )
    .with_created_at_ms(created_at_ms);
    let revision = node_revision(&node, event.id);

    let outcome = history_store::record_change_with(
        conn,
        command,
        "bible_graph.create_node",
        &event,
        &[revision],
        |tx| bible_graph_store::insert_node_in_transaction(tx, &node, event.id),
    )?;

    let projection = bible_graph_store::load_node_detail_projection_envelope(conn, &node.id)?
        .ok_or_else(|| {
            BibleGraphCommandError::Store(HistoryStoreError::InvalidValue(format!(
                "bible graph node projection missing after create: {}",
                node.id.as_str()
            )))
        })?;

    Ok((outcome, projection))
}

fn validate_command(command: &CreateBibleGraphNodeCommand) -> Result<(), BibleGraphCommandError> {
    if command.name.trim().is_empty() {
        return Err(BibleGraphCommandError::InvalidCommand(
            "name is required".to_string(),
        ));
    }
    Ok(())
}

fn node_revision(
    node: &BibleGraphNode,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> ObjectRevision {
    let mut revision = ObjectRevision::new(
        ObjectKind::BibleNode,
        node.id.as_str(),
        event_id,
        RevisionOperation::Create,
    )
    .with_field(FieldDelta::new(
        "name",
        None,
        Some(FieldValue::Text(node.name.clone())),
    ))
    .with_field(FieldDelta::new(
        "schema_key",
        None,
        Some(FieldValue::Text(node.schema_key.as_str().to_string())),
    ))
    .with_field(FieldDelta::new(
        "sort_order",
        None,
        Some(FieldValue::Integer(i64::from(node.sort_order))),
    ));

    if let Some(parent_id) = node.parent_id.as_ref() {
        revision = revision.with_field(FieldDelta::new(
            "parent_id",
            None,
            Some(FieldValue::ObjectRef {
                kind: ObjectKind::BibleNode,
                id: parent_id.as_str().to_string(),
            }),
        ));
    }

    revision
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum BibleGraphCommandError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),
    #[error(transparent)]
    Store(#[from] HistoryStoreError),
}

#[cfg(test)]
#[path = "bible_graph_command_tests.rs"]
mod tests;

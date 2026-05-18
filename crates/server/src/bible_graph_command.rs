use eidetic_core::contracts::{
    BibleGraphNode, BibleNodeDetailProjection, ChangeEvent, ChangeEventKind, CommandEnvelope,
    CreateBibleGraphNodeCommand, EnsureCanonicalBibleRootsCommand, FieldDelta, FieldValue,
    ObjectKind, ObjectRevision, ProjectionEnvelope, RevisionOperation, SetBibleGraphFieldCommand,
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
    validate_parent_exists(conn, &command.payload)?;

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

pub(crate) fn apply_ensure_canonical_bible_roots(
    conn: &mut Connection,
    command: &CommandEnvelope<EnsureCanonicalBibleRootsCommand>,
    created_at_ms: u64,
) -> Result<
    (
        RecordChangeOutcome,
        ProjectionEnvelope<eidetic_core::contracts::BibleGraphNodeListProjection>,
    ),
    BibleGraphCommandError,
> {
    bible_graph_store::create_schema(conn)?;
    let missing_roots = bible_graph_store::missing_canonical_root_nodes(conn)?;
    if missing_roots.is_empty() {
        let projection = bible_graph_store::load_node_list_projection_envelope(conn)?;
        return Ok((RecordChangeOutcome::AlreadyRecorded, projection));
    }

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::SystemRepair,
        "ensure canonical bible roots",
    )
    .with_created_at_ms(created_at_ms);
    let revisions: Vec<ObjectRevision> = missing_roots
        .iter()
        .map(|node| node_revision(node, event.id))
        .collect();

    let outcome = history_store::record_change_with(
        conn,
        command,
        "bible_graph.ensure_canonical_roots",
        &event,
        &revisions,
        |tx| {
            let _ = bible_graph_store::insert_missing_canonical_roots_in_transaction(tx, event.id)?;
            Ok(())
        },
    )?;
    let projection = bible_graph_store::load_node_list_projection_envelope(conn)?;

    Ok((outcome, projection))
}

pub(crate) fn apply_set_bible_graph_field(
    conn: &mut Connection,
    command: &CommandEnvelope<SetBibleGraphFieldCommand>,
    created_at_ms: u64,
) -> Result<
    (
        RecordChangeOutcome,
        ProjectionEnvelope<BibleNodeDetailProjection>,
    ),
    BibleGraphCommandError,
> {
    validate_field_command(&command.payload)?;
    bible_graph_store::create_schema(conn)?;

    let before = bible_graph_store::load_node_detail_projection(conn, &command.payload.node_id)?
        .ok_or_else(|| {
            BibleGraphCommandError::InvalidCommand(format!(
                "bible graph node does not exist: {}",
                command.payload.node_id.as_str()
            ))
        })?;
    let old_value = before
        .parts
        .iter()
        .flat_map(|part| part.fields.iter())
        .find(|field| field.id == command.payload.field_id)
        .and_then(|field| field.value.clone());
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!(
            "set bible graph field {}",
            command.payload.field_key.as_str()
        ),
    )
    .with_created_at_ms(created_at_ms);
    let revision = ObjectRevision::new(
        ObjectKind::BiblePartField,
        command.payload.field_id.as_str(),
        event.id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "value",
        old_value,
        command.payload.value.clone(),
    ));

    let outcome = history_store::record_change_with(
        conn,
        command,
        "bible_graph.set_field",
        &event,
        &[revision],
        |tx| bible_graph_store::set_field_in_transaction(tx, &command.payload, event.id),
    )?;
    let projection =
        bible_graph_store::load_node_detail_projection_envelope(conn, &command.payload.node_id)?
            .ok_or_else(|| {
                BibleGraphCommandError::Store(HistoryStoreError::InvalidValue(format!(
                    "bible graph node projection missing after field update: {}",
                    command.payload.node_id.as_str()
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

fn validate_parent_exists(
    conn: &Connection,
    command: &CreateBibleGraphNodeCommand,
) -> Result<(), BibleGraphCommandError> {
    let Some(parent_id) = command.parent_id.as_ref() else {
        return Ok(());
    };
    if bible_graph_store::node_exists(conn, parent_id)? {
        return Ok(());
    }

    Err(BibleGraphCommandError::InvalidCommand(format!(
        "parent bible graph node does not exist: {}",
        parent_id.as_str()
    )))
}

fn validate_field_command(
    command: &SetBibleGraphFieldCommand,
) -> Result<(), BibleGraphCommandError> {
    if command.part_name.trim().is_empty() {
        return Err(BibleGraphCommandError::InvalidCommand(
            "part_name is required".to_string(),
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

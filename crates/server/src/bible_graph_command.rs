use eidetic_core::contracts::{
    BibleGraphEdge, BibleGraphNode, BibleNodeDetailProjection, ChangeEvent, ChangeEventKind,
    CommandEnvelope, CreateBibleGraphNodeCommand, EnsureCanonicalBibleRootsCommand, FieldDelta,
    FieldValue, ObjectKind, ObjectRevision, ProjectionEnvelope, RevisionOperation,
    SetBibleGraphEdgeCommand, SetBibleGraphFieldCommand, SetBibleGraphSnapshotFieldCommand,
    builtin_bible_graph_schema,
};
use rusqlite::Connection;

use crate::bible_graph_edge_store;
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
    validate_field_schema(&before, &command.payload)?;
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

pub(crate) fn apply_set_bible_graph_edge(
    conn: &mut Connection,
    command: &CommandEnvelope<SetBibleGraphEdgeCommand>,
    created_at_ms: u64,
) -> Result<
    (
        RecordChangeOutcome,
        ProjectionEnvelope<BibleNodeDetailProjection>,
    ),
    BibleGraphCommandError,
> {
    validate_edge_command(&command.payload)?;
    bible_graph_store::create_schema(conn)?;
    validate_edge_endpoint_exists(conn, &command.payload.from_node_id, "from")?;
    validate_edge_endpoint_exists(conn, &command.payload.to_node_id, "to")?;

    let before = bible_graph_edge_store::load_edge(conn, &command.payload.edge_id)?;
    let edge = command.payload.clone().into_edge();
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("set bible graph edge {}", edge.label),
    )
    .with_created_at_ms(created_at_ms);
    let revision = edge_revision(&edge, before.as_ref(), event.id);

    let outcome = history_store::record_change_with(
        conn,
        command,
        "bible_graph.set_edge",
        &event,
        &[revision],
        |tx| bible_graph_edge_store::set_edge_in_transaction(tx, &command.payload, event.id),
    )?;
    let projection = bible_graph_store::load_node_detail_projection_envelope(
        conn,
        &command.payload.from_node_id,
    )?
    .ok_or_else(|| {
        BibleGraphCommandError::Store(HistoryStoreError::InvalidValue(format!(
            "bible graph source node projection missing after edge update: {}",
            command.payload.from_node_id.as_str()
        )))
    })?;

    Ok((outcome, projection))
}

pub(crate) fn apply_set_bible_graph_snapshot_field(
    conn: &mut Connection,
    command: &CommandEnvelope<SetBibleGraphSnapshotFieldCommand>,
    created_at_ms: u64,
) -> Result<
    (
        RecordChangeOutcome,
        ProjectionEnvelope<BibleNodeDetailProjection>,
    ),
    BibleGraphCommandError,
> {
    validate_snapshot_field_command(&command.payload)?;
    bible_graph_store::create_schema(conn)?;

    let before = bible_graph_store::load_node_detail_projection(conn, &command.payload.node_id)?
        .ok_or_else(|| {
            BibleGraphCommandError::InvalidCommand(format!(
                "bible graph node does not exist: {}",
                command.payload.node_id.as_str()
            ))
        })?;
    validate_snapshot_field_schema(&before, &command.payload)?;
    let before_snapshot = before
        .snapshots
        .iter()
        .find(|snapshot| snapshot.snapshot.id == command.payload.snapshot_id);
    let old_value = before_snapshot
        .iter()
        .flat_map(|snapshot| snapshot.fields.iter())
        .find(|field| field.id == command.payload.field_id)
        .and_then(|field| field.value.clone());
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!(
            "set bible graph snapshot field {}",
            command.payload.field_key.as_str()
        ),
    )
    .with_created_at_ms(created_at_ms);
    let revision = snapshot_revision(
        &command.payload,
        old_value,
        before_snapshot.is_some(),
        event.id,
    );

    let outcome = history_store::record_change_with(
        conn,
        command,
        "bible_graph.set_snapshot_field",
        &event,
        &[revision],
        |tx| bible_graph_store::set_snapshot_field_in_transaction(tx, &command.payload, event.id),
    )?;
    let projection =
        bible_graph_store::load_node_detail_projection_envelope(conn, &command.payload.node_id)?
            .ok_or_else(|| {
                BibleGraphCommandError::Store(HistoryStoreError::InvalidValue(format!(
                    "bible graph node projection missing after snapshot field update: {}",
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

fn validate_edge_endpoint_exists(
    conn: &Connection,
    node_id: &eidetic_core::contracts::BibleGraphNodeId,
    role: &'static str,
) -> Result<(), BibleGraphCommandError> {
    if bible_graph_store::node_exists(conn, node_id)? {
        return Ok(());
    }

    Err(BibleGraphCommandError::InvalidCommand(format!(
        "{role} bible graph node does not exist: {}",
        node_id.as_str()
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

fn validate_field_schema(
    projection: &BibleNodeDetailProjection,
    command: &SetBibleGraphFieldCommand,
) -> Result<(), BibleGraphCommandError> {
    let Some(schema) = builtin_bible_graph_schema(&projection.node.schema_key) else {
        return Ok(());
    };
    let Some(part) = schema.part(&command.part_key) else {
        return Err(BibleGraphCommandError::InvalidCommand(format!(
            "part_key {} is not valid for bible graph schema {}",
            command.part_key.as_str(),
            projection.node.schema_key.as_str()
        )));
    };
    if part.name != command.part_name {
        return Err(BibleGraphCommandError::InvalidCommand(format!(
            "part_name {} does not match schema part {}",
            command.part_name, part.name
        )));
    }
    if part.field(&command.field_key).is_none() {
        return Err(BibleGraphCommandError::InvalidCommand(format!(
            "field_key {} is not valid for bible graph schema {} part {}",
            command.field_key.as_str(),
            projection.node.schema_key.as_str(),
            command.part_key.as_str()
        )));
    }
    Ok(())
}

fn validate_edge_command(command: &SetBibleGraphEdgeCommand) -> Result<(), BibleGraphCommandError> {
    if command.label.trim().is_empty() {
        return Err(BibleGraphCommandError::InvalidCommand(
            "label is required".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_snapshot_field_command(
    command: &SetBibleGraphSnapshotFieldCommand,
) -> Result<(), BibleGraphCommandError> {
    if command.label.trim().is_empty() {
        return Err(BibleGraphCommandError::InvalidCommand(
            "label is required".to_string(),
        ));
    }
    if command.part_name.trim().is_empty() {
        return Err(BibleGraphCommandError::InvalidCommand(
            "part_name is required".to_string(),
        ));
    }
    if i64::try_from(command.at_ms).is_err() {
        return Err(BibleGraphCommandError::InvalidCommand(
            "at_ms is too large".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_snapshot_field_schema(
    projection: &BibleNodeDetailProjection,
    command: &SetBibleGraphSnapshotFieldCommand,
) -> Result<(), BibleGraphCommandError> {
    let Some(schema) = builtin_bible_graph_schema(&projection.node.schema_key) else {
        return Ok(());
    };
    let Some(part) = schema.part(&command.part_key) else {
        return Err(BibleGraphCommandError::InvalidCommand(format!(
            "part_key {} is not valid for bible graph schema {}",
            command.part_key.as_str(),
            projection.node.schema_key.as_str()
        )));
    };
    if part.name != command.part_name {
        return Err(BibleGraphCommandError::InvalidCommand(format!(
            "part_name {} does not match schema part {}",
            command.part_name, part.name
        )));
    }
    if part.field(&command.field_key).is_none() {
        return Err(BibleGraphCommandError::InvalidCommand(format!(
            "field_key {} is not valid for bible graph schema {} part {}",
            command.field_key.as_str(),
            projection.node.schema_key.as_str(),
            command.part_key.as_str()
        )));
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

pub(crate) fn snapshot_revision(
    command: &SetBibleGraphSnapshotFieldCommand,
    old_value: Option<FieldValue>,
    snapshot_exists: bool,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> ObjectRevision {
    let operation = if snapshot_exists {
        RevisionOperation::Update
    } else {
        RevisionOperation::Create
    };
    ObjectRevision::new(
        ObjectKind::BibleSnapshot,
        command.snapshot_id.as_str(),
        event_id,
        operation,
    )
    .with_field(FieldDelta::new(
        "node_id",
        None,
        Some(FieldValue::ObjectRef {
            kind: ObjectKind::BibleNode,
            id: command.node_id.as_str().to_string(),
        }),
    ))
    .with_field(FieldDelta::new(
        "at_ms",
        None,
        Some(FieldValue::Integer(
            i64::try_from(command.at_ms).unwrap_or(i64::MAX),
        )),
    ))
    .with_field(FieldDelta::new(
        "label",
        None,
        Some(FieldValue::Text(command.label.clone())),
    ))
    .with_field(FieldDelta::new(
        format!(
            "field.{}.{}",
            command.part_key.as_str(),
            command.field_key.as_str()
        ),
        old_value,
        command.value.clone(),
    ))
}

fn edge_revision(
    edge: &BibleGraphEdge,
    before: Option<&BibleGraphEdge>,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> ObjectRevision {
    let operation = if before.is_some() {
        RevisionOperation::Update
    } else {
        RevisionOperation::Create
    };
    ObjectRevision::new(ObjectKind::BibleEdge, edge.id.as_str(), event_id, operation)
        .with_field(FieldDelta::new(
            "from_node_id",
            before.map(|edge| FieldValue::ObjectRef {
                kind: ObjectKind::BibleNode,
                id: edge.from_node_id.as_str().to_string(),
            }),
            Some(FieldValue::ObjectRef {
                kind: ObjectKind::BibleNode,
                id: edge.from_node_id.as_str().to_string(),
            }),
        ))
        .with_field(FieldDelta::new(
            "to_node_id",
            before.map(|edge| FieldValue::ObjectRef {
                kind: ObjectKind::BibleNode,
                id: edge.to_node_id.as_str().to_string(),
            }),
            Some(FieldValue::ObjectRef {
                kind: ObjectKind::BibleNode,
                id: edge.to_node_id.as_str().to_string(),
            }),
        ))
        .with_field(FieldDelta::new(
            "edge_kind",
            before.map(|edge| FieldValue::Text(format!("{:?}", edge.edge_kind))),
            Some(FieldValue::Text(format!("{:?}", edge.edge_kind))),
        ))
        .with_field(FieldDelta::new(
            "label",
            before.map(|edge| FieldValue::Text(edge.label.clone())),
            Some(FieldValue::Text(edge.label.clone())),
        ))
        .with_field(FieldDelta::new(
            "directed",
            before.map(|edge| FieldValue::Bool(edge.directed)),
            Some(FieldValue::Bool(edge.directed)),
        ))
        .with_field(FieldDelta::new(
            "sort_order",
            before.map(|edge| FieldValue::Integer(i64::from(edge.sort_order))),
            Some(FieldValue::Integer(i64::from(edge.sort_order))),
        ))
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

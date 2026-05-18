use super::*;
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphFieldKey, BibleGraphPartKey,
    BibleGraphSchemaKey, BibleGraphSnapshotFieldId, BibleGraphSnapshotId, ChangeEventKind,
    CommandEnvelope, FieldValue, SetBibleGraphEdgeCommand, SetBibleGraphFieldCommand,
    SetBibleGraphSnapshotFieldCommand,
};

#[derive(Debug, serde::Serialize)]
struct TestCommand;

fn memory_connection() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    create_schema(&conn).unwrap();
    conn
}

#[test]
fn missing_node_projection_returns_none() {
    let conn = memory_connection();
    let node_id = BibleGraphNodeId::new("node.missing").unwrap();

    let projection = load_node_detail_projection_envelope(&conn, &node_id).unwrap();

    assert!(projection.is_none());
}

#[test]
fn missing_canonical_roots_reports_unpersisted_roots() {
    let conn = memory_connection();

    let missing = missing_canonical_root_nodes(&conn).unwrap();

    assert_eq!(missing.len(), 8);
    assert_eq!(missing[0].id.as_str(), "canonical.characters");
}

#[test]
fn node_projection_envelope_uses_revision_history_version() {
    let mut conn = memory_connection();
    let command = CommandEnvelope::new(TestCommand);
    let event = eidetic_core::contracts::ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        "create node",
    );
    let node = BibleGraphNode {
        id: BibleGraphNodeId::new("node.place.beach").unwrap(),
        parent_id: None,
        schema_key: BibleGraphSchemaKey::new("place").unwrap(),
        name: "Beach".to_string(),
        system_owned: false,
        sort_order: 3,
    };
    let revision = eidetic_core::contracts::ObjectRevision::new(
        ObjectKind::BibleNode,
        node.id.as_str(),
        event.id,
        eidetic_core::contracts::RevisionOperation::Create,
    );

    history_store::record_change_with(
        &mut conn,
        &command,
        "test.create_node",
        &event,
        &[revision],
        |tx| insert_node_in_transaction(tx, &node, event.id),
    )
    .unwrap();

    let projection = load_node_detail_projection_envelope(&conn, &node.id)
        .unwrap()
        .unwrap();

    assert_eq!(projection.version, ProjectionVersion(2));
    assert_eq!(projection.change_event_id, Some(event.id));
    assert_eq!(projection.payload.node.name, "Beach");
}

#[test]
fn known_schema_node_detail_projection_includes_default_parts_without_persisting_rows() {
    let mut conn = memory_connection();
    let command = CommandEnvelope::new(TestCommand);
    let event = eidetic_core::contracts::ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        "create character",
    );
    let node = BibleGraphNode {
        id: BibleGraphNodeId::new("node.character.ada").unwrap(),
        parent_id: None,
        schema_key: BibleGraphSchemaKey::new("character").unwrap(),
        name: "Ada".to_string(),
        system_owned: false,
        sort_order: 3,
    };
    let revision = eidetic_core::contracts::ObjectRevision::new(
        ObjectKind::BibleNode,
        node.id.as_str(),
        event.id,
        eidetic_core::contracts::RevisionOperation::Create,
    );
    history_store::record_change_with(
        &mut conn,
        &command,
        "test.create_node",
        &event,
        &[revision],
        |tx| insert_node_in_transaction(tx, &node, event.id),
    )
    .unwrap();

    let projection = load_node_detail_projection_envelope(&conn, &node.id)
        .unwrap()
        .unwrap();

    assert_eq!(
        projection.payload.parts[0].part.part_key.as_str(),
        "profile"
    );
    assert_eq!(
        projection.payload.parts[0].fields[1].field_key.as_str(),
        "tagline"
    );
    assert_eq!(table_count(&conn, "bible_graph_parts"), 0);
    assert_eq!(table_count(&conn, "bible_graph_fields"), 0);
}

#[test]
fn node_list_projection_returns_nodes_in_stable_order() {
    let mut conn = memory_connection();
    seed_node(&mut conn, "node.place.beach", "Beach", 20);
    seed_node(&mut conn, "node.character.ada", "Ada", 10);

    let projection = load_node_list_projection_envelope(&conn).unwrap();

    assert_eq!(projection.version, ProjectionVersion(3));
    assert_eq!(projection.payload.nodes.len(), 2);
    assert_eq!(
        projection.payload.nodes[0].id.as_str(),
        "node.character.ada"
    );
    assert_eq!(projection.payload.nodes[1].id.as_str(), "node.place.beach");
}

#[test]
fn node_detail_projection_includes_parts_and_fields() {
    let mut conn = memory_connection();
    seed_node(&mut conn, "node.character.ada", "Ada", 10);
    let command = CommandEnvelope::new(SetBibleGraphFieldCommand {
        node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
        part_id: eidetic_core::contracts::BibleGraphPartId::new("part.character.profile").unwrap(),
        part_key: eidetic_core::contracts::BibleGraphPartKey::new("profile").unwrap(),
        part_name: "Profile".to_string(),
        part_sort_order: 1,
        field_id: eidetic_core::contracts::BibleGraphFieldId::new("field.character.tagline")
            .unwrap(),
        field_key: eidetic_core::contracts::BibleGraphFieldKey::new("tagline").unwrap(),
        value: Some(FieldValue::Text("Reluctant detective".to_string())),
        field_sort_order: 2,
    });
    let event = eidetic_core::contracts::ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        "set field",
    );
    let revision = eidetic_core::contracts::ObjectRevision::new(
        ObjectKind::BiblePartField,
        command.payload.field_id.as_str(),
        event.id,
        eidetic_core::contracts::RevisionOperation::Update,
    );
    history_store::record_change_with(
        &mut conn,
        &command,
        "test.set_field",
        &event,
        &[revision],
        |tx| set_field_in_transaction(tx, &command.payload, event.id),
    )
    .unwrap();

    let projection = load_node_detail_projection_envelope(
        &conn,
        &BibleGraphNodeId::new("node.character.ada").unwrap(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(projection.version, ProjectionVersion(3));
    assert_eq!(projection.payload.parts.len(), 1);
    assert_eq!(projection.payload.parts[0].fields.len(), 1);
    assert_eq!(
        projection.payload.parts[0].fields[0].value,
        Some(FieldValue::Text("Reluctant detective".to_string()))
    );
}

#[test]
fn node_detail_projection_includes_incoming_and_outgoing_edges() {
    let mut conn = memory_connection();
    seed_node(&mut conn, "node.character.ada", "Ada", 10);
    seed_node(&mut conn, "node.place.beach", "Beach", 20);
    let command = CommandEnvelope::new(SetBibleGraphEdgeCommand {
        edge_id: BibleGraphEdgeId::new("edge.ada.beach").unwrap(),
        from_node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
        to_node_id: BibleGraphNodeId::new("node.place.beach").unwrap(),
        edge_kind: BibleGraphEdgeKind::LocatedIn,
        label: "located in".to_string(),
        directed: true,
        sort_order: 1,
    });
    let event = eidetic_core::contracts::ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        "set edge",
    );
    let revision = eidetic_core::contracts::ObjectRevision::new(
        ObjectKind::BibleEdge,
        command.payload.edge_id.as_str(),
        event.id,
        eidetic_core::contracts::RevisionOperation::Create,
    );
    history_store::record_change_with(
        &mut conn,
        &command,
        "test.set_edge",
        &event,
        &[revision],
        |tx| crate::bible_graph_edge_store::set_edge_in_transaction(tx, &command.payload, event.id),
    )
    .unwrap();

    let source_projection = load_node_detail_projection_envelope(
        &conn,
        &BibleGraphNodeId::new("node.character.ada").unwrap(),
    )
    .unwrap()
    .unwrap();
    let target_projection = load_node_detail_projection_envelope(
        &conn,
        &BibleGraphNodeId::new("node.place.beach").unwrap(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(source_projection.version, ProjectionVersion(3));
    assert_eq!(source_projection.payload.outgoing_edges.len(), 1);
    assert_eq!(source_projection.payload.incoming_edges.len(), 0);
    assert_eq!(target_projection.payload.incoming_edges.len(), 1);
    assert_eq!(target_projection.payload.outgoing_edges.len(), 0);
    assert_eq!(
        target_projection.payload.incoming_edges[0]
            .from_node_id
            .as_str(),
        "node.character.ada"
    );
}

#[test]
fn node_detail_projection_includes_snapshots_and_fields() {
    let mut conn = memory_connection();
    seed_node(&mut conn, "node.place.beach", "Beach", 10);
    let command = CommandEnvelope::new(SetBibleGraphSnapshotFieldCommand {
        snapshot_id: BibleGraphSnapshotId::new("snapshot.beach.sequence-1").unwrap(),
        node_id: BibleGraphNodeId::new("node.place.beach").unwrap(),
        at_ms: 12_000,
        label: "Sequence 1 state".to_string(),
        snapshot_sort_order: 1,
        field_id: BibleGraphSnapshotFieldId::new("snapshot-field.beach.weather.current").unwrap(),
        part_key: BibleGraphPartKey::new("weather").unwrap(),
        part_name: "Weather".to_string(),
        field_key: BibleGraphFieldKey::new("current").unwrap(),
        value: Some(FieldValue::Text("rainy".to_string())),
        field_sort_order: 2,
    });
    let event = eidetic_core::contracts::ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        "set snapshot field",
    );
    let revision = eidetic_core::contracts::ObjectRevision::new(
        ObjectKind::BibleSnapshot,
        command.payload.snapshot_id.as_str(),
        event.id,
        eidetic_core::contracts::RevisionOperation::Update,
    );
    history_store::record_change_with(
        &mut conn,
        &command,
        "test.set_snapshot_field",
        &event,
        &[revision],
        |tx| set_snapshot_field_in_transaction(tx, &command.payload, event.id),
    )
    .unwrap();

    let projection = load_node_detail_projection_envelope(
        &conn,
        &BibleGraphNodeId::new("node.place.beach").unwrap(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(projection.version, ProjectionVersion(3));
    assert_eq!(projection.payload.snapshots.len(), 1);
    assert_eq!(projection.payload.snapshots[0].snapshot.at_ms, 12_000);
    assert_eq!(projection.payload.snapshots[0].fields.len(), 1);
    assert_eq!(
        projection.payload.snapshots[0].fields[0].value,
        Some(FieldValue::Text("rainy".to_string()))
    );
}

fn seed_node(conn: &mut Connection, node_id: &str, name: &str, sort_order: u32) {
    let command = CommandEnvelope::new(TestCommand);
    let event =
        eidetic_core::contracts::ChangeEvent::new(command.id, ChangeEventKind::UserEdit, name);
    let node = BibleGraphNode {
        id: BibleGraphNodeId::new(node_id).unwrap(),
        parent_id: None,
        schema_key: BibleGraphSchemaKey::new("test").unwrap(),
        name: name.to_string(),
        system_owned: false,
        sort_order,
    };
    let revision = eidetic_core::contracts::ObjectRevision::new(
        ObjectKind::BibleNode,
        node.id.as_str(),
        event.id,
        eidetic_core::contracts::RevisionOperation::Create,
    );

    history_store::record_change_with(
        conn,
        &command,
        "test.create_node",
        &event,
        &[revision],
        |tx| insert_node_in_transaction(tx, &node, event.id),
    )
    .unwrap();
}

fn table_count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}

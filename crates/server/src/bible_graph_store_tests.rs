use super::*;
use eidetic_core::contracts::{BibleGraphSchemaKey, ChangeEventKind, CommandEnvelope};

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

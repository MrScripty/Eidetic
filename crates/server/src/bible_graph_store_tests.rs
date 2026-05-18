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

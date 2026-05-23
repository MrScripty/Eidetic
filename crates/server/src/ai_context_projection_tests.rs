use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphFieldId, BibleGraphFieldKey, BibleGraphNodeId,
    BibleGraphPartId, BibleGraphPartKey, BibleGraphSchemaKey, BibleGraphSnapshotFieldId,
    BibleGraphSnapshotId, CommandEnvelope, CreateBibleGraphNodeCommand, FieldValue,
    SetBibleGraphEdgeCommand, SetBibleGraphFieldCommand, SetBibleGraphSnapshotFieldCommand,
};
use eidetic_core::timeline::node::NodeId;
use rusqlite::Connection;

use super::load_ai_bible_context_projection;

#[test]
fn ai_context_projection_loads_graph_facts_for_prompting() {
    let mut conn = Connection::open_in_memory().unwrap();
    seed_graph(&mut conn);

    let projection = load_ai_bible_context_projection(&conn, NodeId::new()).unwrap();

    assert_eq!(projection.version.0, 6);
    assert_eq!(projection.payload.nodes.len(), 2);
    let ada = projection
        .payload
        .nodes
        .iter()
        .find(|node| node.node_id.as_str() == "node.character.ada")
        .expect("ada context node");
    assert_eq!(ada.name, "Ada");
    assert_eq!(ada.fields[0].field_key.as_str(), "tagline");
    assert_eq!(
        ada.fields[0].value,
        FieldValue::Text("Reluctant detective".to_string())
    );
    assert_eq!(ada.snapshots[0].label, "Opening");
    assert_eq!(
        ada.outgoing_edges[0].to_node_id.as_str(),
        "node.place.beach"
    );
}

#[test]
fn ai_context_projection_uses_bounded_render_graph_defaults() {
    let mut conn = Connection::open_in_memory().unwrap();
    for index in 0..205 {
        seed_basic_node(
            &mut conn,
            &format!("node.place.{index:03}"),
            &format!("Place {index:03}"),
            index,
        );
    }

    let projection = load_ai_bible_context_projection(&conn, NodeId::new()).unwrap();

    assert_eq!(projection.payload.nodes.len(), 200);
    assert_eq!(
        projection.payload.nodes[0].node_id.as_str(),
        "node.place.000"
    );
    assert_eq!(
        projection.payload.nodes[199].node_id.as_str(),
        "node.place.199"
    );
}

fn seed_graph(conn: &mut Connection) {
    seed_node(conn, "node.character.ada", "character", "Ada", 10, 100);
    seed_node(conn, "node.place.beach", "place", "Beach", 20, 200);
    crate::bible_graph_command::apply_set_bible_graph_field(
        conn,
        &CommandEnvelope::new(SetBibleGraphFieldCommand {
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            part_id: BibleGraphPartId::new("part.character.profile").unwrap(),
            part_key: BibleGraphPartKey::new("profile").unwrap(),
            part_name: "Profile".to_string(),
            part_sort_order: 10,
            field_id: BibleGraphFieldId::new("field.character.tagline").unwrap(),
            field_key: BibleGraphFieldKey::new("tagline").unwrap(),
            value: Some(FieldValue::Text("Reluctant detective".to_string())),
            field_sort_order: 20,
        }),
        300,
    )
    .unwrap();
    crate::bible_graph_command::apply_set_bible_graph_edge(
        conn,
        &CommandEnvelope::new(SetBibleGraphEdgeCommand {
            edge_id: BibleGraphEdgeId::new("edge.ada.beach").unwrap(),
            from_node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            to_node_id: BibleGraphNodeId::new("node.place.beach").unwrap(),
            edge_kind: BibleGraphEdgeKind::LocatedIn,
            label: "visits".to_string(),
            directed: true,
            sort_order: 10,
        }),
        400,
    )
    .unwrap();
    crate::bible_graph_command::apply_set_bible_graph_snapshot_field(
        conn,
        &CommandEnvelope::new(SetBibleGraphSnapshotFieldCommand {
            snapshot_id: BibleGraphSnapshotId::new("snapshot.character.ada.opening").unwrap(),
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            at_ms: 1_000,
            label: "Opening".to_string(),
            snapshot_sort_order: 10,
            field_id: BibleGraphSnapshotFieldId::new("snapshot-field.character.tagline").unwrap(),
            part_key: BibleGraphPartKey::new("profile").unwrap(),
            part_name: "Profile".to_string(),
            field_key: BibleGraphFieldKey::new("tagline").unwrap(),
            value: Some(FieldValue::Text("Rain-soaked".to_string())),
            field_sort_order: 10,
        }),
        500,
    )
    .unwrap();
}

fn seed_basic_node(conn: &mut Connection, node_id: &str, name: &str, sort_order: u32) {
    seed_node(
        conn,
        node_id,
        "place",
        name,
        sort_order,
        u64::from(sort_order),
    );
}

fn seed_node(
    conn: &mut Connection,
    node_id: &str,
    schema_key: &str,
    name: &str,
    sort_order: u32,
    created_at_ms: u64,
) {
    crate::bible_graph_command::apply_create_bible_graph_node(
        conn,
        &CommandEnvelope::new(CreateBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new(node_id).unwrap(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new(schema_key).unwrap(),
            name: name.to_string(),
            sort_order,
        }),
        created_at_ms,
    )
    .unwrap();
}

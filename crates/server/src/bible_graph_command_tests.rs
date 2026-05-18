use super::*;
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey,
    BibleGraphSchemaKey, BibleGraphSnapshotFieldId, BibleGraphSnapshotId, CommandEnvelope,
    EnsureCanonicalBibleRootsCommand, FieldValue, SetBibleGraphEdgeCommand,
    SetBibleGraphFieldCommand, SetBibleGraphSnapshotFieldCommand,
};

fn memory_connection() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    bible_graph_store::create_schema(&conn).unwrap();
    conn
}

fn create_command(node_id: &str, name: &str) -> CommandEnvelope<CreateBibleGraphNodeCommand> {
    create_command_with_parent(node_id, None, name)
}

fn create_command_with_parent(
    node_id: &str,
    parent_id: Option<&str>,
    name: &str,
) -> CommandEnvelope<CreateBibleGraphNodeCommand> {
    CommandEnvelope::new(CreateBibleGraphNodeCommand {
        node_id: BibleGraphNodeId::new(node_id).unwrap(),
        parent_id: parent_id.map(|value| BibleGraphNodeId::new(value).unwrap()),
        schema_key: BibleGraphSchemaKey::new("character").unwrap(),
        name: name.to_string(),
        sort_order: 7,
    })
}

fn field_command(value: Option<FieldValue>) -> CommandEnvelope<SetBibleGraphFieldCommand> {
    CommandEnvelope::new(SetBibleGraphFieldCommand {
        node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
        part_id: eidetic_core::contracts::BibleGraphPartId::new("part.character.profile").unwrap(),
        part_key: eidetic_core::contracts::BibleGraphPartKey::new("profile").unwrap(),
        part_name: "Profile".to_string(),
        part_sort_order: 1,
        field_id: eidetic_core::contracts::BibleGraphFieldId::new("field.character.tagline")
            .unwrap(),
        field_key: eidetic_core::contracts::BibleGraphFieldKey::new("tagline").unwrap(),
        value,
        field_sort_order: 2,
    })
}

fn edge_command() -> CommandEnvelope<SetBibleGraphEdgeCommand> {
    CommandEnvelope::new(SetBibleGraphEdgeCommand {
        edge_id: BibleGraphEdgeId::new("edge.ada.beach").unwrap(),
        from_node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
        to_node_id: BibleGraphNodeId::new("node.place.beach").unwrap(),
        edge_kind: BibleGraphEdgeKind::LocatedIn,
        label: "located in".to_string(),
        directed: true,
        sort_order: 4,
    })
}

fn snapshot_field_command(
    value: Option<FieldValue>,
) -> CommandEnvelope<SetBibleGraphSnapshotFieldCommand> {
    CommandEnvelope::new(SetBibleGraphSnapshotFieldCommand {
        snapshot_id: BibleGraphSnapshotId::new("snapshot.character.ada.sequence-1").unwrap(),
        node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
        at_ms: 12_000,
        label: "Sequence 1 state".to_string(),
        snapshot_sort_order: 1,
        field_id: BibleGraphSnapshotFieldId::new("snapshot-field.character.status").unwrap(),
        part_key: BibleGraphPartKey::new("profile").unwrap(),
        part_name: "Profile".to_string(),
        field_key: BibleGraphFieldKey::new("tagline").unwrap(),
        value,
        field_sort_order: 2,
    })
}

#[test]
fn create_child_node_requires_existing_parent() {
    let mut conn = memory_connection();
    let parent = create_command("node.group.protagonists", "Protagonists");
    apply_create_bible_graph_node(&mut conn, &parent, 100).unwrap();
    let child =
        create_command_with_parent("node.character.ada", Some("node.group.protagonists"), "Ada");

    let (outcome, projection) = apply_create_bible_graph_node(&mut conn, &child, 200).unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(
        projection
            .payload
            .node
            .parent_id
            .as_ref()
            .map(BibleGraphNodeId::as_str),
        Some("node.group.protagonists")
    );
    assert_eq!(table_count(&conn, "commands"), 2);
    assert_eq!(table_count(&conn, "change_events"), 2);
    assert_eq!(table_count(&conn, "object_revisions"), 2);
    assert_eq!(table_count(&conn, "bible_graph_nodes"), 2);
}

#[test]
fn create_child_node_rejects_missing_parent_without_writing() {
    let mut conn = memory_connection();
    let command =
        create_command_with_parent("node.character.ada", Some("node.group.missing"), "Ada");

    let error = apply_create_bible_graph_node(&mut conn, &command, 100).unwrap_err();

    assert!(matches!(error, BibleGraphCommandError::InvalidCommand(_)));
    assert_eq!(table_count(&conn, "commands"), 0);
    assert_eq!(table_count(&conn, "change_events"), 0);
    assert_eq!(table_count(&conn, "object_revisions"), 0);
    assert_eq!(table_count(&conn, "bible_graph_nodes"), 0);
}

#[test]
fn create_bible_graph_node_records_history_and_projection() {
    let mut conn = memory_connection();
    let command = create_command("node.character.ada", "Ada");

    let (outcome, projection) = apply_create_bible_graph_node(&mut conn, &command, 100).unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(
        projection.version,
        eidetic_core::contracts::ProjectionVersion(2)
    );
    assert_eq!(projection.payload.node.id.as_str(), "node.character.ada");
    assert_eq!(projection.payload.node.name, "Ada");
    assert_eq!(projection.payload.node.schema_key.as_str(), "character");
    assert_eq!(
        projection.payload.parts[0].part.part_key.as_str(),
        "profile"
    );
    assert_eq!(
        projection.payload.parts[0].fields[1].field_key.as_str(),
        "tagline"
    );
    assert!(
        projection
            .payload
            .parts
            .iter()
            .flat_map(|part| part.fields.iter())
            .all(|field| field.value.is_none())
    );
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "change_events"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 1);
    assert_eq!(table_count(&conn, "bible_graph_nodes"), 1);
}

#[test]
fn duplicate_create_command_is_idempotent() {
    let mut conn = memory_connection();
    let command = create_command("node.character.ada", "Ada");

    let (first, _) = apply_create_bible_graph_node(&mut conn, &command, 100).unwrap();
    let (second, projection) = apply_create_bible_graph_node(&mut conn, &command, 100).unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(projection.payload.node.name, "Ada");
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 1);
    assert_eq!(table_count(&conn, "bible_graph_nodes"), 1);
}

#[test]
fn create_command_rejects_blank_name_without_writing() {
    let mut conn = memory_connection();
    let command = create_command("node.character.ada", " ");

    let error = apply_create_bible_graph_node(&mut conn, &command, 100).unwrap_err();

    assert!(matches!(error, BibleGraphCommandError::InvalidCommand(_)));
    assert_eq!(table_count(&conn, "commands"), 0);
    assert_eq!(table_count(&conn, "bible_graph_nodes"), 0);
}

#[test]
fn duplicate_node_id_with_different_command_rolls_back_history() {
    let mut conn = memory_connection();
    let first = create_command("node.character.ada", "Ada");
    let second = create_command("node.character.ada", "Ada Clone");
    apply_create_bible_graph_node(&mut conn, &first, 100).unwrap();

    let error = apply_create_bible_graph_node(&mut conn, &second, 200).unwrap_err();

    assert!(matches!(error, BibleGraphCommandError::Store(_)));
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "change_events"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 1);
    assert_eq!(table_count(&conn, "bible_graph_nodes"), 1);
}

#[test]
fn ensure_canonical_roots_persists_system_owned_nodes() {
    let mut conn = memory_connection();
    let command = CommandEnvelope::new(EnsureCanonicalBibleRootsCommand {});

    let (outcome, projection) =
        apply_ensure_canonical_bible_roots(&mut conn, &command, 100).unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(
        projection.version,
        eidetic_core::contracts::ProjectionVersion(9)
    );
    assert_eq!(projection.payload.nodes.len(), 8);
    assert_eq!(
        projection.payload.nodes[0].id.as_str(),
        "canonical.characters"
    );
    assert!(
        projection
            .payload
            .nodes
            .iter()
            .all(|node| node.system_owned)
    );
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "change_events"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 8);
    assert_eq!(table_count(&conn, "bible_graph_nodes"), 8);
}

#[test]
fn ensure_canonical_roots_replays_duplicate_command_without_extra_rows() {
    let mut conn = memory_connection();
    let command = CommandEnvelope::new(EnsureCanonicalBibleRootsCommand {});

    let (first, _) = apply_ensure_canonical_bible_roots(&mut conn, &command, 100).unwrap();
    let (second, projection) =
        apply_ensure_canonical_bible_roots(&mut conn, &command, 100).unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(projection.payload.nodes.len(), 8);
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "change_events"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 8);
    assert_eq!(table_count(&conn, "bible_graph_nodes"), 8);
}

#[test]
fn ensure_canonical_roots_is_noop_when_roots_already_exist_for_new_command() {
    let mut conn = memory_connection();
    let first = CommandEnvelope::new(EnsureCanonicalBibleRootsCommand {});
    let second = CommandEnvelope::new(EnsureCanonicalBibleRootsCommand {});
    apply_ensure_canonical_bible_roots(&mut conn, &first, 100).unwrap();

    let (outcome, projection) =
        apply_ensure_canonical_bible_roots(&mut conn, &second, 200).unwrap();

    assert_eq!(outcome, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(projection.payload.nodes.len(), 8);
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "change_events"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 8);
    assert_eq!(table_count(&conn, "bible_graph_nodes"), 8);
}

#[test]
fn set_bible_graph_field_creates_part_field_and_updates_projection() {
    let mut conn = memory_connection();
    let node = create_command("node.character.ada", "Ada");
    apply_create_bible_graph_node(&mut conn, &node, 100).unwrap();
    let field = field_command(Some(FieldValue::Text("Reluctant detective".to_string())));

    let (outcome, projection) = apply_set_bible_graph_field(&mut conn, &field, 200).unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(
        projection.version,
        eidetic_core::contracts::ProjectionVersion(3)
    );
    let profile = projection
        .payload
        .parts
        .iter()
        .find(|part| part.part.part_key.as_str() == "profile")
        .unwrap();
    assert_eq!(profile.part.name, "Profile");
    assert_eq!(
        profile
            .fields
            .iter()
            .find(|field| field.field_key.as_str() == "tagline")
            .unwrap()
            .value,
        Some(FieldValue::Text("Reluctant detective".to_string()))
    );
    assert_eq!(table_count(&conn, "commands"), 2);
    assert_eq!(table_count(&conn, "change_events"), 2);
    assert_eq!(table_count(&conn, "object_revisions"), 2);
    assert_eq!(table_count(&conn, "bible_graph_parts"), 1);
    assert_eq!(table_count(&conn, "bible_graph_fields"), 1);
}

#[test]
fn duplicate_set_field_command_is_idempotent() {
    let mut conn = memory_connection();
    let node = create_command("node.character.ada", "Ada");
    apply_create_bible_graph_node(&mut conn, &node, 100).unwrap();
    let field = field_command(Some(FieldValue::Text("Reluctant detective".to_string())));

    let (first, _) = apply_set_bible_graph_field(&mut conn, &field, 200).unwrap();
    let (second, projection) = apply_set_bible_graph_field(&mut conn, &field, 200).unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
    assert!(
        projection
            .payload
            .parts
            .iter()
            .flat_map(|part| part.fields.iter())
            .any(|field| field.field_key.as_str() == "tagline"
                && field.value == Some(FieldValue::Text("Reluctant detective".to_string())))
    );
    assert_eq!(table_count(&conn, "commands"), 2);
    assert_eq!(table_count(&conn, "object_revisions"), 2);
}

#[test]
fn set_field_rejects_unknown_field_for_known_schema_without_history_rows() {
    let mut conn = memory_connection();
    let node = create_command("node.character.ada", "Ada");
    apply_create_bible_graph_node(&mut conn, &node, 100).unwrap();
    let mut field = field_command(Some(FieldValue::Text("Reluctant detective".to_string())));
    field.payload.field_key = eidetic_core::contracts::BibleGraphFieldKey::new("unknown").unwrap();

    let error = apply_set_bible_graph_field(&mut conn, &field, 200).unwrap_err();

    assert!(matches!(error, BibleGraphCommandError::InvalidCommand(_)));
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 1);
    assert_eq!(table_count(&conn, "bible_graph_parts"), 0);
    assert_eq!(table_count(&conn, "bible_graph_fields"), 0);
}

#[test]
fn set_field_rejects_missing_node_without_history_rows() {
    let mut conn = memory_connection();
    let field = field_command(Some(FieldValue::Text("Reluctant detective".to_string())));

    let error = apply_set_bible_graph_field(&mut conn, &field, 200).unwrap_err();

    assert!(matches!(error, BibleGraphCommandError::InvalidCommand(_)));
    assert_eq!(table_count(&conn, "commands"), 0);
    assert_eq!(table_count(&conn, "object_revisions"), 0);
    assert_eq!(table_count(&conn, "bible_graph_parts"), 0);
    assert_eq!(table_count(&conn, "bible_graph_fields"), 0);
}

#[test]
fn set_bible_graph_edge_creates_edge_and_updates_projection() {
    let mut conn = memory_connection();
    let source = create_command("node.character.ada", "Ada");
    let target = create_command("node.place.beach", "Beach");
    apply_create_bible_graph_node(&mut conn, &source, 100).unwrap();
    apply_create_bible_graph_node(&mut conn, &target, 200).unwrap();
    let edge = edge_command();

    let (outcome, projection) = apply_set_bible_graph_edge(&mut conn, &edge, 300).unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(
        projection.version,
        eidetic_core::contracts::ProjectionVersion(3)
    );
    assert_eq!(projection.payload.outgoing_edges.len(), 1);
    assert_eq!(
        projection.payload.outgoing_edges[0].id.as_str(),
        "edge.ada.beach"
    );
    assert_eq!(
        projection.payload.outgoing_edges[0].to_node_id.as_str(),
        "node.place.beach"
    );
    assert_eq!(table_count(&conn, "commands"), 3);
    assert_eq!(table_count(&conn, "change_events"), 3);
    assert_eq!(table_count(&conn, "object_revisions"), 3);
    assert_eq!(table_count(&conn, "bible_graph_edges"), 1);
}

#[test]
fn duplicate_set_edge_command_is_idempotent() {
    let mut conn = memory_connection();
    let source = create_command("node.character.ada", "Ada");
    let target = create_command("node.place.beach", "Beach");
    apply_create_bible_graph_node(&mut conn, &source, 100).unwrap();
    apply_create_bible_graph_node(&mut conn, &target, 200).unwrap();
    let edge = edge_command();

    let (first, _) = apply_set_bible_graph_edge(&mut conn, &edge, 300).unwrap();
    let (second, projection) = apply_set_bible_graph_edge(&mut conn, &edge, 300).unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(projection.payload.outgoing_edges.len(), 1);
    assert_eq!(table_count(&conn, "commands"), 3);
    assert_eq!(table_count(&conn, "object_revisions"), 3);
    assert_eq!(table_count(&conn, "bible_graph_edges"), 1);
}

#[test]
fn set_edge_rejects_missing_target_without_history_rows() {
    let mut conn = memory_connection();
    let source = create_command("node.character.ada", "Ada");
    apply_create_bible_graph_node(&mut conn, &source, 100).unwrap();
    let edge = edge_command();

    let error = apply_set_bible_graph_edge(&mut conn, &edge, 300).unwrap_err();

    assert!(matches!(error, BibleGraphCommandError::InvalidCommand(_)));
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 1);
    assert_eq!(table_count(&conn, "bible_graph_edges"), 0);
}

#[test]
fn set_bible_graph_snapshot_field_creates_snapshot_and_updates_projection() {
    let mut conn = memory_connection();
    let node = create_command("node.character.ada", "Ada");
    apply_create_bible_graph_node(&mut conn, &node, 100).unwrap();
    let snapshot = snapshot_field_command(Some(FieldValue::Text("Rain-soaked".to_string())));

    let (outcome, projection) =
        apply_set_bible_graph_snapshot_field(&mut conn, &snapshot, 200).unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(
        projection.version,
        eidetic_core::contracts::ProjectionVersion(3)
    );
    assert_eq!(projection.payload.snapshots.len(), 1);
    assert_eq!(
        projection.payload.snapshots[0].snapshot.label,
        "Sequence 1 state"
    );
    assert_eq!(
        projection.payload.snapshots[0].fields[0].value,
        Some(FieldValue::Text("Rain-soaked".to_string()))
    );
    assert_eq!(table_count(&conn, "commands"), 2);
    assert_eq!(table_count(&conn, "change_events"), 2);
    assert_eq!(table_count(&conn, "object_revisions"), 2);
    assert_eq!(table_count(&conn, "bible_graph_snapshots"), 1);
    assert_eq!(table_count(&conn, "bible_graph_snapshot_fields"), 1);
}

#[test]
fn duplicate_set_snapshot_field_command_is_idempotent() {
    let mut conn = memory_connection();
    let node = create_command("node.character.ada", "Ada");
    apply_create_bible_graph_node(&mut conn, &node, 100).unwrap();
    let snapshot = snapshot_field_command(Some(FieldValue::Text("Rain-soaked".to_string())));

    let (first, _) = apply_set_bible_graph_snapshot_field(&mut conn, &snapshot, 200).unwrap();
    let (second, projection) =
        apply_set_bible_graph_snapshot_field(&mut conn, &snapshot, 200).unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(projection.payload.snapshots.len(), 1);
    assert_eq!(table_count(&conn, "commands"), 2);
    assert_eq!(table_count(&conn, "object_revisions"), 2);
    assert_eq!(table_count(&conn, "bible_graph_snapshots"), 1);
    assert_eq!(table_count(&conn, "bible_graph_snapshot_fields"), 1);
}

#[test]
fn set_snapshot_field_rejects_missing_node_without_history_rows() {
    let mut conn = memory_connection();
    let snapshot = snapshot_field_command(Some(FieldValue::Text("Rain-soaked".to_string())));

    let error = apply_set_bible_graph_snapshot_field(&mut conn, &snapshot, 200).unwrap_err();

    assert!(matches!(error, BibleGraphCommandError::InvalidCommand(_)));
    assert_eq!(table_count(&conn, "commands"), 0);
    assert_eq!(table_count(&conn, "object_revisions"), 0);
    assert_eq!(table_count(&conn, "bible_graph_snapshots"), 0);
    assert_eq!(table_count(&conn, "bible_graph_snapshot_fields"), 0);
}

#[test]
fn set_snapshot_field_rejects_blank_label_without_history_rows() {
    let mut conn = memory_connection();
    let node = create_command("node.character.ada", "Ada");
    apply_create_bible_graph_node(&mut conn, &node, 100).unwrap();
    let mut snapshot = snapshot_field_command(Some(FieldValue::Text("Rain-soaked".to_string())));
    snapshot.payload.label = " ".to_string();

    let error = apply_set_bible_graph_snapshot_field(&mut conn, &snapshot, 200).unwrap_err();

    assert!(matches!(error, BibleGraphCommandError::InvalidCommand(_)));
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 1);
    assert_eq!(table_count(&conn, "bible_graph_snapshots"), 0);
    assert_eq!(table_count(&conn, "bible_graph_snapshot_fields"), 0);
}

#[test]
fn set_snapshot_field_rejects_unknown_field_for_known_schema_without_history_rows() {
    let mut conn = memory_connection();
    let node = create_command("node.character.ada", "Ada");
    apply_create_bible_graph_node(&mut conn, &node, 100).unwrap();
    let mut snapshot = snapshot_field_command(Some(FieldValue::Text("Rain-soaked".to_string())));
    snapshot.payload.field_key = BibleGraphFieldKey::new("unknown").unwrap();

    let error = apply_set_bible_graph_snapshot_field(&mut conn, &snapshot, 200).unwrap_err();

    assert!(matches!(error, BibleGraphCommandError::InvalidCommand(_)));
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 1);
    assert_eq!(table_count(&conn, "bible_graph_snapshots"), 0);
    assert_eq!(table_count(&conn, "bible_graph_snapshot_fields"), 0);
}

fn table_count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}

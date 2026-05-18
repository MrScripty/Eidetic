use super::*;
use eidetic_core::contracts::{
    BibleGraphNodeId, BibleGraphSchemaKey, CommandEnvelope, EnsureCanonicalBibleRootsCommand,
};

fn memory_connection() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    bible_graph_store::create_schema(&conn).unwrap();
    conn
}

fn create_command(node_id: &str, name: &str) -> CommandEnvelope<CreateBibleGraphNodeCommand> {
    CommandEnvelope::new(CreateBibleGraphNodeCommand {
        node_id: BibleGraphNodeId::new(node_id).unwrap(),
        parent_id: None,
        schema_key: BibleGraphSchemaKey::new("character").unwrap(),
        name: name.to_string(),
        sort_order: 7,
    })
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
    assert!(projection.payload.parts.is_empty());
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

fn table_count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}

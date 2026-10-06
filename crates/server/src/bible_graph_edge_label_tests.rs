use super::*;
use crate::bible_graph_command;

fn setup(conn: &mut Connection) -> SetBibleGraphEdgeCommand {
    for id in ["Mara", "Eli"] {
        bible_graph_command::apply_create_bible_graph_node(
            conn,
            &CommandEnvelope::new(CreateBibleGraphNodeCommand {
                node_id: BibleGraphNodeId::new(id).unwrap(),
                parent_id: None,
                schema_key: BibleGraphSchemaKey::new("character").unwrap(),
                name: id.into(),
                sort_order: 0,
            }),
            1,
        )
        .unwrap();
    }
    let edge = SetBibleGraphEdgeCommand {
        edge_id: BibleGraphEdgeId::new("Mara.Eli").unwrap(),
        from_node_id: BibleGraphNodeId::new("Mara").unwrap(),
        to_node_id: BibleGraphNodeId::new("Eli").unwrap(),
        edge_kind: BibleGraphEdgeKind::References,
        label: "Mara trusts Eli".into(),
        directed: true,
        sort_order: 7,
    };
    set(conn, &edge);
    edge
}
fn set(conn: &mut Connection, edge: &SetBibleGraphEdgeCommand) {
    bible_graph_command::apply_set_bible_graph_edge(conn, &CommandEnvelope::new(edge.clone()), 10)
        .unwrap();
}
fn edit(
    conn: &Connection,
    edge: &SetBibleGraphEdgeCommand,
) -> CommandEnvelope<SetBibleGraphEdgeLabelCommand> {
    let projection = bible_graph_store::load_node_detail_projection(conn, &edge.from_node_id)
        .unwrap()
        .unwrap();
    CommandEnvelope::new(SetBibleGraphEdgeLabelCommand {
        edge_id: edge.edge_id.clone(),
        label: "Mara doubts Eli".into(),
        expected_revision_event_id: projection.edge_revision_event_ids[&edge.edge_id],
    })
}
fn rows(conn: &Connection) -> Vec<(String, Vec<Vec<rusqlite::types::Value>>)> {
    let tables = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").unwrap()
        .query_map([], |row| row.get::<_, String>(0)).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    tables
        .into_iter()
        .map(|table| {
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT * FROM \"{}\" ORDER BY rowid",
                    table.replace('"', "\"\"")
                ))
                .unwrap();
            let count = stmt.column_count();
            let values = stmt
                .query_map([], |row| (0..count).map(|i| row.get(i)).collect())
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            (table, values)
        })
        .collect()
}
fn delete(conn: &mut Connection, edge: &SetBibleGraphEdgeCommand) {
    bible_graph_command::apply_delete_bible_graph_edge(
        conn,
        &CommandEnvelope::new(DeleteBibleGraphEdgeCommand {
            edge_id: edge.edge_id.clone(),
        }),
        11,
    )
    .unwrap();
}

#[test]
fn label_only_update_preserves_structure_and_owned_history_and_replays_without_writes() {
    let mut conn = Connection::open_in_memory().unwrap();
    let edge = setup(&mut conn);
    let command = edit(&conn, &edge);
    let (outcome, projection) = apply(&mut conn, &command, 1).unwrap(); // deliberately earlier timestamp
    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    let projection = projection.expect("fresh write returns its source projection");
    let mut expected = edge.clone().into_edge();
    expected.label = command.payload.label.clone();
    assert_eq!(projection.payload.outgoing_edges, vec![expected]);
    let latest = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::BibleEdge,
        edge.edge_id.as_str(),
    )
    .unwrap()
    .pop()
    .unwrap();
    assert_eq!(latest.fields.len(), 1);
    assert_eq!(latest.fields[0].field_key, "label");
    assert_ne!(
        latest.change_event_id,
        command.payload.expected_revision_event_id
    );
    assert_eq!(
        projection.payload.edge_revision_event_ids[&edge.edge_id],
        latest.change_event_id
    );
    crate::bible_relationship_lineage::capture(
        &conn,
        &crate::ai_context_projection::load_ai_bible_context_projection(
            &conn,
            eidetic_core::timeline::node::NodeId(uuid::Uuid::nil()),
            None,
        )
        .unwrap()
        .payload,
    )
    .unwrap();
    let before = rows(&conn);
    let (outcome, projection) = apply(&mut conn, &command, 20).unwrap();
    assert_eq!(outcome, RecordChangeOutcome::AlreadyRecorded);
    assert!(projection.is_none());
    assert_eq!(rows(&conn), before);
    delete(&mut conn, &edge);
    let before = rows(&conn);
    let (outcome, projection) = apply(&mut conn, &command, 30).unwrap();
    assert_eq!(outcome, RecordChangeOutcome::AlreadyRecorded);
    assert!(projection.is_none());
    assert_eq!(rows(&conn), before);
    assert!(
        bible_graph_edge_store::load_edge(&conn, &edge.edge_id)
            .unwrap()
            .is_none()
    );
}

#[test]
fn committed_label_replay_after_edge_and_source_deletion_is_read_only() {
    let mut conn = Connection::open_in_memory().unwrap();
    let edge = setup(&mut conn);
    let command = edit(&conn, &edge);
    apply(&mut conn, &command, 12).unwrap();
    delete(&mut conn, &edge);
    bible_graph_command::apply_delete_bible_graph_node(
        &mut conn,
        &CommandEnvelope::new(DeleteBibleGraphNodeCommand {
            node_id: edge.from_node_id.clone(),
        }),
        13,
    )
    .unwrap();
    assert!(
        bible_graph_store::load_node_detail_projection(&conn, &edge.from_node_id)
            .unwrap()
            .is_none()
    );
    let before = rows(&conn);
    let changes = conn.total_changes();
    conn.pragma_update(None, "query_only", true).unwrap();
    let (outcome, projection) = apply(&mut conn, &command, 14).unwrap();
    assert_eq!(outcome, RecordChangeOutcome::AlreadyRecorded);
    assert!(projection.is_none());
    assert_eq!(conn.total_changes(), changes);
    assert_eq!(rows(&conn), before);
    conn.pragma_update(None, "query_only", false).unwrap();
    let fresh = CommandEnvelope::new(command.payload.clone());
    assert!(
        apply(&mut conn, &fresh, 15)
            .unwrap_err()
            .to_string()
            .contains("Relationship changed while editing")
    );
    assert_eq!(rows(&conn), before);
    let mut mismatched = command;
    mismatched.payload.label = "Different replay payload".into();
    assert!(apply(&mut conn, &mismatched, 16).is_err());
    assert_eq!(rows(&conn), before);
}

#[test]
fn stale_label_editor_cannot_overwrite_structure_label_aba_or_resurrect_deleted_identity() {
    for conflict in [
        "structure",
        "label",
        "label_aba",
        "delete",
        "delete_recreate",
    ] {
        let mut conn = Connection::open_in_memory().unwrap();
        let edge = setup(&mut conn);
        let command = edit(&conn, &edge);
        let mut newer = edge.clone();
        match conflict {
            "structure" => {
                newer.directed = false;
                newer.edge_kind = BibleGraphEdgeKind::ConflictsWith;
                newer.sort_order = 99;
                set(&mut conn, &newer);
            }
            "label" | "label_aba" => {
                newer.label = "New manual relationship".into();
                set(&mut conn, &newer);
                if conflict == "label_aba" {
                    set(&mut conn, &edge);
                }
            }
            _ => {
                delete(&mut conn, &edge);
                if conflict == "delete_recreate" {
                    set(&mut conn, &edge);
                }
            }
        }
        let before = rows(&conn);
        let error = apply(&mut conn, &command, 0).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("Relationship changed while editing"),
            "{conflict}: {error}"
        );
        assert_eq!(rows(&conn), before, "{conflict}");
        if conflict != "delete" {
            let fresh = edit(&conn, &edge);
            apply(&mut conn, &fresh, 0).unwrap();
            let saved = bible_graph_edge_store::load_edge(&conn, &edge.edge_id)
                .unwrap()
                .unwrap();
            let baseline = if conflict == "structure" { newer } else { edge };
            assert_eq!(
                (saved.directed, saved.edge_kind, saved.sort_order),
                (baseline.directed, baseline.edge_kind, baseline.sort_order)
            );
        }
    }
}

#[test]
fn label_store_transaction_guard_checks_owned_revision_even_if_graph_row_did_not_change() {
    let mut conn = Connection::open_in_memory().unwrap();
    let edge = setup(&mut conn);
    let command = edit(&conn, &edge);
    let envelope = CommandEnvelope::new(());
    let event = ChangeEvent::new(
        envelope.id,
        ChangeEventKind::UserEdit,
        "concurrent owned revision",
    );
    let revision = ObjectRevision::new(
        ObjectKind::BibleEdge,
        edge.edge_id.as_str(),
        event.id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "label",
        Some(FieldValue::Text(edge.label.clone())),
        Some(FieldValue::Text("Other manual label".into())),
    ));
    history_store::record_change(
        &mut conn,
        &envelope,
        "test.owned_edge_edit",
        &event,
        &[revision],
    )
    .unwrap();
    let before = rows(&conn);
    let tx = conn.transaction().unwrap();
    assert!(
        bible_graph_edge_store::set_edge_label_in_transaction(&tx, &command.payload, event.id)
            .is_err()
    );
    tx.rollback().unwrap();
    assert_eq!(rows(&conn), before);
}

#[test]
fn two_connection_label_and_delete_races_preserve_other_writer_rows() {
    for delete_race in [false, true] {
        let path = std::env::temp_dir().join(format!(
            "eidetic-edge-label-{}.sqlite",
            uuid::Uuid::new_v4()
        ));
        let mut editor = crate::sqlite::open_write_connection(&path).unwrap();
        let mut other = crate::sqlite::open_write_connection(&path).unwrap();
        let mut edge = setup(&mut editor);
        let command = edit(&editor, &edge);
        if delete_race {
            delete(&mut other, &edge);
        } else {
            edge.directed = false;
            edge.edge_kind = BibleGraphEdgeKind::ConflictsWith;
            set(&mut other, &edge);
        }
        let before = rows(&other);
        // This tests the actual store guard after the stale editor's read, in
        // a newly acquired writer transaction following the other commit.
        let tx = editor.transaction().unwrap();
        assert!(
            bible_graph_edge_store::set_edge_label_in_transaction(
                &tx,
                &command.payload,
                ChangeEventId::new()
            )
            .is_err()
        );
        tx.rollback().unwrap();
        assert_eq!(rows(&other), before);
        assert!(apply(&mut editor, &command, 0).is_err());
        assert_eq!(rows(&other), before);
        drop(editor);
        drop(other);
        std::fs::remove_file(path).unwrap();
    }
}

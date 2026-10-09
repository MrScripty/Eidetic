use super::*;
use crate::agent_workflow_harness::AgentWorkflowToolExecutor;
use crate::bible_graph_command as command;
use eidetic_core::timeline::node::NodeId;

fn node(conn: &mut Connection, id: &str, order: u32) -> BibleGraphNodeId {
    let node_id = BibleGraphNodeId::new(id).unwrap();
    command::apply_create_bible_graph_node(
        conn,
        &CommandEnvelope::new(CreateBibleGraphNodeCommand {
            node_id: node_id.clone(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            name: id.into(),
            sort_order: order,
        }),
        1,
    )
    .unwrap();
    node_id
}
fn edge(
    conn: &mut Connection,
    id: &str,
    from: &str,
    to: &str,
    kind: BibleGraphEdgeKind,
    directed: bool,
) {
    command::apply_set_bible_graph_edge(
        conn,
        &CommandEnvelope::new(SetBibleGraphEdgeCommand {
            edge_id: BibleGraphEdgeId::new(id).unwrap(),
            from_node_id: BibleGraphNodeId::new(from).unwrap(),
            to_node_id: BibleGraphNodeId::new(to).unwrap(),
            edge_kind: kind,
            label: id.into(),
            directed,
            sort_order: 0,
        }),
        2,
    )
    .unwrap();
}
fn fact(conn: &mut Connection, id: &str, value: &str) -> ChangeEventId {
    command::apply_set_bible_graph_field(
        conn,
        &CommandEnvelope::new(SetBibleGraphFieldCommand {
            node_id: BibleGraphNodeId::new(id).unwrap(),
            part_id: BibleGraphPartId::new(format!("{id}.profile")).unwrap(),
            part_key: BibleGraphPartKey::new("profile").unwrap(),
            part_name: "Profile".into(),
            part_sort_order: 0,
            field_id: BibleGraphFieldId::new(format!("{id}.tagline")).unwrap(),
            field_key: BibleGraphFieldKey::new("tagline").unwrap(),
            value: Some(FieldValue::Text(value.into())),
            field_sort_order: 0,
        }),
        3,
    )
    .unwrap()
    .1
    .change_event_id
    .unwrap()
}
fn snapshot(
    conn: &mut Connection,
    id: &str,
    at_ms: u64,
    key: &str,
    value: Option<&str>,
) -> ChangeEventId {
    command::apply_set_bible_graph_snapshot_field(
        conn,
        &CommandEnvelope::new(SetBibleGraphSnapshotFieldCommand {
            node_id: BibleGraphNodeId::new("Mara").unwrap(),
            snapshot_id: BibleGraphSnapshotId::new(id).unwrap(),
            at_ms,
            label: id.into(),
            snapshot_sort_order: 0,
            field_id: BibleGraphSnapshotFieldId::new(format!("{id}.{key}")).unwrap(),
            part_key: BibleGraphPartKey::new("profile").unwrap(),
            part_name: "Profile".into(),
            field_key: BibleGraphFieldKey::new(key).unwrap(),
            value: value.map(|v| FieldValue::Text(v.into())),
            field_sort_order: 0,
        }),
        4,
    )
    .unwrap()
    .1
    .change_event_id
    .unwrap()
}
fn query() -> BibleRecallRequest {
    BibleRecallRequest {
        anchor_node_id: BibleGraphNodeId::new("Mara").unwrap(),
        story_time_ms: None,
        direction: BibleRecallDirection::Both,
        edge_kinds: vec![],
        neighbor_limit: 8,
    }
}
fn ids(value: &ProjectionEnvelope<BibleRecallProjection>) -> Vec<&str> {
    value
        .payload
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect()
}
fn counts(conn: &Connection) -> (u64, u64, u64) {
    let count = |table| {
        conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
    };
    (
        count("change_events"),
        count("object_revisions"),
        count("commands"),
    )
}

#[test]
fn exact_one_hop_reaches_beyond_prefix_without_filler_or_generation_changes() {
    let mut conn = Connection::open_in_memory().unwrap();
    node(&mut conn, "Mara", 0);
    for i in 0..205 {
        node(&mut conn, &format!("Filler{i:03}"), i + 1);
    }
    node(&mut conn, "BeachHouse", 10000);
    edge(
        &mut conn,
        "home",
        "Mara",
        "BeachHouse",
        BibleGraphEdgeKind::LocatedIn,
        true,
    );
    fact(&mut conn, "Mara", "blue");
    fact(&mut conn, "BeachHouse", "Dry");
    let target = NodeId::new();
    let before =
        crate::ai_context_projection::load_ai_bible_context_projection(&conn, target, None)
            .unwrap();
    assert_eq!(before.payload.nodes.len(), 200);
    assert!(
        !before
            .payload
            .nodes
            .iter()
            .any(|node| node.node_id.as_str() == "BeachHouse")
    );
    let before_counts = counts(&conn);
    let result = load(&conn, &query()).unwrap();
    assert_eq!(ids(&result), ["Mara", "BeachHouse"]);
    assert_eq!(
        result.payload.nodes[1].fields[0].value,
        FieldValue::Text("Dry".into())
    );
    assert_eq!(
        result.payload.paths[0].relationship.edge.edge_kind,
        BibleGraphEdgeKind::LocatedIn
    );
    assert!(result.payload.relationships_untimed);
    assert_eq!(result.payload.omitted_neighbors, 0);
    let request = AgentToolRequest {
        tool_name: AgentToolName::new("read_bible_recall").unwrap(),
        arguments: AgentToolArguments::ReadBibleRecall { query: query() },
    };
    let output = crate::agent_graph_tools::AgentGraphReadTools::new(&conn)
        .execute_tool(&request)
        .unwrap();
    let AgentToolResultPayload::Text { text } = output else {
        panic!("text evidence");
    };
    assert_eq!(
        serde_json::from_str::<ProjectionEnvelope<BibleRecallProjection>>(&text).unwrap(),
        result
    );
    assert_eq!(counts(&conn), before_counts);
    assert_eq!(
        crate::ai_context_projection::load_ai_bible_context_projection(&conn, target, None)
            .unwrap(),
        before
    );
}

#[test]
fn direction_kinds_undirected_cycles_and_parallel_edges_are_bounded() {
    let mut conn = Connection::open_in_memory().unwrap();
    for id in ["Mara", "A", "B", "C", "Third"] {
        node(&mut conn, id, 0);
    }
    edge(
        &mut conn,
        "out",
        "Mara",
        "A",
        BibleGraphEdgeKind::Owns,
        true,
    );
    edge(
        &mut conn,
        "in",
        "B",
        "Mara",
        BibleGraphEdgeKind::References,
        true,
    );
    edge(
        &mut conn,
        "undirected",
        "C",
        "Mara",
        BibleGraphEdgeKind::Custom("trusts".into()),
        false,
    );
    edge(
        &mut conn,
        "loop",
        "Mara",
        "Mara",
        BibleGraphEdgeKind::Owns,
        true,
    );
    edge(
        &mut conn,
        "cycle",
        "A",
        "Mara",
        BibleGraphEdgeKind::References,
        true,
    );
    edge(
        &mut conn,
        "second-hop",
        "A",
        "Third",
        BibleGraphEdgeKind::Owns,
        true,
    );
    let mut q = query();
    q.direction = BibleRecallDirection::Outgoing;
    assert_eq!(ids(&load(&conn, &q).unwrap()), ["Mara", "A", "C"]);
    q.direction = BibleRecallDirection::Incoming;
    assert_eq!(ids(&load(&conn, &q).unwrap()), ["Mara", "A", "B", "C"]);
    q.edge_kinds = vec![BibleGraphEdgeKind::Custom("trusts".into())];
    let result = load(&conn, &q).unwrap();
    assert_eq!(ids(&result), ["Mara", "C"]);
    assert_eq!(
        result.payload.paths[0]
            .relationship
            .edge
            .from_node_id
            .as_str(),
        "C"
    );
    q = query();
    q.neighbor_limit = 1;
    let result = load(&conn, &q).unwrap();
    assert_eq!(ids(&result), ["Mara", "A"]);
    assert_eq!(result.payload.omitted_neighbors, 2);
    assert_eq!(result.payload.omitted_edges, 2);
    for i in 0..40 {
        edge(
            &mut conn,
            &format!("parallel{i:03}"),
            "Mara",
            "A",
            BibleGraphEdgeKind::Owns,
            true,
        );
    }
    let result = load(&conn, &query()).unwrap();
    assert_eq!(result.payload.paths.len(), 32);
    assert_eq!(result.payload.omitted_edges, 12);
    for id in ["A", "B", "C"] {
        assert!(
            result
                .payload
                .paths
                .iter()
                .any(|p| p.neighbor_node_id.as_str() == id)
        );
    }
    for limit in [0, 9, u32::MAX] {
        q.neighbor_limit = limit;
        assert!(load(&conn, &q).is_err());
    }
}

#[test]
fn temporal_resolution_carries_exact_assertions_and_separate_snapshot_clocks() {
    let mut conn = Connection::open_in_memory().unwrap();
    node(&mut conn, "Mara", 0);
    let baseline = fact(&mut conn, "Mara", "Dry");
    let assertion = snapshot(&mut conn, "Opening", 1000, "tagline", Some("Rain"));
    // Metadata now comes from a later write to another sparse field.
    let metadata = snapshot(&mut conn, "Opening", 1000, "summary", Some("Wet shoes"));
    snapshot(&mut conn, "Future", 2000, "motivation", Some("Witness"));
    let unknown = load(&conn, &query()).unwrap();
    assert!(unknown.payload.nodes[0].fields.is_empty());
    assert_eq!(unknown.payload.nodes[0].unresolved_timed_fields.len(), 3);
    assert!(!serde_json::to_string(&unknown).unwrap().contains("Rain"));
    let mut q = query();
    q.story_time_ms = Some(999);
    let before = load(&conn, &q).unwrap();
    assert_eq!(
        before.payload.nodes[0].fields[0].source,
        BibleRecallFieldSource::Baseline {
            field_id: BibleGraphFieldId::new("Mara.tagline").unwrap(),
            revision_event_id: baseline
        }
    );
    assert_eq!(before.payload.nodes[0].unresolved_timed_fields.len(), 2);
    q.story_time_ms = Some(1000);
    let at = load(&conn, &q).unwrap();
    let field = at.payload.nodes[0]
        .fields
        .iter()
        .find(|f| f.field_key.as_str() == "tagline")
        .unwrap();
    assert_eq!(field.value, FieldValue::Text("Rain".into()));
    assert_eq!(
        field.source,
        BibleRecallFieldSource::Snapshot {
            snapshot_id: BibleGraphSnapshotId::new("Opening").unwrap(),
            snapshot_field_id: BibleGraphSnapshotFieldId::new("Opening.tagline").unwrap(),
            snapshot_revision_event_id: metadata,
            field_revision_event_id: assertion,
            at_ms: 1000,
            label: "Opening".into()
        }
    );
    assert_eq!(at.version, before.version);
    snapshot(&mut conn, "Clear", 1500, "tagline", None);
    q.story_time_ms = Some(1500);
    let cleared = load(&conn, &q).unwrap();
    assert!(
        !cleared.payload.nodes[0]
            .fields
            .iter()
            .any(|f| f.field_key.as_str() == "tagline")
    );
    assert!(
        cleared.payload.nodes[0]
            .unresolved_timed_fields
            .iter()
            .any(|f| f.field_key.as_str() == "tagline")
    );
    snapshot(&mut conn, "Conflict", 1500, "tagline", Some("Storm"));
    assert!(
        load(&conn, &q)
            .unwrap_err()
            .to_string()
            .contains("conflicting story facts")
    );
}

#[test]
fn budgets_omit_whole_values_and_refuse_oversized_identity_evidence() {
    let mut conn = Connection::open_in_memory().unwrap();
    node(&mut conn, "Mara", 0);
    node(&mut conn, "A", 1);
    edge(
        &mut conn,
        "link",
        "Mara",
        "A",
        BibleGraphEdgeKind::References,
        true,
    );
    fact(&mut conn, "Mara", "Preserve whole anchor fact");
    fact(&mut conn, "A", &"x".repeat(BIBLE_RECALL_MAX_BYTES));
    let result = load(&conn, &query()).unwrap();
    assert_eq!(
        result.payload.nodes[0].fields[0].value,
        FieldValue::Text("Preserve whole anchor fact".into())
    );
    assert!(result.payload.nodes[1].fields.is_empty());
    assert_eq!(result.payload.nodes[1].omitted_fields, 1);
    assert_eq!(result.payload.paths.len(), 1);
    assert!(serde_json::to_vec(&result).unwrap().len() <= BIBLE_RECALL_MAX_BYTES);
    command::apply_set_bible_graph_node_name(
        &mut conn,
        &CommandEnvelope::new(SetBibleGraphNodeNameCommand {
            node_id: BibleGraphNodeId::new("A").unwrap(),
            name: "y".repeat(BIBLE_RECALL_MAX_BYTES),
        }),
        5,
    )
    .unwrap();
    assert!(
        load(&conn, &query())
            .unwrap_err()
            .to_string()
            .contains("identities and path evidence")
    );
}

#[test]
fn deletions_and_owned_history_inconsistency_refuse_false_evidence() {
    let mut conn = Connection::open_in_memory().unwrap();
    node(&mut conn, "Mara", 0);
    node(&mut conn, "A", 1);
    edge(
        &mut conn,
        "link",
        "Mara",
        "A",
        BibleGraphEdgeKind::References,
        true,
    );
    fact(&mut conn, "Mara", "blue");
    let result = load(&conn, &query()).unwrap();
    assert!(result.payload.nodes[0].name_revision_event_id.is_some());
    command::apply_delete_bible_graph_edge(
        &mut conn,
        &CommandEnvelope::new(DeleteBibleGraphEdgeCommand {
            edge_id: BibleGraphEdgeId::new("link").unwrap(),
        }),
        5,
    )
    .unwrap();
    assert_eq!(ids(&load(&conn, &query()).unwrap()), ["Mara"]);
    edge(
        &mut conn,
        "link2",
        "Mara",
        "A",
        BibleGraphEdgeKind::References,
        true,
    );
    command::apply_delete_bible_graph_edge(
        &mut conn,
        &CommandEnvelope::new(DeleteBibleGraphEdgeCommand {
            edge_id: BibleGraphEdgeId::new("link2").unwrap(),
        }),
        6,
    )
    .unwrap();
    command::apply_delete_bible_graph_node(
        &mut conn,
        &CommandEnvelope::new(DeleteBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new("A").unwrap(),
        }),
        6,
    )
    .unwrap();
    assert_eq!(ids(&load(&conn, &query()).unwrap()), ["Mara"]);
    conn.execute(
        "UPDATE bible_graph_nodes SET name='forged' WHERE id='Mara'",
        [],
    )
    .unwrap();
    assert!(load(&conn, &query()).is_err());
    conn.execute(
        "UPDATE bible_graph_nodes SET name='Mara' WHERE id='Mara'",
        [],
    )
    .unwrap();
    command::apply_delete_bible_graph_node(
        &mut conn,
        &CommandEnvelope::new(DeleteBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new("Mara").unwrap(),
        }),
        7,
    )
    .unwrap();
    assert!(
        load(&conn, &query())
            .unwrap_err()
            .to_string()
            .contains("no longer exists")
    );
}

#[test]
fn read_transaction_keeps_values_source_revisions_and_version_atomic() {
    let path = std::env::temp_dir().join(format!("eidetic-recall-{}.sqlite", uuid::Uuid::new_v4()));
    let mut writer = crate::sqlite::open_write_connection(&path).unwrap();
    node(&mut writer, "Mara", 0);
    let old_event = fact(&mut writer, "Mara", "blue");
    let reader = crate::sqlite::open_write_connection(&path).unwrap();
    let tx = reader.unchecked_transaction().unwrap();
    let before = load(&tx, &query()).unwrap();
    let new_event = fact(&mut writer, "Mara", "amber");
    assert_eq!(load(&tx, &query()).unwrap(), before);
    let BibleRecallFieldSource::Baseline {
        revision_event_id, ..
    } = before.payload.nodes[0].fields[0].source
    else {
        panic!("baseline");
    };
    assert_eq!(revision_event_id, old_event);
    tx.commit().unwrap();
    let after = load(&reader, &query()).unwrap();
    assert!(after.version.0 > before.version.0);
    assert_eq!(
        after.payload.nodes[0].fields[0].value,
        FieldValue::Text("amber".into())
    );
    let BibleRecallFieldSource::Baseline {
        revision_event_id, ..
    } = after.payload.nodes[0].fields[0].source
    else {
        panic!("baseline");
    };
    assert_eq!(revision_event_id, new_event);
    drop(reader);
    drop(writer);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn inspection_preserves_manual_screenplay_pending_proposals_and_dependency_history() {
    let (mut conn, _, _, b, _) = crate::script_impact_review::tests::fixture();
    node(&mut conn, "Mara", 0);
    fact(&mut conn, "Mara", "blue");
    crate::script_impact_review::tests::propose(&mut conn, &b);
    let document =
        crate::script_store::load_document_projection_envelope(&conn, &b.document_id).unwrap();
    let proposals =
        crate::propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
            .unwrap();
    let before = counts(&conn);
    let dependencies: u64 = conn
        .query_row("SELECT count(*) FROM semantic_dependencies", [], |r| {
            r.get(0)
        })
        .unwrap();
    load(&conn, &query()).unwrap();
    assert_eq!(
        crate::script_store::load_document_projection_envelope(&conn, &b.document_id).unwrap(),
        document
    );
    assert_eq!(
        crate::propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
            .unwrap(),
        proposals
    );
    assert_eq!(counts(&conn), before);
    assert_eq!(
        conn.query_row("SELECT count(*) FROM semantic_dependencies", [], |r| r
            .get::<_, u64>(0))
            .unwrap(),
        dependencies
    );
}

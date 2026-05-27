use super::*;
use eidetic_core::contracts::{
    BibleRenderGraphProjectionRequest, ContextEvaluation, ContextEvaluationId,
    ContextEvaluationTaskKind, ContextInfluenceId, ContextInfluenceKind,
    ContextInfluenceProvenance, ContextInfluenceRecord, RecordContextEvaluationCommand,
};
use eidetic_core::timeline::node::{NodeId, StoryLevel, StoryNode};
use eidetic_core::timeline::timing::TimeRange;

#[test]
fn render_graph_projection_envelope_applies_bounded_request() {
    let mut conn = memory_connection();
    seed_node(&mut conn, "node.character.ada", "Ada", 10);
    seed_node(&mut conn, "node.place.beach", "Beach", 20);
    seed_node(&mut conn, "node.place.tower", "Tower", 30);
    seed_edge(
        &mut conn,
        "edge.ada.beach",
        "node.character.ada",
        "node.place.beach",
        1,
    );
    seed_edge(
        &mut conn,
        "edge.beach.tower",
        "node.place.beach",
        "node.place.tower",
        2,
    );

    let projection = load_render_graph_projection_envelope(
        &conn,
        &BibleRenderGraphProjectionRequest {
            selected_node_id: Some(BibleGraphNodeId::new("node.character.ada").unwrap()),
            neighborhood_depth: 1,
            max_nodes: 10,
            ..BibleRenderGraphProjectionRequest::default()
        },
    )
    .unwrap();

    let node_ids: Vec<_> = projection
        .payload
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();
    assert_eq!(
        node_ids,
        vec!["node.character.ada", "node.place.beach", "node.place.tower"]
    );
    assert_eq!(projection.payload.edges.len(), 2);
    assert_eq!(
        projection.payload.edges[0].edge_id.as_str(),
        "edge.ada.beach"
    );
    assert_eq!(
        projection.payload.edges[1].edge_id.as_str(),
        "edge.beach.tower"
    );
}

#[test]
fn render_graph_projection_limits_default_query() {
    let mut conn = memory_connection();
    for index in 0..25 {
        seed_node(
            &mut conn,
            &format!("node.test.{index:02}"),
            &format!("Node {index:02}"),
            index,
        );
    }

    let projection = load_render_graph_projection_envelope(
        &conn,
        &BibleRenderGraphProjectionRequest {
            max_nodes: 7,
            ..BibleRenderGraphProjectionRequest::default()
        },
    )
    .unwrap();

    let node_ids: Vec<_> = projection
        .payload
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();
    assert_eq!(node_ids.len(), 7);
    assert_eq!(node_ids[0], "node.test.00");
    assert_eq!(node_ids[6], "node.test.06");
}

#[test]
fn render_graph_projection_limits_edges() {
    let mut conn = memory_connection();
    seed_node(&mut conn, "node.character.ada", "Ada", 10);
    seed_node(&mut conn, "node.place.beach", "Beach", 20);
    seed_node(&mut conn, "node.place.tower", "Tower", 30);
    seed_edge(
        &mut conn,
        "edge.ada.beach",
        "node.character.ada",
        "node.place.beach",
        1,
    );
    seed_edge(
        &mut conn,
        "edge.ada.tower",
        "node.character.ada",
        "node.place.tower",
        2,
    );

    let projection = load_render_graph_projection_envelope(
        &conn,
        &BibleRenderGraphProjectionRequest {
            selected_node_id: Some(BibleGraphNodeId::new("node.character.ada").unwrap()),
            neighborhood_depth: 1,
            max_nodes: 10,
            max_edges: 1,
            ..BibleRenderGraphProjectionRequest::default()
        },
    )
    .unwrap();

    assert_eq!(projection.payload.edges.len(), 1);
    assert_eq!(
        projection.payload.edges[0].edge_id.as_str(),
        "edge.ada.beach"
    );
    assert_eq!(projection.payload.neighborhoods.len(), 2);
}

#[test]
fn render_graph_projection_filters_edges_by_kind_before_limit() {
    let mut conn = memory_connection();
    seed_node(&mut conn, "node.character.ada", "Ada", 10);
    seed_node(&mut conn, "node.place.beach", "Beach", 20);
    seed_node(&mut conn, "node.place.tower", "Tower", 30);
    seed_edge_with_kind(
        &mut conn,
        "edge.ada.tower",
        "node.character.ada",
        "node.place.tower",
        BibleGraphEdgeKind::References,
        1,
    );
    seed_edge_with_kind(
        &mut conn,
        "edge.ada.beach",
        "node.character.ada",
        "node.place.beach",
        BibleGraphEdgeKind::LocatedIn,
        2,
    );

    let projection = load_render_graph_projection_envelope(
        &conn,
        &BibleRenderGraphProjectionRequest {
            selected_node_id: Some(BibleGraphNodeId::new("node.character.ada").unwrap()),
            neighborhood_depth: 1,
            edge_kinds: vec![BibleGraphEdgeKind::LocatedIn],
            max_nodes: 10,
            max_edges: 1,
            ..BibleRenderGraphProjectionRequest::default()
        },
    )
    .unwrap();

    assert_eq!(projection.payload.edges.len(), 1);
    assert_eq!(
        projection.payload.edges[0].edge_id.as_str(),
        "edge.ada.beach"
    );
    assert_eq!(
        projection.payload.edges[0].edge_kind,
        BibleGraphEdgeKind::LocatedIn
    );
}

#[test]
fn render_graph_projection_search_treats_like_wildcards_as_literal_text() {
    let mut conn = memory_connection();
    seed_node(&mut conn, "node.place.dry", "Dry Archive", 10);
    seed_node(&mut conn, "node.place.percent", "100% Rain Room", 20);

    let projection = load_render_graph_projection_envelope(
        &conn,
        &BibleRenderGraphProjectionRequest {
            search: Some("%".to_string()),
            max_nodes: 10,
            ..BibleRenderGraphProjectionRequest::default()
        },
    )
    .unwrap();

    let node_ids: Vec<_> = projection
        .payload
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();
    assert_eq!(node_ids, vec!["node.place.percent"]);
}

#[test]
fn render_graph_projection_keeps_empty_search_results_empty() {
    let mut conn = memory_connection();
    seed_node(&mut conn, "node.character.ada", "Ada", 10);
    seed_node(&mut conn, "node.place.beach", "Beach", 20);

    let projection = load_render_graph_projection_envelope(
        &conn,
        &BibleRenderGraphProjectionRequest {
            search: Some("tower".to_string()),
            max_nodes: 10,
            ..BibleRenderGraphProjectionRequest::default()
        },
    )
    .unwrap();

    assert!(projection.payload.nodes.is_empty());
    assert!(projection.payload.edges.is_empty());
    assert!(projection.payload.influences.is_empty());
}

#[test]
fn render_graph_projection_queries_focused_root_descendants() {
    let mut conn = memory_connection();
    seed_parented_node(&mut conn, "node.root", None, "Root", 1);
    seed_parented_node(&mut conn, "node.root.child", Some("node.root"), "Child", 2);
    seed_parented_node(
        &mut conn,
        "node.root.grandchild",
        Some("node.root.child"),
        "Grandchild",
        3,
    );
    seed_parented_node(&mut conn, "node.other", None, "Other", 4);

    let projection = load_render_graph_projection_envelope(
        &conn,
        &BibleRenderGraphProjectionRequest {
            focused_root_id: Some(BibleGraphNodeId::new("node.root").unwrap()),
            neighborhood_depth: 1,
            max_nodes: 10,
            ..BibleRenderGraphProjectionRequest::default()
        },
    )
    .unwrap();

    let node_ids: Vec<_> = projection
        .payload
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();
    assert_eq!(node_ids, vec!["node.root", "node.root.child"]);
}

#[test]
fn render_graph_projection_keeps_ancestor_expansion_within_node_limit() {
    let mut conn = memory_connection();
    seed_parented_node(&mut conn, "node.root", None, "Root", 1);
    seed_parented_node(&mut conn, "node.root.child", Some("node.root"), "Child", 2);
    seed_parented_node(
        &mut conn,
        "node.root.grandchild",
        Some("node.root.child"),
        "Grandchild",
        3,
    );

    let projection = load_render_graph_projection_envelope(
        &conn,
        &BibleRenderGraphProjectionRequest {
            selected_node_id: Some(BibleGraphNodeId::new("node.root.grandchild").unwrap()),
            neighborhood_depth: 1,
            max_nodes: 1,
            ..BibleRenderGraphProjectionRequest::default()
        },
    )
    .unwrap();

    let node_ids: Vec<_> = projection
        .payload
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();
    assert_eq!(node_ids, vec!["node.root.grandchild"]);
}

#[test]
fn render_graph_projection_includes_selected_timeline_influences() {
    let mut conn = memory_connection();
    let timeline_node_id = NodeId::new();
    seed_node(&mut conn, "node.character.ada", "Ada", 10);
    seed_node(&mut conn, "node.place.beach", "Beach", 20);
    seed_node(&mut conn, "node.place.tower", "Tower", 30);
    seed_edge(
        &mut conn,
        "edge.ada.beach",
        "node.character.ada",
        "node.place.beach",
        1,
    );
    seed_context_influence(
        &mut conn,
        timeline_node_id,
        "node.character.ada",
        "edge.ada.beach",
    );

    let projection = load_render_graph_projection_envelope(
        &conn,
        &BibleRenderGraphProjectionRequest {
            selected_timeline_node_id: Some(timeline_node_id),
            max_nodes: 10,
            ..BibleRenderGraphProjectionRequest::default()
        },
    )
    .unwrap();

    let node_ids: Vec<_> = projection
        .payload
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();
    assert_eq!(
        node_ids,
        vec!["node.character.ada", "node.place.beach", "node.place.tower"]
    );
    assert_eq!(projection.payload.edges.len(), 1);
    assert_eq!(projection.payload.influences.len(), 1);
    assert_eq!(
        projection.version,
        eidetic_core::contracts::ProjectionVersion(7)
    );
    assert_eq!(
        projection.payload.influences[0]
            .bible_node_id
            .as_ref()
            .unwrap()
            .as_str(),
        "node.character.ada"
    );
}

#[test]
fn render_graph_projection_includes_active_playhead_context_influences() {
    let mut conn = memory_connection();
    let inactive_timeline_node_id = NodeId::new();
    let active_timeline_node_id = NodeId::new();
    seed_timeline_node(&mut conn, inactive_timeline_node_id, 0, 1_000);
    seed_timeline_node(&mut conn, active_timeline_node_id, 1_000, 2_000);
    seed_node(&mut conn, "node.character.ada", "Ada", 10);
    seed_node(&mut conn, "node.place.beach", "Beach", 20);
    seed_edge(
        &mut conn,
        "edge.ada.beach",
        "node.character.ada",
        "node.place.beach",
        1,
    );
    seed_context_influence(
        &mut conn,
        inactive_timeline_node_id,
        "node.place.beach",
        "edge.ada.beach",
    );
    seed_context_influence(
        &mut conn,
        active_timeline_node_id,
        "node.character.ada",
        "edge.ada.beach",
    );

    let projection = load_render_graph_projection_envelope(
        &conn,
        &BibleRenderGraphProjectionRequest {
            active_timeline_ms: Some(1_500),
            max_nodes: 10,
            ..BibleRenderGraphProjectionRequest::default()
        },
    )
    .unwrap();

    assert_eq!(projection.payload.active_timeline_ms, Some(1_500));
    assert_eq!(projection.payload.influences.len(), 1);
    assert_eq!(
        projection.payload.influences[0].timeline_node_id,
        active_timeline_node_id
    );
    assert_eq!(
        projection.payload.influences[0]
            .bible_node_id
            .as_ref()
            .unwrap()
            .as_str(),
        "node.character.ada"
    );
}

#[test]
fn render_graph_projection_keeps_default_graph_without_active_playhead_influences() {
    let mut conn = memory_connection();
    let timeline_node_id = NodeId::new();
    seed_timeline_node(&mut conn, timeline_node_id, 1_000, 2_000);
    seed_node(&mut conn, "node.character.ada", "Ada", 10);
    seed_node(&mut conn, "node.place.beach", "Beach", 20);

    let projection = load_render_graph_projection_envelope(
        &conn,
        &BibleRenderGraphProjectionRequest {
            active_timeline_ms: Some(1_500),
            max_nodes: 10,
            ..BibleRenderGraphProjectionRequest::default()
        },
    )
    .unwrap();

    let node_ids: Vec<_> = projection
        .payload
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();
    assert_eq!(projection.payload.active_timeline_ms, Some(1_500));
    assert_eq!(projection.payload.influences.len(), 0);
    assert_eq!(node_ids, vec!["node.character.ada", "node.place.beach"]);
}

#[test]
fn render_graph_projection_reloads_selected_context_influences() {
    let path = std::env::temp_dir().join(format!(
        "eidetic-render-graph-reload-{}.sqlite",
        uuid::Uuid::new_v4()
    ));
    let timeline_node_id = NodeId::new();
    let request = BibleRenderGraphProjectionRequest {
        selected_timeline_node_id: Some(timeline_node_id),
        max_nodes: 10,
        ..BibleRenderGraphProjectionRequest::default()
    };

    let original = {
        let mut conn = crate::sqlite::open_write_connection(&path).unwrap();
        create_schema(&conn).unwrap();
        seed_node(&mut conn, "node.character.ada", "Ada", 10);
        seed_node(&mut conn, "node.place.beach", "Beach", 20);
        seed_edge(
            &mut conn,
            "edge.ada.beach",
            "node.character.ada",
            "node.place.beach",
            1,
        );
        seed_context_influence(
            &mut conn,
            timeline_node_id,
            "node.character.ada",
            "edge.ada.beach",
        );
        load_render_graph_projection_envelope(&conn, &request).unwrap()
    };

    let reloaded = {
        let conn = crate::sqlite::open_write_connection(&path).unwrap();
        load_render_graph_projection_envelope(&conn, &request).unwrap()
    };

    cleanup_sqlite_files(&path);
    assert_eq!(reloaded.version, original.version);
    assert_eq!(reloaded.change_event_id, original.change_event_id);
    assert_eq!(reloaded.payload, original.payload);
}

#[test]
fn render_graph_projection_keeps_selected_timeline_influences_when_searching() {
    let mut conn = memory_connection();
    let timeline_node_id = NodeId::new();
    seed_node(&mut conn, "node.character.ada", "Ada", 10);
    seed_node(&mut conn, "node.place.beach", "Beach", 20);
    seed_node(&mut conn, "node.place.tower", "Tower", 30);
    seed_edge(
        &mut conn,
        "edge.ada.beach",
        "node.character.ada",
        "node.place.beach",
        1,
    );
    seed_context_influence(
        &mut conn,
        timeline_node_id,
        "node.character.ada",
        "edge.ada.beach",
    );

    let projection = load_render_graph_projection_envelope(
        &conn,
        &BibleRenderGraphProjectionRequest {
            selected_timeline_node_id: Some(timeline_node_id),
            search: Some("tower".to_string()),
            max_nodes: 10,
            ..BibleRenderGraphProjectionRequest::default()
        },
    )
    .unwrap();

    let node_ids: Vec<_> = projection
        .payload
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();
    assert_eq!(
        node_ids,
        vec!["node.character.ada", "node.place.beach", "node.place.tower"]
    );
    assert_eq!(projection.payload.edges.len(), 1);
    assert_eq!(projection.payload.influences.len(), 1);
}

fn seed_context_influence(
    conn: &mut Connection,
    timeline_node_id: NodeId,
    bible_node_id: &str,
    bible_edge_id: &str,
) {
    let evaluation_id = ContextEvaluationId::new();
    let command = CommandEnvelope::new(RecordContextEvaluationCommand {
        evaluation: ContextEvaluation {
            id: evaluation_id,
            target_node_id: timeline_node_id,
            task_kind: ContextEvaluationTaskKind::GenerateTimelineContext,
            summary: "Scene graph context".to_string(),
            distilled_context: Some("Ada is at the beach.".to_string()),
            created_at_ms: 100,
        },
        influences: vec![ContextInfluenceRecord {
            id: ContextInfluenceId::new(),
            evaluation_id,
            timeline_node_id,
            source_layer: StoryLevel::Scene,
            influence_kind: ContextInfluenceKind::Direct,
            confidence: 0.9,
            reason: "Scene uses Ada at the beach.".to_string(),
            provenance: ContextInfluenceProvenance::AiSelected,
            bible_node_id: Some(BibleGraphNodeId::new(bible_node_id).unwrap()),
            bible_edge_id: Some(BibleGraphEdgeId::new(bible_edge_id).unwrap()),
            introduced_by_node_id: Some(timeline_node_id),
            sort_order: 1,
        }],
    });
    crate::context_influence_store::record_context_evaluation(conn, &command, 100).unwrap();
}

fn seed_timeline_node(conn: &mut Connection, node_id: NodeId, start_ms: u64, end_ms: u64) {
    let mut node = StoryNode::new(
        "Timeline node",
        StoryLevel::Scene,
        TimeRange::new(start_ms, end_ms).unwrap(),
    );
    node.id = node_id;
    let tx = conn.transaction().unwrap();
    crate::timeline_node_store::upsert_nodes_in_transaction(&tx, &[node]).unwrap();
    tx.commit().unwrap();
}

fn cleanup_sqlite_files(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
}

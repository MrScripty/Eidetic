use super::*;
use crate::contracts::{
    BibleGraphSchemaKey, ContextEvaluationId, ContextInfluenceId, ContextInfluenceKind,
    ContextInfluenceProvenance, ContextInfluenceRecord,
};
use crate::timeline::node::{NodeId, StoryLevel};

#[test]
fn render_graph_projection_is_deterministic_and_indexes_neighbors() {
    let root = BibleGraphNode {
        id: BibleGraphNodeId::new("canonical.characters").unwrap(),
        parent_id: None,
        schema_key: BibleGraphSchemaKey::new("canonical.root.characters").unwrap(),
        name: "Characters".to_string(),
        system_owned: true,
        sort_order: 0,
    };
    let ada = BibleGraphNode {
        id: BibleGraphNodeId::new("node.character.ada").unwrap(),
        parent_id: Some(root.id.clone()),
        schema_key: BibleGraphSchemaKey::new("character").unwrap(),
        name: "Ada".to_string(),
        system_owned: false,
        sort_order: 1,
    };
    let beach = BibleGraphNode {
        id: BibleGraphNodeId::new("node.place.beach").unwrap(),
        parent_id: None,
        schema_key: BibleGraphSchemaKey::new("place").unwrap(),
        name: "Beach".to_string(),
        system_owned: false,
        sort_order: 2,
    };
    let edge = BibleGraphEdge {
        id: BibleGraphEdgeId::new("edge.ada.beach").unwrap(),
        from_node_id: ada.id.clone(),
        to_node_id: beach.id.clone(),
        edge_kind: BibleGraphEdgeKind::LocatedIn,
        label: "located in".to_string(),
        directed: true,
        sort_order: 0,
    };

    let projection = BibleRenderGraphProjection::from_graph(vec![beach, ada, root], vec![edge]);

    assert_eq!(projection.nodes[0].node_id.as_str(), "canonical.characters");
    assert_eq!(projection.nodes[0].position.z, SYSTEM_NODE_Z);
    assert_eq!(projection.nodes[1].depth, 1);
    assert_eq!(projection.nodes[1].position.x, NODE_COLUMN_SPACING);
    assert_eq!(projection.edges[0].edge_id.as_str(), "edge.ada.beach");
    assert_eq!(projection.neighborhoods.len(), 2);
    assert_eq!(
        projection.neighborhoods[0].connected_node_ids[0].as_str(),
        "node.place.beach"
    );
}

#[test]
fn render_graph_projection_round_trips() {
    let projection = BibleRenderGraphProjection {
        nodes: vec![BibleRenderGraphNode {
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            label: "Ada".to_string(),
            system_owned: false,
            sort_order: 0,
            depth: 0,
            position: BibleRenderGraphPosition {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        }],
        edges: Vec::new(),
        neighborhoods: Vec::new(),
        influences: Vec::new(),
    };

    let json = serde_json::to_string(&projection).unwrap();
    let round_trip: BibleRenderGraphProjection = serde_json::from_str(&json).unwrap();

    assert_eq!(round_trip, projection);
}

#[test]
fn render_graph_projection_request_filters_selected_neighborhood() {
    let root = graph_node("canonical.characters", None, "root", "Characters", true, 0);
    let ada = graph_node(
        "node.character.ada",
        Some("canonical.characters"),
        "character",
        "Ada",
        false,
        1,
    );
    let beach = graph_node("node.place.beach", None, "place", "Beach", false, 2);
    let tower = graph_node("node.place.tower", None, "place", "Tower", false, 3);
    let ada_beach = graph_edge("edge.ada.beach", &ada.id, &beach.id, 0);
    let beach_tower = graph_edge("edge.beach.tower", &beach.id, &tower.id, 1);

    let projection = BibleRenderGraphProjection::from_graph_for_request(
        vec![root, tower, beach, ada.clone()],
        vec![beach_tower, ada_beach],
        &BibleRenderGraphProjectionRequest {
            selected_node_id: Some(ada.id),
            neighborhood_depth: 1,
            max_nodes: 10,
            ..BibleRenderGraphProjectionRequest::default()
        },
    );

    let node_ids: Vec<_> = projection
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();
    assert_eq!(
        node_ids,
        vec![
            "canonical.characters",
            "node.character.ada",
            "node.place.beach"
        ]
    );
    assert_eq!(projection.edges.len(), 1);
    assert_eq!(projection.edges[0].edge_id.as_str(), "edge.ada.beach");
}

#[test]
fn render_graph_projection_request_bounds_default_projection() {
    let nodes = vec![
        graph_node("node.place.alpha", None, "place", "Alpha", false, 1),
        graph_node("node.place.beta", None, "place", "Beta", false, 2),
        graph_node("node.place.gamma", None, "place", "Gamma", false, 3),
    ];

    let projection = BibleRenderGraphProjection::from_graph_for_request(
        nodes,
        Vec::new(),
        &BibleRenderGraphProjectionRequest {
            max_nodes: 2,
            ..BibleRenderGraphProjectionRequest::default()
        },
    );

    let node_ids: Vec<_> = projection
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();
    assert_eq!(node_ids, vec!["node.place.alpha", "node.place.beta"]);
}

#[test]
fn render_graph_projection_filters_influences_to_visible_graph() {
    let target_node_id = NodeId::new();
    let evaluation_id = ContextEvaluationId::new();
    let ada = graph_node("node.character.ada", None, "character", "Ada", false, 1);
    let beach = graph_node("node.place.beach", None, "place", "Beach", false, 2);
    let tower = graph_node("node.place.tower", None, "place", "Tower", false, 3);
    let ada_beach = graph_edge("edge.ada.beach", &ada.id, &beach.id, 0);
    let invisible = graph_edge("edge.beach.tower", &beach.id, &tower.id, 1);
    let visible_record = influence_record(
        target_node_id,
        evaluation_id,
        Some(ada.id.clone()),
        Some(ada_beach.id.clone()),
        1,
    );
    let hidden_record = influence_record(
        target_node_id,
        evaluation_id,
        Some(tower.id.clone()),
        None,
        2,
    );

    let projection = BibleRenderGraphProjection::from_graph_for_request_with_influences(
        vec![ada.clone(), beach, tower],
        vec![ada_beach, invisible],
        &BibleRenderGraphProjectionRequest {
            selected_node_id: Some(ada.id),
            neighborhood_depth: 1,
            max_nodes: 10,
            ..BibleRenderGraphProjectionRequest::default()
        },
        vec![hidden_record, visible_record],
    );

    assert_eq!(projection.influences.len(), 1);
    assert_eq!(
        projection.influences[0]
            .bible_node_id
            .as_ref()
            .unwrap()
            .as_str(),
        "node.character.ada"
    );
    assert_eq!(
        projection.influences[0]
            .bible_edge_id
            .as_ref()
            .unwrap()
            .as_str(),
        "edge.ada.beach"
    );
}

#[test]
fn render_graph_projection_keeps_influenced_nodes_during_search_filter() {
    let target_node_id = NodeId::new();
    let evaluation_id = ContextEvaluationId::new();
    let ada = graph_node("node.character.ada", None, "character", "Ada", false, 1);
    let beach = graph_node("node.place.beach", None, "place", "Beach", false, 2);
    let tower = graph_node("node.place.tower", None, "place", "Tower", false, 3);
    let ada_beach = graph_edge("edge.ada.beach", &ada.id, &beach.id, 0);
    let visible_record = influence_record(
        target_node_id,
        evaluation_id,
        Some(ada.id.clone()),
        Some(ada_beach.id.clone()),
        1,
    );

    let projection = BibleRenderGraphProjection::from_graph_for_request_with_influences(
        vec![ada, beach, tower],
        vec![ada_beach],
        &BibleRenderGraphProjectionRequest {
            selected_timeline_node_id: Some(target_node_id),
            search: Some("tower".to_string()),
            max_nodes: 10,
            ..BibleRenderGraphProjectionRequest::default()
        },
        vec![visible_record],
    );

    let node_ids: Vec<_> = projection
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();
    assert_eq!(
        node_ids,
        vec!["node.character.ada", "node.place.beach", "node.place.tower"]
    );
    assert_eq!(projection.edges.len(), 1);
    assert_eq!(projection.influences.len(), 1);
}

fn graph_node(
    id: &str,
    parent_id: Option<&str>,
    schema_key: &str,
    name: &str,
    system_owned: bool,
    sort_order: u32,
) -> BibleGraphNode {
    BibleGraphNode {
        id: BibleGraphNodeId::new(id).unwrap(),
        parent_id: parent_id.map(|id| BibleGraphNodeId::new(id).unwrap()),
        schema_key: BibleGraphSchemaKey::new(schema_key).unwrap(),
        name: name.to_string(),
        system_owned,
        sort_order,
    }
}

fn graph_edge(
    id: &str,
    from_node_id: &BibleGraphNodeId,
    to_node_id: &BibleGraphNodeId,
    sort_order: u32,
) -> BibleGraphEdge {
    BibleGraphEdge {
        id: BibleGraphEdgeId::new(id).unwrap(),
        from_node_id: from_node_id.clone(),
        to_node_id: to_node_id.clone(),
        edge_kind: BibleGraphEdgeKind::References,
        label: "references".to_string(),
        directed: true,
        sort_order,
    }
}

fn influence_record(
    target_node_id: NodeId,
    evaluation_id: ContextEvaluationId,
    bible_node_id: Option<BibleGraphNodeId>,
    bible_edge_id: Option<BibleGraphEdgeId>,
    sort_order: u32,
) -> ContextInfluenceRecord {
    ContextInfluenceRecord {
        id: ContextInfluenceId::new(),
        evaluation_id,
        timeline_node_id: target_node_id,
        source_layer: StoryLevel::Scene,
        influence_kind: ContextInfluenceKind::Direct,
        confidence: 0.8,
        reason: "Relevant scene context".to_string(),
        provenance: ContextInfluenceProvenance::AiSelected,
        bible_node_id,
        bible_edge_id,
        introduced_by_node_id: Some(target_node_id),
        sort_order,
    }
}

use super::*;
use crate::contracts::{
    BibleGraphNodeCategory, BibleGraphSchemaKey, ContextEvaluationId, ContextInfluenceId,
    ContextInfluenceKind, ContextInfluenceProvenance, ContextInfluenceRecord,
    builtin_bible_graph_schema_list_projection,
};
use crate::timeline::node::{NodeId, StoryLevel};
use std::collections::BTreeMap;

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
    assert_eq!(
        projection.nodes[0].category,
        BibleGraphNodeCategory::Character
    );
    assert_eq!(
        projection.nodes[2].category,
        BibleGraphNodeCategory::Location
    );
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
fn render_graph_projection_centers_parents_over_visible_child_subtrees() {
    let character_root = graph_node(
        "canonical.characters",
        None,
        "canonical.root.characters",
        "Characters",
        true,
        0,
    );
    let place_root = graph_node(
        "canonical.places",
        None,
        "canonical.root.places",
        "Places",
        true,
        1,
    );
    let ada = graph_node(
        "node.character.ada",
        Some("canonical.characters"),
        "character",
        "Ada",
        false,
        10,
    );
    let grace = graph_node(
        "node.character.grace",
        Some("canonical.characters"),
        "character",
        "Grace",
        false,
        20,
    );
    let harbor = graph_node(
        "node.location.harbor",
        Some("canonical.places"),
        "location",
        "Harbor",
        false,
        10,
    );
    let tower = graph_node(
        "node.location.tower",
        Some("canonical.places"),
        "location",
        "Tower",
        false,
        20,
    );

    let projection = BibleRenderGraphProjection::from_graph(
        vec![tower, grace, harbor, ada, place_root, character_root],
        Vec::new(),
    );
    let positions: BTreeMap<_, _> = projection
        .nodes
        .iter()
        .map(|node| (node.node_id.as_str(), node.position))
        .collect();

    assert_eq!(positions["node.character.ada"].y, 0.0);
    assert_eq!(positions["node.character.grace"].y, NODE_ROW_SPACING);
    assert_eq!(positions["canonical.characters"].y, NODE_ROW_SPACING / 2.0);
    assert_eq!(positions["node.location.harbor"].y, NODE_ROW_SPACING * 2.0);
    assert_eq!(positions["node.location.tower"].y, NODE_ROW_SPACING * 3.0);
    assert_eq!(positions["canonical.places"].y, NODE_ROW_SPACING * 2.5);
}

#[test]
fn bible_graph_category_visual_style_is_backend_owned() {
    assert_eq!(BibleGraphNodeCategory::Character.fill_color(), "#6495ed");
    assert_eq!(BibleGraphNodeCategory::Location.fill_color(), "#22c55e");
    assert_eq!(BibleGraphNodeCategory::Prop.fill_color(), "#f97316");

    let schema_projection = builtin_bible_graph_schema_list_projection();
    let character_category = schema_projection
        .payload
        .categories
        .iter()
        .find(|category| category.category == BibleGraphNodeCategory::Character)
        .expect("character category should be projected");
    assert_eq!(character_category.visual_style.fill_color, "#6495ed");
}

#[test]
fn render_graph_projection_round_trips() {
    let projection = BibleRenderGraphProjection {
        focused_root_id: None,
        selected_node_id: None,
        selected_timeline_node_id: None,
        active_timeline_ms: None,
        nodes: vec![BibleRenderGraphNode {
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            category: BibleGraphNodeCategory::Character,
            label: "Ada".to_string(),
            text_content: None,
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
fn render_graph_projection_request_preserves_default_graph_with_selection() {
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

    assert_eq!(
        projection.selected_node_id.as_ref().map(|id| id.as_str()),
        Some("node.character.ada")
    );
    assert_eq!(projection.focused_root_id, None);
    assert_eq!(projection.selected_timeline_node_id, None);
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
            "node.place.beach",
            "node.place.tower"
        ]
    );
    assert_eq!(projection.edges.len(), 2);
    assert_eq!(projection.edges[0].edge_id.as_str(), "edge.ada.beach");
    assert_eq!(projection.edges[1].edge_id.as_str(), "edge.beach.tower");
}

#[test]
fn render_graph_projection_selection_does_not_filter_default_graph() {
    let ada = graph_node("node.character.ada", None, "character", "Ada", false, 1);
    let beach = graph_node("node.place.beach", None, "place", "Beach", false, 2);
    let tower = graph_node("node.place.tower", None, "place", "Tower", false, 3);

    let projection = BibleRenderGraphProjection::from_graph_for_request(
        vec![tower, beach, ada.clone()],
        Vec::new(),
        &BibleRenderGraphProjectionRequest {
            selected_node_id: Some(ada.id),
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
        vec!["node.character.ada", "node.place.beach", "node.place.tower"]
    );
}

#[test]
fn render_graph_projection_preserves_active_timeline_time_request() {
    let ada = graph_node("node.character.ada", None, "character", "Ada", false, 1);
    let beach = graph_node("node.place.beach", None, "place", "Beach", false, 2);

    let projection = BibleRenderGraphProjection::from_graph_for_request(
        vec![ada, beach],
        Vec::new(),
        &BibleRenderGraphProjectionRequest {
            active_timeline_ms: Some(12_345),
            max_nodes: 10,
            ..BibleRenderGraphProjectionRequest::default()
        },
    );

    assert_eq!(projection.active_timeline_ms, Some(12_345));
    let node_ids: Vec<_> = projection
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();
    assert_eq!(node_ids, vec!["node.character.ada", "node.place.beach"]);
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
fn render_graph_projection_request_bounds_edges() {
    let ada = graph_node("node.character.ada", None, "character", "Ada", false, 1);
    let beach = graph_node("node.place.beach", None, "place", "Beach", false, 2);
    let tower = graph_node("node.place.tower", None, "place", "Tower", false, 3);

    let projection = BibleRenderGraphProjection::from_graph_for_request(
        vec![ada.clone(), beach.clone(), tower.clone()],
        vec![
            graph_edge("edge.ada.beach", &ada.id, &beach.id, 0),
            graph_edge("edge.ada.tower", &ada.id, &tower.id, 1),
        ],
        &BibleRenderGraphProjectionRequest {
            selected_node_id: Some(ada.id),
            neighborhood_depth: 1,
            max_nodes: 10,
            max_edges: 1,
            ..BibleRenderGraphProjectionRequest::default()
        },
    );

    assert_eq!(projection.edges.len(), 1);
    assert_eq!(projection.edges[0].edge_id.as_str(), "edge.ada.beach");
    assert_eq!(projection.neighborhoods.len(), 2);
}

#[test]
fn render_graph_projection_request_filters_edges_by_kind() {
    let ada = graph_node("node.character.ada", None, "character", "Ada", false, 1);
    let beach = graph_node("node.place.beach", None, "place", "Beach", false, 2);
    let tower = graph_node("node.place.tower", None, "place", "Tower", false, 3);

    let projection = BibleRenderGraphProjection::from_graph_for_request(
        vec![ada.clone(), beach.clone(), tower.clone()],
        vec![
            graph_edge_with_kind(
                "edge.ada.beach",
                &ada.id,
                &beach.id,
                BibleGraphEdgeKind::LocatedIn,
                0,
            ),
            graph_edge_with_kind(
                "edge.ada.tower",
                &ada.id,
                &tower.id,
                BibleGraphEdgeKind::References,
                1,
            ),
        ],
        &BibleRenderGraphProjectionRequest {
            selected_node_id: Some(ada.id),
            neighborhood_depth: 1,
            edge_kinds: vec![BibleGraphEdgeKind::LocatedIn],
            max_nodes: 10,
            max_edges: 10,
            ..BibleRenderGraphProjectionRequest::default()
        },
    );

    assert_eq!(projection.edges.len(), 1);
    assert_eq!(projection.edges[0].edge_id.as_str(), "edge.ada.beach");
    assert_eq!(projection.edges[0].edge_kind, BibleGraphEdgeKind::LocatedIn);
}

#[test]
fn render_graph_projection_keeps_empty_request_filters_empty() {
    let nodes = vec![
        graph_node("node.place.alpha", None, "place", "Alpha", false, 1),
        graph_node("node.place.beta", None, "place", "Beta", false, 2),
    ];

    let projection = BibleRenderGraphProjection::from_graph_for_request(
        nodes,
        Vec::new(),
        &BibleRenderGraphProjectionRequest {
            search: Some("tower".to_string()),
            max_nodes: 10,
            ..BibleRenderGraphProjectionRequest::default()
        },
    );

    assert!(projection.nodes.is_empty());
    assert!(projection.edges.is_empty());
    assert!(projection.neighborhoods.is_empty());
    assert!(projection.influences.is_empty());
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
            max_nodes: 2,
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
    graph_edge_with_kind(
        id,
        from_node_id,
        to_node_id,
        BibleGraphEdgeKind::References,
        sort_order,
    )
}

fn graph_edge_with_kind(
    id: &str,
    from_node_id: &BibleGraphNodeId,
    to_node_id: &BibleGraphNodeId,
    edge_kind: BibleGraphEdgeKind,
    sort_order: u32,
) -> BibleGraphEdge {
    BibleGraphEdge {
        id: BibleGraphEdgeId::new(id).unwrap(),
        from_node_id: from_node_id.clone(),
        to_node_id: to_node_id.clone(),
        label: format!("{edge_kind:?}"),
        edge_kind,
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

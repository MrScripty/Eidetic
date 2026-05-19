use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{
    BibleGraphEdge, BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphNode, BibleGraphNodeId,
    BibleGraphSchemaKey,
};

const NODE_COLUMN_SPACING: f32 = 320.0;
const NODE_ROW_SPACING: f32 = 150.0;
const SYSTEM_NODE_Z: f32 = -80.0;
const USER_NODE_Z: f32 = 0.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleRenderGraphProjection {
    #[serde(default)]
    pub nodes: Vec<BibleRenderGraphNode>,
    #[serde(default)]
    pub edges: Vec<BibleRenderGraphEdge>,
    #[serde(default)]
    pub neighborhoods: Vec<BibleRenderGraphNeighborhood>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleRenderGraphNode {
    pub node_id: BibleGraphNodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<BibleGraphNodeId>,
    pub schema_key: BibleGraphSchemaKey,
    pub label: String,
    #[serde(default)]
    pub system_owned: bool,
    #[serde(default)]
    pub sort_order: u32,
    #[serde(default)]
    pub depth: u32,
    pub position: BibleRenderGraphPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BibleRenderGraphPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleRenderGraphEdge {
    pub edge_id: BibleGraphEdgeId,
    pub from_node_id: BibleGraphNodeId,
    pub to_node_id: BibleGraphNodeId,
    pub edge_kind: BibleGraphEdgeKind,
    pub label: String,
    #[serde(default = "default_directed")]
    pub directed: bool,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleRenderGraphNeighborhood {
    pub node_id: BibleGraphNodeId,
    #[serde(default)]
    pub connected_node_ids: Vec<BibleGraphNodeId>,
    #[serde(default)]
    pub edge_ids: Vec<BibleGraphEdgeId>,
}

impl BibleRenderGraphProjection {
    pub fn from_graph(nodes: Vec<BibleGraphNode>, edges: Vec<BibleGraphEdge>) -> Self {
        let sorted_nodes = sorted_nodes(nodes);
        let depths = node_depths(&sorted_nodes);
        let row_indexes = row_indexes_by_depth(&sorted_nodes, &depths);
        let render_nodes = sorted_nodes
            .into_iter()
            .map(|node| {
                let depth = depths.get(&node.id).copied().unwrap_or_default();
                let row_index = row_indexes.get(&node.id).copied().unwrap_or_default();
                BibleRenderGraphNode {
                    node_id: node.id,
                    parent_id: node.parent_id,
                    schema_key: node.schema_key,
                    label: node.name,
                    system_owned: node.system_owned,
                    sort_order: node.sort_order,
                    depth,
                    position: BibleRenderGraphPosition {
                        x: depth as f32 * NODE_COLUMN_SPACING,
                        y: row_index as f32 * NODE_ROW_SPACING,
                        z: if node.system_owned {
                            SYSTEM_NODE_Z
                        } else {
                            USER_NODE_Z
                        },
                    },
                }
            })
            .collect();

        let render_edges = sorted_edges(edges);
        let neighborhoods = neighborhoods_for_edges(&render_edges);

        Self {
            nodes: render_nodes,
            edges: render_edges,
            neighborhoods,
        }
    }
}

fn sorted_nodes(mut nodes: Vec<BibleGraphNode>) -> Vec<BibleGraphNode> {
    nodes.sort_by(|a, b| {
        a.sort_order
            .cmp(&b.sort_order)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.id.as_str().cmp(b.id.as_str()))
    });
    nodes
}

fn sorted_edges(edges: Vec<BibleGraphEdge>) -> Vec<BibleRenderGraphEdge> {
    let mut edges: Vec<_> = edges
        .into_iter()
        .map(|edge| BibleRenderGraphEdge {
            edge_id: edge.id,
            from_node_id: edge.from_node_id,
            to_node_id: edge.to_node_id,
            edge_kind: edge.edge_kind,
            label: edge.label,
            directed: edge.directed,
            sort_order: edge.sort_order,
        })
        .collect();
    edges.sort_by(|a, b| {
        a.sort_order
            .cmp(&b.sort_order)
            .then_with(|| a.label.cmp(&b.label))
            .then_with(|| a.edge_id.as_str().cmp(b.edge_id.as_str()))
    });
    edges
}

fn node_depths(nodes: &[BibleGraphNode]) -> BTreeMap<BibleGraphNodeId, u32> {
    let parent_by_id: BTreeMap<_, _> = nodes
        .iter()
        .map(|node| (node.id.clone(), node.parent_id.clone()))
        .collect();
    nodes
        .iter()
        .map(|node| {
            (
                node.id.clone(),
                depth_for_node(&node.id, &parent_by_id, nodes.len()),
            )
        })
        .collect()
}

fn depth_for_node(
    node_id: &BibleGraphNodeId,
    parent_by_id: &BTreeMap<BibleGraphNodeId, Option<BibleGraphNodeId>>,
    limit: usize,
) -> u32 {
    let mut depth = 0_u32;
    let mut current = node_id;
    for _ in 0..limit {
        let Some(Some(parent_id)) = parent_by_id.get(current) else {
            return depth;
        };
        depth = depth.saturating_add(1);
        current = parent_id;
    }
    depth
}

fn row_indexes_by_depth(
    nodes: &[BibleGraphNode],
    depths: &BTreeMap<BibleGraphNodeId, u32>,
) -> BTreeMap<BibleGraphNodeId, u32> {
    let mut row_counts = BTreeMap::<u32, u32>::new();
    let mut rows = BTreeMap::new();
    for node in nodes {
        let depth = depths.get(&node.id).copied().unwrap_or_default();
        let row = row_counts.entry(depth).or_default();
        rows.insert(node.id.clone(), *row);
        *row = row.saturating_add(1);
    }
    rows
}

fn neighborhoods_for_edges(edges: &[BibleRenderGraphEdge]) -> Vec<BibleRenderGraphNeighborhood> {
    let mut by_node = BTreeMap::<BibleGraphNodeId, NeighborhoodBuilder>::new();
    for edge in edges {
        by_node
            .entry(edge.from_node_id.clone())
            .or_default()
            .connect(edge.to_node_id.clone(), edge.edge_id.clone());
        by_node
            .entry(edge.to_node_id.clone())
            .or_default()
            .connect(edge.from_node_id.clone(), edge.edge_id.clone());
    }
    by_node
        .into_iter()
        .map(|(node_id, builder)| BibleRenderGraphNeighborhood {
            node_id,
            connected_node_ids: builder.connected_node_ids.into_iter().collect(),
            edge_ids: builder.edge_ids.into_iter().collect(),
        })
        .collect()
}

#[derive(Default)]
struct NeighborhoodBuilder {
    connected_node_ids: BTreeSet<BibleGraphNodeId>,
    edge_ids: BTreeSet<BibleGraphEdgeId>,
}

impl NeighborhoodBuilder {
    fn connect(&mut self, node_id: BibleGraphNodeId, edge_id: BibleGraphEdgeId) {
        self.connected_node_ids.insert(node_id);
        self.edge_ids.insert(edge_id);
    }
}

fn default_directed() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::BibleGraphSchemaKey;

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
        };

        let json = serde_json::to_string(&projection).unwrap();
        let round_trip: BibleRenderGraphProjection = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, projection);
    }
}

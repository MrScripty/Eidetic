use std::collections::{BTreeMap, BTreeSet};

use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphNodeId, BibleRenderGraphPosition, BibleRenderGraphProjection,
};
use serde::Serialize;

use crate::category::node_fill_color;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BibleGraphVisualSnapshot {
    pub nodes: Vec<BibleGraphVisualNode>,
    pub edges: Vec<BibleGraphVisualEdge>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BibleGraphVisualNode {
    pub node_id: BibleGraphNodeId,
    pub label: String,
    pub position: BibleRenderGraphPosition,
    pub radius: f32,
    pub fill_color: &'static str,
    pub outline_color: &'static str,
    pub highlighted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BibleGraphVisualEdge {
    pub edge_id: BibleGraphEdgeId,
    pub from_node_id: BibleGraphNodeId,
    pub to_node_id: BibleGraphNodeId,
    pub from_position: BibleRenderGraphPosition,
    pub to_position: BibleRenderGraphPosition,
    pub width: f32,
    pub stroke_color: &'static str,
    pub highlighted: bool,
}

pub fn build_bible_graph_visual_snapshot(
    projection: &BibleRenderGraphProjection,
) -> BibleGraphVisualSnapshot {
    let highlighted_nodes: BTreeSet<BibleGraphNodeId> = projection
        .influences
        .iter()
        .filter_map(|influence| influence.bible_node_id.clone())
        .collect();
    let highlighted_edges: BTreeSet<BibleGraphEdgeId> = projection
        .influences
        .iter()
        .filter_map(|influence| influence.bible_edge_id.clone())
        .collect();
    let node_positions: BTreeMap<BibleGraphNodeId, BibleRenderGraphPosition> = projection
        .nodes
        .iter()
        .map(|node| (node.node_id.clone(), node.position.clone()))
        .collect();

    let nodes = projection
        .nodes
        .iter()
        .map(|node| {
            let highlighted = highlighted_nodes.contains(&node.node_id);
            BibleGraphVisualNode {
                node_id: node.node_id.clone(),
                label: node.label.clone(),
                position: node.position.clone(),
                radius: node_radius(node.depth, highlighted),
                fill_color: node_fill_color(node.schema_key.as_str()),
                outline_color: if highlighted { "#f2c94c" } else { "#40576f" },
                highlighted,
            }
        })
        .collect();

    let edges = projection
        .edges
        .iter()
        .filter_map(|edge| {
            let from_position = node_positions.get(&edge.from_node_id)?.clone();
            let to_position = node_positions.get(&edge.to_node_id)?.clone();
            let highlighted = highlighted_edges.contains(&edge.edge_id);
            Some(BibleGraphVisualEdge {
                edge_id: edge.edge_id.clone(),
                from_node_id: edge.from_node_id.clone(),
                to_node_id: edge.to_node_id.clone(),
                from_position,
                to_position,
                width: if highlighted { 3.0 } else { 1.5 },
                stroke_color: if highlighted { "#f2c94c" } else { "#52687f" },
                highlighted,
            })
        })
        .collect();

    BibleGraphVisualSnapshot { nodes, edges }
}

fn node_radius(depth: u32, highlighted: bool) -> f32 {
    let base = 16.0_f32 - (depth.min(4) as f32 * 1.5);
    if highlighted { base + 3.0 } else { base }
}

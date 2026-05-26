use std::collections::{BTreeMap, BTreeSet};

use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphNodeId, BibleRenderGraphPosition, BibleRenderGraphProjection,
};
use serde::Serialize;

use crate::category::node_fill_color;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BibleGraphVisual3dSnapshot {
    pub nodes: Vec<BibleGraphVisual3dNode>,
    pub edges: Vec<BibleGraphVisual3dEdge>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BibleGraphVisual3dNode {
    pub node_id: BibleGraphNodeId,
    pub label: String,
    pub position: BibleRenderGraphPosition,
    pub radius: f32,
    pub fill_color: &'static str,
    pub outline_color: &'static str,
    pub selected: bool,
    pub highlighted: bool,
    pub dimmed: bool,
    pub label_visible: bool,
    pub label_font_size: f32,
    pub label_color: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BibleGraphVisual3dEdge {
    pub edge_id: BibleGraphEdgeId,
    pub edge_class: BibleGraphVisual3dEdgeClass,
    pub from_node_id: BibleGraphNodeId,
    pub to_node_id: BibleGraphNodeId,
    pub from_position: BibleRenderGraphPosition,
    pub to_position: BibleRenderGraphPosition,
    pub radius: f32,
    pub stroke_color: &'static str,
    pub selected: bool,
    pub highlighted: bool,
    pub dimmed: bool,
    pub directed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BibleGraphVisual3dEdgeClass {
    Structural,
    Semantic,
}

pub fn build_bible_graph_visual_3d_snapshot(
    projection: &BibleRenderGraphProjection,
) -> BibleGraphVisual3dSnapshot {
    let node_positions: BTreeMap<BibleGraphNodeId, BibleRenderGraphPosition> = projection
        .nodes
        .iter()
        .map(|node| (node.node_id.clone(), node.position))
        .collect();
    let selected_node_id = projection.selected_node_id.as_ref();
    let highlighted_nodes = highlighted_node_ids(projection);
    let highlighted_edges = highlighted_edge_ids(projection);
    let selected_neighborhood = selected_node_id.and_then(|node_id| {
        projection
            .neighborhoods
            .iter()
            .find(|neighborhood| &neighborhood.node_id == node_id)
    });
    let selected_adjacent_nodes: BTreeSet<BibleGraphNodeId> = selected_neighborhood
        .map(|neighborhood| neighborhood.connected_node_ids.iter().cloned().collect())
        .unwrap_or_default();
    let selected_incident_edges: BTreeSet<BibleGraphEdgeId> = selected_neighborhood
        .map(|neighborhood| neighborhood.edge_ids.iter().cloned().collect())
        .unwrap_or_default();

    let semantic_edges = projection.edges.iter().filter_map(|edge| {
        let from_position = node_positions.get(&edge.from_node_id)?;
        let to_position = node_positions.get(&edge.to_node_id)?;
        let selected = selected_incident_edges.contains(&edge.edge_id);
        let highlighted = selected || highlighted_edges.contains(&edge.edge_id);
        Some(BibleGraphVisual3dEdge {
            edge_id: edge.edge_id.clone(),
            edge_class: BibleGraphVisual3dEdgeClass::Semantic,
            from_node_id: edge.from_node_id.clone(),
            to_node_id: edge.to_node_id.clone(),
            from_position: *from_position,
            to_position: *to_position,
            radius: if highlighted { 2.8 } else { 1.6 },
            stroke_color: semantic_edge_color(highlighted),
            selected,
            highlighted,
            dimmed: selected_node_id.is_some() && !highlighted,
            directed: edge.directed,
        })
    });

    let structural_edges = projection.nodes.iter().filter_map(|node| {
        let parent_id = node.parent_id.as_ref()?;
        let from_position = node_positions.get(parent_id)?;
        let to_position = node_positions.get(&node.node_id)?;
        let selected = selected_node_id.is_some_and(|selected_node_id| {
            selected_node_id == parent_id || selected_node_id == &node.node_id
        });
        Some(BibleGraphVisual3dEdge {
            edge_id: structural_edge_id(parent_id, &node.node_id),
            edge_class: BibleGraphVisual3dEdgeClass::Structural,
            from_node_id: parent_id.clone(),
            to_node_id: node.node_id.clone(),
            from_position: *from_position,
            to_position: *to_position,
            radius: if selected { 1.6 } else { 0.9 },
            stroke_color: structural_edge_color(selected),
            selected,
            highlighted: selected,
            dimmed: selected_node_id.is_some() && !selected,
            directed: true,
        })
    });

    let edges = semantic_edges.chain(structural_edges).collect();
    let nodes = projection
        .nodes
        .iter()
        .map(|node| {
            let selected = selected_node_id == Some(&node.node_id);
            let highlighted = selected
                || highlighted_nodes.contains(&node.node_id)
                || selected_adjacent_nodes.contains(&node.node_id);
            BibleGraphVisual3dNode {
                node_id: node.node_id.clone(),
                label: node.label.clone(),
                position: node.position,
                radius: node_radius(node.depth, node.system_owned, highlighted),
                fill_color: node_fill_color(node.schema_key.as_str()),
                outline_color: if selected {
                    "#f2c94c"
                } else if highlighted {
                    "#6fc2c9"
                } else {
                    "#40576f"
                },
                selected,
                highlighted,
                dimmed: selected_node_id.is_some() && !highlighted,
                label_visible: true,
                label_font_size: node_label_font_size(node.system_owned, selected, highlighted),
                label_color: node_label_color(selected, highlighted),
            }
        })
        .collect();

    BibleGraphVisual3dSnapshot { nodes, edges }
}

fn highlighted_node_ids(projection: &BibleRenderGraphProjection) -> BTreeSet<BibleGraphNodeId> {
    projection
        .influences
        .iter()
        .filter_map(|influence| influence.bible_node_id.clone())
        .collect()
}

fn highlighted_edge_ids(projection: &BibleRenderGraphProjection) -> BTreeSet<BibleGraphEdgeId> {
    projection
        .influences
        .iter()
        .filter_map(|influence| influence.bible_edge_id.clone())
        .collect()
}

fn structural_edge_id(
    parent_id: &BibleGraphNodeId,
    child_id: &BibleGraphNodeId,
) -> BibleGraphEdgeId {
    BibleGraphEdgeId::new(format!(
        "structural.parent.{}->{}",
        parent_id.as_str(),
        child_id.as_str()
    ))
    .expect("structural edge identifiers are non-empty")
}

fn node_radius(depth: u32, system_owned: bool, highlighted: bool) -> f32 {
    let base = if system_owned { 22.0 } else { 16.0 } - (depth.min(4) as f32 * 1.25);
    if highlighted { base + 3.0 } else { base }
}

fn node_label_font_size(system_owned: bool, selected: bool, highlighted: bool) -> f32 {
    if selected {
        16.0
    } else if highlighted || system_owned {
        14.0
    } else {
        12.0
    }
}

fn node_label_color(selected: bool, highlighted: bool) -> &'static str {
    if selected {
        "#f6d977"
    } else if highlighted {
        "#c9f3f5"
    } else {
        "#dbe3ea"
    }
}

fn semantic_edge_color(highlighted: bool) -> &'static str {
    if highlighted { "#f2c94c" } else { "#6e879c" }
}

fn structural_edge_color(highlighted: bool) -> &'static str {
    if highlighted { "#6fc2c9" } else { "#34495e" }
}

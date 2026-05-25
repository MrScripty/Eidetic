use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::timeline::node::{NodeId, StoryLevel};

use super::{
    BibleGraphEdge, BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphNode, BibleGraphNodeId,
    BibleGraphSchemaKey, ContextInfluenceId, ContextInfluenceKind, ContextInfluenceProvenance,
    ContextInfluenceRecord,
    bible_render_graph_filter::{BibleRenderGraphProjectionRequest, included_node_ids_for_request},
};

const NODE_COLUMN_SPACING: f32 = 320.0;
const NODE_ROW_SPACING: f32 = 150.0;
const SYSTEM_NODE_Z: f32 = -80.0;
const USER_NODE_Z: f32 = 0.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleRenderGraphProjection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused_root_id: Option<BibleGraphNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_node_id: Option<BibleGraphNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_timeline_node_id: Option<NodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_timeline_ms: Option<u64>,
    #[serde(default)]
    pub nodes: Vec<BibleRenderGraphNode>,
    #[serde(default)]
    pub edges: Vec<BibleRenderGraphEdge>,
    #[serde(default)]
    pub neighborhoods: Vec<BibleRenderGraphNeighborhood>,
    #[serde(default)]
    pub influences: Vec<BibleRenderGraphInfluence>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleRenderGraphInfluence {
    pub influence_id: ContextInfluenceId,
    pub timeline_node_id: NodeId,
    pub source_layer: StoryLevel,
    pub influence_kind: ContextInfluenceKind,
    pub confidence: f32,
    pub reason: String,
    pub provenance: ContextInfluenceProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bible_node_id: Option<BibleGraphNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bible_edge_id: Option<BibleGraphEdgeId>,
    #[serde(default)]
    pub sort_order: u32,
}

impl BibleRenderGraphProjection {
    pub fn from_graph(nodes: Vec<BibleGraphNode>, edges: Vec<BibleGraphEdge>) -> Self {
        Self::from_graph_for_request(nodes, edges, &BibleRenderGraphProjectionRequest::default())
    }

    pub fn from_graph_for_request(
        nodes: Vec<BibleGraphNode>,
        edges: Vec<BibleGraphEdge>,
        request: &BibleRenderGraphProjectionRequest,
    ) -> Self {
        Self::from_graph_for_request_with_influences(nodes, edges, request, Vec::new())
    }

    pub fn from_graph_for_request_with_influences(
        nodes: Vec<BibleGraphNode>,
        edges: Vec<BibleGraphEdge>,
        request: &BibleRenderGraphProjectionRequest,
        influences: Vec<ContextInfluenceRecord>,
    ) -> Self {
        let request = request.normalized();
        let sorted_nodes = sorted_nodes(nodes);
        let sorted_edges = sorted_graph_edges(edges);
        let mut included_node_ids =
            included_node_ids_for_request(&sorted_nodes, &sorted_edges, &request);
        if request.selected_timeline_node_id.is_some() {
            include_influenced_node_ids(&mut included_node_ids, &sorted_edges, &influences);
        }
        let required_edge_ids = required_edge_ids(&influences);
        let filtered_nodes: Vec<_> = sorted_nodes
            .into_iter()
            .filter(|node| included_node_ids.contains(&node.id))
            .collect();
        let filtered_edges = limit_edges(
            sorted_edges
                .into_iter()
                .filter(|edge| {
                    included_node_ids.contains(&edge.from_node_id)
                        && included_node_ids.contains(&edge.to_node_id)
                        && request_matches_edge_kind(&request, edge)
                })
                .collect(),
            &required_edge_ids,
            request.max_edges as usize,
        );

        let depths = node_depths(&filtered_nodes);
        let row_indexes = row_indexes_by_depth(&filtered_nodes, &depths);
        let render_nodes: Vec<_> = filtered_nodes
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

        let render_edges = render_edges(filtered_edges);
        let neighborhoods = neighborhoods_for_edges(&render_edges);
        let influences = render_influences(influences, &render_nodes, &render_edges);

        Self {
            focused_root_id: request.focused_root_id.clone(),
            selected_node_id: request.selected_node_id.clone(),
            selected_timeline_node_id: request.selected_timeline_node_id,
            active_timeline_ms: request.active_timeline_ms,
            nodes: render_nodes,
            edges: render_edges,
            neighborhoods,
            influences,
        }
    }
}

fn request_matches_edge_kind(
    request: &BibleRenderGraphProjectionRequest,
    edge: &BibleGraphEdge,
) -> bool {
    request.edge_kinds.is_empty()
        || request
            .edge_kinds
            .iter()
            .any(|kind| kind == &edge.edge_kind)
}

fn required_edge_ids(influences: &[ContextInfluenceRecord]) -> BTreeSet<BibleGraphEdgeId> {
    influences
        .iter()
        .filter_map(|record| record.bible_edge_id.as_ref().cloned())
        .collect()
}

fn include_influenced_node_ids(
    included_node_ids: &mut BTreeSet<BibleGraphNodeId>,
    edges: &[BibleGraphEdge],
    influences: &[ContextInfluenceRecord],
) {
    let edges_by_id: BTreeMap<_, _> = edges.iter().map(|edge| (edge.id.clone(), edge)).collect();
    for influence in influences {
        if let Some(node_id) = &influence.bible_node_id {
            included_node_ids.insert(node_id.clone());
        }
        if let Some(edge_id) = &influence.bible_edge_id
            && let Some(edge) = edges_by_id.get(edge_id)
        {
            included_node_ids.insert(edge.from_node_id.clone());
            included_node_ids.insert(edge.to_node_id.clone());
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

fn sorted_graph_edges(mut edges: Vec<BibleGraphEdge>) -> Vec<BibleGraphEdge> {
    edges.sort_by(|a, b| {
        a.sort_order
            .cmp(&b.sort_order)
            .then_with(|| a.label.cmp(&b.label))
            .then_with(|| a.id.as_str().cmp(b.id.as_str()))
    });
    edges
}

fn render_edges(edges: Vec<BibleGraphEdge>) -> Vec<BibleRenderGraphEdge> {
    edges
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
        .collect()
}

fn limit_edges(
    edges: Vec<BibleGraphEdge>,
    required_edge_ids: &BTreeSet<BibleGraphEdgeId>,
    max_edges: usize,
) -> Vec<BibleGraphEdge> {
    let mut included_edge_ids = BTreeSet::new();
    let mut limited = Vec::new();

    for edge in edges
        .iter()
        .filter(|edge| required_edge_ids.contains(&edge.id))
    {
        if limited.len() >= max_edges {
            return limited;
        }
        included_edge_ids.insert(edge.id.clone());
        limited.push(edge.clone());
    }

    for edge in edges {
        if limited.len() >= max_edges {
            break;
        }
        if included_edge_ids.insert(edge.id.clone()) {
            limited.push(edge);
        }
    }

    limited
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

fn render_influences(
    mut influences: Vec<ContextInfluenceRecord>,
    nodes: &[BibleRenderGraphNode],
    edges: &[BibleRenderGraphEdge],
) -> Vec<BibleRenderGraphInfluence> {
    let node_ids: BTreeSet<_> = nodes.iter().map(|node| node.node_id.clone()).collect();
    let edge_ids: BTreeSet<_> = edges.iter().map(|edge| edge.edge_id.clone()).collect();
    influences.sort_by(|a, b| {
        a.sort_order
            .cmp(&b.sort_order)
            .then_with(|| a.id.0.cmp(&b.id.0))
    });
    influences
        .into_iter()
        .filter(|record| {
            record
                .bible_node_id
                .as_ref()
                .is_none_or(|node_id| node_ids.contains(node_id))
                && record
                    .bible_edge_id
                    .as_ref()
                    .is_none_or(|edge_id| edge_ids.contains(edge_id))
        })
        .map(|record| BibleRenderGraphInfluence {
            influence_id: record.id,
            timeline_node_id: record.timeline_node_id,
            source_layer: record.source_layer,
            influence_kind: record.influence_kind,
            confidence: record.confidence,
            reason: record.reason,
            provenance: record.provenance,
            bible_node_id: record.bible_node_id,
            bible_edge_id: record.bible_edge_id,
            sort_order: record.sort_order,
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
#[path = "bible_render_graph_tests.rs"]
mod tests;

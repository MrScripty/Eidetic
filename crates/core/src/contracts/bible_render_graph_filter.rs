use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::timeline::node::NodeId;

use super::{BibleGraphEdge, BibleGraphEdgeKind, BibleGraphNode, BibleGraphNodeId};

const DEFAULT_MAX_RENDER_GRAPH_NODES: u32 = 200;
const MAX_RENDER_GRAPH_NODES: u32 = 500;
const DEFAULT_MAX_RENDER_GRAPH_EDGES: u32 = 500;
const MAX_RENDER_GRAPH_EDGES: u32 = 1_000;
const DEFAULT_NEIGHBORHOOD_DEPTH: u32 = 1;
const MAX_NEIGHBORHOOD_DEPTH: u32 = 6;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BibleRenderGraphProjectionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused_root_id: Option<BibleGraphNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_node_id: Option<BibleGraphNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_timeline_node_id: Option<NodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_timeline_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edge_kinds: Vec<BibleGraphEdgeKind>,
    #[serde(default = "default_neighborhood_depth")]
    pub neighborhood_depth: u32,
    #[serde(default = "default_max_render_graph_nodes")]
    pub max_nodes: u32,
    #[serde(default = "default_max_render_graph_edges")]
    pub max_edges: u32,
}

impl Default for BibleRenderGraphProjectionRequest {
    fn default() -> Self {
        Self {
            focused_root_id: None,
            selected_node_id: None,
            selected_timeline_node_id: None,
            active_timeline_ms: None,
            search: None,
            edge_kinds: Vec::new(),
            neighborhood_depth: DEFAULT_NEIGHBORHOOD_DEPTH,
            max_nodes: DEFAULT_MAX_RENDER_GRAPH_NODES,
            max_edges: DEFAULT_MAX_RENDER_GRAPH_EDGES,
        }
    }
}

impl BibleRenderGraphProjectionRequest {
    pub fn normalized(&self) -> Self {
        Self {
            focused_root_id: self.focused_root_id.clone(),
            selected_node_id: self.selected_node_id.clone(),
            selected_timeline_node_id: self.selected_timeline_node_id,
            active_timeline_ms: self.active_timeline_ms,
            search: normalized_search(self.search.as_deref()),
            edge_kinds: self.edge_kinds.clone(),
            neighborhood_depth: self.neighborhood_depth.min(MAX_NEIGHBORHOOD_DEPTH),
            max_nodes: self.max_nodes.clamp(1, MAX_RENDER_GRAPH_NODES),
            max_edges: self.max_edges.clamp(1, MAX_RENDER_GRAPH_EDGES),
        }
    }
}

pub(super) fn included_node_ids_for_request(
    nodes: &[BibleGraphNode],
    _edges: &[BibleGraphEdge],
    request: &BibleRenderGraphProjectionRequest,
) -> BTreeSet<BibleGraphNodeId> {
    let request = request.normalized();
    let node_ids: BTreeSet<_> = nodes.iter().map(|node| node.id.clone()).collect();
    let parent_by_id: BTreeMap<_, _> = nodes
        .iter()
        .map(|node| (node.id.clone(), node.parent_id.clone()))
        .collect();
    let children_by_parent = children_by_parent(nodes);
    let mut required = BTreeSet::new();
    let mut candidates = BTreeSet::new();

    if let Some(root_id) = &request.focused_root_id
        && node_ids.contains(root_id)
    {
        required.insert(root_id.clone());
        candidates.extend(descendant_ids(
            root_id,
            &children_by_parent,
            request.neighborhood_depth,
        ));
    }

    if let Some(selected_node_id) = &request.selected_node_id
        && node_ids.contains(selected_node_id)
    {
        required.insert(selected_node_id.clone());
    }

    if let Some(search) = &request.search {
        candidates.extend(nodes.iter().filter_map(|node| {
            let node_id = node.id.as_str().to_ascii_lowercase();
            let schema_key = node.schema_key.as_str().to_ascii_lowercase();
            let name = node.name.to_ascii_lowercase();
            (node_id.contains(search) || schema_key.contains(search) || name.contains(search))
                .then(|| node.id.clone())
        }));
    }

    if candidates.is_empty() && should_load_default_node_ids(&request) {
        candidates.extend(nodes.iter().map(|node| node.id.clone()));
    }

    let mut included = BTreeSet::new();
    for node_id in required.iter().chain(candidates.iter()) {
        if node_ids.contains(node_id) {
            included.insert(node_id.clone());
            included.extend(ancestor_ids(node_id, &parent_by_id));
        }
    }

    limit_node_ids(nodes, included, &required, request.max_nodes as usize)
}

fn should_load_default_node_ids(request: &BibleRenderGraphProjectionRequest) -> bool {
    request.focused_root_id.is_none()
        && request.selected_timeline_node_id.is_none()
        && request.active_timeline_ms.is_none()
        && request.search.is_none()
}

fn children_by_parent(
    nodes: &[BibleGraphNode],
) -> BTreeMap<BibleGraphNodeId, BTreeSet<BibleGraphNodeId>> {
    let mut children = BTreeMap::<BibleGraphNodeId, BTreeSet<BibleGraphNodeId>>::new();
    for node in nodes {
        if let Some(parent_id) = &node.parent_id {
            children
                .entry(parent_id.clone())
                .or_default()
                .insert(node.id.clone());
        }
    }
    children
}

fn descendant_ids(
    root_id: &BibleGraphNodeId,
    children_by_parent: &BTreeMap<BibleGraphNodeId, BTreeSet<BibleGraphNodeId>>,
    depth_limit: u32,
) -> BTreeSet<BibleGraphNodeId> {
    let mut visited = BTreeSet::new();
    let mut frontier = BTreeSet::from([root_id.clone()]);
    for _ in 0..=depth_limit {
        let mut next = BTreeSet::new();
        for node_id in frontier {
            if !visited.insert(node_id.clone()) {
                continue;
            }
            if let Some(children) = children_by_parent.get(&node_id) {
                next.extend(children.iter().cloned());
            }
        }
        frontier = next;
    }
    visited
}

fn ancestor_ids(
    node_id: &BibleGraphNodeId,
    parent_by_id: &BTreeMap<BibleGraphNodeId, Option<BibleGraphNodeId>>,
) -> BTreeSet<BibleGraphNodeId> {
    let mut ancestors = BTreeSet::new();
    let mut current = node_id;
    for _ in 0..parent_by_id.len() {
        let Some(Some(parent_id)) = parent_by_id.get(current) else {
            break;
        };
        if !ancestors.insert(parent_id.clone()) {
            break;
        }
        current = parent_id;
    }
    ancestors
}

fn limit_node_ids(
    nodes: &[BibleGraphNode],
    included: BTreeSet<BibleGraphNodeId>,
    required: &BTreeSet<BibleGraphNodeId>,
    max_nodes: usize,
) -> BTreeSet<BibleGraphNodeId> {
    let mut limited = BTreeSet::new();
    for node in nodes.iter().filter(|node| required.contains(&node.id)) {
        if limited.len() >= max_nodes {
            return limited;
        }
        limited.insert(node.id.clone());
    }
    for node in nodes.iter().filter(|node| included.contains(&node.id)) {
        if limited.len() >= max_nodes {
            break;
        }
        limited.insert(node.id.clone());
    }
    limited
}

fn default_max_render_graph_nodes() -> u32 {
    DEFAULT_MAX_RENDER_GRAPH_NODES
}

fn default_max_render_graph_edges() -> u32 {
    DEFAULT_MAX_RENDER_GRAPH_EDGES
}

fn default_neighborhood_depth() -> u32 {
    DEFAULT_NEIGHBORHOOD_DEPTH
}

fn normalized_search(search: Option<&str>) -> Option<String> {
    search
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
}

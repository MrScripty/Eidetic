use std::collections::{BTreeSet, HashSet};

use eidetic_core::contracts::{
    BibleGraphEdge, BibleGraphEdgeId, BibleGraphNode, BibleGraphNodeId,
    BibleRenderGraphProjectionRequest, ContextInfluenceRecord,
};
use eidetic_core::timeline::node::NodeId;
use rusqlite::{Connection, params, params_from_iter};

use crate::bible_graph_edge_store;
use crate::history_store::HistoryStoreError;

const MAX_PARENT_DEPTH: u32 = 64;

pub(crate) struct BoundedBibleRenderGraph {
    pub(crate) nodes: Vec<BibleGraphNode>,
    pub(crate) edges: Vec<BibleGraphEdge>,
    pub(crate) influences: Vec<ContextInfluenceRecord>,
}

struct RenderGraphQueryInput {
    request: BibleRenderGraphProjectionRequest,
    max_nodes: i64,
}

struct RenderGraphScope {
    required_node_ids: BTreeSet<BibleGraphNodeId>,
    candidate_node_ids: BTreeSet<BibleGraphNodeId>,
    influences: Vec<ContextInfluenceRecord>,
}

pub(crate) fn load_bounded_render_graph(
    conn: &Connection,
    request: &BibleRenderGraphProjectionRequest,
) -> Result<BoundedBibleRenderGraph, HistoryStoreError> {
    let input = RenderGraphQueryInput::new(request);
    let scope = load_render_graph_scope(conn, &input)?;
    let nodes = load_scoped_render_graph_nodes(conn, &scope, input.request.max_nodes as usize)?;
    let node_ids: Vec<_> = nodes.iter().map(|node| node.id.clone()).collect();
    let edges = bible_graph_edge_store::load_edges_between_nodes_for_kinds(
        conn,
        &node_ids,
        &input.request.edge_kinds,
        input.request.max_edges,
    )?;

    Ok(BoundedBibleRenderGraph {
        nodes,
        edges,
        influences: scope.influences,
    })
}

impl RenderGraphQueryInput {
    fn new(request: &BibleRenderGraphProjectionRequest) -> Self {
        let request = request.normalized();
        Self {
            max_nodes: i64::from(request.max_nodes),
            request,
        }
    }
}

fn load_render_graph_scope(
    conn: &Connection,
    input: &RenderGraphQueryInput,
) -> Result<RenderGraphScope, HistoryStoreError> {
    let request = &input.request;
    let mut required_node_ids = BTreeSet::new();
    let mut candidate_node_ids = BTreeSet::new();
    let influences = load_selected_context_influences(conn, request)?;

    if let Some(root_id) = &request.focused_root_id {
        required_node_ids.insert(root_id.clone());
        candidate_node_ids.extend(load_descendant_node_ids(
            conn,
            root_id,
            request.neighborhood_depth,
            input.max_nodes,
        )?);
    }

    if let Some(selected_node_id) = &request.selected_node_id {
        required_node_ids.insert(selected_node_id.clone());
    }

    if let Some(search) = &request.search {
        candidate_node_ids.extend(load_search_node_ids(conn, search, input.max_nodes)?);
    }

    required_node_ids.extend(
        influences
            .iter()
            .filter_map(|record| record.bible_node_id.as_ref().cloned()),
    );
    required_node_ids.extend(load_influenced_edge_endpoint_ids(conn, &influences)?);

    if should_load_default_node_ids(request) {
        candidate_node_ids.extend(load_default_node_ids(conn, input.max_nodes)?);
    }

    Ok(RenderGraphScope {
        required_node_ids,
        candidate_node_ids,
        influences,
    })
}

fn load_scoped_render_graph_nodes(
    conn: &Connection,
    scope: &RenderGraphScope,
    max_nodes: usize,
) -> Result<Vec<BibleGraphNode>, HistoryStoreError> {
    let included_node_ids = scope
        .required_node_ids
        .iter()
        .chain(scope.candidate_node_ids.iter())
        .cloned()
        .collect();
    let ids_with_ancestors = include_ancestors(conn, &included_node_ids)?;
    Ok(limit_nodes(
        load_nodes_by_id(conn, &ids_with_ancestors)?,
        &scope.required_node_ids,
        &scope.candidate_node_ids,
        max_nodes,
    ))
}

fn load_selected_context_influences(
    conn: &Connection,
    request: &BibleRenderGraphProjectionRequest,
) -> Result<Vec<ContextInfluenceRecord>, HistoryStoreError> {
    let mut target_node_ids = Vec::new();
    let mut seen_target_node_ids = HashSet::new();
    if let Some(target_node_id) = request.selected_timeline_node_id
        && seen_target_node_ids.insert(target_node_id)
    {
        target_node_ids.push(target_node_id);
    }
    if let Some(active_timeline_ms) = request.active_timeline_ms {
        for target_node_id in load_active_timeline_node_ids(conn, active_timeline_ms)? {
            if seen_target_node_ids.insert(target_node_id) {
                target_node_ids.push(target_node_id);
            }
        }
    }

    let mut records = Vec::new();
    let mut seen_record_ids = HashSet::new();
    for target_node_id in target_node_ids {
        for record in crate::context_influence_store::load_latest_context_influence_records(
            conn,
            target_node_id,
        )? {
            if seen_record_ids.insert(record.id) {
                records.push(record);
            }
        }
    }
    Ok(records)
}

fn load_active_timeline_node_ids(
    conn: &Connection,
    active_timeline_ms: u64,
) -> Result<Vec<NodeId>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT id
         FROM nodes
         WHERE start_ms <= ?1 AND end_ms > ?1
         ORDER BY level ASC, start_ms ASC, sort_order ASC, id ASC",
    )?;
    let rows = statement.query_map([active_timeline_ms as i64], |row| row.get::<_, String>(0))?;
    let mut node_ids = Vec::new();
    for row in rows {
        let raw_id = row?;
        let id = uuid::Uuid::parse_str(&raw_id)
            .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?;
        node_ids.push(NodeId(id));
    }
    Ok(node_ids)
}

fn should_load_default_node_ids(request: &BibleRenderGraphProjectionRequest) -> bool {
    request.focused_root_id.is_none() && request.search.is_none()
}

fn load_influenced_edge_endpoint_ids(
    conn: &Connection,
    influences: &[ContextInfluenceRecord],
) -> Result<Vec<BibleGraphNodeId>, HistoryStoreError> {
    let edge_ids: Vec<_> = influences
        .iter()
        .filter_map(|record| record.bible_edge_id.as_ref().cloned())
        .collect();
    let edges = bible_graph_edge_store::load_edges_by_ids(conn, &deduplicate_edge_ids(edge_ids))?;
    let mut node_ids = BTreeSet::new();
    for edge in edges {
        node_ids.insert(edge.from_node_id);
        node_ids.insert(edge.to_node_id);
    }
    Ok(node_ids.into_iter().collect())
}

fn load_default_node_ids(
    conn: &Connection,
    max_nodes: i64,
) -> Result<Vec<BibleGraphNodeId>, HistoryStoreError> {
    load_node_ids(
        conn,
        "SELECT id
         FROM bible_graph_nodes
         WHERE deleted_event_id IS NULL
         ORDER BY sort_order ASC, name ASC, id ASC
         LIMIT ?1",
        [max_nodes],
    )
}

fn load_descendant_node_ids(
    conn: &Connection,
    root_id: &BibleGraphNodeId,
    depth: u32,
    max_nodes: i64,
) -> Result<Vec<BibleGraphNodeId>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "WITH RECURSIVE descendants(id, depth) AS (
            SELECT id, 0
            FROM bible_graph_nodes
            WHERE id = ?1 AND deleted_event_id IS NULL
            UNION ALL
            SELECT child.id, descendants.depth + 1
            FROM bible_graph_nodes child
            INNER JOIN descendants ON child.parent_id = descendants.id
            WHERE child.deleted_event_id IS NULL AND descendants.depth < ?2
         )
         SELECT nodes.id
         FROM bible_graph_nodes nodes
         INNER JOIN descendants ON descendants.id = nodes.id
         ORDER BY nodes.sort_order ASC, nodes.name ASC, nodes.id ASC
         LIMIT ?3",
    )?;
    let rows = statement.query_map(
        params![root_id.as_str(), i64::from(depth), max_nodes],
        |row| row.get::<_, String>(0),
    )?;
    rows_to_node_ids(rows)
}

fn load_search_node_ids(
    conn: &Connection,
    search: &str,
    max_nodes: i64,
) -> Result<Vec<BibleGraphNodeId>, HistoryStoreError> {
    let pattern = escaped_like_pattern(search);
    let mut statement = conn.prepare(
        "SELECT id
         FROM bible_graph_nodes
         WHERE deleted_event_id IS NULL
           AND (
                lower(id) LIKE ?1 ESCAPE '\\'
             OR lower(schema_key) LIKE ?1 ESCAPE '\\'
             OR lower(name) LIKE ?1 ESCAPE '\\'
           )
         ORDER BY sort_order ASC, name ASC, id ASC
         LIMIT ?2",
    )?;
    let rows = statement.query_map(params![pattern, max_nodes], |row| row.get::<_, String>(0))?;
    rows_to_node_ids(rows)
}

fn escaped_like_pattern(search: &str) -> String {
    let mut pattern = String::from("%");
    for character in search.chars() {
        match character {
            '%' | '_' | '\\' => {
                pattern.push('\\');
                pattern.push(character);
            }
            _ => pattern.push(character),
        }
    }
    pattern.push('%');
    pattern
}

fn include_ancestors(
    conn: &Connection,
    node_ids: &BTreeSet<BibleGraphNodeId>,
) -> Result<BTreeSet<BibleGraphNodeId>, HistoryStoreError> {
    let mut ids = BTreeSet::new();
    for node_id in node_ids {
        ids.extend(load_ancestor_node_ids(conn, node_id)?);
    }
    Ok(ids)
}

fn load_ancestor_node_ids(
    conn: &Connection,
    node_id: &BibleGraphNodeId,
) -> Result<Vec<BibleGraphNodeId>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "WITH RECURSIVE ancestors(id, parent_id, depth) AS (
            SELECT id, parent_id, 0
            FROM bible_graph_nodes
            WHERE id = ?1 AND deleted_event_id IS NULL
            UNION ALL
            SELECT parent.id, parent.parent_id, ancestors.depth + 1
            FROM bible_graph_nodes parent
            INNER JOIN ancestors ON ancestors.parent_id = parent.id
            WHERE parent.deleted_event_id IS NULL AND ancestors.depth < ?2
         )
         SELECT id
         FROM ancestors",
    )?;
    let rows = statement.query_map(
        params![node_id.as_str(), i64::from(MAX_PARENT_DEPTH)],
        |row| row.get::<_, String>(0),
    )?;
    rows_to_node_ids(rows)
}

fn load_nodes_by_id(
    conn: &Connection,
    node_ids: &BTreeSet<BibleGraphNodeId>,
) -> Result<Vec<BibleGraphNode>, HistoryStoreError> {
    if node_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = placeholders(node_ids.len());
    let sql = format!(
        "SELECT id, parent_id, schema_key, name, system_owned, sort_order
         FROM bible_graph_nodes
         WHERE deleted_event_id IS NULL AND id IN ({placeholders})
         ORDER BY sort_order ASC, name ASC, id ASC"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(
        params_from_iter(node_ids.iter().map(BibleGraphNodeId::as_str)),
        crate::bible_graph_store::row_to_node,
    )?;

    let mut nodes = Vec::new();
    for row in rows {
        nodes.push(row?);
    }
    Ok(nodes)
}

fn limit_nodes(
    nodes: Vec<BibleGraphNode>,
    required_node_ids: &BTreeSet<BibleGraphNodeId>,
    candidate_node_ids: &BTreeSet<BibleGraphNodeId>,
    max_nodes: usize,
) -> Vec<BibleGraphNode> {
    let mut limited_node_ids = BTreeSet::new();

    for node in nodes
        .iter()
        .filter(|node| required_node_ids.contains(&node.id))
    {
        if limited_node_ids.len() >= max_nodes {
            break;
        }
        limited_node_ids.insert(node.id.clone());
    }

    for node in nodes.iter().filter(|node| {
        required_node_ids.contains(&node.id) || candidate_node_ids.contains(&node.id)
    }) {
        if limited_node_ids.len() >= max_nodes {
            break;
        }
        limited_node_ids.insert(node.id.clone());
    }

    nodes
        .into_iter()
        .filter(|node| limited_node_ids.contains(&node.id))
        .collect()
}

fn load_node_ids<P>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> Result<Vec<BibleGraphNodeId>, HistoryStoreError>
where
    P: rusqlite::Params,
{
    let mut statement = conn.prepare(sql)?;
    let rows = statement.query_map(params, |row| row.get::<_, String>(0))?;
    rows_to_node_ids(rows)
}

fn rows_to_node_ids(
    rows: impl Iterator<Item = Result<String, rusqlite::Error>>,
) -> Result<Vec<BibleGraphNodeId>, HistoryStoreError> {
    let mut ids = Vec::new();
    for row in rows {
        ids.push(BibleGraphNodeId::new(row?).map_err(|e| {
            HistoryStoreError::InvalidValue(format!("invalid bible graph node id: {e}"))
        })?);
    }
    Ok(ids)
}

fn placeholders(count: usize) -> String {
    std::iter::repeat_n("?", count)
        .collect::<Vec<_>>()
        .join(", ")
}

fn deduplicate_edge_ids(edge_ids: Vec<BibleGraphEdgeId>) -> Vec<BibleGraphEdgeId> {
    edge_ids
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

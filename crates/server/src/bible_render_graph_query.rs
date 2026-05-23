use std::collections::BTreeSet;

use eidetic_core::contracts::{
    BibleGraphEdge, BibleGraphEdgeId, BibleGraphNode, BibleGraphNodeId,
    BibleRenderGraphProjectionRequest, ContextInfluenceRecord,
};
use rusqlite::{Connection, params, params_from_iter};

use crate::bible_graph_edge_store;
use crate::history_store::HistoryStoreError;

const MAX_PARENT_DEPTH: u32 = 64;

pub(crate) struct BoundedBibleRenderGraph {
    pub(crate) nodes: Vec<BibleGraphNode>,
    pub(crate) edges: Vec<BibleGraphEdge>,
    pub(crate) influences: Vec<ContextInfluenceRecord>,
}

pub(crate) fn load_bounded_render_graph(
    conn: &Connection,
    request: &BibleRenderGraphProjectionRequest,
) -> Result<BoundedBibleRenderGraph, HistoryStoreError> {
    let request = request.normalized();
    let max_nodes = i64::from(request.max_nodes);
    let mut node_ids = BTreeSet::new();
    let influences = load_selected_context_influences(conn, &request)?;

    if let Some(root_id) = &request.focused_root_id {
        node_ids.extend(load_descendant_node_ids(
            conn,
            root_id,
            request.neighborhood_depth,
            max_nodes,
        )?);
    }

    if let Some(selected_node_id) = &request.selected_node_id {
        node_ids.extend(load_edge_neighborhood_node_ids(
            conn,
            selected_node_id,
            request.neighborhood_depth,
            max_nodes,
        )?);
    }

    if let Some(search) = &request.search {
        node_ids.extend(load_search_node_ids(conn, search, max_nodes)?);
    }

    node_ids.extend(
        influences
            .iter()
            .filter_map(|record| record.bible_node_id.as_ref().cloned()),
    );
    node_ids.extend(load_influenced_edge_endpoint_ids(conn, &influences)?);

    if node_ids.is_empty() && should_load_default_node_ids(&request) {
        node_ids.extend(load_default_node_ids(conn, max_nodes)?);
    }

    let required_node_ids = node_ids.clone();
    let ids_with_ancestors = include_ancestors(conn, &node_ids)?;
    let nodes = limit_nodes(
        load_nodes_by_id(conn, &ids_with_ancestors)?,
        &required_node_ids,
        request.max_nodes as usize,
    );
    let node_ids: Vec<_> = nodes.iter().map(|node| node.id.clone()).collect();
    let edges = bible_graph_edge_store::load_edges_between_nodes(conn, &node_ids)?;

    Ok(BoundedBibleRenderGraph {
        nodes,
        edges,
        influences,
    })
}

fn load_selected_context_influences(
    conn: &Connection,
    request: &BibleRenderGraphProjectionRequest,
) -> Result<Vec<ContextInfluenceRecord>, HistoryStoreError> {
    let Some(target_node_id) = request.selected_timeline_node_id else {
        return Ok(Vec::new());
    };
    crate::context_influence_store::load_latest_context_influence_records(conn, target_node_id)
}

fn should_load_default_node_ids(request: &BibleRenderGraphProjectionRequest) -> bool {
    request.focused_root_id.is_none()
        && request.selected_node_id.is_none()
        && request.search.is_none()
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

fn load_edge_neighborhood_node_ids(
    conn: &Connection,
    node_id: &BibleGraphNodeId,
    depth: u32,
    max_nodes: i64,
) -> Result<Vec<BibleGraphNodeId>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "WITH RECURSIVE neighborhood(id, depth) AS (
            SELECT id, 0
            FROM bible_graph_nodes
            WHERE id = ?1 AND deleted_event_id IS NULL
            UNION
            SELECT
                CASE
                    WHEN edges.from_node_id = neighborhood.id THEN edges.to_node_id
                    ELSE edges.from_node_id
                END,
                neighborhood.depth + 1
            FROM neighborhood
            INNER JOIN bible_graph_edges edges
                ON edges.deleted_event_id IS NULL
               AND (edges.from_node_id = neighborhood.id OR edges.to_node_id = neighborhood.id)
            INNER JOIN bible_graph_nodes next_node
                ON next_node.deleted_event_id IS NULL
               AND next_node.id = CASE
                    WHEN edges.from_node_id = neighborhood.id THEN edges.to_node_id
                    ELSE edges.from_node_id
                END
            WHERE neighborhood.depth < ?2
         )
         SELECT DISTINCT nodes.id
         FROM bible_graph_nodes nodes
         INNER JOIN neighborhood ON neighborhood.id = nodes.id
         ORDER BY nodes.sort_order ASC, nodes.name ASC, nodes.id ASC
         LIMIT ?3",
    )?;
    let rows = statement.query_map(
        params![node_id.as_str(), i64::from(depth), max_nodes],
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
    max_nodes: usize,
) -> Vec<BibleGraphNode> {
    let mut included_node_ids = BTreeSet::new();
    let mut limited = Vec::new();

    for node in nodes
        .iter()
        .filter(|node| required_node_ids.contains(&node.id))
    {
        if limited.len() >= max_nodes {
            return limited;
        }
        included_node_ids.insert(node.id.clone());
        limited.push(node.clone());
    }

    for node in nodes {
        if limited.len() >= max_nodes {
            break;
        }
        if included_node_ids.insert(node.id.clone()) {
            limited.push(node);
        }
    }

    limited
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

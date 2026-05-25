use eidetic_core::contracts::{
    BibleGraphEdge, BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphNodeId, ChangeEventId,
    SetBibleGraphEdgeCommand,
};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params, params_from_iter};

use crate::history_store::HistoryStoreError;

pub(crate) fn set_edge_in_transaction(
    tx: &Transaction<'_>,
    command: &SetBibleGraphEdgeCommand,
    event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    if !node_exists_in_transaction(tx, &command.from_node_id)? {
        return Err(HistoryStoreError::InvalidValue(format!(
            "from bible graph node does not exist: {}",
            command.from_node_id.as_str()
        )));
    }
    if !node_exists_in_transaction(tx, &command.to_node_id)? {
        return Err(HistoryStoreError::InvalidValue(format!(
            "to bible graph node does not exist: {}",
            command.to_node_id.as_str()
        )));
    }

    let edge = command.clone().into_edge();
    let kind = SqlGraphEdgeKind::from_edge_kind(&edge.edge_kind);
    tx.execute(
        "INSERT INTO bible_graph_edges (
            id, from_node_id, to_node_id, edge_kind, custom_kind, label, directed, sort_order,
            created_event_id, updated_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
         ON CONFLICT(id) DO UPDATE SET
            from_node_id = excluded.from_node_id,
            to_node_id = excluded.to_node_id,
            edge_kind = excluded.edge_kind,
            custom_kind = excluded.custom_kind,
            label = excluded.label,
            directed = excluded.directed,
            sort_order = excluded.sort_order,
            updated_event_id = excluded.updated_event_id,
            deleted_event_id = NULL",
        params![
            edge.id.as_str(),
            edge.from_node_id.as_str(),
            edge.to_node_id.as_str(),
            kind.kind,
            kind.custom,
            edge.label,
            if edge.directed { 1_i64 } else { 0_i64 },
            edge.sort_order as i64,
            event_id.0.to_string(),
        ],
    )?;
    Ok(())
}

pub(crate) fn load_edge(
    conn: &Connection,
    edge_id: &BibleGraphEdgeId,
) -> Result<Option<BibleGraphEdge>, HistoryStoreError> {
    conn.query_row(
        edge_select_sql("WHERE id = ?1 AND deleted_event_id IS NULL").as_str(),
        [edge_id.as_str()],
        row_to_edge,
    )
    .optional()
    .map_err(HistoryStoreError::from)
}

pub(crate) fn load_outgoing_edges(
    conn: &Connection,
    node_id: &BibleGraphNodeId,
) -> Result<Vec<BibleGraphEdge>, HistoryStoreError> {
    load_edges_for_node(conn, node_id, "from_node_id")
}

pub(crate) fn load_incoming_edges(
    conn: &Connection,
    node_id: &BibleGraphNodeId,
) -> Result<Vec<BibleGraphEdge>, HistoryStoreError> {
    load_edges_for_node(conn, node_id, "to_node_id")
}

pub(crate) fn load_edges_between_nodes_for_kinds(
    conn: &Connection,
    node_ids: &[BibleGraphNodeId],
    edge_kinds: &[BibleGraphEdgeKind],
    limit: u32,
) -> Result<Vec<BibleGraphEdge>, HistoryStoreError> {
    if node_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = placeholders(node_ids.len());
    let (edge_kind_filter, edge_kind_params) = edge_kind_filter(edge_kinds);
    let edge_kind_clause = edge_kind_filter
        .map(|filter| format!(" AND ({filter})"))
        .unwrap_or_default();
    let sql = edge_select_sql(format!(
        "WHERE deleted_event_id IS NULL
            AND from_node_id IN ({placeholders})
            AND to_node_id IN ({placeholders})
            {edge_kind_clause}
         ORDER BY sort_order ASC, label ASC, id ASC
         LIMIT ?"
    ));
    let params: Vec<_> = node_ids
        .iter()
        .chain(node_ids.iter())
        .map(|node_id| rusqlite::types::Value::from(node_id.as_str().to_string()))
        .chain(edge_kind_params)
        .chain(std::iter::once(rusqlite::types::Value::from(i64::from(
            limit,
        ))))
        .collect();
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(params), row_to_edge)?;

    let mut edges = Vec::new();
    for row in rows {
        edges.push(row?);
    }
    Ok(edges)
}

fn edge_kind_filter(
    edge_kinds: &[BibleGraphEdgeKind],
) -> (Option<String>, Vec<rusqlite::types::Value>) {
    if edge_kinds.is_empty() {
        return (None, Vec::new());
    }

    let mut clauses = Vec::new();
    let mut params = Vec::new();
    for edge_kind in edge_kinds {
        let kind = SqlGraphEdgeKind::from_edge_kind(edge_kind);
        match kind.custom {
            Some(custom) => {
                clauses.push("(edge_kind = ? AND custom_kind = ?)".to_string());
                params.push(rusqlite::types::Value::from(kind.kind));
                params.push(rusqlite::types::Value::from(custom));
            }
            None => {
                clauses.push("edge_kind = ?".to_string());
                params.push(rusqlite::types::Value::from(kind.kind));
            }
        }
    }

    (Some(clauses.join(" OR ")), params)
}

pub(crate) fn load_edges_by_ids(
    conn: &Connection,
    edge_ids: &[BibleGraphEdgeId],
) -> Result<Vec<BibleGraphEdge>, HistoryStoreError> {
    if edge_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = placeholders(edge_ids.len());
    let sql = edge_select_sql(format!(
        "WHERE deleted_event_id IS NULL
            AND id IN ({placeholders})
         ORDER BY sort_order ASC, label ASC, id ASC"
    ));
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(
        params_from_iter(edge_ids.iter().map(BibleGraphEdgeId::as_str)),
        row_to_edge,
    )?;

    let mut edges = Vec::new();
    for row in rows {
        edges.push(row?);
    }
    Ok(edges)
}

fn load_edges_for_node(
    conn: &Connection,
    node_id: &BibleGraphNodeId,
    column: &'static str,
) -> Result<Vec<BibleGraphEdge>, HistoryStoreError> {
    let sql = edge_select_sql(format!(
        "WHERE {column} = ?1 AND deleted_event_id IS NULL ORDER BY sort_order ASC, label ASC, id ASC"
    ));
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map([node_id.as_str()], row_to_edge)?;

    let mut edges = Vec::new();
    for row in rows {
        edges.push(row?);
    }
    Ok(edges)
}

fn node_exists_in_transaction(
    tx: &Transaction<'_>,
    node_id: &BibleGraphNodeId,
) -> Result<bool, HistoryStoreError> {
    tx.query_row(
        "SELECT 1 FROM bible_graph_nodes WHERE id = ?1 AND deleted_event_id IS NULL",
        [node_id.as_str()],
        |_| Ok(()),
    )
    .optional()
    .map(|value| value.is_some())
    .map_err(HistoryStoreError::from)
}

fn edge_select_sql(where_clause: impl AsRef<str>) -> String {
    format!(
        "SELECT id, from_node_id, to_node_id, edge_kind, custom_kind, label, directed, sort_order
         FROM bible_graph_edges {}",
        where_clause.as_ref()
    )
}

fn placeholders(count: usize) -> String {
    std::iter::repeat_n("?", count)
        .collect::<Vec<_>>()
        .join(", ")
}

fn row_to_edge(row: &Row<'_>) -> Result<BibleGraphEdge, rusqlite::Error> {
    let id: String = row.get(0)?;
    let from_node_id: String = row.get(1)?;
    let to_node_id: String = row.get(2)?;
    let kind = SqlGraphEdgeKind {
        kind: row.get(3)?,
        custom: row.get(4)?,
    }
    .into_edge_kind()
    .map_err(|e| conversion_failure(row, 3, e))?;
    let sort_order: i64 = row.get(7)?;

    Ok(BibleGraphEdge {
        id: BibleGraphEdgeId::new(id).map_err(|e| conversion_failure(row, 0, e))?,
        from_node_id: BibleGraphNodeId::new(from_node_id)
            .map_err(|e| conversion_failure(row, 1, e))?,
        to_node_id: BibleGraphNodeId::new(to_node_id).map_err(|e| conversion_failure(row, 2, e))?,
        edge_kind: kind,
        label: row.get(5)?,
        directed: row.get::<_, i64>(6)? != 0,
        sort_order: u32::try_from(sort_order).map_err(|e| conversion_failure(row, 7, e))?,
    })
}

struct SqlGraphEdgeKind {
    kind: String,
    custom: Option<String>,
}

impl SqlGraphEdgeKind {
    fn from_edge_kind(value: &BibleGraphEdgeKind) -> Self {
        match value {
            BibleGraphEdgeKind::References => Self::built_in("references"),
            BibleGraphEdgeKind::LocatedIn => Self::built_in("located_in"),
            BibleGraphEdgeKind::Owns => Self::built_in("owns"),
            BibleGraphEdgeKind::MemberOf => Self::built_in("member_of"),
            BibleGraphEdgeKind::ConflictsWith => Self::built_in("conflicts_with"),
            BibleGraphEdgeKind::SupportsTheme => Self::built_in("supports_theme"),
            BibleGraphEdgeKind::Custom(value) => Self {
                kind: "custom".to_string(),
                custom: Some(value.clone()),
            },
        }
    }

    fn built_in(kind: &'static str) -> Self {
        Self {
            kind: kind.to_string(),
            custom: None,
        }
    }

    fn into_edge_kind(self) -> Result<BibleGraphEdgeKind, HistoryStoreError> {
        match self.kind.as_str() {
            "references" => Ok(BibleGraphEdgeKind::References),
            "located_in" => Ok(BibleGraphEdgeKind::LocatedIn),
            "owns" => Ok(BibleGraphEdgeKind::Owns),
            "member_of" => Ok(BibleGraphEdgeKind::MemberOf),
            "conflicts_with" => Ok(BibleGraphEdgeKind::ConflictsWith),
            "supports_theme" => Ok(BibleGraphEdgeKind::SupportsTheme),
            "custom" => Ok(BibleGraphEdgeKind::Custom(
                self.custom
                    .ok_or(HistoryStoreError::MissingColumn("custom_kind"))?,
            )),
            other => Err(HistoryStoreError::InvalidValue(format!(
                "unknown graph edge kind: {other}"
            ))),
        }
    }
}

fn conversion_failure<E>(row: &Row<'_>, index: usize, error: E) -> rusqlite::Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    let value_type = row
        .get_ref(index)
        .map(|value| value.data_type())
        .unwrap_or(rusqlite::types::Type::Null);
    rusqlite::Error::FromSqlConversionFailure(index, value_type, Box::new(error))
}

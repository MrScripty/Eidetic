use eidetic_core::timeline::node::StoryNode;
use rusqlite::{Transaction, params};

use crate::history_store::HistoryStoreError;

const TIMELINE_NODE_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS nodes (
    id           TEXT PRIMARY KEY,
    parent_id    TEXT,
    level        TEXT NOT NULL,
    sort_order   INTEGER NOT NULL DEFAULT 0,
    start_ms     INTEGER NOT NULL,
    end_ms       INTEGER NOT NULL,
    name         TEXT NOT NULL,
    content_json TEXT NOT NULL DEFAULT '{}',
    beat_type    TEXT,
    locked       INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_nodes_parent ON nodes(parent_id) WHERE parent_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_nodes_level ON nodes(level);
"#;

pub(crate) fn upsert_nodes_in_transaction(
    tx: &Transaction<'_>,
    nodes: &[StoryNode],
) -> Result<(), HistoryStoreError> {
    tx.execute_batch(TIMELINE_NODE_SCHEMA_SQL)?;

    for node in nodes {
        upsert_node(tx, node)?;
    }

    Ok(())
}

fn upsert_node(tx: &Transaction<'_>, node: &StoryNode) -> Result<(), HistoryStoreError> {
    let content_json = serde_json::to_string(&node.content)?;
    let beat_type_json = node
        .beat_type
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;
    let parent_id = node.parent_id.map(|id| id.0.to_string());

    tx.execute(
        "INSERT INTO nodes (
             id, parent_id, level, sort_order, start_ms, end_ms, name, content_json, beat_type, locked
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
             parent_id = excluded.parent_id,
             level = excluded.level,
             sort_order = excluded.sort_order,
             start_ms = excluded.start_ms,
             end_ms = excluded.end_ms,
             name = excluded.name,
             content_json = excluded.content_json,
             beat_type = excluded.beat_type,
             locked = excluded.locked",
        params![
            node.id.0.to_string(),
            parent_id,
            node.level.label(),
            node.sort_order as i64,
            node.time_range.start_ms as i64,
            node.time_range.end_ms as i64,
            node.name,
            content_json,
            beat_type_json,
            node.locked as i64,
        ],
    )?;

    Ok(())
}

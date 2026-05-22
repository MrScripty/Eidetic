use eidetic_core::story::arc::ArcId;
use eidetic_core::timeline::node::{
    BeatType, ContentStatus, NodeArc, NodeContent, NodeId, StoryLevel, StoryNode,
};
use eidetic_core::timeline::timing::TimeRange;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use uuid::Uuid;

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

CREATE TABLE IF NOT EXISTS node_arcs (
    node_id TEXT NOT NULL,
    arc_id  TEXT NOT NULL,
    PRIMARY KEY (node_id, arc_id)
);
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

pub(crate) fn delete_nodes_in_transaction(
    tx: &Transaction<'_>,
    node_ids: &[NodeId],
) -> Result<(), HistoryStoreError> {
    tx.execute_batch(TIMELINE_NODE_SCHEMA_SQL)?;

    for node_id in node_ids {
        tx.execute(
            "DELETE FROM node_arcs WHERE node_id = ?1",
            [node_id.0.to_string()],
        )?;
        tx.execute("DELETE FROM nodes WHERE id = ?1", [node_id.0.to_string()])?;
    }

    Ok(())
}

pub(crate) fn replace_node_arcs_in_transaction(
    tx: &Transaction<'_>,
    node_arcs: &[NodeArc],
) -> Result<(), HistoryStoreError> {
    tx.execute_batch(TIMELINE_NODE_SCHEMA_SQL)?;
    tx.execute("DELETE FROM node_arcs", [])?;

    for node_arc in node_arcs {
        tx.execute(
            "INSERT INTO node_arcs (node_id, arc_id) VALUES (?1, ?2)",
            params![
                node_arc.node_id.0.to_string(),
                node_arc.arc_id.0.to_string()
            ],
        )?;
    }

    Ok(())
}

pub(crate) fn load_nodes(conn: &Connection) -> Result<Vec<StoryNode>, HistoryStoreError> {
    conn.execute_batch(TIMELINE_NODE_SCHEMA_SQL)?;
    let mut stmt = conn.prepare(
        "SELECT id, parent_id, level, sort_order, start_ms, end_ms,
                name, content_json, beat_type, locked
         FROM nodes ORDER BY level, start_ms",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i32>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, Option<String>>(8)?,
            row.get::<_, i32>(9)?,
        ))
    })?;

    let mut nodes = Vec::new();
    for row in rows {
        let (
            id,
            parent_id,
            level,
            sort_order,
            start_ms,
            end_ms,
            name,
            content_json,
            beat_type_json,
            locked,
        ) = row?;
        nodes.push(StoryNode {
            id: NodeId(parse_uuid(&id)?),
            parent_id: parent_id
                .map(|id| parse_uuid(&id).map(NodeId))
                .transpose()?,
            level: parse_story_level(&level)?,
            sort_order: sort_order as u32,
            time_range: TimeRange {
                start_ms: start_ms as u64,
                end_ms: end_ms as u64,
            },
            name,
            content: serde_json::from_str::<NodeContent>(&content_json)?,
            beat_type: beat_type_json
                .map(|beat_type| serde_json::from_str::<BeatType>(&beat_type))
                .transpose()?,
            locked: locked != 0,
        });
    }

    Ok(nodes)
}

pub(crate) fn load_node_ancestor_stack(
    conn: &Connection,
    target_node_id: NodeId,
) -> Result<Vec<StoryNode>, HistoryStoreError> {
    conn.execute_batch(TIMELINE_NODE_SCHEMA_SQL)?;
    let mut stmt = conn.prepare(
        "WITH RECURSIVE stack(
            id, parent_id, level, sort_order, start_ms, end_ms,
            name, content_json, beat_type, locked, depth
         ) AS (
            SELECT id, parent_id, level, sort_order, start_ms, end_ms,
                name, content_json, beat_type, locked, 0
            FROM nodes
            WHERE id = ?1
            UNION ALL
            SELECT parent.id, parent.parent_id, parent.level, parent.sort_order,
                parent.start_ms, parent.end_ms, parent.name, parent.content_json,
                parent.beat_type, parent.locked, stack.depth + 1
            FROM nodes parent
            INNER JOIN stack ON stack.parent_id = parent.id
         )
         SELECT id, parent_id, level, sort_order, start_ms, end_ms,
            name, content_json, beat_type, locked
         FROM stack
         ORDER BY depth DESC",
    )?;
    let rows = stmt.query_map([target_node_id.0.to_string()], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i32>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, Option<String>>(8)?,
            row.get::<_, i32>(9)?,
        ))
    })?;

    let mut nodes = Vec::new();
    for row in rows {
        nodes.push(story_node_from_row(row?)?);
    }
    Ok(nodes)
}

pub(crate) fn load_node_arcs(conn: &Connection) -> Result<Vec<NodeArc>, HistoryStoreError> {
    conn.execute_batch(TIMELINE_NODE_SCHEMA_SQL)?;
    let mut stmt = conn.prepare("SELECT node_id, arc_id FROM node_arcs")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut node_arcs = Vec::new();
    for row in rows {
        let (node_id, arc_id) = row?;
        node_arcs.push(NodeArc {
            node_id: NodeId(parse_uuid(&node_id)?),
            arc_id: ArcId(parse_uuid(&arc_id)?),
        });
    }

    Ok(node_arcs)
}

fn story_node_from_row(
    row: (
        String,
        Option<String>,
        String,
        i32,
        i64,
        i64,
        String,
        String,
        Option<String>,
        i32,
    ),
) -> Result<StoryNode, HistoryStoreError> {
    let (
        id,
        parent_id,
        level,
        sort_order,
        start_ms,
        end_ms,
        name,
        content_json,
        beat_type_json,
        locked,
    ) = row;
    Ok(StoryNode {
        id: NodeId(parse_uuid(&id)?),
        parent_id: parent_id
            .map(|id| parse_uuid(&id).map(NodeId))
            .transpose()?,
        level: parse_story_level(&level)?,
        sort_order: sort_order as u32,
        time_range: TimeRange {
            start_ms: start_ms as u64,
            end_ms: end_ms as u64,
        },
        name,
        content: serde_json::from_str::<NodeContent>(&content_json)?,
        beat_type: beat_type_json
            .map(|beat_type| serde_json::from_str::<BeatType>(&beat_type))
            .transpose()?,
        locked: locked != 0,
    })
}

pub(crate) fn update_node_content_status(
    conn: &Connection,
    node_id: NodeId,
    status: ContentStatus,
) -> Result<(), HistoryStoreError> {
    update_node_content(conn, node_id, |content| {
        content.status = status;
    })
}

pub(crate) fn update_node_scene_recap(
    conn: &Connection,
    node_id: NodeId,
    scene_recap: String,
) -> Result<(), HistoryStoreError> {
    update_node_content(conn, node_id, |content| {
        content.scene_recap = Some(scene_recap);
    })
}

fn update_node_content(
    conn: &Connection,
    node_id: NodeId,
    update: impl FnOnce(&mut NodeContent),
) -> Result<(), HistoryStoreError> {
    conn.execute_batch(TIMELINE_NODE_SCHEMA_SQL)?;
    let content_json = conn
        .query_row(
            "SELECT content_json FROM nodes WHERE id = ?1",
            [node_id.0.to_string()],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or_else(|| {
            HistoryStoreError::InvalidValue(format!("timeline node not found: {}", node_id.0))
        })?;
    let mut content = serde_json::from_str::<NodeContent>(&content_json)?;
    update(&mut content);
    let content_json = serde_json::to_string(&content)?;
    conn.execute(
        "UPDATE nodes SET content_json = ?1 WHERE id = ?2",
        params![content_json, node_id.0.to_string()],
    )?;

    Ok(())
}

fn parse_uuid(value: &str) -> Result<Uuid, HistoryStoreError> {
    Uuid::parse_str(value).map_err(|error| HistoryStoreError::InvalidId(error.to_string()))
}

fn parse_story_level(value: &str) -> Result<StoryLevel, HistoryStoreError> {
    match value {
        "Premise" => Ok(StoryLevel::Premise),
        "Act" => Ok(StoryLevel::Act),
        "Sequence" => Ok(StoryLevel::Sequence),
        "Scene" => Ok(StoryLevel::Scene),
        "Beat" => Ok(StoryLevel::Beat),
        _ => Err(HistoryStoreError::InvalidValue(format!(
            "unknown story level: {value}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use eidetic_core::timeline::node::{StoryLevel, StoryNode};

    #[test]
    fn updates_node_content_status() {
        let conn = Connection::open_in_memory().expect("open sqlite");
        let node = StoryNode::new(
            "Scene",
            StoryLevel::Scene,
            TimeRange::new(0, 1_000).expect("range"),
        );
        let node_id = node.id;
        let tx = conn.unchecked_transaction().expect("transaction");
        upsert_nodes_in_transaction(&tx, &[node]).expect("seed node");
        tx.commit().expect("commit");

        update_node_content_status(&conn, node_id, ContentStatus::Generating)
            .expect("update status");

        let nodes = load_nodes(&conn).expect("load nodes");
        assert_eq!(nodes[0].content.status, ContentStatus::Generating);
    }

    #[test]
    fn updates_node_scene_recap() {
        let conn = Connection::open_in_memory().expect("open sqlite");
        let node = StoryNode::new(
            "Scene",
            StoryLevel::Scene,
            TimeRange::new(0, 1_000).expect("range"),
        );
        let node_id = node.id;
        let tx = conn.unchecked_transaction().expect("transaction");
        upsert_nodes_in_transaction(&tx, &[node]).expect("seed node");
        tx.commit().expect("commit");

        update_node_scene_recap(&conn, node_id, "Ada leaves in rain.".to_string())
            .expect("update recap");

        let nodes = load_nodes(&conn).expect("load nodes");
        assert_eq!(
            nodes[0].content.scene_recap.as_deref(),
            Some("Ada leaves in rain.")
        );
    }

    #[test]
    fn loads_node_ancestor_stack_from_root_to_target() {
        let conn = Connection::open_in_memory().expect("open sqlite");
        let premise = StoryNode::new(
            "Premise",
            StoryLevel::Premise,
            TimeRange::new(0, 10_000).expect("range"),
        );
        let mut act = StoryNode::new(
            "Act",
            StoryLevel::Act,
            TimeRange::new(0, 10_000).expect("range"),
        );
        act.parent_id = Some(premise.id);
        let mut scene = StoryNode::new(
            "Scene",
            StoryLevel::Scene,
            TimeRange::new(0, 10_000).expect("range"),
        );
        scene.parent_id = Some(act.id);
        let scene_id = scene.id;
        let tx = conn.unchecked_transaction().expect("transaction");
        upsert_nodes_in_transaction(&tx, &[scene, act, premise]).expect("seed nodes");
        tx.commit().expect("commit");

        let stack = load_node_ancestor_stack(&conn, scene_id).expect("load stack");

        assert_eq!(stack.len(), 3);
        assert_eq!(stack[0].level, StoryLevel::Premise);
        assert_eq!(stack[1].level, StoryLevel::Act);
        assert_eq!(stack[2].id, scene_id);
    }
}

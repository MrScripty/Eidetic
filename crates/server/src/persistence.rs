use std::path::{Path, PathBuf};

use eidetic_core::Project;
use eidetic_core::contracts::ObjectKind;
use eidetic_core::reference::{ReferenceDocument, ReferenceType};
use eidetic_core::story::arc::{ArcId, ArcType, Color, StoryArc};
use eidetic_core::timeline::Timeline;
use eidetic_core::timeline::node::{BeatType, NodeArc, NodeContent, NodeId, StoryLevel, StoryNode};
use eidetic_core::timeline::relationship::{Relationship, RelationshipId, RelationshipType};
use eidetic_core::timeline::structure::EpisodeStructure;
use eidetic_core::timeline::timing::TimeRange;
use eidetic_core::timeline::track::{Track, TrackId};
use rusqlite::{Connection, params};
use serde::Serialize;
use tokio::fs;
use uuid::Uuid;

/// Metadata for a saved project on disk.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectEntry {
    pub name: String,
    pub path: PathBuf,
    pub modified: String,
}

/// Default base directory for project storage.
pub fn default_project_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("eidetic")
        .join("projects")
}

/// Generate a save path for a project based on its name.
pub fn project_save_path(name: &str) -> PathBuf {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    default_project_dir().join(sanitized).join("project.db")
}

// ─── Schema ────────────────────────────────────────────────────────

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS schema_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
INSERT OR IGNORE INTO schema_meta (key, value) VALUES ('version', '3');

CREATE TABLE IF NOT EXISTS project (
    id                INTEGER PRIMARY KEY CHECK (id = 1),
    name              TEXT NOT NULL,
    premise           TEXT NOT NULL DEFAULT '',
    total_duration_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS episode_structure (
    id            INTEGER PRIMARY KEY CHECK (id = 1),
    template_name TEXT NOT NULL,
    segments_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS arcs (
    id            TEXT PRIMARY KEY,
    parent_arc_id TEXT,
    name          TEXT NOT NULL,
    description   TEXT NOT NULL DEFAULT '',
    arc_type      TEXT NOT NULL,
    color_r       INTEGER NOT NULL,
    color_g       INTEGER NOT NULL,
    color_b       INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS tracks (
    id         TEXT PRIMARY KEY,
    level      TEXT NOT NULL,
    label      TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    collapsed  INTEGER NOT NULL DEFAULT 0
);

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

CREATE TABLE IF NOT EXISTS relationships (
    id                TEXT PRIMARY KEY,
    from_node_id      TEXT NOT NULL,
    to_node_id        TEXT NOT NULL,
    relationship_type TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS reference_documents (
    id       TEXT PRIMARY KEY,
    name     TEXT NOT NULL,
    content  TEXT NOT NULL,
    doc_type TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ydoc_state (
    id    INTEGER PRIMARY KEY CHECK (id = 1),
    state BLOB NOT NULL
);
"#;

fn create_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(SCHEMA_SQL)
        .map_err(|e| format!("schema error: {e}"))?;
    crate::history_store::create_schema(conn).map_err(|e| format!("history schema error: {e}"))
}

fn clear_all_tables(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "DELETE FROM node_arcs;
         DELETE FROM relationships;
         DELETE FROM nodes;
         DELETE FROM tracks;
         DELETE FROM arcs;
         DELETE FROM reference_documents;
         DELETE FROM episode_structure;
         DELETE FROM project;
         DELETE FROM ydoc_state;",
    )
    .map_err(|e| format!("clear error: {e}"))
}

// ─── Save ──────────────────────────────────────────────────────────

/// Save a project to disk as a SQLite database.
///
/// If `ydoc_state` is provided, it is persisted atomically alongside the
/// project data in the same transaction.
pub async fn save_project(
    project: &Project,
    path: &Path,
    ydoc_state: Option<Vec<u8>>,
) -> Result<(), String> {
    let project = project.clone();
    let path = path.to_path_buf();

    tokio::task::spawn_blocking(move || save_project_sync(&project, &path, ydoc_state.as_deref()))
        .await
        .map_err(|e| format!("spawn_blocking error: {e}"))?
}

fn save_project_sync(
    project: &Project,
    path: &Path,
    ydoc_state: Option<&[u8]>,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir error: {e}"))?;
    }

    let conn = crate::sqlite::open_write_connection(path)
        .map_err(|e| format!("sqlite open error: {e}"))?;

    create_schema(&conn)?;
    let arcs = arcs_for_project_save(&conn, project)?;
    let timeline = timeline_for_project_save(&conn, project)?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("transaction error: {e}"))?;

    clear_all_tables(&tx)?;

    // Project metadata.
    tx.execute(
        "INSERT INTO project (id, name, premise, total_duration_ms) VALUES (1, ?1, ?2, ?3)",
        params![
            project.name,
            project.premise,
            timeline.total_duration_ms as i64
        ],
    )
    .map_err(|e| format!("insert project: {e}"))?;

    // Episode structure.
    let segments_json = serde_json::to_string(&timeline.structure.segments)
        .map_err(|e| format!("serialize segments: {e}"))?;
    tx.execute(
        "INSERT INTO episode_structure (id, template_name, segments_json) VALUES (1, ?1, ?2)",
        params![timeline.structure.template_name, segments_json],
    )
    .map_err(|e| format!("insert episode_structure: {e}"))?;

    // Arcs.
    for arc in &arcs {
        insert_arc(&tx, arc)?;
    }

    // Tracks.
    for track in &timeline.tracks {
        insert_track(&tx, track)?;
    }

    // Nodes.
    for node in &timeline.nodes {
        insert_node(&tx, node)?;
    }

    // Node-Arc tags.
    for node_arc in &timeline.node_arcs {
        insert_node_arc(&tx, node_arc)?;
    }

    // Relationships.
    for rel in &timeline.relationships {
        insert_relationship(&tx, rel)?;
    }

    // Reference documents.
    for doc in &project.references {
        insert_reference_document(&tx, doc)?;
    }

    // Y.Doc CRDT state (persisted atomically with structural data).
    if let Some(state) = ydoc_state {
        tx.execute(
            "INSERT INTO ydoc_state (id, state) VALUES (1, ?1)",
            params![state],
        )
        .map_err(|e| format!("insert ydoc_state: {e}"))?;
    }

    tx.commit().map_err(|e| format!("commit error: {e}"))?;

    tracing::debug!("saved project to {}", path.display());
    Ok(())
}

fn arcs_for_project_save(conn: &Connection, project: &Project) -> Result<Vec<StoryArc>, String> {
    let persisted_arcs =
        crate::story_arc_store::load_arcs(conn).map_err(|e| format!("load persisted arcs: {e}"))?;
    let story_arc_revisions =
        crate::history_store::load_revision_summary_for_kind(conn, ObjectKind::StoryArc)
            .map_err(|e| format!("load story arc revision summary: {e}"))?
            .revision_count;

    if !persisted_arcs.is_empty() || story_arc_revisions > 0 {
        return Ok(persisted_arcs);
    }

    Ok(project.arcs.clone())
}

fn timeline_for_project_save(conn: &Connection, project: &Project) -> Result<Timeline, String> {
    let timeline_revision_count =
        crate::history_store::load_revision_summary_for_kind(conn, ObjectKind::TimelineNode)
            .map_err(|e| format!("load timeline node revision summary: {e}"))?
            .revision_count
            + crate::history_store::load_revision_summary_for_kind(
                conn,
                ObjectKind::TimelineRelationship,
            )
            .map_err(|e| format!("load timeline relationship revision summary: {e}"))?
            .revision_count;

    if timeline_revision_count == 0 && !timeline_current_state_exists(conn)? {
        return Ok(project.timeline.clone());
    }

    let (total_duration_ms, structure) = persisted_timeline_metadata(conn)?.unwrap_or_else(|| {
        (
            project.timeline.total_duration_ms,
            project.timeline.structure.clone(),
        )
    });
    let tracks = persisted_tracks_or_project_tracks(conn, project)?;
    Ok(Timeline {
        total_duration_ms,
        tracks,
        nodes: read_nodes(conn)?,
        node_arcs: read_node_arcs(conn)?,
        relationships: read_relationships(conn)?,
        structure,
    })
}

fn timeline_current_state_exists(conn: &Connection) -> Result<bool, String> {
    Ok(table_has_rows(conn, "nodes")?
        || table_has_rows(conn, "node_arcs")?
        || table_has_rows(conn, "relationships")?)
}

fn table_has_rows(conn: &Connection, table_name: &str) -> Result<bool, String> {
    let sql = format!("SELECT EXISTS(SELECT 1 FROM {table_name} LIMIT 1)");
    conn.query_row(&sql, [], |row| row.get::<_, bool>(0))
        .map_err(|e| format!("read {table_name} row presence: {e}"))
}

fn persisted_timeline_metadata(
    conn: &Connection,
) -> Result<Option<(u64, EpisodeStructure)>, String> {
    let total_duration_ms = match conn.query_row(
        "SELECT total_duration_ms FROM project WHERE id = 1",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(total_duration_ms) => total_duration_ms as u64,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(format!("read project timeline metadata: {e}")),
    };

    Ok(Some((total_duration_ms, read_episode_structure(conn)?)))
}

fn persisted_tracks_or_project_tracks(
    conn: &Connection,
    project: &Project,
) -> Result<Vec<Track>, String> {
    let tracks = read_tracks(conn)?;
    if tracks.is_empty() {
        Ok(project.timeline.tracks.clone())
    } else {
        Ok(tracks)
    }
}

fn insert_arc(conn: &Connection, arc: &StoryArc) -> Result<(), String> {
    let arc_type_json =
        serde_json::to_string(&arc.arc_type).map_err(|e| format!("serialize arc_type: {e}"))?;
    let parent_arc_id = arc.parent_arc_id.map(|id| id.0.to_string());
    conn.execute(
        "INSERT INTO arcs (id, parent_arc_id, name, description, arc_type, color_r, color_g, color_b)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            arc.id.0.to_string(),
            parent_arc_id,
            arc.name,
            arc.description,
            arc_type_json,
            arc.color.r,
            arc.color.g,
            arc.color.b,
        ],
    )
    .map_err(|e| format!("insert arc: {e}"))?;
    Ok(())
}

fn insert_track(conn: &Connection, track: &Track) -> Result<(), String> {
    let level_str = track.level.label();
    conn.execute(
        "INSERT INTO tracks (id, level, label, sort_order, collapsed)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            track.id.0.to_string(),
            level_str,
            track.label,
            track.sort_order as i32,
            track.collapsed as i32,
        ],
    )
    .map_err(|e| format!("insert track: {e}"))?;
    Ok(())
}

fn insert_node(conn: &Connection, node: &StoryNode) -> Result<(), String> {
    let content_json =
        serde_json::to_string(&node.content).map_err(|e| format!("serialize content: {e}"))?;
    let beat_type_json = node
        .beat_type
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|e| format!("serialize beat_type: {e}"))?;
    let parent_id = node.parent_id.map(|id| id.0.to_string());
    let level_str = node.level.label();

    conn.execute(
        "INSERT INTO nodes (id, parent_id, level, sort_order, start_ms, end_ms,
                            name, content_json, beat_type, locked)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            node.id.0.to_string(),
            parent_id,
            level_str,
            node.sort_order as i32,
            node.time_range.start_ms as i64,
            node.time_range.end_ms as i64,
            node.name,
            content_json,
            beat_type_json,
            node.locked as i32,
        ],
    )
    .map_err(|e| format!("insert node: {e}"))?;
    Ok(())
}

fn insert_node_arc(conn: &Connection, node_arc: &NodeArc) -> Result<(), String> {
    conn.execute(
        "INSERT INTO node_arcs (node_id, arc_id) VALUES (?1, ?2)",
        params![
            node_arc.node_id.0.to_string(),
            node_arc.arc_id.0.to_string()
        ],
    )
    .map_err(|e| format!("insert node_arc: {e}"))?;
    Ok(())
}

fn insert_relationship(conn: &Connection, rel: &Relationship) -> Result<(), String> {
    let rel_type_json = serde_json::to_string(&rel.relationship_type)
        .map_err(|e| format!("serialize relationship_type: {e}"))?;
    conn.execute(
        "INSERT INTO relationships (id, from_node_id, to_node_id, relationship_type)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            rel.id.0.to_string(),
            rel.from_node.0.to_string(),
            rel.to_node.0.to_string(),
            rel_type_json,
        ],
    )
    .map_err(|e| format!("insert relationship: {e}"))?;
    Ok(())
}

fn insert_reference_document(conn: &Connection, doc: &ReferenceDocument) -> Result<(), String> {
    let doc_type_json =
        serde_json::to_string(&doc.doc_type).map_err(|e| format!("serialize doc_type: {e}"))?;
    conn.execute(
        "INSERT INTO reference_documents (id, name, content, doc_type) VALUES (?1, ?2, ?3, ?4)",
        params![doc.id.0.to_string(), doc.name, doc.content, doc_type_json,],
    )
    .map_err(|e| format!("insert reference_document: {e}"))?;
    Ok(())
}

// ─── Load ──────────────────────────────────────────────────────────

/// Load a project from the current SQLite project database format.
///
/// Returns `(project, ydoc_state)` where `ydoc_state` is the persisted CRDT
/// blob (if any). When `None`, the caller should populate Y.Doc from the
/// project's cached text fields.
pub async fn load_project(path: &Path) -> Result<(Project, Option<Vec<u8>>), String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || load_project_sync(&path))
        .await
        .map_err(|e| format!("spawn_blocking error: {e}"))?
}

fn load_project_sync(path: &Path) -> Result<(Project, Option<Vec<u8>>), String> {
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("sqlite open error: {e}"))?;

    let version = read_schema_version(&conn);
    if version != 3 {
        return Err(format!(
            "unsupported project schema version {version}; expected 3"
        ));
    }

    let project = load_project_v2(&conn, path)?;
    let ydoc_state = read_ydoc_state(&conn)?;

    Ok((project, ydoc_state))
}

fn read_schema_version(conn: &Connection) -> u32 {
    conn.query_row(
        "SELECT value FROM schema_meta WHERE key = 'version'",
        [],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .and_then(|v| v.parse::<u32>().ok())
    .unwrap_or(1)
}

fn read_ydoc_state(conn: &Connection) -> Result<Option<Vec<u8>>, String> {
    match conn.query_row("SELECT state FROM ydoc_state WHERE id = 1", [], |row| {
        row.get::<_, Vec<u8>>(0)
    }) {
        Ok(state) => Ok(Some(state)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("read ydoc_state: {e}")),
    }
}

fn load_project_v2(conn: &Connection, path: &Path) -> Result<Project, String> {
    // Project metadata.
    let (name, premise, total_duration_ms): (String, String, i64) = conn
        .query_row(
            "SELECT name, premise, total_duration_ms FROM project WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("read project: {e}"))?;

    // Episode structure.
    let structure = read_episode_structure(conn)?;

    // Arcs.
    let arcs = read_arcs(conn)?;

    // Tracks.
    let tracks = read_tracks(conn)?;

    // Nodes.
    let nodes = read_nodes(conn)?;

    // Node-Arc tags.
    let node_arcs = read_node_arcs(conn)?;

    // Relationships.
    let relationships = read_relationships(conn)?;

    // Reference documents.
    let references = read_reference_documents(conn)?;

    let timeline = Timeline {
        total_duration_ms: total_duration_ms as u64,
        tracks,
        nodes,
        node_arcs,
        relationships,
        structure,
    };

    let project = Project {
        name,
        premise,
        timeline,
        arcs,
        references,
    };

    tracing::debug!("loaded project from {}", path.display());
    Ok(project)
}

fn parse_uuid(s: &str) -> Result<Uuid, String> {
    Uuid::parse_str(s).map_err(|e| format!("parse UUID '{s}': {e}"))
}

fn parse_story_level(s: &str) -> Result<StoryLevel, String> {
    match s {
        "Premise" => Ok(StoryLevel::Premise),
        "Act" => Ok(StoryLevel::Act),
        "Sequence" => Ok(StoryLevel::Sequence),
        "Scene" => Ok(StoryLevel::Scene),
        "Beat" => Ok(StoryLevel::Beat),
        _ => Err(format!("unknown story level: '{s}'")),
    }
}

fn read_episode_structure(conn: &Connection) -> Result<EpisodeStructure, String> {
    let (template_name, segments_json): (String, String) = conn
        .query_row(
            "SELECT template_name, segments_json FROM episode_structure WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("read episode_structure: {e}"))?;

    let segments =
        serde_json::from_str(&segments_json).map_err(|e| format!("parse segments: {e}"))?;

    Ok(EpisodeStructure {
        template_name,
        segments,
    })
}

fn read_arcs(conn: &Connection) -> Result<Vec<StoryArc>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, parent_arc_id, name, description, arc_type,
                    color_r, color_g, color_b FROM arcs",
        )
        .map_err(|e| format!("prepare arcs: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, u8>(5)?,
                row.get::<_, u8>(6)?,
                row.get::<_, u8>(7)?,
            ))
        })
        .map_err(|e| format!("query arcs: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (id_str, parent_arc_id_str, name, description, arc_type_json, r, g, b) =
            row.map_err(|e| format!("read arc row: {e}"))?;
        let id = ArcId(parse_uuid(&id_str)?);
        let parent_arc_id = parent_arc_id_str
            .map(|s| parse_uuid(&s).map(ArcId))
            .transpose()?;
        let arc_type: ArcType =
            serde_json::from_str(&arc_type_json).map_err(|e| format!("parse arc_type: {e}"))?;
        result.push(StoryArc {
            id,
            parent_arc_id,
            name,
            description,
            arc_type,
            color: Color::new(r, g, b),
        });
    }
    Ok(result)
}

fn read_tracks(conn: &Connection) -> Result<Vec<Track>, String> {
    let mut stmt = conn
        .prepare("SELECT id, level, label, sort_order, collapsed FROM tracks ORDER BY sort_order")
        .map_err(|e| format!("prepare tracks: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, i32>(4)?,
            ))
        })
        .map_err(|e| format!("query tracks: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (id_str, level_str, label, sort_order, collapsed) =
            row.map_err(|e| format!("read track row: {e}"))?;
        result.push(Track {
            id: TrackId(parse_uuid(&id_str)?),
            level: parse_story_level(&level_str)?,
            label,
            sort_order: sort_order as u32,
            collapsed: collapsed != 0,
        });
    }
    Ok(result)
}

fn read_nodes(conn: &Connection) -> Result<Vec<StoryNode>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, parent_id, level, sort_order, start_ms, end_ms,
                    name, content_json, beat_type, locked
             FROM nodes ORDER BY level, start_ms",
        )
        .map_err(|e| format!("prepare nodes: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
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
        })
        .map_err(|e| format!("query nodes: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (
            id_str,
            parent_id_str,
            level_str,
            sort_order,
            start_ms,
            end_ms,
            name,
            content_json,
            beat_type_json,
            locked,
        ) = row.map_err(|e| format!("read node row: {e}"))?;

        let parent_id = parent_id_str
            .map(|s| parse_uuid(&s).map(NodeId))
            .transpose()?;
        let level = parse_story_level(&level_str)?;
        let content: NodeContent =
            serde_json::from_str(&content_json).map_err(|e| format!("parse content: {e}"))?;
        let beat_type: Option<BeatType> = beat_type_json
            .map(|j| serde_json::from_str(&j))
            .transpose()
            .map_err(|e| format!("parse beat_type: {e}"))?;

        result.push(StoryNode {
            id: NodeId(parse_uuid(&id_str)?),
            parent_id,
            level,
            sort_order: sort_order as u32,
            time_range: TimeRange {
                start_ms: start_ms as u64,
                end_ms: end_ms as u64,
            },
            name,
            content,
            beat_type,
            locked: locked != 0,
        });
    }
    Ok(result)
}

fn read_node_arcs(conn: &Connection) -> Result<Vec<NodeArc>, String> {
    let mut stmt = conn
        .prepare("SELECT node_id, arc_id FROM node_arcs")
        .map_err(|e| format!("prepare node_arcs: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("query node_arcs: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (node_id_str, arc_id_str) = row.map_err(|e| format!("read node_arc row: {e}"))?;
        result.push(NodeArc {
            node_id: NodeId(parse_uuid(&node_id_str)?),
            arc_id: ArcId(parse_uuid(&arc_id_str)?),
        });
    }
    Ok(result)
}

fn read_relationships(conn: &Connection) -> Result<Vec<Relationship>, String> {
    let mut stmt = conn
        .prepare("SELECT id, from_node_id, to_node_id, relationship_type FROM relationships")
        .map_err(|e| format!("prepare relationships: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| format!("query relationships: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (id_str, from_str, to_str, rel_type_json) =
            row.map_err(|e| format!("read relationship row: {e}"))?;
        let rel_type: RelationshipType = serde_json::from_str(&rel_type_json)
            .map_err(|e| format!("parse relationship_type: {e}"))?;
        result.push(Relationship {
            id: RelationshipId(parse_uuid(&id_str)?),
            from_node: NodeId(parse_uuid(&from_str)?),
            to_node: NodeId(parse_uuid(&to_str)?),
            relationship_type: rel_type,
        });
    }
    Ok(result)
}

fn read_reference_documents(conn: &Connection) -> Result<Vec<ReferenceDocument>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, content, doc_type FROM reference_documents")
        .map_err(|e| format!("prepare reference_documents: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| format!("query reference_documents: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (id_str, name, content, doc_type_json) =
            row.map_err(|e| format!("read reference_document row: {e}"))?;
        let doc_type: ReferenceType =
            serde_json::from_str(&doc_type_json).map_err(|e| format!("parse doc_type: {e}"))?;
        result.push(ReferenceDocument {
            id: eidetic_core::reference::ReferenceId(parse_uuid(&id_str)?),
            name,
            content,
            doc_type,
        });
    }
    Ok(result)
}

// ─── List Projects ─────────────────────────────────────────────────

/// List saved projects under a base directory.
pub async fn list_projects(base_dir: &Path) -> Vec<ProjectEntry> {
    let mut entries = Vec::new();

    let Ok(mut read_dir) = fs::read_dir(base_dir).await else {
        return entries;
    };

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();

        let project_file = path.join("project.db");
        if !project_file.exists() {
            continue;
        }

        let modified = entry
            .metadata()
            .await
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs().to_string())
            })
            .unwrap_or_default();

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        entries.push(ProjectEntry {
            name,
            path: project_file,
            modified,
        });
    }

    entries
}

#[cfg(test)]
mod tests {
    use eidetic_core::Template;
    use eidetic_core::contracts::{
        CommandEnvelope, DeleteStoryArcCommand, DeleteTimelineNodeCommand,
    };
    use eidetic_core::story::arc::{ArcType, Color, StoryArc};
    use eidetic_core::timeline::Timeline;
    use eidetic_core::timeline::relationship::{Relationship, RelationshipType};
    use eidetic_core::timeline::structure::EpisodeStructure;
    use uuid::Uuid;

    use super::{load_project_sync, save_project_sync};

    fn temp_project_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("eidetic-persistence-{label}-{}.db", Uuid::new_v4()))
    }

    fn project_with_arc(name: &str) -> eidetic_core::Project {
        let mut project = eidetic_core::Project::new(
            "Persistence Test",
            Timeline::new(1_320_000, EpisodeStructure::standard_30_min()),
        );
        project
            .arcs
            .push(StoryArc::new(name, ArcType::APlot, Color::new(10, 20, 30)));
        project
    }

    #[test]
    fn broad_save_preserves_existing_sqlite_story_arcs_when_project_mirror_is_stale() {
        let path = temp_project_path("preserve-arcs");
        let mut project = project_with_arc("Project Mirror");
        let arc_id = project.arcs[0].id;

        save_project_sync(&project, &path, None).expect("initial save");
        {
            let conn = crate::sqlite::open_write_connection(&path).expect("open sqlite");
            conn.execute(
                "UPDATE arcs SET name = ?1 WHERE id = ?2",
                rusqlite::params!["SQLite Canonical", arc_id.0.to_string()],
            )
            .expect("rename sqlite arc");
        }

        project.arcs.clear();
        save_project_sync(&project, &path, None).expect("second save");

        let (loaded, _) = load_project_sync(&path).expect("load project");
        assert_eq!(loaded.arcs.len(), 1);
        assert_eq!(loaded.arcs[0].name, "SQLite Canonical");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn broad_save_does_not_resurrect_deleted_story_arcs_after_history_exists() {
        let path = temp_project_path("deleted-arcs");
        let project = project_with_arc("Deleted Arc");
        let arc_id = project.arcs[0].id;

        save_project_sync(&project, &path, None).expect("initial save");
        {
            let mut conn = crate::sqlite::open_write_connection(&path).expect("open sqlite");
            crate::story_arc_command::record_delete_story_arc_history(
                &mut conn,
                &CommandEnvelope::new(DeleteStoryArcCommand { arc_id }),
                1,
            )
            .expect("delete story arc");
        }

        save_project_sync(&project, &path, None).expect("second save");

        let (loaded, _) = load_project_sync(&path).expect("load project");
        assert!(loaded.arcs.is_empty());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn broad_save_preserves_existing_sqlite_timeline_nodes_when_project_mirror_is_stale() {
        let path = temp_project_path("preserve-timeline-nodes");
        let mut project = Template::MultiCam.build_project("Persistence Test");
        let node_id = project.timeline.nodes[0].id;

        save_project_sync(&project, &path, None).expect("initial save");
        {
            let conn = crate::sqlite::open_write_connection(&path).expect("open sqlite");
            conn.execute(
                "UPDATE nodes SET name = ?1 WHERE id = ?2",
                rusqlite::params!["SQLite Canonical Node", node_id.0.to_string()],
            )
            .expect("rename sqlite node");
        }

        project.timeline.node_mut(node_id).unwrap().name = "Stale Mirror Node".to_string();
        save_project_sync(&project, &path, None).expect("second save");

        let (loaded, _) = load_project_sync(&path).expect("load project");
        let loaded_node = loaded.timeline.node(node_id).expect("loaded node");
        assert_eq!(loaded_node.name, "SQLite Canonical Node");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn broad_save_does_not_resurrect_deleted_timeline_nodes_after_history_exists() {
        let path = temp_project_path("deleted-timeline-nodes");
        let project = Template::MultiCam.build_project("Persistence Test");
        let node_id = project.timeline.nodes[0].id;

        save_project_sync(&project, &path, None).expect("initial save");
        {
            let mut conn = crate::sqlite::open_write_connection(&path).expect("open sqlite");
            crate::timeline_command::record_delete_timeline_node_history(
                &mut conn,
                &project,
                &CommandEnvelope::new(DeleteTimelineNodeCommand { node_id }),
                1,
            )
            .expect("delete timeline node");
        }

        save_project_sync(&project, &path, None).expect("second save");

        let (loaded, _) = load_project_sync(&path).expect("load project");
        assert!(loaded.timeline.node(node_id).is_err());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn broad_save_preserves_existing_sqlite_timeline_relationships() {
        let path = temp_project_path("preserve-timeline-relationships");
        let mut project = Template::MultiCam.build_project("Persistence Test");
        let from_node = project.timeline.nodes[0].id;
        let to_node = project.timeline.nodes[1].id;
        let relationship = Relationship::new(from_node, to_node, RelationshipType::Thematic);
        let relationship_id = relationship.id;
        project.timeline.add_relationship(relationship).unwrap();

        save_project_sync(&project, &path, None).expect("initial save");

        project.timeline.relationships.clear();
        save_project_sync(&project, &path, None).expect("second save");

        let (loaded, _) = load_project_sync(&path).expect("load project");
        assert!(
            loaded
                .timeline
                .relationships
                .iter()
                .any(|relationship| relationship.id == relationship_id)
        );

        let _ = std::fs::remove_file(path);
    }
}

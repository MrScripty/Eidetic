use std::path::{Path, PathBuf};

use eidetic_core::reference::{ReferenceDocument, ReferenceType};
use eidetic_core::story::arc::{ArcId, ArcType, Color, StoryArc};
use eidetic_core::story::bible::{
    Entity, EntityCategory, EntityDetails, EntityId, EntityRelation, EntitySnapshot,
    SnapshotOverrides, StoryBible,
};
use eidetic_core::timeline::node::{
    BeatType, NodeArc, NodeContent, NodeId, StoryLevel, StoryNode,
};
use eidetic_core::timeline::relationship::{Relationship, RelationshipId, RelationshipType};
use eidetic_core::timeline::structure::EpisodeStructure;
use eidetic_core::timeline::timing::TimeRange;
use eidetic_core::timeline::track::{Track, TrackId};
use eidetic_core::timeline::Timeline;
use eidetic_core::Project;
use rusqlite::{params, Connection};
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
INSERT OR IGNORE INTO schema_meta (key, value) VALUES ('version', '2');

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

CREATE TABLE IF NOT EXISTS entities (
    id           TEXT PRIMARY KEY,
    category     TEXT NOT NULL,
    name         TEXT NOT NULL,
    tagline      TEXT NOT NULL DEFAULT '',
    description  TEXT NOT NULL DEFAULT '',
    details_json TEXT NOT NULL DEFAULT '{}',
    color_r      INTEGER NOT NULL,
    color_g      INTEGER NOT NULL,
    color_b      INTEGER NOT NULL,
    locked       INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS entity_snapshots (
    entity_id      TEXT NOT NULL REFERENCES entities(id),
    at_ms          INTEGER NOT NULL,
    source_node_id TEXT,
    description    TEXT NOT NULL,
    overrides_json TEXT,
    PRIMARY KEY (entity_id, at_ms)
);

CREATE TABLE IF NOT EXISTS entity_node_refs (
    entity_id TEXT NOT NULL REFERENCES entities(id),
    node_id   TEXT NOT NULL,
    PRIMARY KEY (entity_id, node_id)
);

CREATE TABLE IF NOT EXISTS entity_relations (
    entity_id        TEXT NOT NULL REFERENCES entities(id),
    target_entity_id TEXT NOT NULL,
    label            TEXT NOT NULL,
    sort_order       INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS reference_documents (
    id       TEXT PRIMARY KEY,
    name     TEXT NOT NULL,
    content  TEXT NOT NULL,
    doc_type TEXT NOT NULL
);
"#;

fn create_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(SCHEMA_SQL)
        .map_err(|e| format!("schema error: {e}"))
}

fn clear_all_tables(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "DELETE FROM entity_relations;
         DELETE FROM entity_node_refs;
         DELETE FROM entity_snapshots;
         DELETE FROM entities;
         DELETE FROM node_arcs;
         DELETE FROM relationships;
         DELETE FROM nodes;
         DELETE FROM tracks;
         DELETE FROM arcs;
         DELETE FROM reference_documents;
         DELETE FROM episode_structure;
         DELETE FROM project;",
    )
    .map_err(|e| format!("clear error: {e}"))
}

// ─── Save ──────────────────────────────────────────────────────────

/// Save a project to disk as a SQLite database.
pub async fn save_project(project: &Project, path: &Path) -> Result<(), String> {
    let project = project.clone();
    let path = path.to_path_buf();

    tokio::task::spawn_blocking(move || save_project_sync(&project, &path))
        .await
        .map_err(|e| format!("spawn_blocking error: {e}"))?
}

fn save_project_sync(project: &Project, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir error: {e}"))?;
    }

    let conn = Connection::open(path).map_err(|e| format!("sqlite open error: {e}"))?;

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;",
    )
    .map_err(|e| format!("pragma error: {e}"))?;

    create_schema(&conn)?;

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
            project.timeline.total_duration_ms as i64
        ],
    )
    .map_err(|e| format!("insert project: {e}"))?;

    // Episode structure.
    let segments_json = serde_json::to_string(&project.timeline.structure.segments)
        .map_err(|e| format!("serialize segments: {e}"))?;
    tx.execute(
        "INSERT INTO episode_structure (id, template_name, segments_json) VALUES (1, ?1, ?2)",
        params![project.timeline.structure.template_name, segments_json],
    )
    .map_err(|e| format!("insert episode_structure: {e}"))?;

    // Arcs.
    for arc in &project.arcs {
        insert_arc(&tx, arc)?;
    }

    // Tracks.
    for track in &project.timeline.tracks {
        insert_track(&tx, track)?;
    }

    // Nodes.
    for node in &project.timeline.nodes {
        insert_node(&tx, node)?;
    }

    // Node-Arc tags.
    for node_arc in &project.timeline.node_arcs {
        insert_node_arc(&tx, node_arc)?;
    }

    // Relationships.
    for rel in &project.timeline.relationships {
        insert_relationship(&tx, rel)?;
    }

    // Entities.
    for entity in &project.bible.entities {
        insert_entity(&tx, entity)?;
        for snapshot in &entity.snapshots {
            insert_entity_snapshot(&tx, &entity.id, snapshot)?;
        }
        for node_id in &entity.node_refs {
            insert_entity_node_ref(&tx, &entity.id, node_id)?;
        }
        for (idx, relation) in entity.relations.iter().enumerate() {
            insert_entity_relation(&tx, &entity.id, relation, idx)?;
        }
    }

    // Reference documents.
    for doc in &project.references {
        insert_reference_document(&tx, doc)?;
    }

    tx.commit().map_err(|e| format!("commit error: {e}"))?;

    tracing::debug!("saved project to {}", path.display());
    Ok(())
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
        .map(|bt| serde_json::to_string(bt))
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

fn insert_entity(conn: &Connection, entity: &Entity) -> Result<(), String> {
    let details_json = serde_json::to_string(&entity.details)
        .map_err(|e| format!("serialize entity details: {e}"))?;
    conn.execute(
        "INSERT INTO entities (id, category, name, tagline, description,
                               details_json, color_r, color_g, color_b, locked)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            entity.id.0.to_string(),
            entity.category.to_string(),
            entity.name,
            entity.tagline,
            entity.description,
            details_json,
            entity.color.r,
            entity.color.g,
            entity.color.b,
            entity.locked as i32,
        ],
    )
    .map_err(|e| format!("insert entity: {e}"))?;
    Ok(())
}

fn insert_entity_snapshot(
    conn: &Connection,
    entity_id: &EntityId,
    snapshot: &EntitySnapshot,
) -> Result<(), String> {
    let overrides_json = snapshot
        .state_overrides
        .as_ref()
        .map(|o| serde_json::to_string(o))
        .transpose()
        .map_err(|e| format!("serialize overrides: {e}"))?;
    let source_node = snapshot.source_node_id.map(|id| id.0.to_string());

    conn.execute(
        "INSERT INTO entity_snapshots (entity_id, at_ms, source_node_id, description, overrides_json)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            entity_id.0.to_string(),
            snapshot.at_ms as i64,
            source_node,
            snapshot.description,
            overrides_json,
        ],
    )
    .map_err(|e| format!("insert entity_snapshot: {e}"))?;
    Ok(())
}

fn insert_entity_node_ref(
    conn: &Connection,
    entity_id: &EntityId,
    node_id: &NodeId,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO entity_node_refs (entity_id, node_id) VALUES (?1, ?2)",
        params![entity_id.0.to_string(), node_id.0.to_string()],
    )
    .map_err(|e| format!("insert entity_node_ref: {e}"))?;
    Ok(())
}

fn insert_entity_relation(
    conn: &Connection,
    entity_id: &EntityId,
    relation: &EntityRelation,
    sort_order: usize,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO entity_relations (entity_id, target_entity_id, label, sort_order)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            entity_id.0.to_string(),
            relation.target_entity_id.0.to_string(),
            relation.label,
            sort_order as i32,
        ],
    )
    .map_err(|e| format!("insert entity_relation: {e}"))?;
    Ok(())
}

fn insert_reference_document(conn: &Connection, doc: &ReferenceDocument) -> Result<(), String> {
    let doc_type_json =
        serde_json::to_string(&doc.doc_type).map_err(|e| format!("serialize doc_type: {e}"))?;
    conn.execute(
        "INSERT INTO reference_documents (id, name, content, doc_type) VALUES (?1, ?2, ?3, ?4)",
        params![
            doc.id.0.to_string(),
            doc.name,
            doc.content,
            doc_type_json,
        ],
    )
    .map_err(|e| format!("insert reference_document: {e}"))?;
    Ok(())
}

// ─── Load ──────────────────────────────────────────────────────────

/// Load a project from disk. Handles both SQLite (.db) and legacy JSON (.json) files.
/// If only a .json sibling exists for a .db path, auto-migrates to SQLite.
pub async fn load_project(path: &Path) -> Result<Project, String> {
    let path = path.to_path_buf();

    // Legacy JSON path.
    if path.extension().map_or(false, |ext| ext == "json") {
        return load_project_json(&path).await;
    }

    // If .db doesn't exist, check for a .json sibling and auto-migrate.
    if !path.exists() {
        let json_sibling = path.with_file_name("project.json");
        if json_sibling.exists() {
            tracing::info!(
                "SQLite file not found, loading from JSON sibling: {}",
                json_sibling.display()
            );
            let project = load_project_json(&json_sibling).await?;
            save_project(&project, &path).await?;
            tracing::info!("migrated project from JSON to SQLite: {}", path.display());
            return Ok(project);
        }
    }

    tokio::task::spawn_blocking(move || load_project_sync(&path))
        .await
        .map_err(|e| format!("spawn_blocking error: {e}"))?
}

fn load_project_sync(path: &Path) -> Result<Project, String> {
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("sqlite open error: {e}"))?;

    // Check schema version to handle migration from v1.
    let version = read_schema_version(&conn);

    if version == 1 {
        drop(conn);
        return load_and_migrate_v1(path);
    }

    // v2 schema — load directly.
    load_project_v2(&conn, path)
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

    // Entities.
    let mut entities = read_entities(conn)?;
    for entity in &mut entities {
        entity.snapshots = read_entity_snapshots(conn, &entity.id)?;
        entity.node_refs = read_entity_node_refs(conn, &entity.id)?;
        entity.relations = read_entity_relations(conn, &entity.id)?;
    }

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
        bible: StoryBible { entities },
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

    let segments = serde_json::from_str(&segments_json)
        .map_err(|e| format!("parse segments: {e}"))?;

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
        let (id_str, parent_id_str, level_str, sort_order, start_ms, end_ms, name, content_json, beat_type_json, locked) =
            row.map_err(|e| format!("read node row: {e}"))?;

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
        let (node_id_str, arc_id_str) =
            row.map_err(|e| format!("read node_arc row: {e}"))?;
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

fn read_entities(conn: &Connection) -> Result<Vec<Entity>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, category, name, tagline, description, details_json,
                    color_r, color_g, color_b, locked
             FROM entities",
        )
        .map_err(|e| format!("prepare entities: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, u8>(6)?,
                row.get::<_, u8>(7)?,
                row.get::<_, u8>(8)?,
                row.get::<_, i32>(9)?,
            ))
        })
        .map_err(|e| format!("query entities: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (id_str, category_str, name, tagline, description, details_json, r, g, b, locked) =
            row.map_err(|e| format!("read entity row: {e}"))?;

        let category: EntityCategory = serde_json::from_str(&format!("\"{}\"", category_str))
            .map_err(|e| format!("parse category '{}': {e}", category_str))?;
        let details: EntityDetails =
            serde_json::from_str(&details_json).map_err(|e| format!("parse entity details: {e}"))?;

        result.push(Entity {
            id: EntityId(parse_uuid(&id_str)?),
            category,
            name,
            tagline,
            description,
            details,
            snapshots: Vec::new(),
            node_refs: Vec::new(),
            relations: Vec::new(),
            color: Color::new(r, g, b),
            locked: locked != 0,
        });
    }
    Ok(result)
}

fn read_entity_snapshots(
    conn: &Connection,
    entity_id: &EntityId,
) -> Result<Vec<EntitySnapshot>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT at_ms, source_node_id, description, overrides_json
             FROM entity_snapshots WHERE entity_id = ?1 ORDER BY at_ms",
        )
        .map_err(|e| format!("prepare entity_snapshots: {e}"))?;

    let rows = stmt
        .query_map(params![entity_id.0.to_string()], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(|e| format!("query entity_snapshots: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (at_ms, source_node_str, description, overrides_json) =
            row.map_err(|e| format!("read snapshot row: {e}"))?;

        let source_node_id = source_node_str
            .map(|s| parse_uuid(&s).map(NodeId))
            .transpose()?;
        let state_overrides: Option<SnapshotOverrides> = overrides_json
            .map(|j| serde_json::from_str(&j))
            .transpose()
            .map_err(|e| format!("parse overrides: {e}"))?;

        result.push(EntitySnapshot {
            at_ms: at_ms as u64,
            source_node_id,
            description,
            state_overrides,
        });
    }
    Ok(result)
}

fn read_entity_node_refs(conn: &Connection, entity_id: &EntityId) -> Result<Vec<NodeId>, String> {
    let mut stmt = conn
        .prepare("SELECT node_id FROM entity_node_refs WHERE entity_id = ?1")
        .map_err(|e| format!("prepare entity_node_refs: {e}"))?;

    let rows = stmt
        .query_map(params![entity_id.0.to_string()], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|e| format!("query entity_node_refs: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let id_str = row.map_err(|e| format!("read node_ref row: {e}"))?;
        result.push(NodeId(parse_uuid(&id_str)?));
    }
    Ok(result)
}

fn read_entity_relations(
    conn: &Connection,
    entity_id: &EntityId,
) -> Result<Vec<EntityRelation>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT target_entity_id, label FROM entity_relations
             WHERE entity_id = ?1 ORDER BY sort_order",
        )
        .map_err(|e| format!("prepare entity_relations: {e}"))?;

    let rows = stmt
        .query_map(params![entity_id.0.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("query entity_relations: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (target_str, label) = row.map_err(|e| format!("read relation row: {e}"))?;
        result.push(EntityRelation {
            target_entity_id: EntityId(parse_uuid(&target_str)?),
            label,
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
        let doc_type: ReferenceType = serde_json::from_str(&doc_type_json)
            .map_err(|e| format!("parse doc_type: {e}"))?;
        result.push(ReferenceDocument {
            id: eidetic_core::reference::ReferenceId(parse_uuid(&id_str)?),
            name,
            content,
            doc_type,
        });
    }
    Ok(result)
}

// ─── V1 → V2 Migration ────────────────────────────────────────────

/// Load a v1 database, migrate the data structures, and save as v2.
fn load_and_migrate_v1(path: &Path) -> Result<Project, String> {
    let conn = Connection::open(path).map_err(|e| format!("sqlite open error: {e}"))?;

    // Read v1 project metadata.
    let (name, total_duration_ms): (String, i64) = conn
        .query_row(
            "SELECT name, total_duration_ms FROM project WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("read v1 project: {e}"))?;

    let structure = read_episode_structure(&conn)?;

    // Read v1 arcs (no parent_arc_id in v1).
    let arcs = read_v1_arcs(&conn)?;

    // Build the arc_id-to-arc map for tagging.
    let arc_map: std::collections::HashMap<Uuid, ArcId> = arcs
        .iter()
        .map(|a| (a.id.0, a.id))
        .collect();

    // Read v1 tracks and clips.
    let mut nodes = Vec::new();
    let mut node_arcs = Vec::new();

    let v1_tracks = read_v1_tracks(&conn)?;
    for (track_id_uuid, arc_id_uuid) in &v1_tracks {
        let arc_id = arc_map.get(arc_id_uuid).copied();

        // Read main clips (is_sub_beat = 0) → Scene level.
        let v1_clips = read_v1_clips(&conn, track_id_uuid, false)?;
        for clip in v1_clips {
            if let Some(aid) = arc_id {
                node_arcs.push(NodeArc {
                    node_id: clip.id,
                    arc_id: aid,
                });
            }
            nodes.push(clip);
        }

        // Read sub-beats (is_sub_beat = 1) → Beat level.
        let v1_sub_beats = read_v1_clips(&conn, track_id_uuid, true)?;
        for beat in v1_sub_beats {
            if let Some(aid) = arc_id {
                node_arcs.push(NodeArc {
                    node_id: beat.id,
                    arc_id: aid,
                });
            }
            nodes.push(beat);
        }
    }

    // Relationships.
    let relationships = read_v1_relationships(&conn)?;

    // Read entities with v1 column names.
    let mut entities = read_entities(&conn)?;
    for entity in &mut entities {
        entity.snapshots = read_v1_entity_snapshots(&conn, &entity.id)?;
        entity.node_refs = read_v1_entity_node_refs(&conn, &entity.id)?;
        entity.relations = read_entity_relations(&conn, &entity.id)?;
    }

    let references = read_reference_documents(&conn)?;

    // Create v2 tracks (4 level-based tracks).
    let tracks = Track::default_set();

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
        premise: String::new(),
        timeline,
        arcs,
        bible: StoryBible { entities },
        references,
    };

    drop(conn);

    // Rewrite the database as v2.
    save_project_sync(&project, path)?;

    tracing::info!("migrated project from v1 to v2: {}", path.display());
    Ok(project)
}

fn read_v1_arcs(conn: &Connection) -> Result<Vec<StoryArc>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, description, arc_type, color_r, color_g, color_b FROM arcs")
        .map_err(|e| format!("prepare v1 arcs: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, u8>(4)?,
                row.get::<_, u8>(5)?,
                row.get::<_, u8>(6)?,
            ))
        })
        .map_err(|e| format!("query v1 arcs: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (id_str, name, description, arc_type_json, r, g, b) =
            row.map_err(|e| format!("read v1 arc row: {e}"))?;
        let id = ArcId(parse_uuid(&id_str)?);
        let arc_type: ArcType =
            serde_json::from_str(&arc_type_json).map_err(|e| format!("parse arc_type: {e}"))?;
        result.push(StoryArc {
            id,
            parent_arc_id: None,
            name,
            description,
            arc_type,
            color: Color::new(r, g, b),
        });
    }
    Ok(result)
}

fn read_v1_tracks(conn: &Connection) -> Result<Vec<(Uuid, Uuid)>, String> {
    let mut stmt = conn
        .prepare("SELECT id, arc_id FROM tracks ORDER BY sort_order")
        .map_err(|e| format!("prepare v1 tracks: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("query v1 tracks: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (id_str, arc_id_str) = row.map_err(|e| format!("read v1 track row: {e}"))?;
        result.push((parse_uuid(&id_str)?, parse_uuid(&arc_id_str)?));
    }
    Ok(result)
}

fn read_v1_clips(conn: &Connection, track_id: &Uuid, is_sub_beat: bool) -> Result<Vec<StoryNode>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, parent_clip_id, start_ms, end_ms, beat_type, name, content_json, locked
             FROM clips WHERE track_id = ?1 AND is_sub_beat = ?2
             ORDER BY start_ms",
        )
        .map_err(|e| format!("prepare v1 clips: {e}"))?;

    let level = if is_sub_beat {
        StoryLevel::Beat
    } else {
        StoryLevel::Scene
    };

    let rows = stmt
        .query_map(params![track_id.to_string(), is_sub_beat as i32], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, i32>(7)?,
            ))
        })
        .map_err(|e| format!("query v1 clips: {e}"))?;

    let mut result = Vec::new();
    for (i, row) in rows.enumerate() {
        let (id_str, parent_id_str, start_ms, end_ms, beat_type_json, name, content_json, locked) =
            row.map_err(|e| format!("read v1 clip row: {e}"))?;

        let parent_id = parent_id_str
            .map(|s| parse_uuid(&s).map(NodeId))
            .transpose()?;

        // Parse v1 BeatContent into NodeContent.
        let content = parse_v1_content(&content_json)?;

        let beat_type: Option<BeatType> = serde_json::from_str(&beat_type_json).ok();

        result.push(StoryNode {
            id: NodeId(parse_uuid(&id_str)?),
            parent_id,
            level,
            sort_order: i as u32,
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

/// Parse v1 BeatContent JSON into NodeContent.
///
/// NodeContent's custom Deserialize handles legacy v2 fields (`generated_text`,
/// `user_refined_text`) and old ContentStatus variants via serde aliases. We
/// try that first, then fall back to v1 field names (`beat_notes`, etc.).
fn parse_v1_content(json: &str) -> Result<NodeContent, String> {
    // Try parsing as current or v2 NodeContent first (custom Deserialize
    // handles legacy fields and status variants automatically).
    if let Ok(content) = serde_json::from_str::<NodeContent>(json) {
        return Ok(content);
    }

    // Fall back to v1 BeatContent (different field names entirely).
    #[derive(serde::Deserialize)]
    struct V1Content {
        #[serde(default)]
        beat_notes: String,
        #[serde(default)]
        generated_script: Option<String>,
        #[serde(default)]
        user_refined_script: Option<String>,
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        scene_recap: Option<String>,
    }

    let v1: V1Content =
        serde_json::from_str(json).map_err(|e| format!("parse v1 content: {e}"))?;

    use eidetic_core::timeline::node::ContentStatus;
    let status = match v1.status.as_deref() {
        Some("\"NotesOnly\"") | Some("NotesOnly") => ContentStatus::NotesOnly,
        Some("\"Generating\"") | Some("Generating") => ContentStatus::Generating,
        Some("\"Generated\"") | Some("Generated")
        | Some("\"UserRefined\"") | Some("UserRefined")
        | Some("\"UserWritten\"") | Some("UserWritten") => ContentStatus::HasContent,
        _ => ContentStatus::Empty,
    };

    // Collapse: user_refined > generated > empty.
    let content = v1.user_refined_script
        .or(v1.generated_script)
        .unwrap_or_default();

    Ok(NodeContent {
        notes: v1.beat_notes,
        content,
        status,
        scene_recap: v1.scene_recap,
    })
}

fn read_v1_relationships(conn: &Connection) -> Result<Vec<Relationship>, String> {
    let mut stmt = conn
        .prepare("SELECT id, from_clip_id, to_clip_id, relationship_type FROM relationships")
        .map_err(|e| format!("prepare v1 relationships: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| format!("query v1 relationships: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (id_str, from_str, to_str, rel_type_json) =
            row.map_err(|e| format!("read v1 relationship row: {e}"))?;
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

fn read_v1_entity_snapshots(
    conn: &Connection,
    entity_id: &EntityId,
) -> Result<Vec<EntitySnapshot>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT at_ms, source_clip_id, description, overrides_json
             FROM entity_snapshots WHERE entity_id = ?1 ORDER BY at_ms",
        )
        .map_err(|e| format!("prepare v1 entity_snapshots: {e}"))?;

    let rows = stmt
        .query_map(params![entity_id.0.to_string()], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(|e| format!("query v1 entity_snapshots: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (at_ms, source_clip_str, description, overrides_json) =
            row.map_err(|e| format!("read v1 snapshot row: {e}"))?;

        let source_node_id = source_clip_str
            .map(|s| parse_uuid(&s).map(NodeId))
            .transpose()?;
        let state_overrides: Option<SnapshotOverrides> = overrides_json
            .map(|j| serde_json::from_str(&j))
            .transpose()
            .map_err(|e| format!("parse overrides: {e}"))?;

        result.push(EntitySnapshot {
            at_ms: at_ms as u64,
            source_node_id,
            description,
            state_overrides,
        });
    }
    Ok(result)
}

fn read_v1_entity_node_refs(conn: &Connection, entity_id: &EntityId) -> Result<Vec<NodeId>, String> {
    let mut stmt = conn
        .prepare("SELECT clip_id FROM entity_clip_refs WHERE entity_id = ?1")
        .map_err(|e| format!("prepare v1 entity_clip_refs: {e}"))?;

    let rows = stmt
        .query_map(params![entity_id.0.to_string()], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|e| format!("query v1 entity_clip_refs: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let id_str = row.map_err(|e| format!("read v1 clip_ref row: {e}"))?;
        result.push(NodeId(parse_uuid(&id_str)?));
    }
    Ok(result)
}

// ─── Legacy JSON Support ───────────────────────────────────────────

/// Load a project from a JSON file (legacy format).
async fn load_project_json(path: &Path) -> Result<Project, String> {
    let data = fs::read_to_string(path)
        .await
        .map_err(|e| format!("read error: {e}"))?;

    let mut project: Project =
        serde_json::from_str(&data).map_err(|e| format!("deserialize error: {e}"))?;

    // Migrate v1 characters into bible entities if bible is empty and JSON had characters.
    if project.bible.entities.is_empty() {
        if let Ok(raw) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(chars) = raw.get("characters").and_then(|v| v.as_array()) {
                migrate_v1_characters(&mut project, chars);
                if !project.bible.entities.is_empty() {
                    tracing::info!(
                        "migrated {} v1 characters to bible entities",
                        project.bible.entities.len()
                    );
                }
            }
        }
    }

    tracing::debug!("loaded project from JSON: {}", path.display());
    Ok(project)
}

/// Convert v1 Character objects into Entity objects in the story bible.
fn migrate_v1_characters(project: &mut Project, chars: &[serde_json::Value]) {
    use eidetic_core::story::arc::Color;
    use eidetic_core::story::bible::{Entity, EntityCategory, EntityDetails};

    for ch in chars {
        let name = ch
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let description = ch
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let voice_notes = ch
            .get("voice_notes")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let color = ch
            .get("color")
            .and_then(|v| {
                let r = v.get("r").and_then(|x| x.as_u64())? as u8;
                let g = v.get("g").and_then(|x| x.as_u64())? as u8;
                let b = v.get("b").and_then(|x| x.as_u64())? as u8;
                Some(Color::new(r, g, b))
            })
            .unwrap_or(Color::new(200, 200, 200));

        let mut entity = Entity::new(name, EntityCategory::Character, color);
        entity.description = description.clone();
        entity.tagline = if description.len() > 100 {
            format!("{}...", &description[..97])
        } else {
            description
        };
        entity.details = EntityDetails::Character {
            traits: Vec::new(),
            voice_notes,
            character_relations: Vec::new(),
            audience_knowledge: String::new(),
        };

        project.bible.add_entity(entity);
    }
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

        let project_file = if path.join("project.db").exists() {
            path.join("project.db")
        } else if path.join("project.json").exists() {
            path.join("project.json")
        } else {
            continue;
        };

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

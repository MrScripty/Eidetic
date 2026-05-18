use rusqlite::Connection;

use crate::history_store::{self, HistoryStoreError};

const BIBLE_GRAPH_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS bible_graph_nodes (
    id               TEXT PRIMARY KEY CHECK (id <> ''),
    parent_id        TEXT,
    schema_key       TEXT NOT NULL CHECK (schema_key <> ''),
    name             TEXT NOT NULL CHECK (name <> ''),
    system_owned     INTEGER NOT NULL CHECK (system_owned IN (0, 1)),
    sort_order       INTEGER NOT NULL,
    created_event_id TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id TEXT REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_bible_graph_nodes_parent
    ON bible_graph_nodes(parent_id, sort_order);
CREATE INDEX IF NOT EXISTS idx_bible_graph_nodes_schema
    ON bible_graph_nodes(schema_key);

CREATE TABLE IF NOT EXISTS bible_graph_parts (
    id               TEXT PRIMARY KEY CHECK (id <> ''),
    node_id          TEXT NOT NULL REFERENCES bible_graph_nodes(id),
    part_key         TEXT NOT NULL CHECK (part_key <> ''),
    name             TEXT NOT NULL CHECK (name <> ''),
    system_owned     INTEGER NOT NULL CHECK (system_owned IN (0, 1)),
    sort_order       INTEGER NOT NULL,
    created_event_id TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id TEXT REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_bible_graph_parts_node
    ON bible_graph_parts(node_id, sort_order, name);

CREATE TABLE IF NOT EXISTS bible_graph_fields (
    id               TEXT PRIMARY KEY CHECK (id <> ''),
    part_id          TEXT NOT NULL REFERENCES bible_graph_parts(id),
    field_key        TEXT NOT NULL CHECK (field_key <> ''),
    value_type       TEXT,
    text_value       TEXT,
    integer_value    INTEGER,
    number_value     REAL,
    bool_value       INTEGER CHECK (bool_value IS NULL OR bool_value IN (0, 1)),
    ref_kind         TEXT,
    ref_id           TEXT,
    asset_ref        TEXT,
    sort_order       INTEGER NOT NULL,
    updated_event_id TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id TEXT REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_bible_graph_fields_part
    ON bible_graph_fields(part_id, sort_order, field_key);

CREATE TABLE IF NOT EXISTS bible_graph_edges (
    id               TEXT PRIMARY KEY CHECK (id <> ''),
    from_node_id     TEXT NOT NULL REFERENCES bible_graph_nodes(id),
    to_node_id       TEXT NOT NULL REFERENCES bible_graph_nodes(id),
    edge_kind        TEXT NOT NULL CHECK (edge_kind <> ''),
    custom_kind      TEXT,
    label            TEXT NOT NULL CHECK (label <> ''),
    directed         INTEGER NOT NULL CHECK (directed IN (0, 1)),
    sort_order       INTEGER NOT NULL,
    created_event_id TEXT NOT NULL REFERENCES change_events(id),
    updated_event_id TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id TEXT REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_bible_graph_edges_from
    ON bible_graph_edges(from_node_id, sort_order, id);
CREATE INDEX IF NOT EXISTS idx_bible_graph_edges_to
    ON bible_graph_edges(to_node_id, sort_order, id);
"#;

pub(crate) fn create_schema(conn: &Connection) -> Result<(), HistoryStoreError> {
    history_store::create_schema(conn)?;
    conn.execute_batch(BIBLE_GRAPH_SCHEMA_SQL)?;
    Ok(())
}

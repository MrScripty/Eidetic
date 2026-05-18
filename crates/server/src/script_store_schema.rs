use rusqlite::Connection;

use crate::history_store::{self, HistoryStoreError};

const SCRIPT_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS script_documents (
    id               TEXT PRIMARY KEY CHECK (id <> ''),
    title            TEXT NOT NULL CHECK (title <> ''),
    sort_order       INTEGER NOT NULL,
    created_event_id TEXT NOT NULL REFERENCES change_events(id),
    updated_event_id TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id TEXT REFERENCES change_events(id)
);

CREATE TABLE IF NOT EXISTS script_segments (
    id               TEXT PRIMARY KEY CHECK (id <> ''),
    document_id      TEXT NOT NULL REFERENCES script_documents(id),
    source_node_id   TEXT,
    start_ms         INTEGER NOT NULL,
    end_ms           INTEGER NOT NULL,
    status           TEXT NOT NULL CHECK (status <> ''),
    sort_order       INTEGER NOT NULL,
    created_event_id TEXT NOT NULL REFERENCES change_events(id),
    updated_event_id TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id TEXT REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_script_segments_document
    ON script_segments(document_id, sort_order, start_ms, id);

CREATE TABLE IF NOT EXISTS script_blocks (
    id               TEXT PRIMARY KEY CHECK (id <> ''),
    segment_id       TEXT NOT NULL REFERENCES script_segments(id),
    block_kind       TEXT NOT NULL CHECK (block_kind <> ''),
    text             TEXT NOT NULL,
    sort_order       INTEGER NOT NULL,
    updated_event_id TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id TEXT REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_script_blocks_segment
    ON script_blocks(segment_id, sort_order, id);

CREATE TABLE IF NOT EXISTS script_spans (
    id               TEXT PRIMARY KEY CHECK (id <> ''),
    block_id         TEXT NOT NULL REFERENCES script_blocks(id),
    start_byte       INTEGER NOT NULL,
    end_byte         INTEGER NOT NULL,
    provenance       TEXT NOT NULL CHECK (provenance <> ''),
    updated_event_id TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id TEXT REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_script_spans_block
    ON script_spans(block_id, start_byte, end_byte, id);

CREATE TABLE IF NOT EXISTS script_locks (
    id               TEXT PRIMARY KEY CHECK (id <> ''),
    span_id          TEXT NOT NULL REFERENCES script_spans(id),
    reason           TEXT NOT NULL CHECK (reason <> ''),
    created_event_id TEXT NOT NULL REFERENCES change_events(id),
    updated_event_id TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id TEXT REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_script_locks_span
    ON script_locks(span_id, id);
"#;

pub(crate) fn create_schema(conn: &Connection) -> Result<(), HistoryStoreError> {
    history_store::create_schema(conn)?;
    conn.execute_batch(SCRIPT_SCHEMA_SQL)?;
    Ok(())
}

use eidetic_core::contracts::{
    ChangeEventId, ObjectKind, ProjectionEnvelope, ProjectionVersion, SetStoryArcMetadataCommand,
    StoryArcListProjection,
};
use eidetic_core::story::arc::{ArcId, ArcType, Color, StoryArc};
use rusqlite::{Connection, OptionalExtension, Row, Transaction};
use uuid::Uuid;

use crate::history_store::{self, HistoryStoreError};

const STORY_ARC_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS arcs (
    id            TEXT PRIMARY KEY CHECK (id <> ''),
    parent_arc_id TEXT,
    name          TEXT NOT NULL CHECK (name <> ''),
    description   TEXT NOT NULL DEFAULT '',
    arc_type      TEXT NOT NULL CHECK (arc_type <> ''),
    color_r       INTEGER NOT NULL,
    color_g       INTEGER NOT NULL,
    color_b       INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_arcs_parent
    ON arcs(parent_arc_id, name, id);
"#;

pub(crate) fn create_schema(conn: &Connection) -> Result<(), HistoryStoreError> {
    history_store::create_schema(conn)?;
    conn.execute_batch(STORY_ARC_SCHEMA_SQL)?;
    Ok(())
}

pub(crate) fn insert_arc_in_transaction(
    tx: &Transaction<'_>,
    arc: &StoryArc,
    _event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    if arc_exists_in_transaction(tx, &arc.id)? {
        return Err(HistoryStoreError::InvalidValue(format!(
            "story arc already exists: {}",
            arc.id.0
        )));
    }
    tx.execute(
        "INSERT INTO arcs (
            id, parent_arc_id, name, description, arc_type, color_r, color_g, color_b
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        story_arc_params(arc)?,
    )?;
    Ok(())
}

pub(crate) fn update_arc_metadata_in_transaction(
    tx: &Transaction<'_>,
    command: &SetStoryArcMetadataCommand,
    _event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let Some(mut arc) = load_arc_in_transaction(tx, &command.arc_id)? else {
        return Err(HistoryStoreError::InvalidValue(format!(
            "story arc not found: {}",
            command.arc_id.0
        )));
    };

    if let Some(name) = &command.name {
        arc.name = name.clone();
    }
    if let Some(description) = &command.description {
        arc.description = description.clone();
    }
    if let Some(arc_type) = &command.arc_type {
        arc.arc_type = arc_type.clone();
    }
    if let Some(color) = command.color {
        arc.color = color;
    }

    tx.execute(
        "UPDATE arcs
         SET parent_arc_id = ?2,
             name = ?3,
             description = ?4,
             arc_type = ?5,
             color_r = ?6,
             color_g = ?7,
             color_b = ?8
         WHERE id = ?1",
        story_arc_params(&arc)?,
    )?;
    Ok(())
}

pub(crate) fn delete_arc_in_transaction(
    tx: &Transaction<'_>,
    arc_id: &ArcId,
    _event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    tx.execute("DELETE FROM arcs WHERE id = ?1", [arc_id.0.to_string()])?;
    Ok(())
}

pub(crate) fn load_arcs(conn: &Connection) -> Result<Vec<StoryArc>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT id, parent_arc_id, name, description, arc_type, color_r, color_g, color_b
         FROM arcs
         ORDER BY name ASC, id ASC",
    )?;
    let rows = statement.query_map([], row_to_arc)?;

    let mut arcs = Vec::new();
    for row in rows {
        arcs.push(row?);
    }
    Ok(arcs)
}

pub(crate) fn load_arc(
    conn: &Connection,
    arc_id: &ArcId,
) -> Result<Option<StoryArc>, HistoryStoreError> {
    conn.query_row(
        "SELECT id, parent_arc_id, name, description, arc_type, color_r, color_g, color_b
         FROM arcs
         WHERE id = ?1",
        [arc_id.0.to_string()],
        row_to_arc,
    )
    .optional()
    .map_err(Into::into)
}

pub(crate) fn load_arc_list_projection_envelope(
    conn: &Connection,
) -> Result<ProjectionEnvelope<StoryArcListProjection>, HistoryStoreError> {
    let projection = StoryArcListProjection::from_arcs(&load_arcs(conn)?);
    let summary = history_store::load_revision_summary_for_kind(conn, ObjectKind::StoryArc)?;

    match summary.latest_change_event_id {
        Some(change_event_id) => Ok(ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            change_event_id,
            projection,
        )),
        None => Ok(ProjectionEnvelope::initial(projection)),
    }
}

fn arc_exists_in_transaction(
    tx: &Transaction<'_>,
    arc_id: &ArcId,
) -> Result<bool, HistoryStoreError> {
    tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM arcs WHERE id = ?1)",
        [arc_id.0.to_string()],
        |row| row.get::<_, bool>(0),
    )
    .map_err(Into::into)
}

fn load_arc_in_transaction(
    tx: &Transaction<'_>,
    arc_id: &ArcId,
) -> Result<Option<StoryArc>, HistoryStoreError> {
    tx.query_row(
        "SELECT id, parent_arc_id, name, description, arc_type, color_r, color_g, color_b
         FROM arcs
         WHERE id = ?1",
        [arc_id.0.to_string()],
        row_to_arc,
    )
    .optional()
    .map_err(Into::into)
}

fn story_arc_params(arc: &StoryArc) -> Result<[rusqlite::types::Value; 8], HistoryStoreError> {
    Ok([
        rusqlite::types::Value::Text(arc.id.0.to_string()),
        arc.parent_arc_id
            .map(|arc_id| rusqlite::types::Value::Text(arc_id.0.to_string()))
            .unwrap_or(rusqlite::types::Value::Null),
        rusqlite::types::Value::Text(arc.name.clone()),
        rusqlite::types::Value::Text(arc.description.clone()),
        rusqlite::types::Value::Text(encode_arc_type(&arc.arc_type)?),
        rusqlite::types::Value::Integer(i64::from(arc.color.r)),
        rusqlite::types::Value::Integer(i64::from(arc.color.g)),
        rusqlite::types::Value::Integer(i64::from(arc.color.b)),
    ])
}

fn row_to_arc(row: &Row<'_>) -> rusqlite::Result<StoryArc> {
    let id = row_to_arc_id(row.get::<_, String>(0)?, 0)?;
    let parent_arc_id = row
        .get::<_, Option<String>>(1)?
        .map(|value| row_to_arc_id(value, 1))
        .transpose()?;
    let arc_type = decode_arc_type(row.get::<_, String>(4)?, 4)?;
    Ok(StoryArc {
        id,
        parent_arc_id,
        name: row.get(2)?,
        description: row.get(3)?,
        arc_type,
        color: Color::new(row.get(5)?, row.get(6)?, row.get(7)?),
    })
}

fn row_to_arc_id(value: String, column: usize) -> rusqlite::Result<ArcId> {
    Uuid::parse_str(&value).map(ArcId).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            column,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

fn decode_arc_type(value: String, column: usize) -> rusqlite::Result<ArcType> {
    serde_json::from_str(&value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            column,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

fn encode_arc_type(arc_type: &ArcType) -> Result<String, HistoryStoreError> {
    serde_json::to_string(arc_type).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::story::arc::{ArcType, Color};

    #[test]
    fn stores_and_projects_story_arcs() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let tx = conn.transaction().unwrap();
        let arc = StoryArc::new("Mystery", ArcType::APlot, Color::A_PLOT);

        insert_arc_in_transaction(&tx, &arc, ChangeEventId::new()).unwrap();
        tx.commit().unwrap();

        let projection = load_arc_list_projection_envelope(&conn).unwrap();
        assert_eq!(projection.payload.arcs.len(), 1);
        assert_eq!(projection.payload.arcs[0].name, "Mystery");
    }

    #[test]
    fn updates_and_deletes_story_arc_rows() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let arc = StoryArc::new("Mystery", ArcType::APlot, Color::A_PLOT);
        let tx = conn.transaction().unwrap();
        insert_arc_in_transaction(&tx, &arc, ChangeEventId::new()).unwrap();
        tx.commit().unwrap();

        let tx = conn.transaction().unwrap();
        update_arc_metadata_in_transaction(
            &tx,
            &SetStoryArcMetadataCommand {
                arc_id: arc.id,
                name: Some("Renamed".to_string()),
                description: None,
                arc_type: None,
                color: Some(Color::new(1, 2, 3)),
            },
            ChangeEventId::new(),
        )
        .unwrap();
        tx.commit().unwrap();

        let arcs = load_arcs(&conn).unwrap();
        assert_eq!(arcs[0].name, "Renamed");
        assert_eq!(arcs[0].color, Color::new(1, 2, 3));

        let tx = conn.transaction().unwrap();
        delete_arc_in_transaction(&tx, &arc.id, ChangeEventId::new()).unwrap();
        tx.commit().unwrap();

        assert!(load_arcs(&conn).unwrap().is_empty());
    }
}

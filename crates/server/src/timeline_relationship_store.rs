use eidetic_core::timeline::node::NodeId;
use eidetic_core::timeline::relationship::Relationship;
use eidetic_core::timeline::relationship::RelationshipId;
use eidetic_core::timeline::relationship::RelationshipType;
use rusqlite::{Connection, Transaction, params};
use uuid::Uuid;

use crate::history_store::HistoryStoreError;

const TIMELINE_RELATIONSHIP_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS relationships (
    id                TEXT PRIMARY KEY,
    from_node_id      TEXT NOT NULL,
    to_node_id        TEXT NOT NULL,
    relationship_type TEXT NOT NULL
);
"#;

pub(crate) fn upsert_relationship_in_transaction(
    tx: &Transaction<'_>,
    relationship: &Relationship,
) -> Result<(), HistoryStoreError> {
    tx.execute_batch(TIMELINE_RELATIONSHIP_SCHEMA_SQL)?;
    upsert_relationship(tx, relationship)
}

pub(crate) fn upsert_relationships_in_transaction(
    tx: &Transaction<'_>,
    relationships: &[Relationship],
) -> Result<(), HistoryStoreError> {
    tx.execute_batch(TIMELINE_RELATIONSHIP_SCHEMA_SQL)?;

    for relationship in relationships {
        upsert_relationship(tx, relationship)?;
    }

    Ok(())
}

fn upsert_relationship(
    tx: &Transaction<'_>,
    relationship: &Relationship,
) -> Result<(), HistoryStoreError> {
    let relationship_type = serde_json::to_string(&relationship.relationship_type)?;
    tx.execute(
        "INSERT INTO relationships (id, from_node_id, to_node_id, relationship_type)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET
             from_node_id = excluded.from_node_id,
             to_node_id = excluded.to_node_id,
             relationship_type = excluded.relationship_type",
        params![
            relationship.id.0.to_string(),
            relationship.from_node.0.to_string(),
            relationship.to_node.0.to_string(),
            relationship_type,
        ],
    )?;

    Ok(())
}

pub(crate) fn delete_relationship_in_transaction(
    tx: &Transaction<'_>,
    relationship_id: RelationshipId,
) -> Result<(), HistoryStoreError> {
    tx.execute_batch(TIMELINE_RELATIONSHIP_SCHEMA_SQL)?;
    tx.execute(
        "DELETE FROM relationships WHERE id = ?1",
        [relationship_id.0.to_string()],
    )?;
    Ok(())
}

pub(crate) fn delete_relationships_in_transaction(
    tx: &Transaction<'_>,
    relationship_ids: &[RelationshipId],
) -> Result<(), HistoryStoreError> {
    tx.execute_batch(TIMELINE_RELATIONSHIP_SCHEMA_SQL)?;

    for relationship_id in relationship_ids {
        tx.execute(
            "DELETE FROM relationships WHERE id = ?1",
            [relationship_id.0.to_string()],
        )?;
    }

    Ok(())
}

pub(crate) fn load_relationships(
    conn: &Connection,
) -> Result<Vec<Relationship>, HistoryStoreError> {
    conn.execute_batch(TIMELINE_RELATIONSHIP_SCHEMA_SQL)?;
    let mut stmt =
        conn.prepare("SELECT id, from_node_id, to_node_id, relationship_type FROM relationships")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    let mut relationships = Vec::new();
    for row in rows {
        let (id, from_node, to_node, relationship_type_json) = row?;
        relationships.push(Relationship {
            id: RelationshipId(parse_uuid(&id)?),
            from_node: NodeId(parse_uuid(&from_node)?),
            to_node: NodeId(parse_uuid(&to_node)?),
            relationship_type: serde_json::from_str::<RelationshipType>(&relationship_type_json)?,
        });
    }

    Ok(relationships)
}

fn parse_uuid(value: &str) -> Result<Uuid, HistoryStoreError> {
    Uuid::parse_str(value).map_err(|error| HistoryStoreError::InvalidId(error.to_string()))
}

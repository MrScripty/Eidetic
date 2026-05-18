use eidetic_core::timeline::relationship::Relationship;
use eidetic_core::timeline::relationship::RelationshipId;
use rusqlite::{Transaction, params};

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

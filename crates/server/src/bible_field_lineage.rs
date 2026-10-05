//! Revision custody for untimed fields actually supplied by the Bible resolver.
use eidetic_core::contracts::*;
use rusqlite::{Connection, OptionalExtension, params};

use crate::history_store::HistoryStoreError;

/// Caller owns a read transaction covering both context and revision capture.
/// Only baseline fields surviving temporal resolution participate. An effective
/// timed override is not evidence that the baseline value was consumed.
pub(crate) fn capture(
    conn: &Connection,
    context: &AiBibleContextProjection,
) -> Result<Vec<BibleFieldInput>, HistoryStoreError> {
    let mut inputs = Vec::new();
    for node in &context.nodes {
        let detail = crate::bible_graph_store::load_node_detail_projection(conn, &node.node_id)?
            .ok_or_else(invalid)?;
        for field in &node.fields {
            let stored = detail
                .parts
                .iter()
                .filter(|part| part.part.part_key == field.part_key)
                .flat_map(|part| &part.fields)
                .find(|stored| stored.field_key == field.field_key)
                .ok_or_else(invalid)?;
            if stored.value.as_ref() != Some(&field.value) {
                return Err(invalid());
            }
            let revision: String = conn.query_row(
                "SELECT updated_event_id FROM bible_graph_fields WHERE id = ?1 AND deleted_event_id IS NULL",
                [stored.id.as_str()], |row| row.get(0))?;
            inputs.push(BibleFieldInput {
                node_id: node.node_id.clone(),
                part_key: field.part_key.clone(),
                field_key: field.field_key.clone(),
                field_id: stored.id.clone(),
                revision_event_id: parse_event(&revision)?,
                value: field.value.clone(),
            });
        }
    }
    inputs.sort_by(|a, b| a.field_id.as_str().cmp(b.field_id.as_str()));
    Ok(inputs)
}

pub(crate) fn endpoint(input: &BibleFieldInput) -> SemanticDependencyEndpoint {
    SemanticDependencyEndpoint::BibleField {
        node_id: input.node_id.clone(),
        part_key: input.part_key.clone(),
        field_key: input.field_key.clone(),
        field_id: Some(input.field_id.clone()),
    }
}

/// Late output binds historical evidence, even after its source changes/deletes.
pub(crate) fn validate_history(
    conn: &Connection,
    input: &BibleFieldInput,
) -> Result<(), HistoryStoreError> {
    let owner: Option<(String, String, String)> = conn
        .query_row(
            "SELECT p.node_id, p.part_key, f.field_key FROM bible_graph_fields f
         JOIN bible_graph_parts p ON p.id = f.part_id WHERE f.id = ?1",
            [input.field_id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    let history = crate::revision_projection::load_object_field_projection_at_event(
        conn,
        ObjectKind::BiblePartField,
        input.field_id.as_str(),
        input.revision_event_id,
    )?;
    if owner
        != Some((
            input.node_id.as_str().into(),
            input.part_key.as_str().into(),
            input.field_key.as_str().into(),
        ))
        || !history.is_some_and(|history| {
            !history.deleted && history.fields.get("value") == Some(&input.value)
        })
    {
        return Err(invalid());
    }
    Ok(())
}

/// Live identity and revision, including owner deletion. Never infer a current
/// identity for an older unbound/generic Bible dependency.
pub(crate) fn current_revision(
    conn: &Connection,
    endpoint: &SemanticDependencyEndpoint,
) -> Result<Option<ChangeEventId>, HistoryStoreError> {
    let SemanticDependencyEndpoint::BibleField {
        node_id,
        part_key,
        field_key,
        field_id: Some(field_id),
    } = endpoint
    else {
        return Ok(None);
    };
    let revision: Option<String> = conn.query_row(
        "SELECT f.updated_event_id FROM bible_graph_fields f
         JOIN bible_graph_parts p ON p.id = f.part_id JOIN bible_graph_nodes n ON n.id = p.node_id
         WHERE f.id = ?1 AND n.id = ?2 AND p.part_key = ?3 AND f.field_key = ?4
           AND f.deleted_event_id IS NULL AND p.deleted_event_id IS NULL AND n.deleted_event_id IS NULL",
        params![field_id.as_str(), node_id.as_str(), part_key.as_str(), field_key.as_str()],
        |row| row.get(0)).optional()?;
    revision.map(|revision| parse_event(&revision)).transpose()
}

pub(crate) fn excerpt(
    conn: &Connection,
    endpoint: &SemanticDependencyEndpoint,
    event: ChangeEventId,
) -> Result<Option<String>, HistoryStoreError> {
    let SemanticDependencyEndpoint::BibleField {
        field_id: Some(field_id),
        ..
    } = endpoint
    else {
        return Ok(None);
    };
    let history = crate::revision_projection::load_object_field_projection_at_event(
        conn,
        ObjectKind::BiblePartField,
        field_id.as_str(),
        event,
    )?;
    Ok(history.and_then(|history| {
        history.fields.get("value").map(|value| {
            let text = match value {
                FieldValue::Text(text) => text.clone(),
                value => format!("{value:?}"),
            };
            text.chars().take(120).collect()
        })
    }))
}

fn invalid() -> HistoryStoreError {
    HistoryStoreError::InvalidValue(
        "Bible input does not match canonical field revision history".into(),
    )
}

fn parse_event(value: &str) -> Result<ChangeEventId, HistoryStoreError> {
    uuid::Uuid::parse_str(value)
        .map(ChangeEventId)
        .map_err(|error| HistoryStoreError::InvalidId(error.to_string()))
}

#[cfg(test)]
#[path = "bible_field_lineage_tests.rs"]
mod tests;

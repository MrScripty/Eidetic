//! Custody for authored names actually included in Bible prompt headers.
use std::collections::BTreeMap;

use eidetic_core::contracts::*;
use rusqlite::{Connection, OptionalExtension};

use crate::history_store::HistoryStoreError;
use crate::semantic_dependency_store::{
    DependencyDirection, DependencyEndpointFilter, SemanticDependencyFilter,
};

fn invalid() -> HistoryStoreError {
    HistoryStoreError::InvalidValue(
        "Bible name input does not match canonical revision history".into(),
    )
}

/// Resolution and receipt capture must share the caller's read snapshot.
pub(crate) fn capture(
    conn: &Connection,
    context: &AiBibleContextProjection,
) -> Result<Vec<BibleNodeNameInput>, HistoryStoreError> {
    let mut inputs = BTreeMap::new();
    for node in &context.nodes {
        let stored =
            crate::bible_graph_store::load_node(conn, &node.node_id)?.ok_or_else(invalid)?;
        if stored.name != node.name {
            return Err(invalid());
        }
        let input = BibleNodeNameInput {
            node_id: node.node_id.clone(),
            name: node.name.clone(),
            revision_event_id: current_revision(
                conn,
                &SemanticDependencyEndpoint::BibleNode {
                    node_id: node.node_id.clone(),
                },
            )?
            .ok_or_else(invalid)?,
        };
        validate_history(conn, &input)?;
        if let Some(previous) = inputs.insert(node.node_id.as_str().to_owned(), input.clone())
            && previous != input
        {
            return Err(invalid());
        }
    }
    Ok(inputs.into_values().collect())
}

pub(crate) fn endpoint(input: &BibleNodeNameInput) -> SemanticDependencyEndpoint {
    SemanticDependencyEndpoint::BibleNode {
        node_id: input.node_id.clone(),
    }
}

/// Validate the owned historical name, even if generation finished after a rename.
pub(crate) fn validate_history(
    conn: &Connection,
    input: &BibleNodeNameInput,
) -> Result<(), HistoryStoreError> {
    let owns_name: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id
         WHERE r.object_kind='bible_node' AND r.object_id=?1 AND r.change_event_id=?2
         AND f.field_key='name' AND f.new_type='text' AND f.new_text=?3)",
        rusqlite::params![input.node_id.as_str(), input.revision_event_id.0.to_string(), input.name],
        |row| row.get(0),
    )?;
    if !owns_name {
        return Err(invalid());
    }
    let history = crate::revision_projection::load_object_field_projection_at_event(
        conn,
        ObjectKind::BibleNode,
        input.node_id.as_str(),
        input.revision_event_id,
    )?
    .ok_or_else(invalid)?;
    if history.deleted || history.fields.get("name") != Some(&FieldValue::Text(input.name.clone()))
    {
        return Err(invalid());
    }
    Ok(())
}

/// A name clock excludes parent, order and other metadata-only revisions.
pub(crate) fn current_revision(
    conn: &Connection,
    endpoint: &SemanticDependencyEndpoint,
) -> Result<Option<ChangeEventId>, HistoryStoreError> {
    let revision = known_revision(conn, endpoint)?;
    if revision.is_none()
        && let SemanticDependencyEndpoint::BibleNode { node_id } = endpoint
        && crate::bible_graph_store::load_node(conn, node_id)?.is_some()
    {
        return Err(invalid());
    }
    Ok(revision)
}

/// Inspection may explicitly report unknown lineage for imported names.
pub(crate) fn known_revision(
    conn: &Connection,
    endpoint: &SemanticDependencyEndpoint,
) -> Result<Option<ChangeEventId>, HistoryStoreError> {
    let SemanticDependencyEndpoint::BibleNode { node_id } = endpoint else {
        return Ok(None);
    };
    if crate::bible_graph_store::load_node(conn, node_id)?.is_none() {
        return Ok(None);
    }
    let event: Option<String> = conn
        .query_row(
            "SELECT r.change_event_id FROM object_revisions r
         JOIN change_events e ON e.id=r.change_event_id
         WHERE r.object_kind='bible_node' AND r.object_id=?1
         AND (r.operation='delete' OR EXISTS(SELECT 1 FROM object_revision_fields f
              WHERE f.revision_id=r.id AND f.field_key='name' AND f.new_type='text'))
         ORDER BY e.rowid DESC,r.rowid DESC LIMIT 1",
            [node_id.as_str()],
            |row| row.get(0),
        )
        .optional()?;
    event
        .map(|event| {
            uuid::Uuid::parse_str(&event)
                .map(ChangeEventId)
                .map_err(|error| HistoryStoreError::InvalidId(error.to_string()))
        })
        .transpose()
}

pub(crate) fn excerpt(
    conn: &Connection,
    endpoint: &SemanticDependencyEndpoint,
    event: ChangeEventId,
) -> Result<Option<String>, HistoryStoreError> {
    let SemanticDependencyEndpoint::BibleNode { node_id } = endpoint else {
        return Ok(None);
    };
    let history = crate::revision_projection::load_object_field_projection_at_event(
        conn,
        ObjectKind::BibleNode,
        node_id.as_str(),
        event,
    )?;
    Ok(
        history.and_then(|history| match history.fields.get("name") {
            Some(FieldValue::Text(name)) => Some(name.chars().take(120).collect()),
            _ => None,
        }),
    )
}

/// Replacing lineage must not silently discard any live, previously consumed name.
pub(crate) fn validate_preview_inputs(
    conn: &Connection,
    generation: ChangeEventId,
    segment: &ScriptSegmentId,
    inputs: &[BibleNodeNameInput],
) -> Result<(), HistoryStoreError> {
    let dependencies = crate::semantic_dependency_store::load_semantic_dependency_projection(
        conn,
        &SemanticDependencyFilter {
            endpoint: DependencyEndpointFilter {
                kind: "script_segment".into(),
                id: segment.as_str().into(),
                part_key: None,
                field_key: None,
            },
            direction: DependencyDirection::Source,
        },
    )
    .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?;
    for dependency in dependencies.payload.dependencies {
        if !dependency
            .revision_binding
            .as_ref()
            .is_some_and(|binding| binding.source_revision_event_id == generation)
        {
            continue;
        }
        if let SemanticDependencyEndpoint::BibleNode { node_id } = &dependency.target
            && current_revision(conn, &dependency.target)?.is_some()
            && !inputs.iter().any(|input| input.node_id == *node_id)
        {
            return Err(HistoryStoreError::InvalidValue("Bible name review source is outside the current context; restore its context before previewing".into()));
        }
    }
    Ok(())
}

/// Deleted names retain owned history custody rather than a fabricated receipt.
pub(crate) fn capture_absence_revisions(
    conn: &Connection,
    causes: &[ScriptImpactCause],
) -> Result<Vec<(BibleGraphNodeId, ChangeEventId)>, HistoryStoreError> {
    let mut missing = BTreeMap::new();
    for cause in causes {
        if let SemanticDependencyEndpoint::BibleNode { node_id } = &cause.input
            && cause.current_revision_event_id.is_none()
        {
            let latest = crate::history_store::load_revisions_for_object(
                conn,
                ObjectKind::BibleNode,
                node_id.as_str(),
            )?
            .last()
            .map(|revision| revision.change_event_id)
            .ok_or_else(invalid)?;
            missing.insert(node_id.as_str().to_owned(), (node_id.clone(), latest));
        }
    }
    Ok(missing.into_values().collect())
}

#[cfg(test)]
#[path = "bible_node_name_lineage_tests.rs"]
mod tests;

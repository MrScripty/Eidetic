//! Revision custody for relationships actually supplied to screenplay generation.
use std::collections::BTreeMap;

use eidetic_core::contracts::*;
use rusqlite::{Connection, OptionalExtension};

use crate::history_store::HistoryStoreError;

/// Caller owns the read snapshot covering both resolution and revision capture.
pub(crate) fn capture(
    conn: &Connection,
    context: &AiBibleContextProjection,
) -> Result<Vec<BibleRelationshipInput>, HistoryStoreError> {
    let mut inputs = BTreeMap::new();
    for edge in context
        .nodes
        .iter()
        .flat_map(|node| node.incoming_edges.iter().chain(&node.outgoing_edges))
    {
        let stored =
            crate::bible_graph_edge_store::load_edge(conn, &edge.edge_id)?.ok_or_else(invalid)?;
        if stored.from_node_id != edge.from_node_id
            || stored.to_node_id != edge.to_node_id
            || stored.edge_kind != edge.edge_kind
            || stored.label != edge.label
            || stored.directed != edge.directed
        {
            return Err(invalid());
        }
        let revision = current_revision(conn, &endpoint_id(&edge.edge_id))?.ok_or_else(invalid)?;
        let input = BibleRelationshipInput {
            edge: edge.clone(),
            revision_event_id: revision,
        };
        // Sparse generic revisions can diverge from graph storage. Refuse an
        // inconsistent read instead of labelling stale payload as consumed latest.
        validate_history(conn, &input)?;
        if let Some(previous) = inputs.insert(edge.edge_id.as_str().to_owned(), input.clone())
            && previous != input
        {
            return Err(invalid());
        }
    }
    Ok(inputs.into_values().collect())
}

fn invalid() -> HistoryStoreError {
    HistoryStoreError::InvalidValue(
        "Bible relationship input does not match canonical revision history".into(),
    )
}

/// Reuse owned object revisions just as child-plan Bible custody does. Missing
/// consumed edges must retain identity history: recreate/delete is still drift.
pub(crate) fn capture_absence_revisions(
    conn: &Connection,
    causes: &[ScriptImpactCause],
) -> Result<Vec<(BibleGraphEdgeId, ChangeEventId)>, HistoryStoreError> {
    let mut missing = BTreeMap::new();
    for cause in causes {
        if let SemanticDependencyEndpoint::BibleEdge { edge_id } = &cause.input
            && cause.current_revision_event_id.is_none()
        {
            let latest = crate::history_store::load_revisions_for_object(
                conn,
                ObjectKind::BibleEdge,
                edge_id.as_str(),
            )?
            .last()
            .map(|revision| revision.change_event_id)
            .ok_or_else(invalid)?;
            missing.insert(edge_id.as_str().to_owned(), (edge_id.clone(), latest));
        }
    }
    Ok(missing.into_values().collect())
}

fn endpoint_id(edge_id: &BibleGraphEdgeId) -> SemanticDependencyEndpoint {
    SemanticDependencyEndpoint::BibleEdge {
        edge_id: edge_id.clone(),
    }
}

pub(crate) fn endpoint(input: &BibleRelationshipInput) -> SemanticDependencyEndpoint {
    endpoint_id(&input.edge.edge_id)
}

/// Late output retains its original consumed revision; never rebind to latest.
pub(crate) fn validate_history(
    conn: &Connection,
    input: &BibleRelationshipInput,
) -> Result<(), HistoryStoreError> {
    let owns_revision: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM object_revisions WHERE object_kind='bible_edge' AND object_id=?1 AND change_event_id=?2)",
        rusqlite::params![input.edge.edge_id.as_str(), input.revision_event_id.0.to_string()],
        |row| row.get(0),
    )?;
    if !owns_revision {
        return Err(invalid());
    }
    let history = crate::revision_projection::load_object_field_projection_at_event(
        conn,
        ObjectKind::BibleEdge,
        input.edge.edge_id.as_str(),
        input.revision_event_id,
    )?
    .ok_or_else(invalid)?;
    let edge = &input.edge;
    let expected = [
        (
            "from_node_id",
            FieldValue::ObjectRef {
                kind: ObjectKind::BibleNode,
                id: edge.from_node_id.as_str().into(),
            },
        ),
        (
            "to_node_id",
            FieldValue::ObjectRef {
                kind: ObjectKind::BibleNode,
                id: edge.to_node_id.as_str().into(),
            },
        ),
        (
            "edge_kind",
            FieldValue::Text(format!("{:?}", edge.edge_kind)),
        ),
        ("label", FieldValue::Text(edge.label.clone())),
        ("directed", FieldValue::Bool(edge.directed)),
    ];
    if history.deleted
        || expected
            .iter()
            .any(|(key, value)| history.fields.get(*key) != Some(value))
    {
        return Err(invalid());
    }
    Ok(())
}

pub(crate) fn current_revision(
    conn: &Connection,
    endpoint: &SemanticDependencyEndpoint,
) -> Result<Option<ChangeEventId>, HistoryStoreError> {
    let SemanticDependencyEndpoint::BibleEdge { edge_id } = endpoint else {
        return Ok(None);
    };
    let revision: Option<String> = conn.query_row(
        "SELECT r.change_event_id FROM bible_graph_edges e
         JOIN bible_graph_nodes f ON f.id=e.from_node_id JOIN bible_graph_nodes t ON t.id=e.to_node_id
         JOIN object_revisions r ON r.object_kind='bible_edge' AND r.object_id=e.id
         JOIN change_events c ON c.id=r.change_event_id
         WHERE e.id=?1 AND e.deleted_event_id IS NULL AND f.deleted_event_id IS NULL AND t.deleted_event_id IS NULL
         ORDER BY c.rowid DESC,r.rowid DESC LIMIT 1",
        [edge_id.as_str()], |row| row.get(0),
    ).optional()?;
    revision
        .map(|value| {
            uuid::Uuid::parse_str(&value)
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
    let SemanticDependencyEndpoint::BibleEdge { edge_id } = endpoint else {
        return Ok(None);
    };
    let history = crate::revision_projection::load_object_field_projection_at_event(
        conn,
        ObjectKind::BibleEdge,
        edge_id.as_str(),
        event,
    )?;
    Ok(
        history.and_then(|history| match history.fields.get("label") {
            Some(FieldValue::Text(label)) => Some(label.chars().take(120).collect()),
            _ => None,
        }),
    )
}

#[cfg(test)]
#[path = "bible_relationship_lineage_tests.rs"]
mod tests;

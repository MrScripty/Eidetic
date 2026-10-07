//! Selected clip Notes consumption in existing command/proposal/dependency history.
use eidetic_core::contracts::*;
use eidetic_core::timeline::node::NodeId;
use rusqlite::{Connection, OptionalExtension, params};

use crate::history_store::HistoryStoreError;

fn invalid() -> HistoryStoreError {
    HistoryStoreError::InvalidValue(
        "Timeline Notes evidence does not match owned field history".into(),
    )
}

fn owned_revision(
    conn: &Connection,
    node: NodeId,
    through: Option<ChangeEventId>,
) -> Result<Option<ChangeEventId>, HistoryStoreError> {
    let event: Option<String> = conn.query_row(
        "SELECT r.change_event_id FROM object_revisions r JOIN change_events e ON e.id=r.change_event_id
         WHERE r.object_kind='timeline_node' AND r.object_id=?1
         AND (r.operation='delete' OR EXISTS(SELECT 1 FROM object_revision_fields f
             WHERE f.revision_id=r.id AND f.field_key='notes'))
         AND (?2 IS NULL OR e.rowid <= (SELECT rowid FROM change_events WHERE id=?2))
         ORDER BY e.rowid DESC,r.rowid DESC LIMIT 1",
        params![node.0.to_string(), through.map(|event| event.0.to_string())], |row| row.get(0)
    ).optional()?;
    event
        .map(|event| {
            uuid::Uuid::parse_str(&event)
                .map(ChangeEventId)
                .map_err(|error| HistoryStoreError::InvalidId(error.to_string()))
        })
        .transpose()
}

fn validate_history(
    conn: &Connection,
    input: &TimelineNotesInput,
) -> Result<(), HistoryStoreError> {
    let Some(event) = input.revision_event_id else {
        return Ok(());
    };
    let owns: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id
         WHERE r.object_kind='timeline_node' AND r.object_id=?1 AND r.change_event_id=?2
         AND f.field_key='notes' AND f.new_type='text' AND f.new_text=?3)",
        params![input.node_id.0.to_string(), event.0.to_string(), input.notes], |row| row.get(0))?;
    let history = crate::revision_projection::load_object_field_projection_at_event(
        conn,
        ObjectKind::TimelineNode,
        &input.node_id.0.to_string(),
        event,
    )?
    .ok_or_else(invalid)?;
    if !owns
        || history.deleted
        || history.fields.get("notes") != Some(&FieldValue::Text(input.notes.clone()))
    {
        return Err(invalid());
    }
    Ok(())
}

/// The target already captured the exact prompt Notes before provider I/O.
/// Resolve its field clock at that historical node event, never at today's state.
pub(crate) fn from_target(
    conn: &Connection,
    target: &ScriptGenerationTarget,
) -> Result<TimelineNotesInput, HistoryStoreError> {
    let revision_event_id = if let Some(event) = target.node_revision_event_id {
        let owns: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM object_revisions WHERE object_kind='timeline_node'
             AND object_id=?1 AND change_event_id=?2)",
            params![target.node_id.0.to_string(), event.0.to_string()],
            |row| row.get(0),
        )?;
        if !owns {
            return Err(invalid());
        }
        owned_revision(conn, target.node_id, Some(event))?
    } else {
        None
    };
    let input = TimelineNotesInput {
        node_id: target.node_id,
        notes: target.notes.clone(),
        revision_event_id,
    };
    validate_history(conn, &input)?;
    Ok(input)
}

pub(crate) fn capture(
    conn: &Connection,
    node: NodeId,
) -> Result<TimelineNotesInput, HistoryStoreError> {
    let node = crate::timeline_node_store::load_node_ancestor_stack(conn, node)?
        .into_iter()
        .find(|candidate| candidate.id == node)
        .ok_or_else(invalid)?;
    let input = TimelineNotesInput {
        node_id: node.id,
        notes: node.content.notes,
        revision_event_id: owned_revision(conn, node.id, None)?,
    };
    validate_history(conn, &input)?;
    Ok(input)
}

/// Missing legacy target/review receipts remain unknown; no backfill from current Notes.
pub(crate) fn recorded(
    conn: &Connection,
    event: ChangeEventId,
) -> Result<Option<TimelineNotesInput>, HistoryStoreError> {
    let (kind, json): (String, String) = conn.query_row(
        "SELECT c.payload_type,c.payload_json FROM commands c JOIN change_events e ON e.command_id=c.id WHERE e.id=?1",
        [event.0.to_string()], |row| Ok((row.get(0)?,row.get(1)?)))?;
    let input = match kind.as_str() {
        "script.generate_block" => serde_json::from_str::<GenerateScriptBlockCommand>(&json)?
            .target_binding
            .as_ref()
            .map(|target| from_target(conn, target))
            .transpose()?,
        "semantic.propagation_accept" => {
            let command: AcceptPropagationProposalCommand = serde_json::from_str(&json)?;
            let json: String = conn.query_row(
                "SELECT binding_json FROM script_impact_proposal_bindings WHERE proposal_id=?1",
                [command.proposal_id.as_str()],
                |row| row.get(0),
            )?;
            serde_json::from_str::<ScriptImpactProposalBinding>(&json)?.timeline_notes_current
        }
        _ => None,
    };
    if let Some(input) = &input {
        validate_history(conn, input)?;
    }
    Ok(input)
}

fn dependency_id(event: ChangeEventId) -> SemanticDependencyId {
    SemanticDependencyId::new(format!("generation.{}.timeline_notes", event.0))
        .expect("nonempty Notes dependency")
}

pub(crate) fn dependency(
    input: &TimelineNotesInput,
    segment: &ScriptSegmentId,
    event: ChangeEventId,
    created_at_ms: u64,
) -> Option<SemanticDependency> {
    input.revision_event_id.map(|revision| SemanticDependency {
        id: dependency_id(event),
        source: SemanticDependencyEndpoint::ScriptSegment {
            segment_id: segment.clone(),
        },
        target: SemanticDependencyEndpoint::TimelineNode {
            node_id: input.node_id,
        },
        kind: SemanticDependencyKind::DerivesFrom,
        rationale: Some("Selected clip Notes supplied to screenplay generation".into()),
        confidence: None,
        created_at_ms,
        revision_binding: Some(SemanticDependencyRevisionBinding {
            source_revision_event_id: event,
            target_revision_event_id: revision,
        }),
    })
}

pub(crate) fn cause(
    conn: &Connection,
    event: ChangeEventId,
    dependency: &SemanticDependency,
) -> Result<Option<ScriptImpactCause>, HistoryStoreError> {
    if dependency.id != dependency_id(event) {
        return Ok(None);
    }
    let input = recorded(conn, event)?.ok_or_else(invalid)?;
    if dependency.target
        != (SemanticDependencyEndpoint::TimelineNode {
            node_id: input.node_id,
        })
        || dependency
            .revision_binding
            .as_ref()
            .map(|binding| binding.target_revision_event_id)
            != input.revision_event_id
    {
        return Err(invalid());
    }
    let live: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM nodes WHERE id=?1)",
        [input.node_id.0.to_string()],
        |row| row.get(0),
    )?;
    let (current_revision_event_id, reason) = if live {
        let current = capture(conn, input.node_id)?;
        if current == input {
            return Ok(None);
        }
        (current.revision_event_id, ScriptImpactReason::Changed)
    } else {
        (None, ScriptImpactReason::Deleted)
    };
    Ok(Some(ScriptImpactCause {
        dependency_id: dependency.id.clone(),
        input: dependency.target.clone(),
        consumed_revision_event_id: input.revision_event_id.ok_or_else(invalid)?,
        current_revision_event_id,
        reason,
        input_excerpt: Some(input.notes.chars().take(180).collect()),
    }))
}

/// Existing target receipts already prove Notes consumption even when older
/// generations predate the graph dependency. Missing receipts remain unknown.
pub(crate) fn impact(
    conn: &Connection,
    event: ChangeEventId,
    segment: &ScriptSegmentId,
) -> Result<Option<ScriptImpactCause>, HistoryStoreError> {
    let Some(input) = recorded(conn, event)? else {
        return Ok(None);
    };
    let Some(dependency) = dependency(&input, segment, event, 0) else {
        return Ok(None);
    };
    cause(conn, event, &dependency)
}

#[cfg(test)]
#[path = "timeline_notes_lineage_tests.rs"]
mod tests;

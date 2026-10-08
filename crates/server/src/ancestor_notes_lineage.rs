//! Consumed ancestor Notes in existing generation/proposal/dependency history.
use eidetic_core::contracts::*;
use eidetic_core::timeline::node::{NodeId, StoryNode};
use rusqlite::Connection;

use crate::{history_store::HistoryStoreError, timeline_notes_lineage as notes};

fn invalid() -> HistoryStoreError {
    HistoryStoreError::InvalidValue("ancestor Notes evidence is stale or invalid".into())
}

fn ancestors(conn: &Connection, target: NodeId) -> Result<Vec<StoryNode>, HistoryStoreError> {
    let stack = crate::timeline_node_store::load_node_ancestor_stack(conn, target)?;
    if !stack.iter().any(|node| node.id == target) {
        return Err(invalid());
    }
    Ok(stack
        .into_iter()
        .rev()
        .filter(|node| node.id != target)
        .collect())
}

/// Bind the exact prompt chain to one canonical snapshot before provider I/O.
pub(crate) fn capture_generation(
    conn: &Connection,
    target: NodeId,
    expected: &[StoryNode],
) -> Result<Vec<TimelineNotesInput>, HistoryStoreError> {
    let current = ancestors(conn, target)?;
    if current
        .iter()
        .map(|node| (node.id, &node.content.notes))
        .collect::<Vec<_>>()
        != expected
            .iter()
            .map(|node| (node.id, &node.content.notes))
            .collect::<Vec<_>>()
    {
        return Err(invalid());
    }
    capture_nonempty(conn, &current)
}

fn capture_nonempty(
    conn: &Connection,
    ancestors: &[StoryNode],
) -> Result<Vec<TimelineNotesInput>, HistoryStoreError> {
    ancestors
        .iter()
        .filter(|node| !node.content.notes.is_empty())
        .map(|node| notes::capture(conn, node.id))
        .collect()
}

pub(crate) fn validate_inputs(
    conn: &Connection,
    inputs: &[TimelineNotesInput],
) -> Result<(), HistoryStoreError> {
    let mut seen = std::collections::BTreeSet::new();
    for input in inputs {
        if !seen.insert(input.node_id.0) {
            return Err(invalid());
        }
        notes::validate_history(conn, input)?;
    }
    Ok(())
}

/// Writer-lock admission rejects a changed/cleared/ABA ancestor read atomically.
pub(crate) fn validate_admission(
    conn: &Connection,
    command: &GenerateScriptBlockCommand,
) -> Result<(), HistoryStoreError> {
    let Some(inputs) = &command.ancestor_notes_inputs else {
        return Ok(());
    };
    validate_inputs(conn, inputs)?;
    let target = command
        .block
        .source_node_id
        .as_deref()
        .ok_or_else(invalid)?;
    let target = NodeId(uuid::Uuid::parse_str(target).map_err(|_| invalid())?);
    if capture_nonempty(conn, &ancestors(conn, target)?)? != *inputs {
        return Err(invalid());
    }
    Ok(())
}

fn dependency_id(event: ChangeEventId, node: NodeId) -> SemanticDependencyId {
    SemanticDependencyId::new(format!("generation.{}.ancestor_notes.{}", event.0, node.0))
        .expect("nonempty ancestor Notes identity")
}

pub(crate) fn dependencies(
    conn: &Connection,
    inputs: Option<&[TimelineNotesInput]>,
    segment: &ScriptSegmentId,
    event: ChangeEventId,
    created_at_ms: u64,
) -> Result<Vec<SemanticDependency>, HistoryStoreError> {
    let Some(inputs) = inputs else {
        return Ok(Vec::new());
    };
    validate_inputs(conn, inputs)?;
    Ok(inputs
        .iter()
        .filter_map(|input| {
            input.revision_event_id.map(|revision| SemanticDependency {
                id: dependency_id(event, input.node_id),
                source: SemanticDependencyEndpoint::ScriptSegment {
                    segment_id: segment.clone(),
                },
                target: SemanticDependencyEndpoint::TimelineNode {
                    node_id: input.node_id,
                },
                kind: SemanticDependencyKind::DerivesFrom,
                rationale: Some(
                    "Ancestor Notes supplied to screenplay generation or reviewed update".into(),
                ),
                confidence: None,
                created_at_ms,
                revision_binding: Some(SemanticDependencyRevisionBinding {
                    source_revision_event_id: event,
                    target_revision_event_id: revision,
                }),
            })
        })
        .collect())
}

/// Missing legacy receipts stay unknown. Never replace them with current prose.
pub(crate) fn recorded(
    conn: &Connection,
    event: ChangeEventId,
) -> Result<Option<Vec<TimelineNotesInput>>, HistoryStoreError> {
    let (kind, json): (String, String) = conn.query_row(
        "SELECT c.payload_type,c.payload_json FROM commands c JOIN change_events e ON e.command_id=c.id WHERE e.id=?1",
        [event.0.to_string()], |row| Ok((row.get(0)?,row.get(1)?)))?;
    let inputs = match kind.as_str() {
        "script.generate_block" => {
            serde_json::from_str::<GenerateScriptBlockCommand>(&json)?.ancestor_notes_inputs
        }
        "semantic.propagation_accept" => {
            let command: AcceptPropagationProposalCommand = serde_json::from_str(&json)?;
            let json: String = conn.query_row(
                "SELECT binding_json FROM script_impact_proposal_bindings WHERE proposal_id=?1",
                [command.proposal_id.as_str()],
                |row| row.get(0),
            )?;
            serde_json::from_str::<ScriptImpactProposalBinding>(&json)?.ancestor_notes_current
        }
        _ => None,
    };
    if let Some(inputs) = &inputs {
        validate_inputs(conn, inputs)?;
    }
    Ok(inputs)
}

pub(crate) fn impact(
    conn: &Connection,
    event: ChangeEventId,
) -> Result<Vec<ScriptImpactCause>, HistoryStoreError> {
    let mut causes = Vec::new();
    for input in recorded(conn, event)?.iter().flatten() {
        // Unowned legacy field history cannot be assigned a fabricated revision.
        if input.revision_event_id.is_some()
            && let Some(cause) =
                notes::input_cause(conn, input, dependency_id(event, input.node_id))?
        {
            causes.push(cause);
        }
    }
    Ok(causes)
}

pub(crate) struct PreviewInputs {
    pub previous: Option<Vec<TimelineNotesInput>>,
    pub current: Option<Vec<TimelineNotesInput>>,
    pub absent: Option<Vec<(NodeId, ChangeEventId)>>,
}

/// Refresh only proven consumed identities; no unrelated/new ancestor expansion.
pub(crate) fn preview_inputs(
    conn: &Connection,
    event: ChangeEventId,
) -> Result<PreviewInputs, HistoryStoreError> {
    let previous = recorded(conn, event)?;
    let Some(inputs) = &previous else {
        return Ok(PreviewInputs {
            previous: None,
            current: None,
            absent: None,
        });
    };
    let mut current = Vec::new();
    let mut absent = Vec::new();
    for input in inputs {
        let live: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE id=?1)",
            [input.node_id.0.to_string()],
            |row| row.get(0),
        )?;
        if live {
            // Clearing is an exact read with an owned field clock, not absence.
            current.push(notes::capture(conn, input.node_id)?);
        } else {
            absent.push((
                input.node_id,
                notes::owned_revision(conn, input.node_id, None)?.ok_or_else(invalid)?,
            ));
        }
    }
    Ok(PreviewInputs {
        previous,
        current: Some(current),
        absent: Some(absent),
    })
}

#[cfg(test)]
#[path = "ancestor_notes_lineage_tests.rs"]
mod tests;

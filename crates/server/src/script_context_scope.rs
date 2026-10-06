//! Complete screenplay-window custody in existing command/proposal history.
//! The dependency targets the selected timeline anchor; its revision binding is
//! the screenplay selection epoch, not a fictional-time or node-fact revision.
use crate::history_store::HistoryStoreError;
use eidetic_core::contracts::*;
use eidetic_core::timeline::node::NodeId;
use rusqlite::{Connection, OptionalExtension};
use std::collections::BTreeSet;

/// Caller holds the same read snapshot as the complete context read.
pub(crate) fn capture(
    conn: &Connection,
    node_id: NodeId,
    start_ms: u64,
    end_ms: u64,
    inputs: &[ScriptContextBlock],
) -> Result<ScriptContextScope, HistoryStoreError> {
    let mut seen = BTreeSet::new();
    let segment_ids = inputs
        .iter()
        .filter(|input| seen.insert(input.segment_id.clone()))
        .map(|input| input.segment_id.clone())
        .collect();
    Ok(ScriptContextScope {
        node_id,
        start_ms,
        end_ms,
        segment_ids,
        revision_event_id: epoch(conn)?,
    })
}

// Existing append-only segment revisions detect an unconsumed membership ABA.
// Unrelated block text edits do not advance this conservative placement epoch.
fn epoch(conn: &Connection) -> Result<Option<ChangeEventId>, HistoryStoreError> {
    let value: Option<String> = conn
        .query_row(
            "SELECT r.change_event_id FROM object_revisions r
         JOIN change_events e ON e.id = r.change_event_id
         JOIN script_segments s ON s.id = r.object_id
         WHERE r.object_kind = 'script_segment' AND s.document_id = 'script.document.main'
         ORDER BY e.rowid DESC, r.sort_order DESC, r.rowid DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    value
        .map(|value| {
            uuid::Uuid::parse_str(&value)
                .map(ChangeEventId)
                .map_err(|e| HistoryStoreError::InvalidId(e.to_string()))
        })
        .transpose()
}

pub(crate) fn dependency_id(event: ChangeEventId) -> SemanticDependencyId {
    SemanticDependencyId::new(format!("generation.{}.context", event.0))
        .expect("nonempty context id")
}

pub(crate) fn dependency(
    command: &GenerateScriptBlockCommand,
    event: ChangeEventId,
    created_at_ms: u64,
) -> Option<SemanticDependency> {
    command
        .script_context_scope
        .as_ref()
        .map(|scope| SemanticDependency {
            id: dependency_id(event),
            source: SemanticDependencyEndpoint::ScriptSegment {
                segment_id: command.block.segment_id.clone(),
            },
            target: SemanticDependencyEndpoint::TimelineNode {
                node_id: scope.node_id,
            },
            kind: SemanticDependencyKind::DerivesFrom,
            rationale: Some("Complete screenplay continuity window supplied to generation".into()),
            confidence: None,
            created_at_ms,
            revision_binding: Some(SemanticDependencyRevisionBinding {
                source_revision_event_id: event,
                target_revision_event_id: scope.revision_event_id.unwrap_or(event),
            }),
        })
}

pub(crate) fn validate(
    conn: &Connection,
    command: &GenerateScriptBlockCommand,
) -> Result<(), HistoryStoreError> {
    let Some(scope) = &command.script_context_scope else {
        return Ok(());
    };
    let inputs = command.script_inputs.as_ref().ok_or_else(invalid)?;
    if command.block.document_id.as_str() != "script.document.main"
        || command.block.source_node_id.as_deref() != Some(scope.node_id.0.to_string().as_str())
        || scope.start_ms >= scope.end_ms
        || scope.segment_ids.iter().collect::<BTreeSet<_>>().len() != scope.segment_ids.len()
    {
        return Err(invalid());
    }
    // Preview can supplement a proven source outside the window. That evidence
    // is still consumed, but must not become invented normal-window membership.
    let selected: BTreeSet<_> = scope.segment_ids.iter().collect();
    let mut seen = BTreeSet::new();
    let ordered = inputs
        .iter()
        .filter(|input| {
            selected.contains(&input.segment_id) && seen.insert(input.segment_id.clone())
        })
        .map(|input| input.segment_id.clone())
        .collect::<Vec<_>>();
    if ordered != scope.segment_ids {
        return Err(invalid());
    }
    if let Some(event) = scope.revision_event_id {
        let valid: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM object_revisions r JOIN script_segments s ON s.id=r.object_id
             WHERE r.object_kind='script_segment' AND s.document_id='script.document.main' AND r.change_event_id=?1)",
            [event.0.to_string()], |row| row.get(0))?;
        if !valid {
            return Err(invalid());
        }
    } else if !scope.segment_ids.is_empty() {
        return Err(invalid());
    }
    Ok(())
}

fn recorded(
    conn: &Connection,
    event: ChangeEventId,
) -> Result<Option<(ScriptContextScope, Vec<ScriptContextBlock>)>, HistoryStoreError> {
    let (kind, json): (String, String) = conn.query_row(
        "SELECT c.payload_type,c.payload_json FROM commands c JOIN change_events e ON e.command_id=c.id WHERE e.id=?1",
        [event.0.to_string()], |row| Ok((row.get(0)?,row.get(1)?)))?;
    match kind.as_str() {
        "script.generate_block" => {
            let command: GenerateScriptBlockCommand = serde_json::from_str(&json)?;
            Ok(command.script_context_scope.zip(command.script_inputs))
        }
        "semantic.propagation_accept" => {
            let command: AcceptPropagationProposalCommand = serde_json::from_str(&json)?;
            let binding: String = conn.query_row(
                "SELECT binding_json FROM script_impact_proposal_bindings WHERE proposal_id=?1",
                [command.proposal_id.as_str()],
                |row| row.get(0),
            )?;
            let binding: ScriptImpactProposalBinding = serde_json::from_str(&binding)?;
            Ok(binding
                .script_context_scope
                .map(|scope| (scope, binding.script_inputs)))
        }
        _ => Ok(None),
    }
}

fn signature(
    scope: &ScriptContextScope,
    inputs: &[ScriptContextBlock],
    output: &ScriptSegmentId,
) -> Vec<(ScriptSegmentId, u8)> {
    scope
        .segment_ids
        .iter()
        .filter(|id| *id != output)
        .filter_map(|id| {
            inputs
                .iter()
                .find(|input| input.segment_id == *id)
                .map(|input| {
                    let position = if input.end_ms <= scope.start_ms {
                        0
                    } else if input.start_ms >= scope.end_ms {
                        2
                    } else {
                        1
                    };
                    (id.clone(), position)
                })
        })
        .collect()
}

pub(crate) fn cause(
    conn: &Connection,
    event: ChangeEventId,
    output: &ScriptSegmentId,
    dependency: &SemanticDependency,
) -> Result<Option<ScriptImpactCause>, HistoryStoreError> {
    if dependency.id != dependency_id(event) {
        return Ok(None);
    }
    let Some((before, old_inputs)) = recorded(conn, event)? else {
        return Ok(None);
    };
    let (node, start, end): (String, u64, u64) = conn.query_row(
        "SELECT source_node_id,start_ms,end_ms FROM script_segments WHERE id=?1",
        [output.as_str()],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    let node_id = NodeId(
        uuid::Uuid::parse_str(&node).map_err(|e| HistoryStoreError::InvalidId(e.to_string()))?,
    );
    let inputs = crate::ai_script_context::load_script_context(conn, node_id, start, end)?;
    let current = capture(conn, node_id, start, end, &inputs)?;
    let old = signature(&before, &old_inputs, output);
    let new = signature(&current, &inputs, output);
    if before.node_id == current.node_id && old == new {
        return Ok(None);
    }
    let old_ids: BTreeSet<_> = old.iter().map(|(id, _)| id).collect();
    let new_ids: BTreeSet<_> = new.iter().map(|(id, _)| id).collect();
    let entered = new_ids
        .difference(&old_ids)
        .map(|id| name(conn, id, &inputs))
        .collect::<Result<Vec<_>, _>>()?;
    let left = old_ids
        .difference(&new_ids)
        .map(|id| name(conn, id, &old_inputs))
        .collect::<Result<Vec<_>, _>>()?;
    let mut explanation = Vec::new();
    if !entered.is_empty() {
        explanation.push(format!("Entered: {}.", entered.join(", ")));
    }
    if !left.is_empty() {
        explanation.push(format!("Left: {}.", left.join(", ")));
    }
    if explanation.is_empty() {
        explanation.push("Scene order changed around this scene.".into());
    }
    Ok(Some(ScriptImpactCause {
        dependency_id: dependency.id.clone(),
        input: dependency.target.clone(),
        consumed_revision_event_id: before.revision_event_id.unwrap_or(event),
        current_revision_event_id: current.revision_event_id,
        reason: ScriptImpactReason::ContextChanged,
        input_excerpt: Some(explanation.join(" ").chars().take(180).collect()),
    }))
}

fn name(
    conn: &Connection,
    id: &ScriptSegmentId,
    inputs: &[ScriptContextBlock],
) -> Result<String, HistoryStoreError> {
    let source = inputs
        .iter()
        .find(|input| input.segment_id == *id)
        .and_then(|input| input.source_node_id.as_deref());
    if let Some(source) = source {
        let name: Option<String> = conn
            .query_row("SELECT name FROM nodes WHERE id=?1", [source], |row| {
                row.get(0)
            })
            .optional()?;
        if let Some(name) = name {
            return Ok(name);
        }
    }
    Ok(id.as_str().into())
}

fn invalid() -> HistoryStoreError {
    HistoryStoreError::InvalidValue(
        "screenplay context selection does not match captured canonical inputs".into(),
    )
}

#[cfg(test)]
#[path = "scene_context_membership_tests.rs"]
pub(crate) mod tests;

//! Exact consumed titles reuse generation commands, proposal bindings and graph edges.
use crate::history_store::HistoryStoreError;
use eidetic_core::ai::backend::GenerateRequest;
use eidetic_core::contracts::*;
use eidetic_core::timeline::node::NodeId;
use rusqlite::{Connection, OptionalExtension, params};

fn invalid() -> HistoryStoreError {
    HistoryStoreError::InvalidValue("consumed timeline title evidence is stale or invalid".into())
}

/// These are precisely the node names formatted by build_chat_prompt. Never
/// resolve a recap name by text: distinct nodes can have identical titles.
pub(crate) fn supplied_titles(
    request: &GenerateRequest,
) -> Result<Vec<(NodeId, String)>, HistoryStoreError> {
    let mut supplied = vec![(request.target_node.id, request.target_node.name.clone())];
    supplied.extend(
        request
            .ancestor_chain
            .iter()
            .chain(&request.siblings)
            .map(|node| (node.id, node.name.clone())),
    );
    for recap in &request.surrounding_context.preceding_recaps {
        supplied.push((recap.node_id.ok_or_else(invalid)?, recap.node_name.clone()));
    }
    let mut unique: Vec<(NodeId, String)> = Vec::new();
    for (id, name) in supplied {
        if let Some((_, prior)) = unique.iter().find(|(node, _)| *node == id) {
            if *prior != name {
                return Err(invalid());
            }
        } else {
            unique.push((id, name));
        }
    }
    Ok(unique)
}

pub(crate) fn capture(
    conn: &Connection,
    node_id: NodeId,
) -> Result<TimelineTitleInput, HistoryStoreError> {
    let name: String = conn
        .query_row(
            "SELECT name FROM nodes WHERE id=?1",
            [node_id.0.to_string()],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(invalid)?;
    let input = TimelineTitleInput {
        node_id,
        name,
        revision_event_id: crate::timeline_node_store::latest_name_event(conn, node_id, None)?,
    };
    validate_history(conn, &input)?;
    Ok(input)
}

pub(crate) fn capture_generation(
    conn: &Connection,
    supplied: &[(NodeId, String)],
) -> Result<Vec<TimelineTitleInput>, HistoryStoreError> {
    let mut inputs = Vec::new();
    for (id, name) in supplied {
        let input = capture(conn, *id)?;
        if input.name != *name {
            return Err(invalid());
        }
        inputs.push(input);
    }
    validate_inputs(conn, &inputs)?;
    Ok(inputs)
}

fn validate_history(
    conn: &Connection,
    input: &TimelineTitleInput,
) -> Result<(), HistoryStoreError> {
    let Some(event) = input.revision_event_id else {
        return Ok(());
    };
    let owns: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id
         WHERE r.object_kind='timeline_node' AND r.object_id=?1 AND r.change_event_id=?2
         AND f.field_key='name' AND f.new_type='text' AND f.new_text=?3)",
        params![input.node_id.0.to_string(), event.0.to_string(), input.name], |row| row.get(0))?;
    let historical = crate::revision_projection::load_object_field_projection_at_event(
        conn,
        ObjectKind::TimelineNode,
        &input.node_id.0.to_string(),
        event,
    )?
    .ok_or_else(invalid)?;
    if !owns
        || historical.deleted
        || historical.fields.get("name") != Some(&FieldValue::Text(input.name.clone()))
    {
        return Err(invalid());
    }
    Ok(())
}

pub(crate) fn validate_inputs(
    conn: &Connection,
    inputs: &[TimelineTitleInput],
) -> Result<(), HistoryStoreError> {
    let mut seen = std::collections::BTreeSet::new();
    for input in inputs {
        if !seen.insert(input.node_id.0) {
            return Err(invalid());
        }
        validate_history(conn, input)?;
    }
    Ok(())
}

/// Writer admission retains exact values and owned clocks, including baseline
/// title edit/restore ABA. An unrelated Notes/placement edit is not a title edit.
pub(crate) fn validate_admission(
    conn: &Connection,
    command: &GenerateScriptBlockCommand,
) -> Result<(), HistoryStoreError> {
    let Some(inputs) = &command.timeline_title_inputs else {
        return Ok(());
    };
    validate_inputs(conn, inputs)?;
    let target = command
        .block
        .source_node_id
        .as_deref()
        .ok_or_else(invalid)?;
    if !inputs
        .iter()
        .any(|input| input.node_id.0.to_string() == target)
    {
        return Err(invalid());
    }
    for input in inputs {
        if capture(conn, input.node_id)? != *input {
            return Err(invalid());
        }
    }
    Ok(())
}

fn dependency_id(event: ChangeEventId, node: NodeId) -> SemanticDependencyId {
    SemanticDependencyId::new(format!("generation.{}.timeline_title.{}", event.0, node.0))
        .expect("title dependency identity")
}

pub(crate) fn dependencies(
    conn: &Connection,
    inputs: Option<&[TimelineTitleInput]>,
    segment: &ScriptSegmentId,
    event: ChangeEventId,
    created_at_ms: u64,
) -> Result<Vec<SemanticDependency>, HistoryStoreError> {
    let Some(inputs) = inputs else {
        return Ok(Vec::new());
    };
    validate_inputs(conn, inputs)?;
    Ok(inputs.iter().map(|input| SemanticDependency {
        id: dependency_id(event, input.node_id),
        source: SemanticDependencyEndpoint::ScriptSegment {segment_id: segment.clone()},
        target: SemanticDependencyEndpoint::TimelineNode {node_id: input.node_id},
        kind: SemanticDependencyKind::DerivesFrom,
        rationale: Some("Exact timeline title supplied to generation or targeted review; baseline clock is a read stamp".into()),
        confidence: None, created_at_ms,
        revision_binding: Some(SemanticDependencyRevisionBinding {
            source_revision_event_id: event,
            // For baseline titles this is the generation read stamp, not an
            // invented name authoring revision. The exact owned clock is None
            // in the durable consumed-title receipt, used for all comparisons.
            target_revision_event_id: input.revision_event_id.unwrap_or(event),
        }),
    }).collect())
}

/// Optional legacy receipts remain unknown, never reconstructed from today.
pub(crate) fn recorded(
    conn: &Connection,
    event: ChangeEventId,
) -> Result<Option<Vec<TimelineTitleInput>>, HistoryStoreError> {
    let (kind, json): (String, String) = conn.query_row(
        "SELECT c.payload_type,c.payload_json FROM commands c JOIN change_events e ON e.command_id=c.id WHERE e.id=?1",
        [event.0.to_string()], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let inputs = match kind.as_str() {
        "script.generate_block" => {
            serde_json::from_str::<GenerateScriptBlockCommand>(&json)?.timeline_title_inputs
        }
        "semantic.propagation_accept" => {
            let command: AcceptPropagationProposalCommand = serde_json::from_str(&json)?;
            let json: String = conn.query_row(
                "SELECT binding_json FROM script_impact_proposal_bindings WHERE proposal_id=?1",
                [command.proposal_id.as_str()],
                |row| row.get(0),
            )?;
            serde_json::from_str::<ScriptImpactProposalBinding>(&json)?.timeline_title_current
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
        let live: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE id=?1)",
            [input.node_id.0.to_string()],
            |row| row.get(0),
        )?;
        let (current_revision_event_id, reason) = if live {
            let current = capture(conn, input.node_id)?;
            if current == *input {
                continue;
            }
            (current.revision_event_id, ScriptImpactReason::Changed)
        } else {
            (None, ScriptImpactReason::Deleted)
        };
        causes.push(ScriptImpactCause {
            dependency_id: dependency_id(event, input.node_id),
            input: SemanticDependencyEndpoint::TimelineNode {
                node_id: input.node_id,
            },
            consumed_revision_event_id: input.revision_event_id.unwrap_or(event),
            current_revision_event_id,
            reason,
            input_excerpt: Some(input.name.clone()),
        });
    }
    Ok(causes)
}

pub(crate) struct PreviewInputs {
    pub previous: Option<Vec<TimelineTitleInput>>,
    pub current: Option<Vec<TimelineTitleInput>>,
    pub absent: Option<Vec<(NodeId, ChangeEventId)>>,
}

/// Refresh only actually consumed identities. Owned deletion clocks bind
/// absence without inventing current title prose or adding unrelated nodes.
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
            current.push(capture(conn, input.node_id)?);
        } else {
            let deletion =
                crate::timeline_node_store::latest_name_event(conn, input.node_id, None)?
                    .ok_or_else(invalid)?;
            let historical = crate::revision_projection::load_object_field_projection_at_event(
                conn,
                ObjectKind::TimelineNode,
                &input.node_id.0.to_string(),
                deletion,
            )?
            .ok_or_else(invalid)?;
            if !historical.deleted {
                return Err(invalid());
            }
            absent.push((input.node_id, deletion));
        }
    }
    Ok(PreviewInputs {
        previous,
        current: Some(current),
        absent: Some(absent),
    })
}

pub(crate) fn append_prompt(prompt: &mut String, binding: &ScriptImpactProposalBinding) {
    for input in binding.timeline_title_previous.iter().flatten() {
        prompt.push_str(&format!(
            "\nORIGINAL CONSUMED TIMELINE TITLE ({}):\n{}\n",
            input.node_id.0, input.name
        ));
    }
    for input in binding.timeline_title_current.iter().flatten() {
        prompt.push_str(&format!(
            "\nCURRENT CONSUMED TIMELINE TITLE ({}):\n{}\n",
            input.node_id.0, input.name
        ));
    }
    for (node, _) in binding.timeline_title_absence_revisions.iter().flatten() {
        prompt.push_str(&format!(
            "\nPREVIOUSLY CONSUMED TIMELINE TITLE SOURCE REMOVED: {}\n",
            node.0
        ));
    }
}

#[cfg(test)]
#[path = "timeline_title_lineage_tests.rs"]
mod tests;

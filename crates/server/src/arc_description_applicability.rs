//! Known-empty description applicability, separate from consumed arc prose.
use std::collections::BTreeSet;

use eidetic_core::contracts::*;
use eidetic_core::timeline::node::NodeId;
use rusqlite::{Connection, params};

use crate::{history_store::HistoryStoreError, story_arc_lineage as arcs};

fn invalid() -> HistoryStoreError {
    HistoryStoreError::InvalidValue("arc description applicability is stale or invalid".into())
}

/// Read only omitted descriptions on the actual selected tags, in the caller's
/// canonical snapshot. Ordinary supplied descriptions keep their existing owner.
pub(crate) fn capture(
    conn: &Connection,
    node: NodeId,
    supplied: &[StoryArcFieldInput],
) -> Result<Vec<StoryArcFieldInput>, HistoryStoreError> {
    let (selected, _) = arcs::capture(conn, node, supplied)?;
    let selected_ids = selected.iter().map(|arc| arc.id.0).collect::<BTreeSet<_>>();
    for field in [StoryArcPromptField::Name, StoryArcPromptField::ArcType] {
        if supplied
            .iter()
            .filter(|input| input.field == field)
            .map(|input| input.arc_id.0)
            .collect::<BTreeSet<_>>()
            != selected_ids
        {
            return Err(invalid());
        }
    }
    let mut result = Vec::new();
    for arc in selected {
        if supplied
            .iter()
            .any(|input| input.arc_id == arc.id && input.field == StoryArcPromptField::Description)
        {
            continue;
        }
        // No omitted-description receipt may acknowledge newly supplied prose.
        if !arc.description.is_empty() {
            return Err(invalid());
        }
        let input = StoryArcFieldInput {
            arc_id: arc.id,
            field: StoryArcPromptField::Description,
            value: String::new(),
            revision_event_id: arcs::current_revision(
                conn,
                &SemanticDependencyEndpoint::StoryArcField {
                    arc_id: arc.id,
                    field: StoryArcPromptField::Description,
                },
            )?,
        };
        result.push(input);
    }
    result.sort_by_key(|input| input.arc_id.0);
    Ok(result)
}

pub(crate) fn validate_inputs(
    conn: &Connection,
    inputs: &[StoryArcFieldInput],
) -> Result<(), HistoryStoreError> {
    let mut seen = BTreeSet::new();
    for input in inputs {
        if input.field != StoryArcPromptField::Description
            || !input.value.is_empty()
            || !seen.insert(input.arc_id.0)
        {
            return Err(invalid());
        }
        arcs::validate_history(conn, input)?;
    }
    Ok(())
}

pub(crate) fn validate_admission(
    conn: &Connection,
    command: &GenerateScriptBlockCommand,
) -> Result<(), HistoryStoreError> {
    let Some(inputs) = &command.arc_description_applicability else {
        return Ok(());
    };
    validate_inputs(conn, inputs)?;
    let node = command
        .block
        .source_node_id
        .as_deref()
        .ok_or_else(invalid)?;
    let node = NodeId(uuid::Uuid::parse_str(node).map_err(|_| invalid())?);
    if capture(
        conn,
        node,
        command.arc_inputs.as_deref().ok_or_else(invalid)?,
    )? != *inputs
    {
        return Err(invalid());
    }
    Ok(())
}

fn dependency_id(event: ChangeEventId, input: &StoryArcFieldInput) -> SemanticDependencyId {
    SemanticDependencyId::new(format!(
        "generation.{}.arc_description_applicability.{}",
        event.0, input.arc_id.0
    ))
    .expect("nonempty applicability identity")
}

pub(crate) fn dependencies(
    conn: &Connection,
    inputs: Option<&[StoryArcFieldInput]>,
    segment: &ScriptSegmentId,
    event: ChangeEventId,
    created_at_ms: u64,
) -> Result<Vec<SemanticDependency>, HistoryStoreError> {
    let Some(inputs) = inputs else {
        return Ok(Vec::new());
    };
    validate_inputs(conn, inputs)?;
    Ok(inputs.iter().filter_map(|input| input.revision_event_id.map(|revision| SemanticDependency {
        id: dependency_id(event, input),
        source: SemanticDependencyEndpoint::ScriptSegment { segment_id: segment.clone() },
        target: arcs::endpoint(input),
        kind: SemanticDependencyKind::DerivesFrom,
        rationale: Some("Known-empty description applicability read; description prose was not supplied".into()),
        confidence: None, created_at_ms,
        revision_binding: Some(SemanticDependencyRevisionBinding { source_revision_event_id: event, target_revision_event_id: revision }),
    })).collect())
}

/// Missing legacy applicability stays unknown; no reconstruction from today's arc.
pub(crate) fn recorded(
    conn: &Connection,
    event: ChangeEventId,
) -> Result<Option<Vec<StoryArcFieldInput>>, HistoryStoreError> {
    let (kind, json): (String, String) = conn.query_row(
        "SELECT c.payload_type,c.payload_json FROM commands c JOIN change_events e ON e.command_id=c.id WHERE e.id=?1",
        [event.0.to_string()], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let inputs = match kind.as_str() {
        "script.generate_block" => {
            serde_json::from_str::<GenerateScriptBlockCommand>(&json)?.arc_description_applicability
        }
        "semantic.propagation_accept" => {
            let command: AcceptPropagationProposalCommand = serde_json::from_str(&json)?;
            let json: String = conn.query_row(
                "SELECT binding_json FROM script_impact_proposal_bindings WHERE proposal_id=?1",
                [command.proposal_id.as_str()],
                |row| row.get(0),
            )?;
            serde_json::from_str::<ScriptImpactProposalBinding>(&json)?
                .arc_description_applicability_current
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
    segment: &ScriptSegmentId,
) -> Result<Vec<ScriptImpactCause>, HistoryStoreError> {
    let node: Option<String> = conn.query_row(
        "SELECT source_node_id FROM script_segments WHERE id=?1",
        [segment.as_str()],
        |row| row.get(0),
    )?;
    let Some(node) = node else {
        return Ok(Vec::new());
    };
    let mut causes = Vec::new();
    for input in recorded(conn, event)?.iter().flatten() {
        let Some(original) = input.revision_event_id else {
            continue;
        };
        let tagged: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM node_arcs WHERE node_id=?1 AND arc_id=?2)",
            params![node, input.arc_id.0.to_string()],
            |row| row.get(0),
        )?;
        let Some(arc) = crate::story_arc_store::load_arc(conn, &input.arc_id)? else {
            continue;
        };
        // Clearing withdraws newly available prose. Ordinary consumed-description
        // clearing remains governed by its existing field dependency instead.
        if !tagged || arc.description.is_empty() {
            continue;
        }
        causes.push(ScriptImpactCause {
            dependency_id: dependency_id(event, input),
            input: arcs::endpoint(input),
            consumed_revision_event_id: original,
            current_revision_event_id: arcs::current_revision(conn, &arcs::endpoint(input))?,
            reason: ScriptImpactReason::ContextChanged,
            input_excerpt: Some("Description was not supplied (known empty).".into()),
        });
    }
    Ok(causes)
}

#[cfg(test)]
#[path = "arc_description_applicability_tests.rs"]
mod tests;

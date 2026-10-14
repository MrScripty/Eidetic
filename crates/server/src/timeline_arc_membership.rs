//! Selected clip arc membership in existing timeline field and generation history.
use eidetic_core::contracts::*;
use eidetic_core::timeline::node::NodeId;
use rusqlite::{Connection, OptionalExtension, params};

use crate::history_store::HistoryStoreError;

fn invalid() -> HistoryStoreError {
    HistoryStoreError::InvalidValue(
        "Timeline arc membership evidence does not match owned history".into(),
    )
}

pub(crate) fn validate_history(
    conn: &Connection,
    input: &TimelineArcMembershipInput,
) -> Result<(), HistoryStoreError> {
    if input.arc_ids.windows(2).any(|ids| ids[0].0 >= ids[1].0) {
        return Err(invalid());
    }
    let Some(event) = input.revision_event_id else {
        return Ok(());
    };
    let stored: Option<String> = conn.query_row(
        "SELECT f.new_text FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id
         WHERE r.object_kind='timeline_node' AND r.object_id=?1 AND r.change_event_id=?2
         AND f.field_key='arc_ids' AND f.new_type='text'",
        params![input.node_id.0.to_string(),event.0.to_string()], |row| row.get(0)).optional()?.flatten();
    let mut ids: Vec<eidetic_core::story::arc::ArcId> =
        serde_json::from_str(&stored.ok_or_else(invalid)?)?;
    ids.sort_by_key(|id| id.0);
    if ids != input.arc_ids {
        return Err(invalid());
    }
    Ok(())
}

pub(crate) fn capture(
    conn: &Connection,
    node: NodeId,
) -> Result<TimelineArcMembershipInput, HistoryStoreError> {
    capture_excluding(conn, node, None)
}

pub(crate) fn capture_excluding(
    conn: &Connection,
    node: NodeId,
    exclude: Option<ChangeEventId>,
) -> Result<TimelineArcMembershipInput, HistoryStoreError> {
    let live: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM nodes WHERE id=?1)",
        [node.0.to_string()],
        |row| row.get(0),
    )?;
    if !live {
        return Err(invalid());
    }
    let mut statement =
        conn.prepare("SELECT arc_id FROM node_arcs WHERE node_id=?1 ORDER BY arc_id")?;
    let arc_ids = statement
        .query_map([node.0.to_string()], |row| row.get::<_, String>(0))?
        .map(|id| {
            uuid::Uuid::parse_str(&id?)
                .map(eidetic_core::story::arc::ArcId)
                .map_err(|error| HistoryStoreError::InvalidId(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let event: Option<String> = conn.query_row(
        "SELECT r.change_event_id FROM object_revisions r JOIN change_events e ON e.id=r.change_event_id
         JOIN object_revision_fields f ON f.revision_id=r.id
         WHERE r.object_kind='timeline_node' AND r.object_id=?1 AND f.field_key='arc_ids'
         AND (?2 IS NULL OR e.id != ?2) ORDER BY e.rowid DESC,r.rowid DESC LIMIT 1",
        params![node.0.to_string(),exclude.map(|id| id.0.to_string())],|row|row.get(0)).optional()?;
    let input = TimelineArcMembershipInput {
        node_id: node,
        arc_ids,
        revision_event_id: event
            .map(|event| {
                uuid::Uuid::parse_str(&event)
                    .map(ChangeEventId)
                    .map_err(|error| HistoryStoreError::InvalidId(error.to_string()))
            })
            .transpose()?,
    };
    validate_history(conn, &input)?;
    Ok(input)
}

pub(crate) fn command_receipt(
    conn: &Connection,
    command: &CommandEnvelope<SetTimelineNodeArcsCommand>,
) -> Result<TimelineArcMembershipInput, HistoryStoreError> {
    let event: String = conn.query_row(
        "SELECT id FROM change_events WHERE command_id=?1",
        [command.id.0.to_string()],
        |row| row.get(0),
    )?;
    let mut arc_ids = command.payload.arc_ids.clone();
    arc_ids.sort_by_key(|id| id.0);
    let input = TimelineArcMembershipInput {
        node_id: command.payload.node_id,
        arc_ids,
        revision_event_id: Some(ChangeEventId(
            uuid::Uuid::parse_str(&event).map_err(|_| invalid())?,
        )),
    };
    validate_history(conn, &input)?;
    Ok(input)
}

/// Missing legacy receipts stay unknown; never infer original tags from current state.
pub(crate) fn recorded(
    conn: &Connection,
    event: ChangeEventId,
) -> Result<Option<TimelineArcMembershipInput>, HistoryStoreError> {
    let (kind,json): (String,String) = conn.query_row("SELECT c.payload_type,c.payload_json FROM commands c JOIN change_events e ON e.command_id=c.id WHERE e.id=?1",[event.0.to_string()],|row|Ok((row.get(0)?,row.get(1)?)))?;
    let input = match kind.as_str() {
        "script.generate_block" => serde_json::from_str::<GenerateScriptBlockCommand>(&json)?
            .target_binding
            .and_then(|target| target.arc_membership),
        "semantic.propagation_accept" => {
            let command: AcceptPropagationProposalCommand = serde_json::from_str(&json)?;
            let json: String = conn.query_row(
                "SELECT binding_json FROM script_impact_proposal_bindings WHERE proposal_id=?1",
                [command.proposal_id.as_str()],
                |row| row.get(0),
            )?;
            serde_json::from_str::<ScriptImpactProposalBinding>(&json)?.arc_membership_current
        }
        _ => None,
    };
    if let Some(input) = &input {
        validate_history(conn, input)?;
    }
    Ok(input)
}

fn dependency_id(event: ChangeEventId) -> SemanticDependencyId {
    SemanticDependencyId::new(format!("generation.{}.arc_membership", event.0))
        .expect("membership dependency")
}

pub(crate) fn dependency(
    input: &TimelineArcMembershipInput,
    segment: &ScriptSegmentId,
    event: ChangeEventId,
    created_at_ms: u64,
) -> SemanticDependency {
    SemanticDependency {
        id: dependency_id(event),
        source: SemanticDependencyEndpoint::ScriptSegment {
            segment_id: segment.clone(),
        },
        target: SemanticDependencyEndpoint::TimelineNode {
            node_id: input.node_id,
        },
        kind: SemanticDependencyKind::DerivesFrom,
        rationale: Some(
            "Selected clip story arc membership supplied to screenplay generation".into(),
        ),
        confidence: None,
        created_at_ms,
        // Known baseline membership is owned by its recorded generation receipt.
        revision_binding: Some(SemanticDependencyRevisionBinding {
            source_revision_event_id: event,
            target_revision_event_id: input.revision_event_id.unwrap_or(event),
        }),
    }
}

pub(crate) fn impact(
    conn: &Connection,
    event: ChangeEventId,
    segment: &ScriptSegmentId,
) -> Result<Option<ScriptImpactCause>, HistoryStoreError> {
    let Some(input) = recorded(conn, event)? else {
        return Ok(None);
    };
    let source: Option<String> = conn.query_row(
        "SELECT source_node_id FROM script_segments WHERE id=?1",
        [segment.as_str()],
        |row| row.get(0),
    )?;
    if source.as_deref() != Some(input.node_id.0.to_string().as_str()) {
        return Err(invalid());
    }
    let live: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM nodes WHERE id=?1)",
        [input.node_id.0.to_string()],
        |row| row.get(0),
    )?;
    if !live {
        return Ok(Some(ScriptImpactCause {
            dependency_id: dependency_id(event),
            input: SemanticDependencyEndpoint::TimelineNode {
                node_id: input.node_id,
            },
            consumed_revision_event_id: input.revision_event_id.unwrap_or(event),
            current_revision_event_id: None,
            reason: ScriptImpactReason::Deleted,
            input_excerpt: Some("Selected clip story arc assignment source was removed.".into()),
        }));
    }
    let current = capture(conn, input.node_id)?;
    if current == input {
        return Ok(None);
    }
    Ok(Some(ScriptImpactCause {
        dependency_id: dependency_id(event),
        input: SemanticDependencyEndpoint::TimelineNode {
            node_id: input.node_id,
        },
        consumed_revision_event_id: input.revision_event_id.unwrap_or(event),
        current_revision_event_id: current.revision_event_id,
        reason: ScriptImpactReason::ContextChanged,
        input_excerpt: Some("Selected clip story arc assignment changed.".into()),
    }))
}

/// Allow withdrawal of live arc prose only when the immutable consumed membership
/// and current canonical receipt explicitly prove that arc left this clip.
pub(crate) fn permits_withdrawal(
    conn: &Connection,
    node: NodeId,
    generation: ChangeEventId,
    arc: eidetic_core::story::arc::ArcId,
) -> Result<bool, HistoryStoreError> {
    let Some(previous) = recorded(conn, generation)? else {
        return Ok(false);
    };
    let current = capture(conn, node)?;
    Ok(current.revision_event_id.is_some()
        && previous.node_id == node
        && previous.arc_ids.contains(&arc)
        && !current.arc_ids.contains(&arc))
}

/// Prompt evidence comes exclusively from the captured proposal binding.
pub(crate) fn append_prompt(user: &mut String, binding: &ScriptImpactProposalBinding) {
    for (label, input, fields) in [
        (
            "ORIGINAL RECORDED STORY ARC ASSIGNMENT (historical, not current guidance)",
            binding.arc_membership_previous.as_ref(),
            binding.arc_previous_inputs.as_deref(),
        ),
        (
            "CURRENT SELECTED CLIP STORY ARC ASSIGNMENT",
            binding.arc_membership_current.as_ref(),
            binding.arc_inputs.as_deref(),
        ),
    ] {
        let Some(input) = input else {
            continue;
        };
        user.push_str(&format!("\n{label} (clip {}):\n", input.node_id.0));
        if input.arc_ids.is_empty() {
            user.push_str("(no arcs)\n");
        }
        for arc in &input.arc_ids {
            let name = fields
                .unwrap_or_default()
                .iter()
                .find(|field| field.arc_id == *arc && field.field == StoryArcPromptField::Name)
                .map(|field| field.value.as_str())
                .unwrap_or("name unavailable");
            user.push_str(&format!("{} name: {}\n", arc.0, name));
        }
    }
    if binding.arc_membership_previous.is_some() && binding.arc_membership_current.is_some() {
        user.push_str("Arcs absent from the current assignment no longer guide this update.\n");
    }
}

#[cfg(test)]
#[path = "timeline_arc_membership_tests.rs"]
mod tests;

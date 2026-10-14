//! Exact tagged-arc prompt fields, with existing sparse history as authority.
use std::collections::BTreeSet;

use eidetic_core::contracts::*;
use eidetic_core::story::arc::{ArcId, StoryArc};
use eidetic_core::timeline::node::NodeId;
use rusqlite::{Connection, OptionalExtension, params};

use crate::history_store::HistoryStoreError;
use crate::semantic_dependency_store::{
    DependencyDirection, DependencyEndpointFilter, SemanticDependencyFilter,
};

fn invalid() -> HistoryStoreError {
    HistoryStoreError::InvalidValue("Story arc input does not match canonical field history".into())
}

pub(crate) fn endpoint(input: &StoryArcFieldInput) -> SemanticDependencyEndpoint {
    SemanticDependencyEndpoint::StoryArcField {
        arc_id: input.arc_id,
        field: input.field,
    }
}

/// Read tags, exact prompt values and owned field revisions in one snapshot.
/// Previously consumed descriptions retain an empty value when explicitly cleared.
pub(crate) fn capture(
    conn: &Connection,
    node: NodeId,
    previous: &[StoryArcFieldInput],
) -> Result<(Vec<StoryArc>, Vec<StoryArcFieldInput>), HistoryStoreError> {
    let mut stmt = conn.prepare("SELECT arc_id FROM node_arcs WHERE node_id=?1")?;
    let ids = stmt
        .query_map([node.0.to_string()], |row| row.get::<_, String>(0))?
        .collect::<Result<BTreeSet<_>, _>>()?;
    let arcs: Vec<_> = crate::story_arc_store::load_arcs(conn)?
        .into_iter()
        .filter(|arc| ids.contains(&arc.id.0.to_string()))
        .collect();
    let mut inputs = Vec::new();
    for arc in &arcs {
        for (field, value) in [
            (StoryArcPromptField::Name, arc.name.clone()),
            (
                StoryArcPromptField::ArcType,
                serde_json::to_string(&arc.arc_type)?,
            ),
            (StoryArcPromptField::Description, arc.description.clone()),
        ] {
            if field == StoryArcPromptField::Description
                && value.is_empty()
                && !previous.iter().any(|old| {
                    old.arc_id == arc.id && old.field == field && old.revision_event_id.is_some()
                })
            {
                continue;
            }
            let input = StoryArcFieldInput {
                arc_id: arc.id,
                field,
                value,
                revision_event_id: owned_revision(conn, arc.id, field)?,
            };
            validate_history(conn, &input)?;
            inputs.push(input);
        }
    }
    Ok((arcs, inputs))
}

fn owned_revision(
    conn: &Connection,
    arc: ArcId,
    field: StoryArcPromptField,
) -> Result<Option<ChangeEventId>, HistoryStoreError> {
    let event: Option<String> = conn.query_row(
        "SELECT r.change_event_id FROM object_revisions r JOIN change_events e ON e.id=r.change_event_id
         WHERE r.object_kind='story_arc' AND r.object_id=?1 AND (r.operation='delete'
         OR EXISTS(SELECT 1 FROM object_revision_fields f WHERE f.revision_id=r.id AND f.field_key=?2))
         ORDER BY e.rowid DESC,r.rowid DESC LIMIT 1",
        params![arc.0.to_string(), field.as_str()], |row| row.get(0)).optional()?;
    event
        .map(|event| {
            uuid::Uuid::parse_str(&event)
                .map(ChangeEventId)
                .map_err(|error| HistoryStoreError::InvalidId(error.to_string()))
        })
        .transpose()
}

pub(crate) fn current_revision(
    conn: &Connection,
    endpoint: &SemanticDependencyEndpoint,
) -> Result<Option<ChangeEventId>, HistoryStoreError> {
    let SemanticDependencyEndpoint::StoryArcField { arc_id, field } = endpoint else {
        return Ok(None);
    };
    let Some(arc) = crate::story_arc_store::load_arc(conn, arc_id)? else {
        return Ok(None);
    };
    let revision_event_id = owned_revision(conn, *arc_id, *field)?;
    let value = match field {
        StoryArcPromptField::Name => arc.name,
        StoryArcPromptField::Description => arc.description,
        StoryArcPromptField::ArcType => serde_json::to_string(&arc.arc_type)?,
    };
    validate_history(
        conn,
        &StoryArcFieldInput {
            arc_id: *arc_id,
            field: *field,
            value,
            revision_event_id,
        },
    )?;
    Ok(revision_event_id)
}

/// Historical consumption survives later edits/deletion and never binds to latest.
pub(crate) fn validate_history(
    conn: &Connection,
    input: &StoryArcFieldInput,
) -> Result<(), HistoryStoreError> {
    if input.field == StoryArcPromptField::ArcType {
        let value: eidetic_core::story::arc::ArcType = serde_json::from_str(&input.value)?;
        if serde_json::to_string(&value)? != input.value {
            return Err(invalid());
        }
    }
    let Some(event) = input.revision_event_id else {
        return Ok(());
    };
    let owns: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id
         WHERE r.object_kind='story_arc' AND r.object_id=?1 AND r.change_event_id=?2
         AND f.field_key=?3 AND f.new_type='text' AND f.new_text=?4)",
        params![input.arc_id.0.to_string(), event.0.to_string(), input.field.as_str(), input.value],
        |row| row.get(0))?;
    let history = crate::revision_projection::load_object_field_projection_at_event(
        conn,
        ObjectKind::StoryArc,
        &input.arc_id.0.to_string(),
        event,
    )?
    .ok_or_else(invalid)?;
    if !owns
        || history.deleted
        || history.fields.get(input.field.as_str()) != Some(&FieldValue::Text(input.value.clone()))
    {
        return Err(invalid());
    }
    Ok(())
}

pub(crate) fn excerpt(
    conn: &Connection,
    endpoint: &SemanticDependencyEndpoint,
    event: ChangeEventId,
) -> Result<Option<String>, HistoryStoreError> {
    let SemanticDependencyEndpoint::StoryArcField { arc_id, field } = endpoint else {
        return Ok(None);
    };
    let history = crate::revision_projection::load_object_field_projection_at_event(
        conn,
        ObjectKind::StoryArc,
        &arc_id.0.to_string(),
        event,
    )?;
    Ok(
        history.and_then(|history| match history.fields.get(field.as_str()) {
            Some(FieldValue::Text(value)) => Some(value.chars().take(120).collect()),
            _ => None,
        }),
    )
}

/// Read the actual durable receipt, including an accepted preview's consumption.
/// Missing legacy receipt stays missing; never reconstruct it from today's tags.
pub(crate) fn recorded(
    conn: &Connection,
    event: ChangeEventId,
) -> Result<Option<Vec<StoryArcFieldInput>>, HistoryStoreError> {
    let (kind, json): (String, String) = conn.query_row(
        "SELECT c.payload_type,c.payload_json FROM commands c JOIN change_events e ON e.command_id=c.id WHERE e.id=?1",
        [event.0.to_string()], |row| Ok((row.get(0)?, row.get(1)?)))?;
    match kind.as_str() {
        "script.generate_block" => {
            Ok(serde_json::from_str::<GenerateScriptBlockCommand>(&json)?.arc_inputs)
        }
        "semantic.propagation_accept" => {
            let command: AcceptPropagationProposalCommand = serde_json::from_str(&json)?;
            let json: String = conn.query_row(
                "SELECT binding_json FROM script_impact_proposal_bindings WHERE proposal_id=?1",
                [command.proposal_id.as_str()],
                |row| row.get(0),
            )?;
            Ok(serde_json::from_str::<ScriptImpactProposalBinding>(&json)?.arc_inputs)
        }
        _ => Ok(None),
    }
}

pub(crate) struct ArcPreviewInputs {
    pub previous: Option<Vec<StoryArcFieldInput>>,
    pub current: Vec<StoryArcFieldInput>,
    pub absent: Vec<(ArcId, ChangeEventId)>,
}

pub(crate) fn preview_inputs(
    conn: &Connection,
    node: NodeId,
    generation: ChangeEventId,
    segment: &ScriptSegmentId,
) -> Result<ArcPreviewInputs, HistoryStoreError> {
    let previous = recorded(conn, generation)?;
    let (arcs, inputs) = capture(conn, node, previous.as_deref().unwrap_or_default())?;
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
    let mut absent = Vec::new();
    let mut seen = BTreeSet::new();
    for dependency in dependencies.payload.dependencies {
        if !dependency
            .revision_binding
            .as_ref()
            .is_some_and(|binding| binding.source_revision_event_id == generation)
        {
            continue;
        }
        let SemanticDependencyEndpoint::StoryArcField { arc_id, field } = dependency.target else {
            continue;
        };
        if crate::story_arc_store::load_arc(conn, &arc_id)?.is_some() {
            // Known-empty applicability has a dependency but supplies no prose.
            // Check actual tags, independently of which fields capture supplied.
            if !arcs.iter().any(|arc| arc.id == arc_id)
                && !crate::timeline_arc_membership::permits_withdrawal(
                    conn, node, generation, arc_id,
                )?
            {
                return Err(HistoryStoreError::InvalidValue("Story arc review source is outside current tags; restore its context before previewing".into()));
            }
        } else if seen.insert(arc_id.0.to_string()) {
            let event = owned_revision(conn, arc_id, field)?.ok_or_else(invalid)?;
            let history = crate::revision_projection::load_object_field_projection_at_event(
                conn,
                ObjectKind::StoryArc,
                &arc_id.0.to_string(),
                event,
            )?
            .ok_or_else(invalid)?;
            if !history.deleted {
                return Err(invalid());
            }
            absent.push((arc_id, event));
        }
    }
    Ok(ArcPreviewInputs {
        previous,
        current: inputs,
        absent,
    })
}

/// Same semantic representation as the generation prompt, with exact field values.
pub(crate) fn append_prompt(user: &mut String, inputs: &[StoryArcFieldInput]) {
    if !inputs.is_empty() {
        user.push_str("\nCURRENT TAGGED STORY ARC FIELDS:\n");
        for input in inputs {
            user.push_str(&format!(
                "{} {}: {}\n",
                input.arc_id.0,
                input.field.as_str(),
                input.value
            ));
        }
    }
}

#[cfg(test)]
#[path = "story_arc_lineage_tests.rs"]
mod tests;

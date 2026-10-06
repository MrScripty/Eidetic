//! Entity-scoped untimed field membership in existing generation/proposal history.
use std::collections::BTreeSet;

use eidetic_core::contracts::*;
use eidetic_core::timeline::node::NodeId;
use rusqlite::{Connection, OptionalExtension};

use crate::history_store::HistoryStoreError;

fn invalid() -> HistoryStoreError {
    HistoryStoreError::InvalidValue(
        "Bible context membership does not match captured provenance".into(),
    )
}

fn explicit_nodes(
    conn: &Connection,
    node: NodeId,
) -> Result<BTreeSet<BibleGraphNodeId>, HistoryStoreError> {
    Ok(
        crate::context_influence_store::load_latest_context_influence_records(conn, node)?
            .into_iter()
            .filter(|record| {
                record.influence_kind == ContextInfluenceKind::Direct
                    && matches!(
                        record.provenance,
                        ContextInfluenceProvenance::UserSelected
                            | ContextInfluenceProvenance::AiSelected
                    )
            })
            .filter_map(|record| record.bible_node_id)
            .collect(),
    )
}

// Reuse append-only Bible/context history. Unrelated edits can stale a pending
// request, but never create membership review unless scoped field IDs differ.
fn epoch(conn: &Connection) -> Result<Option<ChangeEventId>, HistoryStoreError> {
    let value: Option<String> = conn.query_row(
        "SELECT r.change_event_id FROM object_revisions r JOIN change_events e ON e.id=r.change_event_id
         WHERE r.object_kind IN ('bible_node','bible_part_field','bible_snapshot','context_evaluation','context_influence')
         ORDER BY e.rowid DESC,r.rowid DESC LIMIT 1", [], |row| row.get(0)).optional()?;
    value
        .map(|value| {
            uuid::Uuid::parse_str(&value)
                .map(ChangeEventId)
                .map_err(|error| HistoryStoreError::InvalidId(error.to_string()))
        })
        .transpose()
}

fn fields(
    conn: &Connection,
    nodes: &BTreeSet<BibleGraphNodeId>,
    retained: &BTreeSet<BibleGraphFieldId>,
) -> Result<BTreeSet<BibleGraphFieldId>, HistoryStoreError> {
    let mut result = BTreeSet::new();
    for node in nodes {
        let mut statement = conn.prepare(
            "SELECT f.id,EXISTS(SELECT 1 FROM bible_graph_snapshot_fields sf
               JOIN bible_graph_snapshots ss ON ss.id=sf.snapshot_id
               WHERE ss.node_id=n.id AND sf.part_key=p.part_key AND sf.field_key=f.field_key)
             FROM bible_graph_fields f JOIN bible_graph_parts p ON p.id=f.part_id JOIN bible_graph_nodes n ON n.id=p.node_id
             WHERE n.id=?1 AND n.deleted_event_id IS NULL AND p.deleted_event_id IS NULL
               AND f.deleted_event_id IS NULL AND f.value_type IS NOT NULL")?;
        let rows = statement.query_map([node.as_str()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?))
        })?;
        for row in rows {
            let (id, timed) = row?;
            let id = BibleGraphFieldId::new(id).map_err(|_| invalid())?;
            // A later snapshot alone must not remove a previously watched field.
            // Snapshot keys, including tombstones, never introduce new membership.
            if !timed || retained.contains(&id) {
                result.insert(id);
            }
        }
    }
    Ok(result)
}

// Preserve an entity's proven relevance across accepted clears. The latest
// generation is authoritative; unknown legacy provenance is never filled in.
fn prior_record(conn: &Connection, node: NodeId) -> Result<Option<Recorded>, HistoryStoreError> {
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='script_generations')",[],|row|row.get(0))?;
    if !exists {
        return Ok(None);
    }
    let event: Option<String> = conn.query_row("SELECT g.event_id FROM script_generations g JOIN script_segments s ON s.id=g.segment_id WHERE s.source_node_id=?1 AND s.document_id='script.document.main' ORDER BY g.rowid DESC LIMIT 1",[node.0.to_string()],|row|row.get(0)).optional()?;
    let Some(event) = event else {
        return Ok(None);
    };
    let event = uuid::Uuid::parse_str(&event)
        .map(ChangeEventId)
        .map_err(|_| invalid())?;
    recorded(conn, event)
}

fn prior_nodes(
    conn: &Connection,
    node: NodeId,
) -> Result<BTreeSet<BibleGraphNodeId>, HistoryStoreError> {
    Ok(prior_record(conn, node)?
        .map(|record| match record.scope {
            Some(scope) => scope.node_ids.into_iter().collect(),
            None => record
                .inputs
                .unwrap_or_default()
                .into_iter()
                .map(|input| input.node_id)
                .collect(),
        })
        .unwrap_or_default())
}

fn prior_fields(
    conn: &Connection,
    node: NodeId,
) -> Result<BTreeSet<BibleGraphFieldId>, HistoryStoreError> {
    Ok(prior_record(conn, node)?
        .map(|record| match record.scope {
            Some(scope) => scope.field_ids.into_iter().collect(),
            None => record
                .inputs
                .unwrap_or_default()
                .into_iter()
                .map(|input| input.field_id)
                .collect(),
        })
        .unwrap_or_default())
}

pub(crate) fn capture(
    conn: &Connection,
    node: NodeId,
    inputs: &[BibleFieldInput],
) -> Result<BibleContextScope, HistoryStoreError> {
    let mut nodes: BTreeSet<_> = inputs.iter().map(|input| input.node_id.clone()).collect();
    nodes.extend(prior_nodes(conn, node)?);
    nodes.extend(explicit_nodes(conn, node)?);
    // Canonical membership is separate from actual value consumption. Retained
    // entities can be outside the current resolver limit; their unchanged fields
    // must not appear newly entered immediately after a reviewed acceptance.
    let field_ids = fields(conn, &nodes, &prior_fields(conn, node)?)?
        .into_iter()
        .collect();
    Ok(BibleContextScope {
        node_id: node,
        node_ids: nodes.into_iter().collect(),
        field_ids,
        revision_event_id: epoch(conn)?,
    })
}

fn dependency_id(event: ChangeEventId) -> SemanticDependencyId {
    SemanticDependencyId::new(format!("generation.{}.bible_context", event.0))
        .expect("nonempty dependency id")
}

pub(crate) fn dependency(
    command: &GenerateScriptBlockCommand,
    event: ChangeEventId,
    created_at_ms: u64,
) -> Option<SemanticDependency> {
    command
        .bible_context_scope
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
            rationale: Some(
                "Untimed Bible field membership on consumed or explicitly assigned entities".into(),
            ),
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
    let Some(scope) = &command.bible_context_scope else {
        return Ok(());
    };
    let inputs = command.bible_inputs.as_ref().ok_or_else(invalid)?;
    let nodes: BTreeSet<_> = scope.node_ids.iter().cloned().collect();
    let ids: BTreeSet<_> = scope.field_ids.iter().cloned().collect();
    if command.block.source_node_id.as_deref() != Some(scope.node_id.0.to_string().as_str())
        || nodes.len() != scope.node_ids.len()
        || ids.len() != scope.field_ids.len()
        || !inputs.iter().all(|input| nodes.contains(&input.node_id))
    {
        return Err(invalid());
    }
    if let Some(event) = scope.revision_event_id {
        let valid: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM object_revisions WHERE change_event_id=?1
             AND object_kind IN ('bible_node','bible_part_field','bible_snapshot','context_evaluation','context_influence'))",
            [event.0.to_string()], |row| row.get(0))?;
        if !valid {
            return Err(invalid());
        }
        for input in inputs {
            let before: bool = conn.query_row("SELECT (SELECT rowid FROM change_events WHERE id=?1) <= (SELECT rowid FROM change_events WHERE id=?2)",rusqlite::params![input.revision_event_id.0.to_string(),event.0.to_string()],|row|row.get(0))?;
            if !before {
                return Err(invalid());
            }
        }
        if historical_fields(conn, &nodes, &prior_fields(conn, scope.node_id)?, event)? != ids {
            return Err(invalid());
        }
        let mut expected: BTreeSet<_> = inputs.iter().map(|input| input.node_id.clone()).collect();
        expected.extend(prior_nodes(conn, scope.node_id)?);
        expected.extend(historical_explicit_nodes(conn, scope.node_id, event)?);
        if expected != nodes {
            return Err(invalid());
        }
    } else if !nodes.is_empty() || !ids.is_empty() {
        return Err(invalid());
    }
    Ok(())
}

struct Recorded {
    scope: Option<BibleContextScope>,
    inputs: Option<Vec<BibleFieldInput>>,
}

fn recorded(
    conn: &Connection,
    event: ChangeEventId,
) -> Result<Option<Recorded>, HistoryStoreError> {
    let (kind, json): (String,String) = conn.query_row(
        "SELECT c.payload_type,c.payload_json FROM commands c JOIN change_events e ON e.command_id=c.id WHERE e.id=?1",
        [event.0.to_string()], |row| Ok((row.get(0)?,row.get(1)?)))?;
    match kind.as_str() {
        "script.generate_block" => {
            let command: GenerateScriptBlockCommand = serde_json::from_str(&json)?;
            Ok(Some(Recorded {
                scope: command.bible_context_scope,
                inputs: command.bible_inputs,
            }))
        }
        "semantic.propagation_accept" => {
            let command: AcceptPropagationProposalCommand = serde_json::from_str(&json)?;
            let json: String = conn.query_row(
                "SELECT binding_json FROM script_impact_proposal_bindings WHERE proposal_id=?1",
                [command.proposal_id.as_str()],
                |row| row.get(0),
            )?;
            let binding: ScriptImpactProposalBinding = serde_json::from_str(&json)?;
            Ok(Some(Recorded {
                scope: binding.bible_context_scope,
                inputs: Some(binding.bible_inputs),
            }))
        }
        _ => Ok(None),
    }
}

fn historical_explicit_nodes(
    conn: &Connection,
    node: NodeId,
    at: ChangeEventId,
) -> Result<BTreeSet<BibleGraphNodeId>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT r.bible_node_id FROM context_influence_records r
         WHERE r.bible_node_id IS NOT NULL AND r.influence_kind='direct'
           AND r.provenance IN ('user_selected','ai_selected')
           AND r.evaluation_id=(SELECT v.id FROM context_evaluations v
             JOIN change_events e ON e.id=v.created_event_id WHERE v.target_node_id=?1
             AND e.rowid <= (SELECT rowid FROM change_events WHERE id=?2)
             ORDER BY v.created_at_ms DESC,v.id DESC LIMIT 1)",
    )?;
    let rows = statement.query_map(
        rusqlite::params![node.0.to_string(), at.0.to_string()],
        |row| row.get::<_, String>(0),
    )?;
    rows.map(|row| BibleGraphNodeId::new(row?).map_err(|_| invalid()))
        .collect()
}

fn historical_projection(
    conn: &Connection,
    kind: ObjectKind,
    id: &str,
    at: ChangeEventId,
) -> Result<Option<crate::revision_projection::ObjectFieldProjection>, HistoryStoreError> {
    let kind_json = serde_json::to_value(&kind)?;
    let kind_name = kind_json.as_str().ok_or_else(invalid)?;
    let event:Option<String>=conn.query_row("SELECT r.change_event_id FROM object_revisions r JOIN change_events e ON e.id=r.change_event_id WHERE r.object_kind=?1 AND r.object_id=?2 AND e.rowid <= (SELECT rowid FROM change_events WHERE id=?3) ORDER BY e.rowid DESC,r.rowid DESC LIMIT 1",rusqlite::params![kind_name,id,at.0.to_string()],|row|row.get(0)).optional()?;
    let Some(event) = event else {
        return Ok(None);
    };
    let event = uuid::Uuid::parse_str(&event)
        .map(ChangeEventId)
        .map_err(|_| invalid())?;
    crate::revision_projection::load_object_field_projection_at_event(conn, kind, id, event)
}

fn was_absent(
    conn: &Connection,
    id: &BibleGraphFieldId,
    at: ChangeEventId,
) -> Result<bool, HistoryStoreError> {
    Ok(
        historical_projection(conn, ObjectKind::BiblePartField, id.as_str(), at)?
            .is_none_or(|field| field.deleted || !field.fields.contains_key("value")),
    )
}

fn historical_fields(
    conn: &Connection,
    nodes: &BTreeSet<BibleGraphNodeId>,
    retained: &BTreeSet<BibleGraphFieldId>,
    at: ChangeEventId,
) -> Result<BTreeSet<BibleGraphFieldId>, HistoryStoreError> {
    let mut result = BTreeSet::new();
    for node in nodes {
        if historical_projection(conn, ObjectKind::BibleNode, node.as_str(), at)?
            .is_none_or(|node| node.deleted)
        {
            continue;
        }
        let mut statement=conn.prepare("SELECT f.id,p.part_key,f.field_key,p.deleted_event_id FROM bible_graph_fields f JOIN bible_graph_parts p ON p.id=f.part_id WHERE p.node_id=?1")?;
        let rows = statement.query_map([node.as_str()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })?;
        for row in rows {
            let (id, part, key, part_deleted) = row?;
            if let Some(deleted) = part_deleted {
                let deleted:bool=conn.query_row("SELECT (SELECT rowid FROM change_events WHERE id=?1) <= (SELECT rowid FROM change_events WHERE id=?2)",rusqlite::params![deleted,at.0.to_string()],|row|row.get(0))?;
                if deleted {
                    continue;
                }
            }
            let id = BibleGraphFieldId::new(id).map_err(|_| invalid())?;
            if was_absent(conn, &id, at)? {
                continue;
            }
            let timed:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM bible_graph_snapshot_fields f JOIN bible_graph_snapshots s ON s.id=f.snapshot_id JOIN object_revisions r ON r.object_kind='bible_snapshot' AND r.object_id=s.id JOIN object_revision_fields d ON d.revision_id=r.id AND d.field_key=('field.' || f.part_key || '.' || f.field_key) JOIN change_events e ON e.id=r.change_event_id WHERE s.node_id=?1 AND f.part_key=?2 AND f.field_key=?3 AND e.rowid <= (SELECT rowid FROM change_events WHERE id=?4))",rusqlite::params![node.as_str(),part,key,at.0.to_string()],|row|row.get(0))?;
            if !timed || retained.contains(&id) {
                result.insert(id);
            }
        }
    }
    Ok(result)
}

fn label(conn: &Connection, id: &BibleGraphFieldId) -> Result<String, HistoryStoreError> {
    Ok(conn.query_row("SELECT n.name || '.' || p.part_key || '.' || f.field_key FROM bible_graph_fields f JOIN bible_graph_parts p ON p.id=f.part_id JOIN bible_graph_nodes n ON n.id=p.node_id WHERE f.id=?1", [id.as_str()], |row| row.get::<_,String>(0)).optional()?.unwrap_or_else(|| id.as_str().into()))
}

struct MembershipDelta {
    node: NodeId,
    revision: ChangeEventId,
    entered: Vec<BibleGraphFieldId>,
    removed: Vec<BibleGraphFieldId>,
}

fn delta(
    conn: &Connection,
    event: ChangeEventId,
    segment: &ScriptSegmentId,
) -> Result<Option<MembershipDelta>, HistoryStoreError> {
    let Some(before) = recorded(conn, event)? else {
        return Ok(None);
    };
    let Some(inputs) = before.inputs else {
        return Ok(None);
    };
    let node: Option<String> = conn.query_row(
        "SELECT source_node_id FROM script_segments WHERE id=?1",
        [segment.as_str()],
        |row| row.get(0),
    )?;
    let Some(node) = node else {
        return Ok(None);
    };
    let node = NodeId(uuid::Uuid::parse_str(&node).map_err(|_| invalid())?);
    let (mut nodes, old, revision) = match &before.scope {
        Some(scope) => (
            scope.node_ids.iter().cloned().collect::<BTreeSet<_>>(),
            scope.field_ids.iter().cloned().collect::<BTreeSet<_>>(),
            scope.revision_event_id.unwrap_or(event),
        ),
        None => {
            // These entities are proven consumed, not an inferred complete scope.
            let nodes: BTreeSet<_> = inputs.iter().map(|input| input.node_id.clone()).collect();
            if nodes.is_empty() {
                return Ok(None);
            }
            (
                nodes,
                inputs.iter().map(|input| input.field_id.clone()).collect(),
                event,
            )
        }
    };
    if before.scope.is_some() {
        nodes.extend(explicit_nodes(conn, node)?);
    }
    let mut current = fields(conn, &nodes, &old)?;
    if before.scope.is_none() {
        // A missing old receipt never becomes fabricated historical consumption.
        // Only new IDs proven absent/null at generation can create membership.
        let mut filtered = old.intersection(&current).cloned().collect::<BTreeSet<_>>();
        for id in current.difference(&old) {
            if was_absent(conn, id, event)? {
                filtered.insert(id.clone());
            }
        }
        current = filtered;
    }
    if current == old {
        return Ok(None);
    }
    Ok(Some(MembershipDelta {
        node,
        revision,
        entered: current.difference(&old).cloned().collect(),
        removed: old.difference(&current).cloned().collect(),
    }))
}

// Membership IDs alone cannot acknowledge a newly entered value. Every bound
// preview must supply those values, even when another impact cause was selected.
// Reuse the exact delta that owns review, including conservative legacy absence.
pub(crate) fn validate_preview_inputs(
    conn: &Connection,
    event: ChangeEventId,
    segment: &ScriptSegmentId,
    inputs: &[BibleFieldInput],
) -> Result<(), HistoryStoreError> {
    let Some(delta) = delta(conn, event, segment)? else {
        return Ok(());
    };
    let supplied: BTreeSet<_> = inputs.iter().map(|input| &input.field_id).collect();
    if delta.entered.iter().any(|id| !supplied.contains(id)) {
        return Err(HistoryStoreError::InvalidValue(
            "Bible membership review values are outside the current resolved context; restore their context before previewing".into(),
        ));
    }
    Ok(())
}

pub(crate) fn cause(
    conn: &Connection,
    event: ChangeEventId,
    segment: &ScriptSegmentId,
) -> Result<Option<ScriptImpactCause>, HistoryStoreError> {
    let Some(delta) = delta(conn, event, segment)? else {
        return Ok(None);
    };
    let entered = delta
        .entered
        .iter()
        .map(|id| label(conn, id))
        .collect::<Result<Vec<_>, _>>()?;
    let removed = delta
        .removed
        .iter()
        .map(|id| label(conn, id))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(ScriptImpactCause {
        dependency_id: dependency_id(event),
        input: SemanticDependencyEndpoint::TimelineNode {
            node_id: delta.node,
        },
        consumed_revision_event_id: delta.revision,
        current_revision_event_id: epoch(conn)?,
        reason: ScriptImpactReason::ContextChanged,
        input_excerpt: Some(format!(
            "Untimed Bible fields entered: {}; removed: {}",
            entered.join(", "),
            removed.join(", ")
        )),
    }))
}

#[cfg(test)]
#[path = "bible_context_scope_tests.rs"]
mod tests;

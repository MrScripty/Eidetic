//! Child-plan screenplay receipts live in the existing creation command history.
use eidetic_core::contracts::{ChangeEventId, ObjectKind, ScriptContextBlock, ScriptContextScope};
use eidetic_core::timeline::node::{NodeArc, NodeId, StoryNode};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::history_store::{self, HistoryStoreError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChildPlanMemory {
    nodes: Vec<(StoryNode, Option<ChangeEventId>)>,
    parent_arcs: Vec<NodeArc>,
    pub(crate) script_inputs: Vec<ScriptContextBlock>,
    script_scope: ScriptContextScope,
}

pub(crate) fn capture(
    conn: &Connection,
    parent: NodeId,
) -> Result<ChildPlanMemory, HistoryStoreError> {
    capture_before_event(conn, parent, None)
}

fn capture_before_event(
    conn: &Connection,
    parent: NodeId,
    excluded: Option<ChangeEventId>,
) -> Result<ChildPlanMemory, HistoryStoreError> {
    let nodes = crate::timeline_node_store::load_nodes(conn)?;
    let target = nodes
        .iter()
        .find(|node| node.id == parent)
        .ok_or_else(stale)?;
    let mut selected = BTreeSet::from([parent.0]);
    loop {
        let before = selected.len();
        for node in &nodes {
            if node.parent_id.is_some_and(|id| selected.contains(&id.0)) {
                selected.insert(node.id.0);
            }
        }
        if selected.len() == before {
            break;
        }
    }
    let mut bound = nodes
        .iter()
        .filter(|node| selected.contains(&node.id.0))
        .map(|node| {
            let event = history_store::load_revisions_for_object(
                conn,
                ObjectKind::TimelineNode,
                &node.id.0.to_string(),
            )?
            .iter()
            .rev()
            .find(|revision| Some(revision.change_event_id) != excluded)
            .map(|revision| revision.change_event_id);
            Ok((node.clone(), event))
        })
        .collect::<Result<Vec<_>, HistoryStoreError>>()?;
    bound.sort_by_key(|(node, _)| node.id.0);
    let mut parent_arcs = crate::timeline_node_store::load_node_arcs(conn)?
        .into_iter()
        .filter(|tag| tag.node_id == parent)
        .collect::<Vec<_>>();
    parent_arcs.sort_by_key(|tag| tag.arc_id.0);
    let range = target.time_range;
    let script_inputs =
        crate::ai_script_context::load_script_context(conn, parent, range.start_ms, range.end_ms)?;
    let script_scope = crate::script_context_scope::capture(
        conn,
        parent,
        range.start_ms,
        range.end_ms,
        &script_inputs,
    )?;
    Ok(ChildPlanMemory {
        nodes: bound,
        parent_arcs,
        script_inputs,
        script_scope,
    })
}

pub(crate) fn validate(
    conn: &Connection,
    parent: NodeId,
    expected: &ChildPlanMemory,
) -> Result<(), HistoryStoreError> {
    if serde_json::to_value(capture(conn, parent)?)? != serde_json::to_value(expected)? {
        return Err(stale());
    }
    Ok(())
}

pub(crate) fn validate_before_apply(
    conn: &Connection,
    parent: NodeId,
    expected: &ChildPlanMemory,
    event: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    // The history writer has inserted this command's delete/create revisions
    // before its current-state closure. Exclude only that in-flight event while
    // checking the committed source receipts under the same writer transaction.
    if serde_json::to_value(capture_before_event(conn, parent, Some(event))?)?
        != serde_json::to_value(expected)?
    {
        return Err(stale());
    }
    Ok(())
}

pub(crate) fn validate_parent(
    expected: &ChildPlanMemory,
    parent: &StoryNode,
) -> Result<(), HistoryStoreError> {
    let parent = serde_json::to_value(parent)?;
    let found = expected
        .nodes
        .iter()
        .map(|(node, _)| serde_json::to_value(node))
        .collect::<Result<Vec<_>, _>>()?;
    if !found.contains(&parent) {
        return Err(stale());
    }
    Ok(())
}

fn stale() -> HistoryStoreError {
    HistoryStoreError::InvalidValue(
        "Child plan story context changed; generate and review a fresh plan before accepting"
            .into(),
    )
}

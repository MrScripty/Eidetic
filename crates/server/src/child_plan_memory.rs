//! Child-plan source receipts live in the existing creation command history.
use eidetic_core::ai::backend::ChildPlanBibleContext;
use eidetic_core::contracts::{ChangeEventId, ObjectKind, ScriptContextBlock, ScriptContextScope};
use eidetic_core::timeline::node::{NodeArc, NodeId, StoryNode};
use eidetic_core::timeline::relationship::Relationship;
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
    // Absent on older pending plans: unknown edge custody must require a fresh
    // review, not silently assert that no relationships were consumed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    relationships: Option<ChildPlanRelationships>,
    // Missing on older proposals: unknown Bible custody needs fresh review.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bible: Option<crate::child_plan_bible_memory::ChildPlanBibleMemory>,
}

impl ChildPlanMemory {
    pub(crate) fn bible_context(&self) -> Option<ChildPlanBibleContext> {
        self.bible.as_ref().map(|memory| memory.receipt.clone())
    }

    fn story_time_ms(&self) -> Option<u64> {
        self.bible
            .as_ref()
            .and_then(|memory| memory.receipt.context.payload.story_time_ms)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChildPlanRelationships {
    current: Vec<Relationship>,
    revisions: Vec<(String, Option<ChangeEventId>)>,
}

pub(crate) fn capture(
    conn: &Connection,
    parent: NodeId,
    story_time_ms: Option<u64>,
) -> Result<ChildPlanMemory, HistoryStoreError> {
    capture_before_event(conn, parent, story_time_ms, None)
}

fn capture_before_event(
    conn: &Connection,
    parent: NodeId,
    story_time_ms: Option<u64>,
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
    selected.remove(&parent.0);
    let relationships = capture_relationships(conn, &selected, excluded)?;
    Ok(ChildPlanMemory {
        nodes: bound,
        parent_arcs,
        script_inputs,
        script_scope,
        relationships: Some(relationships),
        bible: Some(crate::child_plan_bible_memory::capture(
            conn,
            parent,
            story_time_ms,
        )?),
    })
}

fn capture_relationships(
    conn: &Connection,
    descendants: &BTreeSet<uuid::Uuid>,
    excluded: Option<ChangeEventId>,
) -> Result<ChildPlanRelationships, HistoryStoreError> {
    // Acceptance removes exactly the edges touching replaced descendants,
    // including edges from outside the subtree; parent-only edges survive.
    let mut current = crate::timeline_relationship_store::load_relationships(conn)?
        .into_iter()
        .filter(|edge| {
            descendants.contains(&edge.from_node.0) || descendants.contains(&edge.to_node.0)
        })
        .collect::<Vec<_>>();
    current.sort_by_key(|edge| edge.id.0);
    let mut ids = current
        .iter()
        .map(|edge| edge.id.0.to_string())
        .collect::<BTreeSet<_>>();
    let descendants = descendants
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    // Include deleted identities from endpoint deltas so add/delete and
    // delete/recreate ABA cannot masquerade as an unchanged canonical edge set.
    let mut statement = conn.prepare(
        "SELECT DISTINCT r.object_id, f.old_text, f.new_text
         FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id
         WHERE r.object_kind=?1 AND f.field_key IN ('from_node_id','to_node_id')
           AND (?2 IS NULL OR r.change_event_id<>?2)",
    )?;
    let rows = statement.query_map(
        rusqlite::params![
            serde_json::to_value(ObjectKind::TimelineRelationship)?.as_str(),
            excluded.map(|id| id.0.to_string())
        ],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        },
    )?;
    for row in rows {
        let (id, old, new) = row?;
        if old
            .iter()
            .chain(new.iter())
            .any(|endpoint| descendants.contains(endpoint))
        {
            ids.insert(id);
        }
    }
    let revisions = ids
        .into_iter()
        .map(|id| {
            // Existing history reads committed event order, not timestamps or
            // event-local sort_order. Exclude only this apply's in-flight revisions.
            let latest = history_store::load_revisions_for_object(
                conn,
                ObjectKind::TimelineRelationship,
                &id,
            )?
            .into_iter()
            .rev()
            .find(|revision| Some(revision.change_event_id) != excluded)
            .map(|revision| revision.change_event_id);
            Ok((id, latest))
        })
        .collect::<Result<Vec<_>, HistoryStoreError>>()?;
    Ok(ChildPlanRelationships { current, revisions })
}

pub(crate) fn validate(
    conn: &Connection,
    parent: NodeId,
    expected: &ChildPlanMemory,
) -> Result<(), HistoryStoreError> {
    let current = capture(conn, parent, expected.story_time_ms()).map_err(changed_source)?;
    if !unchanged(current, expected)? {
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
    if !unchanged(
        capture_before_event(conn, parent, expected.story_time_ms(), Some(event))
            .map_err(changed_source)?,
        expected,
    )? {
        return Err(stale());
    }
    Ok(())
}

fn changed_source(error: HistoryStoreError) -> HistoryStoreError {
    // A recorded proposal whose canonical sources no longer resolve is a
    // definite writer refusal, including newly conflicting timed assertions.
    // Preserve infrastructure failures as uncertain rather than relabel them.
    match error {
        HistoryStoreError::InvalidValue(_) => stale(),
        other => other,
    }
}

fn unchanged(
    mut current: ChildPlanMemory,
    expected: &ChildPlanMemory,
) -> Result<bool, HistoryStoreError> {
    let Some(bible) = current.bible.take() else {
        return Ok(false);
    };
    let Some(expected_bible) = &expected.bible else {
        return Ok(false);
    };
    if !bible.unchanged(expected_bible) {
        return Ok(false);
    }
    let mut expected = expected.clone();
    expected.bible = None;
    Ok(serde_json::to_value(current)? == serde_json::to_value(expected)?)
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

pub(crate) fn stale() -> HistoryStoreError {
    HistoryStoreError::InvalidValue(
        "Child plan story context changed; generate and review a fresh plan before accepting"
            .into(),
    )
}

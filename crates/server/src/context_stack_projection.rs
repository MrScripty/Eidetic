//! Shared saved-screenplay evidence for timeline and agent context consumers.
use eidetic_core::contracts::{
    ContextEvaluation, ContextStackProjection, ObjectKind, ProjectionEnvelope, ProjectionVersion,
};
use eidetic_core::timeline::node::NodeId;
use rusqlite::Connection;

use crate::history_store::{self, HistoryStoreError};

pub(crate) fn load(
    conn: &Connection,
    target: NodeId,
) -> Result<Option<ProjectionEnvelope<ContextStackProjection>>, HistoryStoreError> {
    crate::context_influence_store::create_schema(conn)?;
    crate::script_store::create_schema(conn)?;
    if conn.is_autocommit() {
        let tx = conn.unchecked_transaction()?;
        let projection = load_in_snapshot(&tx, target)?;
        tx.commit()?;
        Ok(projection)
    } else {
        load_in_snapshot(conn, target)
    }
}

fn load_in_snapshot(
    conn: &Connection,
    target: NodeId,
) -> Result<Option<ProjectionEnvelope<ContextStackProjection>>, HistoryStoreError> {
    let nodes = crate::timeline_node_store::load_node_ancestor_stack(conn, target)?;
    let Some(target_node) = nodes.iter().find(|node| node.id == target) else {
        return Ok(None);
    };
    let Some(mut projection) = ContextStackProjection::from_nodes(&nodes, target) else {
        return Ok(None);
    };
    let ids: Vec<_> = projection
        .layers
        .iter()
        .map(|layer| layer.node_id)
        .collect();
    let evaluations = crate::context_influence_store::load_latest_context_evaluations(conn, &ids)?;
    apply_distilled_context_overrides(&mut projection, &evaluations);
    projection.script_context = Some(crate::ai_script_context::load_script_context(
        conn,
        target,
        target_node.time_range.start_ms,
        target_node.time_range.end_ms,
    )?);
    let summary = history_store::load_revision_summary_for_kinds(
        conn,
        &[
            ObjectKind::TimelineNode,
            ObjectKind::ContextEvaluation,
            ObjectKind::ContextInfluence,
            ObjectKind::ScriptDocument,
            ObjectKind::ScriptSegment,
            ObjectKind::ScriptBlock,
        ],
    )?;
    Ok(Some(match summary.latest_change_event_id {
        Some(event) => ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            event,
            projection,
        ),
        None => ProjectionEnvelope::initial(projection),
    }))
}

pub(crate) fn apply_distilled_context_overrides(
    projection: &mut ContextStackProjection,
    evaluations: &[ContextEvaluation],
) {
    for layer in &mut projection.layers {
        let Some(evaluation) = evaluations
            .iter()
            .find(|value| value.target_node_id == layer.node_id)
        else {
            continue;
        };
        if let Some(context) = &evaluation.distilled_context {
            layer.distilled_context = Some(context.clone());
        }
    }
}

#[cfg(test)]
#[path = "context_stack_projection_tests.rs"]
mod tests;

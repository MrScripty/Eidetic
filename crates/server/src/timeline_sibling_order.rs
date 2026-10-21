//! Owned sibling snapshots and immutable command receipts; no parallel order state.
use crate::{history_store::HistoryStoreError, timeline_node_store};
use eidetic_core::{
    contracts::*,
    timeline::{node::NodeId, timing::TimeRange},
};
use rusqlite::{Connection, OptionalExtension, params};
fn invalid(message: &str) -> HistoryStoreError {
    HistoryStoreError::InvalidValue(message.into())
}
pub(crate) fn capture(
    conn: &Connection,
    node: NodeId,
) -> Result<TimelineSiblingOrderRead, HistoryStoreError> {
    capture_excluding(conn, node, None)
}
pub(crate) fn capture_excluding(
    conn: &Connection,
    node: NodeId,
    exclude: Option<ChangeEventId>,
) -> Result<TimelineSiblingOrderRead, HistoryStoreError> {
    let nodes = timeline_node_store::load_nodes(conn)?;
    let selected = nodes
        .iter()
        .find(|n| n.id == node)
        .ok_or_else(|| invalid("Clip no longer exists"))?;
    let mut siblings = nodes
        .iter()
        .filter(|n| n.parent_id == selected.parent_id && n.level == selected.level)
        .map(|n| {
            Ok(TimelineSiblingPlacementRead {
                node_id: n.id,
                name: n.name.clone(),
                start_ms: n.time_range.start_ms,
                end_ms: n.time_range.end_ms,
                sort_order: n.sort_order,
                revision_event_id: timeline_node_store::latest_node_event(conn, n.id, exclude)?,
            })
        })
        .collect::<Result<Vec<_>, HistoryStoreError>>()?;
    siblings.sort_by_key(|n| (n.start_ms, n.node_id.0));
    let membership:Option<String>=conn.query_row("SELECT r.change_event_id FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id JOIN change_events e ON e.id=r.change_event_id WHERE r.object_kind='timeline_node' AND f.field_key='parent_id' AND (f.old_text=?1 OR f.new_text=?1) AND (?2 IS NULL OR e.id != ?2) ORDER BY e.rowid DESC,r.rowid DESC LIMIT 1",params![selected.parent_id.map(|id|id.0.to_string()),exclude.map(|id|id.0.to_string())],|r|r.get(0)).optional()?;
    Ok(TimelineSiblingOrderRead {
        node_id: node,
        parent_id: selected.parent_id,
        level: selected.level,
        membership_revision_event_id: membership
            .map(|id| {
                uuid::Uuid::parse_str(&id)
                    .map(ChangeEventId)
                    .map_err(|e| HistoryStoreError::InvalidId(e.to_string()))
            })
            .transpose()?,
        siblings,
    })
}
pub(crate) fn command_receipt(
    conn: &Connection,
    command: &CommandEnvelope<ReorderTimelineSiblingCommand>,
) -> Result<TimelineSiblingOrderRead, HistoryStoreError> {
    let (event,kind,json):(String,String,String) = conn.query_row("SELECT e.id,c.payload_type,c.payload_json FROM change_events e JOIN commands c ON c.id=e.command_id WHERE e.command_id=?1",[command.id.0.to_string()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
    if kind != "timeline.sibling_reorder"
        || serde_json::from_str::<ReorderTimelineSiblingCommand>(&json)? != command.payload
    {
        return Err(invalid(
            "Reorder receipt command differs from owned history",
        ));
    }
    let event = ChangeEventId(
        uuid::Uuid::parse_str(&event).map_err(|e| HistoryStoreError::InvalidId(e.to_string()))?,
    );
    let mut receipt = command.payload.expected.clone();
    let a = receipt
        .siblings
        .iter()
        .position(|n| n.node_id == command.payload.node_id)
        .ok_or_else(|| invalid("Missing original selected clip"))?;
    let b = receipt
        .siblings
        .iter()
        .position(|n| n.node_id == command.payload.neighbor_id)
        .ok_or_else(|| invalid("Missing original neighbor"))?;
    let (first, second) = if a < b { (a, b) } else { (b, a) };
    let (a_range, b_range) = eidetic_core::timeline::sibling_reorder::swapped_adjacent_ranges(
        TimeRange::new(
            receipt.siblings[first].start_ms,
            receipt.siblings[first].end_ms,
        )
        .map_err(|e| invalid(&e.to_string()))?,
        TimeRange::new(
            receipt.siblings[second].start_ms,
            receipt.siblings[second].end_ms,
        )
        .map_err(|e| invalid(&e.to_string()))?,
    )
    .map_err(|e| invalid(&e.to_string()))?;
    let first_order = receipt.siblings[first].sort_order;
    receipt.siblings[first].sort_order = receipt.siblings[second].sort_order;
    receipt.siblings[second].sort_order = first_order;
    for (index, range) in [(first, a_range), (second, b_range)] {
        let n = &mut receipt.siblings[index];
        n.start_ms = range.start_ms;
        n.end_ms = range.end_ms;
        n.revision_event_id = Some(event);
        let stored_order:Option<i64>=conn.query_row("SELECT f.new_integer FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id WHERE r.object_kind='timeline_node' AND r.object_id=?1 AND r.change_event_id=?2 AND f.field_key='sort_order'",params![n.node_id.0.to_string(),event.0.to_string()],|r|r.get(0)).optional()?;
        let previous_order = command
            .payload
            .expected
            .siblings
            .iter()
            .find(|old| old.node_id == n.node_id)
            .expect("original receipt node")
            .sort_order;
        if stored_order != (previous_order != n.sort_order).then_some(i64::from(n.sort_order)) {
            return Err(invalid(
                "Reorder order receipt differs from owned sparse history",
            ));
        }
        for (field, value) in [("start_ms", range.start_ms), ("end_ms", range.end_ms)] {
            let stored:i64=conn.query_row("SELECT f.new_integer FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id WHERE r.object_kind='timeline_node' AND r.object_id=?1 AND r.change_event_id=?2 AND f.field_key=?3",params![n.node_id.0.to_string(),event.0.to_string(),field],|r|r.get(0))?;
            if u64::try_from(stored).ok() != Some(value) {
                return Err(invalid("Reorder receipt differs from owned history"));
            }
        }
    }
    receipt.siblings.sort_by_key(|n| (n.start_ms, n.node_id.0));
    Ok(receipt)
}
#[cfg(test)]
#[path = "timeline_sibling_order_tests.rs"]
mod tests;

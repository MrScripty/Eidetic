//! Atomic two-subtree authoring and source-bound screenplay placement.
use crate::{
    history_store::{self, HistoryStoreError, RecordChangeOutcome},
    timeline_command::TimelineCommandError,
    timeline_node_store,
};
use eidetic_core::{Project, contracts::*};
use rusqlite::Connection;
pub(crate) fn record_reorder_timeline_sibling_history(
    conn: &mut Connection,
    project: &Project,
    command: &CommandEnvelope<ReorderTimelineSiblingCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, TimelineCommandError> {
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "timeline.sibling_reorder")?
    {
        return Ok(outcome);
    }
    if command.payload.expected.node_id != command.payload.node_id {
        return Err(HistoryStoreError::InvalidValue(
            "Sibling snapshot belongs to another clip".into(),
        )
        .into());
    }
    let mut next = project.timeline.clone();
    next.reorder_adjacent_nodes(command.payload.node_id, command.payload.neighbor_id)?;
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!(
            "reorder {} with {}",
            project.timeline.node(command.payload.node_id)?.name,
            project.timeline.node(command.payload.neighbor_id)?.name
        ),
    )
    .with_created_at_ms(created_at_ms);
    let mut revisions = Vec::new();
    let mut ranges = Vec::new();
    for old in &project.timeline.nodes {
        let new = next.node(old.id)?;
        if old.time_range != new.time_range || old.sort_order != new.sort_order {
            let mut revision = ObjectRevision::new(
                ObjectKind::TimelineNode,
                old.id.0.to_string(),
                event.id,
                RevisionOperation::Update,
            );
            if old.time_range != new.time_range {
                for (field, old_value, new_value) in [
                    ("start_ms", old.time_range.start_ms, new.time_range.start_ms),
                    ("end_ms", old.time_range.end_ms, new.time_range.end_ms),
                ] {
                    revision = revision.with_field(FieldDelta::new(
                        field,
                        Some(FieldValue::Integer(crate::script_store_codec::to_i64(
                            old_value, field,
                        )?)),
                        Some(FieldValue::Integer(crate::script_store_codec::to_i64(
                            new_value, field,
                        )?)),
                    ));
                }
                ranges.push((new.id, new.time_range));
            }
            if old.sort_order != new.sort_order {
                revision = revision.with_field(FieldDelta::new(
                    "sort_order",
                    Some(FieldValue::Integer(i64::from(old.sort_order))),
                    Some(FieldValue::Integer(i64::from(new.sort_order))),
                ));
            }
            revisions.push(revision);
        }
    }
    crate::script_store::create_schema(conn)?;
    Ok(history_store::record_change_with(
        conn,
        command,
        "timeline.sibling_reorder",
        &event,
        &revisions,
        |tx| {
            crate::timeline_command_guard::validate_current_timeline(tx, &project.timeline)?;
            if crate::timeline_sibling_order::capture_excluding(
                tx,
                command.payload.node_id,
                Some(event.id),
            )? != command.payload.expected
            {
                return Err(HistoryStoreError::InvalidValue("Sibling placement changed; discard the reorder draft and read current siblings before applying".into()));
            }
            timeline_node_store::upsert_nodes_in_transaction(tx, &next.nodes)?;
            crate::timeline_script_placement::sync_in_transaction(
                tx,
                &ranges,
                event.id,
                revisions.len(),
            )
        },
    )?)
}

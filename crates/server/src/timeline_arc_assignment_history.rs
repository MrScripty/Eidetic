//! Explicit membership authoring through existing node_arcs and sparse revisions.
use crate::{
    history_store::{self, HistoryStoreError, RecordChangeOutcome},
    timeline_arc_membership,
    timeline_command::TimelineCommandError,
};
use eidetic_core::{Project, contracts::*};
use rusqlite::{Connection, params};

pub(crate) fn record_set_timeline_node_arcs_history(
    conn: &mut Connection,
    project: &Project,
    command: &CommandEnvelope<SetTimelineNodeArcsCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, TimelineCommandError> {
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "timeline.node_arcs")?
    {
        return Ok(outcome);
    }
    let node = project.timeline.node(command.payload.node_id)?;
    let mut arc_ids = command.payload.arc_ids.clone();
    arc_ids.sort_by_key(|id| id.0);
    if arc_ids.windows(2).any(|ids| ids[0] == ids[1]) || command.payload.expected.node_id != node.id
    {
        return Err(HistoryStoreError::InvalidValue("Invalid clip arc assignment".into()).into());
    }
    let mut previous = project
        .timeline
        .node_arcs
        .iter()
        .filter(|tag| tag.node_id == node.id)
        .map(|tag| tag.arc_id)
        .collect::<Vec<_>>();
    previous.sort_by_key(|id| id.0);
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("assign story arcs to {}", node.name),
    )
    .with_created_at_ms(created_at_ms);
    let revision = ObjectRevision::new(
        ObjectKind::TimelineNode,
        node.id.0.to_string(),
        event.id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "arc_ids",
        Some(FieldValue::Text(
            serde_json::to_string(&previous).map_err(HistoryStoreError::from)?,
        )),
        Some(FieldValue::Text(
            serde_json::to_string(&arc_ids).map_err(HistoryStoreError::from)?,
        )),
    ));
    Ok(history_store::record_change_with(
        conn,
        command,
        "timeline.node_arcs",
        &event,
        &[revision],
        |tx| {
            crate::timeline_command_guard::validate_current_timeline(tx, &project.timeline)?;
            let current = timeline_arc_membership::capture_excluding(tx, node.id, Some(event.id))?;
            if current != command.payload.expected {
                return Err(HistoryStoreError::InvalidValue(
                    "arc assignment changed; read current arcs before saving".into(),
                ));
            }
            if node.locked {
                return Err(HistoryStoreError::InvalidValue(
                    "clip is locked; unlock before assigning story arcs".into(),
                ));
            }
            if current.arc_ids == arc_ids {
                return Err(HistoryStoreError::InvalidValue(
                    "arc assignment is unchanged".into(),
                ));
            }
            for arc in &arc_ids {
                if crate::story_arc_store::load_arc(tx, arc)?.is_none() {
                    return Err(HistoryStoreError::InvalidValue(
                        "Selected story arc no longer exists; read current arcs before saving"
                            .into(),
                    ));
                }
            }
            tx.execute(
                "DELETE FROM node_arcs WHERE node_id=?1",
                [node.id.0.to_string()],
            )?;
            for arc in &arc_ids {
                tx.execute(
                    "INSERT INTO node_arcs(node_id,arc_id) VALUES(?1,?2)",
                    params![node.id.0.to_string(), arc.0.to_string()],
                )?;
            }
            Ok(())
        },
    )?)
}

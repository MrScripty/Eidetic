use eidetic_core::Project;
use eidetic_core::contracts::{
    ApplyTimelineChildrenCommand, ChangeEventId, CommandEnvelope, ObjectRevision,
};
use rusqlite::{Connection, Transaction};

use crate::child_plan_store;
use crate::history_store::HistoryStoreError;
use crate::timeline_command::TimelineCommandError;

pub(crate) fn child_plan_apply_revision(
    conn: &Connection,
    project: &Project,
    command: &CommandEnvelope<ApplyTimelineChildrenCommand>,
    event_id: ChangeEventId,
) -> Result<Option<ObjectRevision>, TimelineCommandError> {
    let Some(child_plan_id) = command.payload.child_plan_id.as_ref() else {
        return Ok(None);
    };
    let parent = project.timeline.node(command.payload.parent_id)?;
    let expected_child_level = parent.level.child_level().ok_or_else(|| {
        eidetic_core::Error::InvalidHierarchy(format!(
            "{} nodes cannot have children",
            parent.level
        ))
    })?;
    child_plan_store::validate_child_plan_for_apply(
        conn,
        child_plan_id,
        command.payload.parent_id,
        expected_child_level,
    )
    .map_err(child_plan_error)?;
    Ok(Some(child_plan_store::applied_child_plan_revision(
        child_plan_id,
        event_id,
    )?))
}

pub(crate) fn mark_child_plan_applied_in_transaction(
    tx: &Transaction<'_>,
    command: &CommandEnvelope<ApplyTimelineChildrenCommand>,
) -> Result<(), HistoryStoreError> {
    if let Some(child_plan_id) = command.payload.child_plan_id.as_ref() {
        child_plan_store::mark_child_plan_applied_in_transaction(tx, child_plan_id)?;
    }
    Ok(())
}

fn child_plan_error(error: child_plan_store::ChildPlanStoreError) -> TimelineCommandError {
    TimelineCommandError::History(HistoryStoreError::InvalidValue(error.to_string()))
}

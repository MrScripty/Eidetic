use eidetic_core::ai::backend::RejectChildPlanCommand;
use eidetic_core::contracts::{ChangeEvent, ChangeEventKind, CommandEnvelope};
use rusqlite::Connection;

use crate::child_plan_store::{self, ChildPlanStoreError};
use crate::history_store::{self, RecordChangeOutcome};

pub(crate) fn record_reject_child_plan(
    conn: &mut Connection,
    command: &CommandEnvelope<RejectChildPlanCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, ChildPlanStoreError> {
    child_plan_store::create_schema(conn)?;
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "ai.child_plan_reject")?
    {
        return Ok(outcome);
    }
    child_plan_store::validate_child_plan_pending(conn, &command.payload.plan_id)?;

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalRejected,
        format!("reject child plan {}", command.payload.plan_id.as_str()),
    )
    .with_created_at_ms(created_at_ms);
    let revision = child_plan_store::rejected_child_plan_revision(
        &command.payload.plan_id,
        command.payload.reason.as_deref(),
        event.id,
    )?;

    Ok(history_store::record_change_with(
        conn,
        command,
        "ai.child_plan_reject",
        &event,
        &[revision],
        |tx| {
            child_plan_store::mark_child_plan_rejected_in_transaction(tx, &command.payload.plan_id)
        },
    )?)
}

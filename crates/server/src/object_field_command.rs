use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, FieldDelta, ObjectRevision, RevisionOperation,
    SetObjectFieldCommand,
};
use rusqlite::Connection;

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::revision_projection::{self, ObjectFieldProjection};

pub(crate) fn apply_set_object_field(
    conn: &mut Connection,
    command: &CommandEnvelope<SetObjectFieldCommand>,
    created_at_ms: u64,
) -> Result<(RecordChangeOutcome, ObjectFieldProjection), ObjectFieldCommandError> {
    validate_command(&command.payload)?;

    let before = revision_projection::load_object_field_projection(
        conn,
        command.payload.object_kind.clone(),
        &command.payload.object_id,
    )?;
    let old_value = before.fields.get(&command.payload.field_key).cloned();
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("set {}", command.payload.field_key),
    )
    .with_created_at_ms(created_at_ms);
    let revision = ObjectRevision::new(
        command.payload.object_kind.clone(),
        command.payload.object_id.clone(),
        event.id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        command.payload.field_key.clone(),
        old_value,
        command.payload.value.clone(),
    ));

    let outcome =
        history_store::record_change(conn, command, "object.set_field", &event, &[revision])?;
    let projection = revision_projection::load_object_field_projection(
        conn,
        command.payload.object_kind.clone(),
        &command.payload.object_id,
    )?;

    Ok((outcome, projection))
}

fn validate_command(command: &SetObjectFieldCommand) -> Result<(), ObjectFieldCommandError> {
    if command.object_id.trim().is_empty() {
        return Err(ObjectFieldCommandError::InvalidCommand(
            "object_id is required".to_string(),
        ));
    }
    if command.field_key.trim().is_empty() {
        return Err(ObjectFieldCommandError::InvalidCommand(
            "field_key is required".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ObjectFieldCommandError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),
    #[error(transparent)]
    History(#[from] HistoryStoreError),
}

#[cfg(test)]
#[path = "object_field_command_tests.rs"]
mod tests;

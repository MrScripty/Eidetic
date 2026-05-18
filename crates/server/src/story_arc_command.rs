use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, CreateStoryArcCommand, DeleteStoryArcCommand,
    FieldDelta, FieldValue, ObjectKind, ObjectRevision, RevisionOperation,
    SetStoryArcMetadataCommand,
};
use eidetic_core::story::arc::{ArcType, StoryArc};
use rusqlite::Connection;
use thiserror::Error;

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::story_arc_store;

pub(crate) fn record_create_story_arc_history(
    conn: &mut Connection,
    command: &CommandEnvelope<CreateStoryArcCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, StoryArcCommandError> {
    if let Some(outcome) = history_store::check_recorded_command(conn, command, "story_arc.create")?
    {
        return Ok(outcome);
    }
    validate_arc_name(&command.payload.name)?;
    if story_arc_store::load_arc(conn, &command.payload.arc_id)?.is_some() {
        return Err(StoryArcCommandError::InvalidCommand(format!(
            "story arc already exists: {}",
            command.payload.arc_id.0
        )));
    }

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("create story arc {}", command.payload.name),
    )
    .with_created_at_ms(created_at_ms);
    let revision = ObjectRevision::new(
        ObjectKind::StoryArc,
        command.payload.arc_id.0.to_string(),
        event.id,
        RevisionOperation::Create,
    )
    .with_field(FieldDelta::new(
        "name",
        None,
        Some(FieldValue::Text(command.payload.name.clone())),
    ))
    .with_field(FieldDelta::new(
        "description",
        None,
        Some(FieldValue::Text(command.payload.description.clone())),
    ))
    .with_field(FieldDelta::new(
        "parent_arc_id",
        None,
        command
            .payload
            .parent_arc_id
            .map(|arc_id| FieldValue::Text(arc_id.0.to_string())),
    ))
    .with_field(FieldDelta::new(
        "arc_type",
        None,
        Some(FieldValue::Text(encode_arc_type(
            &command.payload.arc_type,
        )?)),
    ))
    .with_field(FieldDelta::new(
        "color_r",
        None,
        Some(FieldValue::Integer(i64::from(command.payload.color.r))),
    ))
    .with_field(FieldDelta::new(
        "color_g",
        None,
        Some(FieldValue::Integer(i64::from(command.payload.color.g))),
    ))
    .with_field(FieldDelta::new(
        "color_b",
        None,
        Some(FieldValue::Integer(i64::from(command.payload.color.b))),
    ));

    let arc = StoryArc {
        id: command.payload.arc_id,
        parent_arc_id: command.payload.parent_arc_id,
        name: command.payload.name.clone(),
        description: command.payload.description.clone(),
        arc_type: command.payload.arc_type.clone(),
        color: command.payload.color,
    };

    Ok(history_store::record_change_with(
        conn,
        command,
        "story_arc.create",
        &event,
        &[revision],
        |tx| story_arc_store::insert_arc_in_transaction(tx, &arc, event.id),
    )?)
}

pub(crate) fn record_set_story_arc_metadata_history(
    conn: &mut Connection,
    command: &CommandEnvelope<SetStoryArcMetadataCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, StoryArcCommandError> {
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "story_arc.set_metadata")?
    {
        return Ok(outcome);
    }
    if let Some(name) = &command.payload.name {
        validate_arc_name(name)?;
    }

    let arc = story_arc_store::load_arc(conn, &command.payload.arc_id)?
        .ok_or_else(|| StoryArcCommandError::NotFound("story arc not found".to_string()))?;

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("update story arc {}", arc.name),
    )
    .with_created_at_ms(created_at_ms);
    let mut revision = ObjectRevision::new(
        ObjectKind::StoryArc,
        command.payload.arc_id.0.to_string(),
        event.id,
        RevisionOperation::Update,
    );

    if let Some(name) = &command.payload.name {
        revision = revision.with_field(FieldDelta::new(
            "name",
            Some(FieldValue::Text(arc.name.clone())),
            Some(FieldValue::Text(name.clone())),
        ));
    }
    if let Some(description) = &command.payload.description {
        revision = revision.with_field(FieldDelta::new(
            "description",
            Some(FieldValue::Text(arc.description.clone())),
            Some(FieldValue::Text(description.clone())),
        ));
    }
    if let Some(arc_type) = &command.payload.arc_type {
        revision = revision.with_field(FieldDelta::new(
            "arc_type",
            Some(FieldValue::Text(encode_arc_type(&arc.arc_type)?)),
            Some(FieldValue::Text(encode_arc_type(arc_type)?)),
        ));
    }
    if let Some(color) = command.payload.color {
        revision = revision
            .with_field(FieldDelta::new(
                "color_r",
                Some(FieldValue::Integer(i64::from(arc.color.r))),
                Some(FieldValue::Integer(i64::from(color.r))),
            ))
            .with_field(FieldDelta::new(
                "color_g",
                Some(FieldValue::Integer(i64::from(arc.color.g))),
                Some(FieldValue::Integer(i64::from(color.g))),
            ))
            .with_field(FieldDelta::new(
                "color_b",
                Some(FieldValue::Integer(i64::from(arc.color.b))),
                Some(FieldValue::Integer(i64::from(color.b))),
            ));
    }

    Ok(history_store::record_change_with(
        conn,
        command,
        "story_arc.set_metadata",
        &event,
        &[revision],
        |tx| story_arc_store::update_arc_metadata_in_transaction(tx, &command.payload, event.id),
    )?)
}

pub(crate) fn record_delete_story_arc_history(
    conn: &mut Connection,
    command: &CommandEnvelope<DeleteStoryArcCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, StoryArcCommandError> {
    if let Some(outcome) = history_store::check_recorded_command(conn, command, "story_arc.delete")?
    {
        return Ok(outcome);
    }
    let arc = story_arc_store::load_arc(conn, &command.payload.arc_id)?;
    let event = ChangeEvent::new(command.id, ChangeEventKind::UserEdit, "delete story arc")
        .with_created_at_ms(created_at_ms);
    let mut revision = ObjectRevision::new(
        ObjectKind::StoryArc,
        command.payload.arc_id.0.to_string(),
        event.id,
        RevisionOperation::Delete,
    );

    if let Some(arc) = arc {
        revision = revision
            .with_field(FieldDelta::new(
                "name",
                Some(FieldValue::Text(arc.name.clone())),
                None,
            ))
            .with_field(FieldDelta::new(
                "description",
                Some(FieldValue::Text(arc.description.clone())),
                None,
            ))
            .with_field(FieldDelta::new(
                "parent_arc_id",
                arc.parent_arc_id
                    .map(|arc_id| FieldValue::Text(arc_id.0.to_string())),
                None,
            ))
            .with_field(FieldDelta::new(
                "arc_type",
                Some(FieldValue::Text(encode_arc_type(&arc.arc_type)?)),
                None,
            ))
            .with_field(FieldDelta::new(
                "color_r",
                Some(FieldValue::Integer(i64::from(arc.color.r))),
                None,
            ))
            .with_field(FieldDelta::new(
                "color_g",
                Some(FieldValue::Integer(i64::from(arc.color.g))),
                None,
            ))
            .with_field(FieldDelta::new(
                "color_b",
                Some(FieldValue::Integer(i64::from(arc.color.b))),
                None,
            ));
    }

    Ok(history_store::record_change_with(
        conn,
        command,
        "story_arc.delete",
        &event,
        &[revision],
        |tx| story_arc_store::delete_arc_in_transaction(tx, &command.payload.arc_id, event.id),
    )?)
}

fn validate_arc_name(name: &str) -> Result<(), StoryArcCommandError> {
    if name.trim().is_empty() {
        return Err(StoryArcCommandError::InvalidCommand(
            "arc name is required".to_string(),
        ));
    }
    Ok(())
}

fn encode_arc_type(arc_type: &ArcType) -> Result<String, StoryArcCommandError> {
    serde_json::to_string(arc_type)
        .map_err(|error| StoryArcCommandError::InvalidCommand(format!("invalid arc type: {error}")))
}

#[derive(Debug, Error)]
pub(crate) enum StoryArcCommandError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),
    #[error("{0}")]
    NotFound(String),
    #[error(transparent)]
    History(#[from] HistoryStoreError),
}

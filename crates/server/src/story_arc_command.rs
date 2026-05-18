use eidetic_core::Project;
use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, CreateStoryArcCommand, DeleteStoryArcCommand,
    FieldDelta, FieldValue, ObjectKind, ObjectRevision, ProjectionEnvelope, RevisionOperation,
    SetStoryArcMetadataCommand, StoryArcListProjection,
};
use eidetic_core::story::arc::{ArcType, StoryArc};
use rusqlite::Connection;
use thiserror::Error;

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};

pub(crate) fn record_create_story_arc_history(
    conn: &mut Connection,
    project: &Project,
    command: &CommandEnvelope<CreateStoryArcCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, StoryArcCommandError> {
    if let Some(outcome) = history_store::check_recorded_command(conn, command, "story_arc.create")?
    {
        return Ok(outcome);
    }
    validate_arc_name(&command.payload.name)?;
    if project
        .arcs
        .iter()
        .any(|arc| arc.id == command.payload.arc_id)
    {
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

    Ok(history_store::record_change(
        conn,
        command,
        "story_arc.create",
        &event,
        &[revision],
    )?)
}

pub(crate) fn record_set_story_arc_metadata_history(
    conn: &mut Connection,
    project: &Project,
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

    let arc = project
        .arcs
        .iter()
        .find(|arc| arc.id == command.payload.arc_id)
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

    Ok(history_store::record_change(
        conn,
        command,
        "story_arc.set_metadata",
        &event,
        &[revision],
    )?)
}

pub(crate) fn record_delete_story_arc_history(
    conn: &mut Connection,
    project: &Project,
    command: &CommandEnvelope<DeleteStoryArcCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, StoryArcCommandError> {
    if let Some(outcome) = history_store::check_recorded_command(conn, command, "story_arc.delete")?
    {
        return Ok(outcome);
    }
    let arc = project
        .arcs
        .iter()
        .find(|arc| arc.id == command.payload.arc_id);
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

    Ok(history_store::record_change(
        conn,
        command,
        "story_arc.delete",
        &event,
        &[revision],
    )?)
}

pub(crate) fn apply_create_story_arc(
    project: &mut Project,
    command: &CommandEnvelope<CreateStoryArcCommand>,
) -> Result<ProjectionEnvelope<StoryArcListProjection>, StoryArcCommandError> {
    validate_arc_name(&command.payload.name)?;
    if project
        .arcs
        .iter()
        .any(|arc| arc.id == command.payload.arc_id)
    {
        return Err(StoryArcCommandError::InvalidCommand(format!(
            "story arc already exists: {}",
            command.payload.arc_id.0
        )));
    }

    project.arcs.push(StoryArc {
        id: command.payload.arc_id,
        parent_arc_id: command.payload.parent_arc_id,
        name: command.payload.name.clone(),
        description: command.payload.description.clone(),
        arc_type: command.payload.arc_type.clone(),
        color: command.payload.color,
    });

    Ok(list_projection(project))
}

pub(crate) fn apply_set_story_arc_metadata(
    project: &mut Project,
    command: &CommandEnvelope<SetStoryArcMetadataCommand>,
) -> Result<ProjectionEnvelope<StoryArcListProjection>, StoryArcCommandError> {
    if let Some(name) = &command.payload.name {
        validate_arc_name(name)?;
    }

    let arc = project
        .arcs
        .iter_mut()
        .find(|arc| arc.id == command.payload.arc_id)
        .ok_or_else(|| StoryArcCommandError::NotFound("story arc not found".to_string()))?;

    if let Some(name) = &command.payload.name {
        arc.name = name.clone();
    }
    if let Some(description) = &command.payload.description {
        arc.description = description.clone();
    }
    if let Some(arc_type) = &command.payload.arc_type {
        arc.arc_type = arc_type.clone();
    }
    if let Some(color) = command.payload.color {
        arc.color = color;
    }

    Ok(list_projection(project))
}

pub(crate) fn apply_delete_story_arc(
    project: &mut Project,
    command: &CommandEnvelope<DeleteStoryArcCommand>,
) -> Result<(bool, ProjectionEnvelope<StoryArcListProjection>), StoryArcCommandError> {
    let before = project.arcs.len();
    project.arcs.retain(|arc| arc.id != command.payload.arc_id);
    Ok((before != project.arcs.len(), list_projection(project)))
}

fn validate_arc_name(name: &str) -> Result<(), StoryArcCommandError> {
    if name.trim().is_empty() {
        return Err(StoryArcCommandError::InvalidCommand(
            "arc name is required".to_string(),
        ));
    }
    Ok(())
}

fn list_projection(project: &Project) -> ProjectionEnvelope<StoryArcListProjection> {
    ProjectionEnvelope::initial(StoryArcListProjection::from_arcs(&project.arcs))
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

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::Template;
    use eidetic_core::contracts::CommandId;
    use eidetic_core::story::arc::{ArcId, ArcType, Color};

    #[test]
    fn create_story_arc_returns_projection_with_new_arc() {
        let mut project = Template::MultiCam.build_project("Arc Command Test");
        let arc_id = ArcId::new();
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: CreateStoryArcCommand {
                arc_id,
                parent_arc_id: None,
                name: "Mystery".to_string(),
                description: "The central investigation".to_string(),
                arc_type: ArcType::APlot,
                color: Color::A_PLOT,
            },
        };

        let projection = apply_create_story_arc(&mut project, &command).unwrap();

        assert!(project.arcs.iter().any(|arc| arc.id == arc_id));
        assert!(
            projection
                .payload
                .arcs
                .iter()
                .any(|arc| arc.id == arc_id && arc.name == "Mystery")
        );
    }

    #[test]
    fn create_story_arc_rejects_blank_name() {
        let mut project = Template::MultiCam.build_project("Arc Command Test");
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: CreateStoryArcCommand {
                arc_id: ArcId::new(),
                parent_arc_id: None,
                name: " ".to_string(),
                description: String::new(),
                arc_type: ArcType::BPlot,
                color: Color::B_PLOT,
            },
        };

        assert!(apply_create_story_arc(&mut project, &command).is_err());
    }

    #[test]
    fn set_story_arc_metadata_updates_existing_arc() {
        let mut project = Template::MultiCam.build_project("Arc Command Test");
        let arc_id = project.arcs[0].id;
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: SetStoryArcMetadataCommand {
                arc_id,
                name: Some("Renamed".to_string()),
                description: Some("Updated description".to_string()),
                arc_type: None,
                color: Some(Color::new(1, 2, 3)),
            },
        };

        let projection = apply_set_story_arc_metadata(&mut project, &command).unwrap();

        let arc = projection
            .payload
            .arcs
            .iter()
            .find(|arc| arc.id == arc_id)
            .expect("updated arc");
        assert_eq!(arc.name, "Renamed");
        assert_eq!(arc.description, "Updated description");
        assert_eq!(arc.color, Color::new(1, 2, 3));
    }

    #[test]
    fn delete_story_arc_returns_deleted_flag() {
        let mut project = Template::MultiCam.build_project("Arc Command Test");
        let arc_id = project.arcs[0].id;
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: DeleteStoryArcCommand { arc_id },
        };

        let (deleted, projection) = apply_delete_story_arc(&mut project, &command).unwrap();

        assert!(deleted);
        assert!(projection.payload.arcs.iter().all(|arc| arc.id != arc_id));
    }
}

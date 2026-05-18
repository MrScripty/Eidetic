use eidetic_core::Project;
use eidetic_core::contracts::{
    CommandEnvelope, CreateStoryArcCommand, DeleteStoryArcCommand, ProjectionEnvelope,
    SetStoryArcMetadataCommand, StoryArcListProjection,
};
use eidetic_core::story::arc::StoryArc;
use thiserror::Error;

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

#[derive(Debug, Error)]
pub(crate) enum StoryArcCommandError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),
    #[error("{0}")]
    NotFound(String),
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

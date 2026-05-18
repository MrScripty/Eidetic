use eidetic_core::contracts::{
    CommandEnvelope, ProjectionEnvelope, SetTimelineNodeRangeCommand, TimelineRenderProjection,
};
use eidetic_core::project::Project;
use eidetic_core::timeline::timing::TimeRange;
use thiserror::Error;

pub(crate) fn apply_set_timeline_node_range(
    project: &mut Project,
    command: &CommandEnvelope<SetTimelineNodeRangeCommand>,
) -> Result<ProjectionEnvelope<TimelineRenderProjection>, TimelineCommandError> {
    let range = TimeRange::new(command.payload.start_ms, command.payload.end_ms)
        .map_err(TimelineCommandError::Core)?;
    project
        .timeline
        .resize_node(command.payload.node_id, range)
        .map_err(TimelineCommandError::Core)?;

    Ok(ProjectionEnvelope::initial(
        TimelineRenderProjection::from_timeline(&project.timeline),
    ))
}

#[derive(Debug, Error)]
pub(crate) enum TimelineCommandError {
    #[error("{0}")]
    Core(#[from] eidetic_core::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::Template;
    use eidetic_core::contracts::CommandId;

    #[test]
    fn set_timeline_node_range_updates_projection() {
        let mut project = Template::MultiCam.build_project("Timeline Command Test");
        let node_id = project.timeline.nodes[0].id;
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: SetTimelineNodeRangeCommand {
                node_id,
                start_ms: 1_000,
                end_ms: 2_000,
            },
        };

        let projection = apply_set_timeline_node_range(&mut project, &command).unwrap();

        let clip = projection
            .payload
            .clips
            .iter()
            .find(|clip| clip.node_id == node_id)
            .expect("updated clip");
        assert_eq!(clip.start_ms, 1_000);
        assert_eq!(clip.end_ms, 2_000);
    }

    #[test]
    fn set_timeline_node_range_rejects_invalid_range() {
        let mut project = Template::MultiCam.build_project("Timeline Command Test");
        let node_id = project.timeline.nodes[0].id;
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: SetTimelineNodeRangeCommand {
                node_id,
                start_ms: 2_000,
                end_ms: 1_000,
            },
        };

        assert!(apply_set_timeline_node_range(&mut project, &command).is_err());
    }
}

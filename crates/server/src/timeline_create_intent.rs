use eidetic_core::Project;
use eidetic_core::contracts::{
    CommandEnvelope, CreateTimelineChildFromParentCommand, CreateTimelineNodeCommand,
};

use crate::timeline_command::TimelineCommandError;

pub(crate) fn derive_create_child_timeline_node_command(
    project: &Project,
    command: CommandEnvelope<CreateTimelineChildFromParentCommand>,
) -> Result<CommandEnvelope<CreateTimelineNodeCommand>, TimelineCommandError> {
    let parent = project.timeline.node(command.payload.parent_id)?;
    let child_level = parent.level.child_level().ok_or_else(|| {
        eidetic_core::Error::InvalidHierarchy(format!(
            "{} nodes cannot have children",
            parent.level
        ))
    })?;

    Ok(CommandEnvelope {
        id: command.id,
        payload: CreateTimelineNodeCommand {
            node_id: command.payload.node_id,
            parent_id: Some(parent.id),
            level: child_level,
            name: format!("New {}", child_level.label()),
            start_ms: parent.time_range.start_ms,
            end_ms: parent.time_range.end_ms,
            beat_type: None,
        },
    })
}

#[cfg(test)]
mod tests {
    use eidetic_core::Template;
    use eidetic_core::contracts::{
        CommandEnvelope, CommandId, CreateTimelineChildFromParentCommand,
    };
    use eidetic_core::timeline::node::{BeatType, NodeId, StoryLevel, StoryNode};
    use eidetic_core::timeline::timing::TimeRange;

    use super::derive_create_child_timeline_node_command;

    #[test]
    fn derives_child_command_from_parent_context() {
        let project = Template::MultiCam.build_project("Create Intent Test");
        let parent = project
            .timeline
            .nodes
            .iter()
            .find(|node| node.level == StoryLevel::Premise)
            .expect("premise node");
        let node_id = NodeId::new();
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: CreateTimelineChildFromParentCommand {
                node_id,
                parent_id: parent.id,
            },
        };

        let derived = derive_create_child_timeline_node_command(&project, command).unwrap();

        assert_eq!(derived.payload.node_id, node_id);
        assert_eq!(derived.payload.parent_id, Some(parent.id));
        assert_eq!(derived.payload.level, StoryLevel::Act);
        assert_eq!(derived.payload.name, "New Act");
        assert_eq!(derived.payload.start_ms, parent.time_range.start_ms);
        assert_eq!(derived.payload.end_ms, parent.time_range.end_ms);
        assert_eq!(derived.payload.beat_type, None);
    }

    #[test]
    fn rejects_parent_without_child_level() {
        let mut project = Template::MultiCam.build_project("Create Intent Test");
        let scene = project
            .timeline
            .nodes
            .iter()
            .find(|node| node.level == StoryLevel::Scene)
            .expect("scene node");
        let parent = StoryNode::new_beat(
            "Beat parent",
            BeatType::Setup,
            TimeRange::new(scene.time_range.start_ms, scene.time_range.end_ms).unwrap(),
            scene.id,
        );
        let parent_id = parent.id;
        project.timeline.add_node(parent).unwrap();
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: CreateTimelineChildFromParentCommand {
                node_id: NodeId::new(),
                parent_id,
            },
        };

        assert!(derive_create_child_timeline_node_command(&project, command).is_err());
    }
}

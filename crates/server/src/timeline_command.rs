use eidetic_core::contracts::{
    ApplyTimelineChildrenCommand, CommandEnvelope, CreateTimelineNodeCommand,
    CreateTimelineRelationshipCommand, DeleteTimelineNodeCommand,
    DeleteTimelineRelationshipCommand, ProjectionEnvelope, SetTimelineNodeLockCommand,
    SetTimelineNodeRangeCommand, SplitTimelineNodeCommand, TimelineRenderProjection,
};
use eidetic_core::project::Project;
use eidetic_core::timeline::node::{ContentStatus, StoryNode};
use eidetic_core::timeline::relationship::Relationship;
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

pub(crate) fn apply_split_timeline_node(
    project: &mut Project,
    command: &CommandEnvelope<SplitTimelineNodeCommand>,
) -> Result<ProjectionEnvelope<TimelineRenderProjection>, TimelineCommandError> {
    project
        .timeline
        .split_node(command.payload.node_id, command.payload.at_ms)
        .map_err(TimelineCommandError::Core)?;

    Ok(ProjectionEnvelope::initial(
        TimelineRenderProjection::from_timeline(&project.timeline),
    ))
}

pub(crate) fn apply_delete_timeline_node(
    project: &mut Project,
    command: &CommandEnvelope<DeleteTimelineNodeCommand>,
) -> Result<ProjectionEnvelope<TimelineRenderProjection>, TimelineCommandError> {
    project
        .timeline
        .remove_node(command.payload.node_id)
        .map_err(TimelineCommandError::Core)?;

    Ok(ProjectionEnvelope::initial(
        TimelineRenderProjection::from_timeline(&project.timeline),
    ))
}

pub(crate) fn apply_set_timeline_node_lock(
    project: &mut Project,
    command: &CommandEnvelope<SetTimelineNodeLockCommand>,
) -> Result<ProjectionEnvelope<TimelineRenderProjection>, TimelineCommandError> {
    let node = project
        .timeline
        .node_mut(command.payload.node_id)
        .map_err(TimelineCommandError::Core)?;
    node.locked = command.payload.locked;

    Ok(ProjectionEnvelope::initial(
        TimelineRenderProjection::from_timeline(&project.timeline),
    ))
}

pub(crate) fn apply_create_timeline_node(
    project: &mut Project,
    command: &CommandEnvelope<CreateTimelineNodeCommand>,
) -> Result<ProjectionEnvelope<TimelineRenderProjection>, TimelineCommandError> {
    let time_range = TimeRange::new(command.payload.start_ms, command.payload.end_ms)
        .map_err(TimelineCommandError::Core)?;
    let mut node = if let Some(parent_id) = command.payload.parent_id {
        StoryNode::new_child(
            command.payload.name.clone(),
            command.payload.level,
            time_range,
            parent_id,
        )
    } else {
        StoryNode::new(
            command.payload.name.clone(),
            command.payload.level,
            time_range,
        )
    };
    node.id = command.payload.node_id;
    node.beat_type = command.payload.beat_type.clone();

    project
        .timeline
        .add_node(node)
        .map_err(TimelineCommandError::Core)?;

    Ok(ProjectionEnvelope::initial(
        TimelineRenderProjection::from_timeline(&project.timeline),
    ))
}

pub(crate) fn apply_timeline_children(
    project: &mut Project,
    command: &CommandEnvelope<ApplyTimelineChildrenCommand>,
) -> Result<ProjectionEnvelope<TimelineRenderProjection>, TimelineCommandError> {
    let parent_id = command.payload.parent_id;
    let (parent_range, child_level) = {
        let parent = project.timeline.node(parent_id)?;
        let child_level = parent.level.child_level().ok_or_else(|| {
            eidetic_core::Error::InvalidHierarchy(format!(
                "{} nodes cannot have children",
                parent.level
            ))
        })?;
        (parent.time_range, child_level)
    };

    project.timeline.clear_children_of(parent_id)?;

    if command.payload.children.is_empty() {
        return Ok(ProjectionEnvelope::initial(
            TimelineRenderProjection::from_timeline(&project.timeline),
        ));
    }

    let total_weight: f32 = command
        .payload
        .children
        .iter()
        .map(|child| child.weight.max(0.1))
        .sum();
    let parent_duration = parent_range.end_ms - parent_range.start_ms;
    let parent_arc_ids = project.timeline.arcs_for_node(parent_id);
    let mut cursor = parent_range.start_ms;

    for (index, child) in command.payload.children.iter().enumerate() {
        let weight = child.weight.max(0.1);
        let duration = if index == command.payload.children.len() - 1 {
            parent_range.end_ms - cursor
        } else {
            ((weight / total_weight) * parent_duration as f32) as u64
        };
        let end_ms = (cursor + duration).min(parent_range.end_ms);
        let time_range = TimeRange::new(cursor, end_ms)?;
        let mut node = StoryNode::new_child(&child.name, child_level, time_range, parent_id);
        node.id = child.node_id;
        node.sort_order = index as u32;
        node.content.notes = child.outline.clone();
        if !node.content.notes.is_empty() {
            node.content.status = ContentStatus::NotesOnly;
        }
        node.beat_type = child.beat_type.clone();

        project.timeline.add_node(node)?;
        for arc_id in &parent_arc_ids {
            project.timeline.tag_node(child.node_id, *arc_id);
        }
        cursor = end_ms;
    }

    Ok(ProjectionEnvelope::initial(
        TimelineRenderProjection::from_timeline(&project.timeline),
    ))
}

pub(crate) fn apply_create_timeline_relationship(
    project: &mut Project,
    command: &CommandEnvelope<CreateTimelineRelationshipCommand>,
) -> Result<ProjectionEnvelope<TimelineRenderProjection>, TimelineCommandError> {
    let mut relationship = Relationship::new(
        command.payload.from_node_id,
        command.payload.to_node_id,
        command.payload.relationship_type.clone(),
    );
    relationship.id = command.payload.relationship_id;

    project
        .timeline
        .add_relationship(relationship)
        .map_err(TimelineCommandError::Core)?;

    Ok(ProjectionEnvelope::initial(
        TimelineRenderProjection::from_timeline(&project.timeline),
    ))
}

pub(crate) fn apply_delete_timeline_relationship(
    project: &mut Project,
    command: &CommandEnvelope<DeleteTimelineRelationshipCommand>,
) -> Result<ProjectionEnvelope<TimelineRenderProjection>, TimelineCommandError> {
    project
        .timeline
        .remove_relationship(command.payload.relationship_id)
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

    #[test]
    fn split_timeline_node_returns_projection_without_original_node() {
        let mut project = Template::MultiCam.build_project("Timeline Command Test");
        let node = project.timeline.nodes[0].clone();
        let split_ms = node.time_range.start_ms + node.time_range.duration_ms() / 2;
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: SplitTimelineNodeCommand {
                node_id: node.id,
                at_ms: split_ms,
            },
        };

        let projection = apply_split_timeline_node(&mut project, &command).unwrap();

        assert!(
            projection
                .payload
                .clips
                .iter()
                .all(|clip| clip.node_id != node.id)
        );
        assert!(
            projection
                .payload
                .clips
                .iter()
                .any(|clip| clip.start_ms == node.time_range.start_ms && clip.end_ms == split_ms)
        );
        assert!(
            projection
                .payload
                .clips
                .iter()
                .any(|clip| clip.start_ms == split_ms && clip.end_ms == node.time_range.end_ms)
        );
    }

    #[test]
    fn delete_timeline_node_returns_projection_without_deleted_subtree() {
        let mut project = Template::MultiCam.build_project("Timeline Command Test");
        let parent = project.timeline.nodes[0].clone();
        let child_id = project
            .timeline
            .nodes
            .iter()
            .find(|node| node.parent_id == Some(parent.id))
            .expect("child node")
            .id;
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: DeleteTimelineNodeCommand { node_id: parent.id },
        };

        let projection = apply_delete_timeline_node(&mut project, &command).unwrap();

        assert!(
            projection
                .payload
                .clips
                .iter()
                .all(|clip| clip.node_id != parent.id && clip.node_id != child_id)
        );
    }

    #[test]
    fn set_timeline_node_lock_updates_projection() {
        let mut project = Template::MultiCam.build_project("Timeline Command Test");
        let node_id = project.timeline.nodes[0].id;
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: SetTimelineNodeLockCommand {
                node_id,
                locked: true,
            },
        };

        let projection = apply_set_timeline_node_lock(&mut project, &command).unwrap();

        let clip = projection
            .payload
            .clips
            .iter()
            .find(|clip| clip.node_id == node_id)
            .expect("locked clip");
        assert!(clip.locked);
    }

    #[test]
    fn set_timeline_node_lock_rejects_unknown_node() {
        let mut project = Template::MultiCam.build_project("Timeline Command Test");
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: SetTimelineNodeLockCommand {
                node_id: eidetic_core::timeline::node::NodeId::new(),
                locked: true,
            },
        };

        assert!(apply_set_timeline_node_lock(&mut project, &command).is_err());
    }

    #[test]
    fn create_timeline_node_returns_projection_with_new_node() {
        let mut project = Template::MultiCam.build_project("Timeline Command Test");
        let parent = project.timeline.nodes[0].clone();
        let node_id = eidetic_core::timeline::node::NodeId::new();
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: CreateTimelineNodeCommand {
                node_id,
                parent_id: Some(parent.id),
                level: parent.level.child_level().expect("child level"),
                name: "Inserted act".to_string(),
                start_ms: parent.time_range.start_ms,
                end_ms: parent.time_range.start_ms + 1_000,
                beat_type: None,
            },
        };

        let projection = apply_create_timeline_node(&mut project, &command).unwrap();

        let clip = projection
            .payload
            .clips
            .iter()
            .find(|clip| clip.node_id == node_id)
            .expect("created clip");
        assert_eq!(clip.parent_id, Some(parent.id));
        assert_eq!(clip.name, "Inserted act");
    }

    #[test]
    fn apply_timeline_children_replaces_existing_children() {
        let mut project = Template::MultiCam.build_project("Timeline Command Test");
        let parent = project.timeline.nodes[0].clone();
        let original_child_id = project
            .timeline
            .children_of(parent.id)
            .first()
            .expect("existing child")
            .id;
        let first_child_id = eidetic_core::timeline::node::NodeId::new();
        let second_child_id = eidetic_core::timeline::node::NodeId::new();
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: ApplyTimelineChildrenCommand {
                parent_id: parent.id,
                children: vec![
                    eidetic_core::contracts::ApplyTimelineChildCommand {
                        node_id: first_child_id,
                        name: "First child".to_string(),
                        outline: "First outline".to_string(),
                        weight: 1.0,
                        beat_type: None,
                    },
                    eidetic_core::contracts::ApplyTimelineChildCommand {
                        node_id: second_child_id,
                        name: "Second child".to_string(),
                        outline: "Second outline".to_string(),
                        weight: 1.0,
                        beat_type: None,
                    },
                ],
            },
        };

        let projection = apply_timeline_children(&mut project, &command).unwrap();

        assert!(
            projection
                .payload
                .clips
                .iter()
                .all(|clip| clip.node_id != original_child_id)
        );
        assert!(
            projection
                .payload
                .clips
                .iter()
                .any(|clip| clip.node_id == first_child_id
                    && clip.parent_id == Some(parent.id)
                    && clip.name == "First child")
        );
        assert!(
            projection
                .payload
                .clips
                .iter()
                .any(|clip| clip.node_id == second_child_id
                    && clip.parent_id == Some(parent.id)
                    && clip.name == "Second child")
        );
    }

    #[test]
    fn create_timeline_relationship_returns_projection_with_relationship() {
        let mut project = Template::MultiCam.build_project("Timeline Command Test");
        let from_node = project.timeline.nodes[0].id;
        let to_node = project.timeline.nodes[1].id;
        let relationship_id = eidetic_core::timeline::relationship::RelationshipId::new();
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: CreateTimelineRelationshipCommand {
                relationship_id,
                from_node_id: from_node,
                to_node_id: to_node,
                relationship_type: eidetic_core::timeline::relationship::RelationshipType::Thematic,
            },
        };

        let projection = apply_create_timeline_relationship(&mut project, &command).unwrap();

        assert!(projection.payload.relationships.iter().any(|relationship| {
            relationship.relationship_id == relationship_id
                && relationship.from_node_id == from_node
                && relationship.to_node_id == to_node
        }));
    }

    #[test]
    fn create_timeline_relationship_rejects_unknown_endpoint() {
        let mut project = Template::MultiCam.build_project("Timeline Command Test");
        let to_node = project.timeline.nodes[0].id;
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: CreateTimelineRelationshipCommand {
                relationship_id: eidetic_core::timeline::relationship::RelationshipId::new(),
                from_node_id: eidetic_core::timeline::node::NodeId::new(),
                to_node_id: to_node,
                relationship_type: eidetic_core::timeline::relationship::RelationshipType::Causal,
            },
        };

        assert!(apply_create_timeline_relationship(&mut project, &command).is_err());
    }

    #[test]
    fn delete_timeline_relationship_returns_projection_without_relationship() {
        let mut project = Template::MultiCam.build_project("Timeline Command Test");
        let from_node = project.timeline.nodes[0].id;
        let to_node = project.timeline.nodes[1].id;
        let mut relationship = Relationship::new(
            from_node,
            to_node,
            eidetic_core::timeline::relationship::RelationshipType::Thematic,
        );
        let relationship_id = eidetic_core::timeline::relationship::RelationshipId::new();
        relationship.id = relationship_id;
        project.timeline.add_relationship(relationship).unwrap();
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: DeleteTimelineRelationshipCommand { relationship_id },
        };

        let projection = apply_delete_timeline_relationship(&mut project, &command).unwrap();

        assert!(
            projection
                .payload
                .relationships
                .iter()
                .all(|relationship| relationship.relationship_id != relationship_id)
        );
    }
}

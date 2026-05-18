use eidetic_core::contracts::{
    ApplyTimelineChildrenCommand, ChangeEvent, ChangeEventKind, CommandEnvelope,
    CreateTimelineNodeCommand, CreateTimelineRelationshipCommand, DeleteTimelineNodeCommand,
    DeleteTimelineRelationshipCommand, FieldDelta, FieldValue, ObjectKind, ObjectRevision,
    ProjectionEnvelope, RevisionOperation, SetTimelineNodeLockCommand, SetTimelineNodeNotesCommand,
    SetTimelineNodeRangeCommand, SplitTimelineNodeCommand, TimelineRenderProjection,
};
use eidetic_core::project::Project;
use eidetic_core::timeline::node::{ContentStatus, StoryNode};
use eidetic_core::timeline::relationship::{Relationship, RelationshipType};
use eidetic_core::timeline::timing::TimeRange;
use rusqlite::Connection;
use thiserror::Error;

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};

pub(crate) fn record_set_timeline_node_range_history(
    conn: &mut Connection,
    project: &Project,
    command: &CommandEnvelope<SetTimelineNodeRangeCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, TimelineCommandError> {
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "timeline.node_range")?
    {
        return Ok(outcome);
    }

    let range = TimeRange::new(command.payload.start_ms, command.payload.end_ms)?;
    if range.end_ms > project.timeline.total_duration_ms {
        return Err(TimelineCommandError::Core(
            eidetic_core::Error::NodeExceedsTimeline {
                node_end_ms: range.end_ms,
                timeline_ms: project.timeline.total_duration_ms,
            },
        ));
    }
    let node = project.timeline.node(command.payload.node_id)?;

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("set timeline node range {}", node.name),
    )
    .with_created_at_ms(created_at_ms);
    let revision = ObjectRevision::new(
        ObjectKind::TimelineNode,
        command.payload.node_id.0.to_string(),
        event.id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "start_ms",
        Some(FieldValue::Integer(node.time_range.start_ms as i64)),
        Some(FieldValue::Integer(command.payload.start_ms as i64)),
    ))
    .with_field(FieldDelta::new(
        "end_ms",
        Some(FieldValue::Integer(node.time_range.end_ms as i64)),
        Some(FieldValue::Integer(command.payload.end_ms as i64)),
    ));

    Ok(history_store::record_change(
        conn,
        command,
        "timeline.node_range",
        &event,
        &[revision],
    )?)
}

pub(crate) fn record_set_timeline_node_lock_history(
    conn: &mut Connection,
    project: &Project,
    command: &CommandEnvelope<SetTimelineNodeLockCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, TimelineCommandError> {
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "timeline.node_lock")?
    {
        return Ok(outcome);
    }

    let node = project.timeline.node(command.payload.node_id)?;
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("set timeline node lock {}", node.name),
    )
    .with_created_at_ms(created_at_ms);
    let revision = ObjectRevision::new(
        ObjectKind::TimelineNode,
        command.payload.node_id.0.to_string(),
        event.id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "locked",
        Some(FieldValue::Bool(node.locked)),
        Some(FieldValue::Bool(command.payload.locked)),
    ));

    Ok(history_store::record_change(
        conn,
        command,
        "timeline.node_lock",
        &event,
        &[revision],
    )?)
}

pub(crate) fn record_set_timeline_node_notes_history(
    conn: &mut Connection,
    project: &Project,
    command: &CommandEnvelope<SetTimelineNodeNotesCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, TimelineCommandError> {
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "timeline.node_notes")?
    {
        return Ok(outcome);
    }

    let node = project.timeline.node(command.payload.node_id)?;
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("set timeline node notes {}", node.name),
    )
    .with_created_at_ms(created_at_ms);
    let new_status =
        if !command.payload.notes.is_empty() && node.content.status == ContentStatus::Empty {
            ContentStatus::NotesOnly
        } else {
            node.content.status
        };
    let mut revision = ObjectRevision::new(
        ObjectKind::TimelineNode,
        command.payload.node_id.0.to_string(),
        event.id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "notes",
        Some(FieldValue::Text(node.content.notes.clone())),
        Some(FieldValue::Text(command.payload.notes.clone())),
    ));

    if new_status != node.content.status {
        revision = revision.with_field(FieldDelta::new(
            "content_status",
            Some(FieldValue::Text(encode_content_status(node.content.status))),
            Some(FieldValue::Text(encode_content_status(new_status))),
        ));
    }

    Ok(history_store::record_change(
        conn,
        command,
        "timeline.node_notes",
        &event,
        &[revision],
    )?)
}

pub(crate) fn record_create_timeline_relationship_history(
    conn: &mut Connection,
    project: &Project,
    command: &CommandEnvelope<CreateTimelineRelationshipCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, TimelineCommandError> {
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "timeline.relationship_create")?
    {
        return Ok(outcome);
    }

    project.timeline.node(command.payload.from_node_id)?;
    project.timeline.node(command.payload.to_node_id)?;

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        "create timeline relationship",
    )
    .with_created_at_ms(created_at_ms);
    let revision = ObjectRevision::new(
        ObjectKind::TimelineRelationship,
        command.payload.relationship_id.0.to_string(),
        event.id,
        RevisionOperation::Create,
    )
    .with_field(FieldDelta::new(
        "from_node_id",
        None,
        Some(FieldValue::Text(command.payload.from_node_id.0.to_string())),
    ))
    .with_field(FieldDelta::new(
        "to_node_id",
        None,
        Some(FieldValue::Text(command.payload.to_node_id.0.to_string())),
    ))
    .with_field(FieldDelta::new(
        "relationship_type",
        None,
        Some(FieldValue::Text(encode_relationship_type(
            &command.payload.relationship_type,
        )?)),
    ));

    Ok(history_store::record_change(
        conn,
        command,
        "timeline.relationship_create",
        &event,
        &[revision],
    )?)
}

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

pub(crate) fn apply_set_timeline_node_notes(
    project: &mut Project,
    command: &CommandEnvelope<SetTimelineNodeNotesCommand>,
) -> Result<ProjectionEnvelope<TimelineRenderProjection>, TimelineCommandError> {
    let node = project
        .timeline
        .node_mut(command.payload.node_id)
        .map_err(TimelineCommandError::Core)?;
    node.content.notes = command.payload.notes.clone();
    if !node.content.notes.is_empty() && node.content.status == ContentStatus::Empty {
        node.content.status = ContentStatus::NotesOnly;
    }

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

fn encode_content_status(status: ContentStatus) -> String {
    match status {
        ContentStatus::Empty => "Empty",
        ContentStatus::NotesOnly => "NotesOnly",
        ContentStatus::Generating => "Generating",
        ContentStatus::HasContent => "HasContent",
    }
    .to_string()
}

fn encode_relationship_type(
    relationship_type: &RelationshipType,
) -> Result<String, TimelineCommandError> {
    serde_json::to_string(relationship_type).map_err(|error| {
        TimelineCommandError::Core(eidetic_core::Error::InvalidOperation(format!(
            "invalid relationship type: {error}"
        )))
    })
}

#[derive(Debug, Error)]
pub(crate) enum TimelineCommandError {
    #[error("{0}")]
    Core(#[from] eidetic_core::Error),
    #[error(transparent)]
    History(#[from] HistoryStoreError),
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
    fn set_timeline_node_notes_updates_projection_status() {
        let mut project = Template::MultiCam.build_project("Timeline Command Test");
        let node_id = project.timeline.nodes[0].id;
        project.timeline.node_mut(node_id).unwrap().content.status = ContentStatus::Empty;
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: SetTimelineNodeNotesCommand {
                node_id,
                notes: "New outline".to_string(),
            },
        };

        let projection = apply_set_timeline_node_notes(&mut project, &command).unwrap();

        assert_eq!(
            project.timeline.node(node_id).unwrap().content.notes,
            "New outline"
        );
        let clip = projection
            .payload
            .clips
            .iter()
            .find(|clip| clip.node_id == node_id)
            .expect("notes clip");
        assert_eq!(clip.content_status, ContentStatus::NotesOnly);
    }

    #[test]
    fn set_timeline_node_notes_rejects_unknown_node() {
        let mut project = Template::MultiCam.build_project("Timeline Command Test");
        let command = CommandEnvelope {
            id: CommandId::new(),
            payload: SetTimelineNodeNotesCommand {
                node_id: eidetic_core::timeline::node::NodeId::new(),
                notes: "New outline".to_string(),
            },
        };

        assert!(apply_set_timeline_node_notes(&mut project, &command).is_err());
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

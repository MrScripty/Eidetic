use eidetic_core::contracts::{
    ApplyTimelineChildrenCommand, CommandEnvelope, CreateTimelineNodeCommand,
    CreateTimelineRelationshipCommand, DeleteTimelineNodeCommand,
    DeleteTimelineRelationshipCommand, ProjectionEnvelope, SetTimelineNodeLockCommand,
    SetTimelineNodeNotesCommand, SetTimelineNodeRangeCommand, SplitTimelineNodeCommand,
    TimelineRenderProjection,
};
use eidetic_core::project::Project;
use eidetic_core::timeline::node::{ContentStatus, StoryNode};
use eidetic_core::timeline::relationship::Relationship;
use eidetic_core::timeline::timing::TimeRange;
use thiserror::Error;

use crate::history_store::HistoryStoreError;
pub(crate) use crate::timeline_command_history::{
    record_create_timeline_node_history, record_create_timeline_relationship_history,
    record_delete_timeline_relationship_history, record_set_timeline_node_lock_history,
    record_set_timeline_node_notes_history, record_set_timeline_node_range_history,
};
pub(crate) use crate::timeline_node_delete_history::record_delete_timeline_node_history;
pub(crate) use crate::timeline_node_split_history::record_split_timeline_node_history;

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
        .split_node(
            command.payload.node_id,
            command.payload.at_ms,
            command.payload.left_node_id,
            command.payload.right_node_id,
        )
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

#[derive(Debug, Error)]
pub(crate) enum TimelineCommandError {
    #[error("{0}")]
    Core(#[from] eidetic_core::Error),
    #[error(transparent)]
    History(#[from] HistoryStoreError),
}

#[cfg(test)]
#[path = "timeline_command_tests.rs"]
mod tests;

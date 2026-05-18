use eidetic_core::Project;
use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, CreateTimelineNodeCommand,
    CreateTimelineRelationshipCommand, DeleteTimelineRelationshipCommand, FieldDelta, FieldValue,
    ObjectKind, ObjectRevision, RevisionOperation, SetTimelineNodeLockCommand,
    SetTimelineNodeNotesCommand, SetTimelineNodeRangeCommand,
};
use eidetic_core::timeline::node::{BeatType, ContentStatus, StoryLevel};
use eidetic_core::timeline::relationship::RelationshipType;
use eidetic_core::timeline::timing::TimeRange;
use rusqlite::Connection;

use crate::history_store::{self, RecordChangeOutcome};
use crate::timeline_command::TimelineCommandError;

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

pub(crate) fn record_delete_timeline_relationship_history(
    conn: &mut Connection,
    project: &Project,
    command: &CommandEnvelope<DeleteTimelineRelationshipCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, TimelineCommandError> {
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "timeline.relationship_delete")?
    {
        return Ok(outcome);
    }

    let relationship = project
        .timeline
        .relationships
        .iter()
        .find(|relationship| relationship.id == command.payload.relationship_id)
        .ok_or_else(|| {
            TimelineCommandError::Core(eidetic_core::Error::RelationshipNotFound(
                command.payload.relationship_id.0,
            ))
        })?;

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        "delete timeline relationship",
    )
    .with_created_at_ms(created_at_ms);
    let revision = ObjectRevision::new(
        ObjectKind::TimelineRelationship,
        command.payload.relationship_id.0.to_string(),
        event.id,
        RevisionOperation::Delete,
    )
    .with_field(FieldDelta::new(
        "from_node_id",
        Some(FieldValue::Text(relationship.from_node.0.to_string())),
        None,
    ))
    .with_field(FieldDelta::new(
        "to_node_id",
        Some(FieldValue::Text(relationship.to_node.0.to_string())),
        None,
    ))
    .with_field(FieldDelta::new(
        "relationship_type",
        Some(FieldValue::Text(encode_relationship_type(
            &relationship.relationship_type,
        )?)),
        None,
    ));

    Ok(history_store::record_change(
        conn,
        command,
        "timeline.relationship_delete",
        &event,
        &[revision],
    )?)
}

pub(crate) fn record_create_timeline_node_history(
    conn: &mut Connection,
    project: &Project,
    command: &CommandEnvelope<CreateTimelineNodeCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, TimelineCommandError> {
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "timeline.node_create")?
    {
        return Ok(outcome);
    }

    let range = validate_create_timeline_node(project, command)?;
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("create timeline node {}", command.payload.name),
    )
    .with_created_at_ms(created_at_ms);
    let mut revision = ObjectRevision::new(
        ObjectKind::TimelineNode,
        command.payload.node_id.0.to_string(),
        event.id,
        RevisionOperation::Create,
    )
    .with_field(FieldDelta::new(
        "name",
        None,
        Some(FieldValue::Text(command.payload.name.clone())),
    ))
    .with_field(FieldDelta::new(
        "parent_id",
        None,
        command
            .payload
            .parent_id
            .map(|node_id| FieldValue::Text(node_id.0.to_string())),
    ))
    .with_field(FieldDelta::new(
        "level",
        None,
        Some(FieldValue::Text(encode_story_level(command.payload.level))),
    ))
    .with_field(FieldDelta::new(
        "start_ms",
        None,
        Some(FieldValue::Integer(range.start_ms as i64)),
    ))
    .with_field(FieldDelta::new(
        "end_ms",
        None,
        Some(FieldValue::Integer(range.end_ms as i64)),
    ))
    .with_field(FieldDelta::new(
        "sort_order",
        None,
        Some(FieldValue::Integer(0)),
    ))
    .with_field(FieldDelta::new(
        "locked",
        None,
        Some(FieldValue::Bool(false)),
    ))
    .with_field(FieldDelta::new(
        "content_status",
        None,
        Some(FieldValue::Text(encode_content_status(
            ContentStatus::Empty,
        ))),
    ));

    if let Some(beat_type) = &command.payload.beat_type {
        revision = revision.with_field(FieldDelta::new(
            "beat_type",
            None,
            Some(FieldValue::Text(encode_beat_type(beat_type)?)),
        ));
    }

    Ok(history_store::record_change(
        conn,
        command,
        "timeline.node_create",
        &event,
        &[revision],
    )?)
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

fn encode_story_level(level: StoryLevel) -> String {
    level.label().to_string()
}

fn encode_beat_type(beat_type: &BeatType) -> Result<String, TimelineCommandError> {
    serde_json::to_string(beat_type).map_err(|error| {
        TimelineCommandError::Core(eidetic_core::Error::InvalidOperation(format!(
            "invalid beat type: {error}"
        )))
    })
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

fn validate_create_timeline_node(
    project: &Project,
    command: &CommandEnvelope<CreateTimelineNodeCommand>,
) -> Result<TimeRange, TimelineCommandError> {
    let range = TimeRange::new(command.payload.start_ms, command.payload.end_ms)?;
    if range.end_ms > project.timeline.total_duration_ms {
        return Err(TimelineCommandError::Core(
            eidetic_core::Error::NodeExceedsTimeline {
                node_end_ms: range.end_ms,
                timeline_ms: project.timeline.total_duration_ms,
            },
        ));
    }

    if let Some(parent_id) = command.payload.parent_id {
        let parent = project.timeline.node(parent_id)?;
        let expected_child_level = parent.level.child_level().ok_or_else(|| {
            eidetic_core::Error::InvalidHierarchy(format!(
                "{} nodes cannot have children",
                parent.level
            ))
        })?;
        if command.payload.level != expected_child_level {
            return Err(TimelineCommandError::Core(
                eidetic_core::Error::InvalidHierarchy(format!(
                    "expected {} child for {} parent, got {}",
                    expected_child_level, parent.level, command.payload.level
                )),
            ));
        }
    } else if command.payload.level != StoryLevel::Premise {
        return Err(TimelineCommandError::Core(
            eidetic_core::Error::InvalidHierarchy(format!(
                "{} nodes must have a parent",
                command.payload.level
            )),
        ));
    } else if project
        .timeline
        .nodes
        .iter()
        .any(|node| node.level == StoryLevel::Premise)
    {
        return Err(TimelineCommandError::Core(
            eidetic_core::Error::InvalidHierarchy("only one Premise node is allowed".to_string()),
        ));
    }

    Ok(range)
}

use std::collections::HashSet;

use eidetic_core::Project;
use eidetic_core::contracts::{
    ApplyTimelineChildCommand, ApplyTimelineChildrenCommand, ChangeEvent, ChangeEventId,
    ChangeEventKind, CommandEnvelope, FieldDelta, FieldValue, ObjectKind, ObjectRevision,
    RevisionOperation,
};
use eidetic_core::story::arc::ArcId;
use eidetic_core::timeline::node::{ContentStatus, NodeId, StoryLevel, StoryNode};
use eidetic_core::timeline::relationship::Relationship;
use eidetic_core::timeline::timing::TimeRange;
use rusqlite::Connection;

use crate::history_store::{self, RecordChangeOutcome};
use crate::timeline_command::TimelineCommandError;
use crate::timeline_command_history_codec::{
    encode_arc_ids, encode_beat_type, encode_content_status, encode_relationship_type,
    encode_story_level,
};
use crate::timeline_node_store;
use crate::timeline_relationship_store;

pub(crate) fn record_apply_timeline_children_history(
    conn: &mut Connection,
    project: &Project,
    command: &CommandEnvelope<ApplyTimelineChildrenCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, TimelineCommandError> {
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "timeline.children_apply")?
    {
        return Ok(outcome);
    }

    let child_plan = validate_and_plan_children(project, command)?;
    let existing_children = collect_removed_children(project, command.payload.parent_id);
    let removed_node_ids: Vec<_> = existing_children.iter().map(|node| node.id).collect();
    let removed_relationships: Vec<_> = project
        .timeline
        .relationships
        .iter()
        .filter(|relationship| {
            removed_node_ids.contains(&relationship.from_node)
                || removed_node_ids.contains(&relationship.to_node)
        })
        .collect();
    let removed_relationship_ids: Vec<_> = removed_relationships
        .iter()
        .map(|relationship| relationship.id)
        .collect();

    let parent = project.timeline.node(command.payload.parent_id)?;
    let parent_arc_ids = project.timeline.arcs_for_node(parent.id);
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("replace timeline children for {}", parent.name),
    )
    .with_created_at_ms(created_at_ms);
    let mut revisions = Vec::new();
    for node in existing_children {
        revisions.push(deleted_node_revision(
            node,
            project.timeline.arcs_for_node(node.id),
            event.id,
        )?);
    }
    for relationship in removed_relationships {
        revisions.push(deleted_relationship_revision(relationship, event.id)?);
    }
    for planned_child in &child_plan {
        revisions.push(created_child_revision(
            planned_child.child,
            command.payload.parent_id,
            planned_child.level,
            planned_child.time_range,
            planned_child.sort_order,
            &parent_arc_ids,
            event.id,
        )?);
    }
    let mut next_timeline = project.timeline.clone();
    next_timeline.clear_children_of(command.payload.parent_id)?;
    for planned_child in &child_plan {
        let mut node = StoryNode::new_child(
            &planned_child.child.name,
            planned_child.level,
            planned_child.time_range,
            command.payload.parent_id,
        );
        node.id = planned_child.child.node_id;
        node.sort_order = planned_child.sort_order;
        node.content.notes = planned_child.child.outline.clone();
        if !node.content.notes.is_empty() {
            node.content.status = ContentStatus::NotesOnly;
        }
        node.beat_type = planned_child.child.beat_type.clone();

        next_timeline.add_node(node)?;
        for arc_id in &parent_arc_ids {
            next_timeline.tag_node(planned_child.child.node_id, *arc_id);
        }
    }

    Ok(history_store::record_change_with(
        conn,
        command,
        "timeline.children_apply",
        &event,
        &revisions,
        |tx| {
            timeline_relationship_store::delete_relationships_in_transaction(
                tx,
                &removed_relationship_ids,
            )?;
            timeline_node_store::delete_nodes_in_transaction(tx, &removed_node_ids)?;
            timeline_node_store::upsert_nodes_in_transaction(tx, &next_timeline.nodes)?;
            timeline_node_store::replace_node_arcs_in_transaction(tx, &next_timeline.node_arcs)?;
            timeline_relationship_store::upsert_relationships_in_transaction(
                tx,
                &next_timeline.relationships,
            )
        },
    )?)
}

struct PlannedChild<'a> {
    child: &'a ApplyTimelineChildCommand,
    level: StoryLevel,
    time_range: TimeRange,
    sort_order: u32,
}

fn validate_and_plan_children<'a>(
    project: &Project,
    command: &'a CommandEnvelope<ApplyTimelineChildrenCommand>,
) -> Result<Vec<PlannedChild<'a>>, TimelineCommandError> {
    let parent = project.timeline.node(command.payload.parent_id)?;
    let child_level = parent.level.child_level().ok_or_else(|| {
        TimelineCommandError::Core(eidetic_core::Error::InvalidHierarchy(format!(
            "{} nodes cannot have children",
            parent.level
        )))
    })?;
    let mut ids = HashSet::new();
    for child in &command.payload.children {
        if !ids.insert(child.node_id) {
            return Err(TimelineCommandError::Core(
                eidetic_core::Error::InvalidOperation(
                    "child node ids must be distinct".to_string(),
                ),
            ));
        }
        if project
            .timeline
            .nodes
            .iter()
            .any(|node| node.id == child.node_id)
        {
            return Err(TimelineCommandError::Core(
                eidetic_core::Error::InvalidOperation("child node id already exists".to_string()),
            ));
        }
    }

    if command.payload.children.is_empty() {
        return Ok(Vec::new());
    }

    let total_weight: f32 = command
        .payload
        .children
        .iter()
        .map(|child| child.weight.max(0.1))
        .sum();
    let parent_duration = parent.time_range.end_ms - parent.time_range.start_ms;
    let mut cursor = parent.time_range.start_ms;
    let mut plan = Vec::with_capacity(command.payload.children.len());
    for (index, child) in command.payload.children.iter().enumerate() {
        let weight = child.weight.max(0.1);
        let duration = if index == command.payload.children.len() - 1 {
            parent.time_range.end_ms - cursor
        } else {
            ((weight / total_weight) * parent_duration as f32) as u64
        };
        let end_ms = (cursor + duration).min(parent.time_range.end_ms);
        let time_range = TimeRange::new(cursor, end_ms)?;
        plan.push(PlannedChild {
            child,
            level: child_level,
            time_range,
            sort_order: index as u32,
        });
        cursor = end_ms;
    }
    Ok(plan)
}

fn collect_removed_children(project: &Project, parent_id: NodeId) -> Vec<&StoryNode> {
    let mut removed = Vec::new();
    for child in project.timeline.children_of(parent_id) {
        removed.push(child);
        removed.extend(project.timeline.descendants_of(child.id));
    }
    removed
}

fn deleted_node_revision(
    node: &StoryNode,
    arc_ids: Vec<ArcId>,
    event_id: ChangeEventId,
) -> Result<ObjectRevision, TimelineCommandError> {
    let mut revision = ObjectRevision::new(
        ObjectKind::TimelineNode,
        node.id.0.to_string(),
        event_id,
        RevisionOperation::Delete,
    )
    .with_field(FieldDelta::new(
        "name",
        Some(FieldValue::Text(node.name.clone())),
        None,
    ))
    .with_field(FieldDelta::new(
        "parent_id",
        node.parent_id
            .map(|parent_id| FieldValue::Text(parent_id.0.to_string())),
        None,
    ))
    .with_field(FieldDelta::new(
        "level",
        Some(FieldValue::Text(encode_story_level(node.level))),
        None,
    ))
    .with_field(FieldDelta::new(
        "start_ms",
        Some(FieldValue::Integer(node.time_range.start_ms as i64)),
        None,
    ))
    .with_field(FieldDelta::new(
        "end_ms",
        Some(FieldValue::Integer(node.time_range.end_ms as i64)),
        None,
    ))
    .with_field(FieldDelta::new(
        "sort_order",
        Some(FieldValue::Integer(node.sort_order as i64)),
        None,
    ))
    .with_field(FieldDelta::new(
        "locked",
        Some(FieldValue::Bool(node.locked)),
        None,
    ))
    .with_field(FieldDelta::new(
        "notes",
        Some(FieldValue::Text(node.content.notes.clone())),
        None,
    ))
    .with_field(FieldDelta::new(
        "content_status",
        Some(FieldValue::Text(encode_content_status(node.content.status))),
        None,
    ));

    if let Some(beat_type) = &node.beat_type {
        revision = revision.with_field(FieldDelta::new(
            "beat_type",
            Some(FieldValue::Text(encode_beat_type(beat_type)?)),
            None,
        ));
    }
    if !arc_ids.is_empty() {
        revision = revision.with_field(FieldDelta::new(
            "arc_ids",
            Some(FieldValue::Text(encode_arc_ids(&arc_ids)?)),
            None,
        ));
    }
    Ok(revision)
}

fn deleted_relationship_revision(
    relationship: &Relationship,
    event_id: ChangeEventId,
) -> Result<ObjectRevision, TimelineCommandError> {
    Ok(ObjectRevision::new(
        ObjectKind::TimelineRelationship,
        relationship.id.0.to_string(),
        event_id,
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
    )))
}

fn created_child_revision(
    child: &ApplyTimelineChildCommand,
    parent_id: NodeId,
    level: StoryLevel,
    time_range: TimeRange,
    sort_order: u32,
    parent_arc_ids: &[ArcId],
    event_id: ChangeEventId,
) -> Result<ObjectRevision, TimelineCommandError> {
    let content_status = if child.outline.is_empty() {
        ContentStatus::Empty
    } else {
        ContentStatus::NotesOnly
    };
    let mut revision = ObjectRevision::new(
        ObjectKind::TimelineNode,
        child.node_id.0.to_string(),
        event_id,
        RevisionOperation::Create,
    )
    .with_field(FieldDelta::new(
        "name",
        None,
        Some(FieldValue::Text(child.name.clone())),
    ))
    .with_field(FieldDelta::new(
        "parent_id",
        None,
        Some(FieldValue::Text(parent_id.0.to_string())),
    ))
    .with_field(FieldDelta::new(
        "level",
        None,
        Some(FieldValue::Text(encode_story_level(level))),
    ))
    .with_field(FieldDelta::new(
        "start_ms",
        None,
        Some(FieldValue::Integer(time_range.start_ms as i64)),
    ))
    .with_field(FieldDelta::new(
        "end_ms",
        None,
        Some(FieldValue::Integer(time_range.end_ms as i64)),
    ))
    .with_field(FieldDelta::new(
        "sort_order",
        None,
        Some(FieldValue::Integer(sort_order as i64)),
    ))
    .with_field(FieldDelta::new(
        "locked",
        None,
        Some(FieldValue::Bool(false)),
    ))
    .with_field(FieldDelta::new(
        "notes",
        None,
        Some(FieldValue::Text(child.outline.clone())),
    ))
    .with_field(FieldDelta::new(
        "content_status",
        None,
        Some(FieldValue::Text(encode_content_status(content_status))),
    ));

    if let Some(beat_type) = &child.beat_type {
        revision = revision.with_field(FieldDelta::new(
            "beat_type",
            None,
            Some(FieldValue::Text(encode_beat_type(beat_type)?)),
        ));
    }
    if !parent_arc_ids.is_empty() {
        revision = revision.with_field(FieldDelta::new(
            "arc_ids",
            None,
            Some(FieldValue::Text(encode_arc_ids(parent_arc_ids)?)),
        ));
    }
    Ok(revision)
}

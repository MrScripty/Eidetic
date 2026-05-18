use eidetic_core::Project;
use eidetic_core::contracts::{
    ChangeEvent, ChangeEventId, ChangeEventKind, CommandEnvelope, FieldDelta, FieldValue,
    ObjectKind, ObjectRevision, RevisionOperation, SplitTimelineNodeCommand,
};
use eidetic_core::story::arc::ArcId;
use eidetic_core::timeline::node::{BeatType, ContentStatus, NodeId, StoryLevel, StoryNode};
use eidetic_core::timeline::relationship::Relationship;
use rusqlite::Connection;

use crate::history_store::{self, RecordChangeOutcome};
use crate::timeline_command::TimelineCommandError;
use crate::timeline_command_history_codec::{
    encode_arc_ids, encode_beat_type, encode_content_status, encode_story_level,
};
use crate::timeline_node_store;
use crate::timeline_relationship_store;

pub(crate) fn record_split_timeline_node_history(
    conn: &mut Connection,
    project: &Project,
    command: &CommandEnvelope<SplitTimelineNodeCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, TimelineCommandError> {
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "timeline.node_split")?
    {
        return Ok(outcome);
    }

    validate_split_timeline_node(project, command)?;
    let node = project.timeline.node(command.payload.node_id)?;
    let arc_ids = project.timeline.arcs_for_node(node.id);
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("split timeline node {}", node.name),
    )
    .with_created_at_ms(created_at_ms);
    let mut revisions = vec![
        split_original_node_revision(node, &arc_ids, event.id)?,
        split_created_node_revision(
            command.payload.left_node_id,
            format!("{} (L)", node.name),
            node.parent_id,
            node.level,
            node.time_range.start_ms,
            command.payload.at_ms,
            node.sort_order,
            node.locked,
            node.beat_type.as_ref(),
            &arc_ids,
            event.id,
        )?,
        split_created_node_revision(
            command.payload.right_node_id,
            format!("{} (R)", node.name),
            node.parent_id,
            node.level,
            command.payload.at_ms,
            node.time_range.end_ms,
            node.sort_order + 1,
            node.locked,
            node.beat_type.as_ref(),
            &arc_ids,
            event.id,
        )?,
    ];

    for child in project.timeline.children_of(node.id) {
        revisions.push(split_child_reparent_revision(
            child.id,
            node.id,
            split_child_parent(child, command),
            event.id,
        ));
    }
    for relationship in &project.timeline.relationships {
        if relationship.from_node == node.id || relationship.to_node == node.id {
            revisions.push(split_relationship_revision(
                relationship,
                command,
                event.id,
            )?);
        }
    }
    let mut next_timeline = project.timeline.clone();
    next_timeline.split_node(
        command.payload.node_id,
        command.payload.at_ms,
        command.payload.left_node_id,
        command.payload.right_node_id,
    )?;

    Ok(history_store::record_change_with(
        conn,
        command,
        "timeline.node_split",
        &event,
        &revisions,
        |tx| {
            timeline_node_store::delete_nodes_in_transaction(tx, &[command.payload.node_id])?;
            timeline_node_store::upsert_nodes_in_transaction(tx, &next_timeline.nodes)?;
            timeline_node_store::replace_node_arcs_in_transaction(tx, &next_timeline.node_arcs)?;
            timeline_relationship_store::upsert_relationships_in_transaction(
                tx,
                &next_timeline.relationships,
            )
        },
    )?)
}

fn validate_split_timeline_node(
    project: &Project,
    command: &CommandEnvelope<SplitTimelineNodeCommand>,
) -> Result<(), TimelineCommandError> {
    if command.payload.left_node_id == command.payload.right_node_id {
        return Err(TimelineCommandError::Core(
            eidetic_core::Error::InvalidOperation("split node ids must be distinct".to_string()),
        ));
    }
    if project.timeline.nodes.iter().any(|node| {
        node.id == command.payload.left_node_id || node.id == command.payload.right_node_id
    }) {
        return Err(TimelineCommandError::Core(
            eidetic_core::Error::InvalidOperation("split node ids already exist".to_string()),
        ));
    }

    let node = project.timeline.node(command.payload.node_id)?;
    if command.payload.at_ms <= node.time_range.start_ms
        || command.payload.at_ms >= node.time_range.end_ms
    {
        return Err(TimelineCommandError::Core(
            eidetic_core::Error::SplitOutOfRange {
                split_ms: command.payload.at_ms,
                start_ms: node.time_range.start_ms,
                end_ms: node.time_range.end_ms,
            },
        ));
    }
    Ok(())
}

fn split_original_node_revision(
    node: &StoryNode,
    arc_ids: &[ArcId],
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
            Some(FieldValue::Text(encode_arc_ids(arc_ids)?)),
            None,
        ));
    }
    Ok(revision)
}

#[allow(clippy::too_many_arguments)]
fn split_created_node_revision(
    node_id: NodeId,
    name: String,
    parent_id: Option<NodeId>,
    level: StoryLevel,
    start_ms: u64,
    end_ms: u64,
    sort_order: u32,
    locked: bool,
    beat_type: Option<&BeatType>,
    arc_ids: &[ArcId],
    event_id: ChangeEventId,
) -> Result<ObjectRevision, TimelineCommandError> {
    let mut revision = ObjectRevision::new(
        ObjectKind::TimelineNode,
        node_id.0.to_string(),
        event_id,
        RevisionOperation::Create,
    )
    .with_field(FieldDelta::new("name", None, Some(FieldValue::Text(name))))
    .with_field(FieldDelta::new(
        "parent_id",
        None,
        parent_id.map(|parent_id| FieldValue::Text(parent_id.0.to_string())),
    ))
    .with_field(FieldDelta::new(
        "level",
        None,
        Some(FieldValue::Text(encode_story_level(level))),
    ))
    .with_field(FieldDelta::new(
        "start_ms",
        None,
        Some(FieldValue::Integer(start_ms as i64)),
    ))
    .with_field(FieldDelta::new(
        "end_ms",
        None,
        Some(FieldValue::Integer(end_ms as i64)),
    ))
    .with_field(FieldDelta::new(
        "sort_order",
        None,
        Some(FieldValue::Integer(sort_order as i64)),
    ))
    .with_field(FieldDelta::new(
        "locked",
        None,
        Some(FieldValue::Bool(locked)),
    ))
    .with_field(FieldDelta::new(
        "content_status",
        None,
        Some(FieldValue::Text(encode_content_status(
            ContentStatus::Empty,
        ))),
    ));

    if let Some(beat_type) = beat_type {
        revision = revision.with_field(FieldDelta::new(
            "beat_type",
            None,
            Some(FieldValue::Text(encode_beat_type(beat_type)?)),
        ));
    }
    if !arc_ids.is_empty() {
        revision = revision.with_field(FieldDelta::new(
            "arc_ids",
            None,
            Some(FieldValue::Text(encode_arc_ids(arc_ids)?)),
        ));
    }
    Ok(revision)
}

fn split_child_parent(
    child: &StoryNode,
    command: &CommandEnvelope<SplitTimelineNodeCommand>,
) -> NodeId {
    let child_mid =
        child.time_range.start_ms + (child.time_range.end_ms - child.time_range.start_ms) / 2;
    if child_mid < command.payload.at_ms {
        command.payload.left_node_id
    } else {
        command.payload.right_node_id
    }
}

fn split_child_reparent_revision(
    child_id: NodeId,
    old_parent_id: NodeId,
    new_parent_id: NodeId,
    event_id: ChangeEventId,
) -> ObjectRevision {
    ObjectRevision::new(
        ObjectKind::TimelineNode,
        child_id.0.to_string(),
        event_id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "parent_id",
        Some(FieldValue::Text(old_parent_id.0.to_string())),
        Some(FieldValue::Text(new_parent_id.0.to_string())),
    ))
}

fn split_relationship_revision(
    relationship: &Relationship,
    command: &CommandEnvelope<SplitTimelineNodeCommand>,
    event_id: ChangeEventId,
) -> Result<ObjectRevision, TimelineCommandError> {
    let mut revision = ObjectRevision::new(
        ObjectKind::TimelineRelationship,
        relationship.id.0.to_string(),
        event_id,
        RevisionOperation::Update,
    );

    if relationship.from_node == command.payload.node_id {
        revision = revision.with_field(FieldDelta::new(
            "from_node_id",
            Some(FieldValue::Text(relationship.from_node.0.to_string())),
            Some(FieldValue::Text(command.payload.left_node_id.0.to_string())),
        ));
    }
    if relationship.to_node == command.payload.node_id {
        revision = revision.with_field(FieldDelta::new(
            "to_node_id",
            Some(FieldValue::Text(relationship.to_node.0.to_string())),
            Some(FieldValue::Text(
                command.payload.right_node_id.0.to_string(),
            )),
        ));
    }

    Ok(revision)
}

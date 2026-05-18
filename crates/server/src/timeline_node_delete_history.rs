use eidetic_core::Project;
use eidetic_core::contracts::{
    ChangeEvent, ChangeEventId, ChangeEventKind, CommandEnvelope, DeleteTimelineNodeCommand,
    FieldDelta, FieldValue, ObjectKind, ObjectRevision, RevisionOperation,
};
use eidetic_core::story::arc::ArcId;
use eidetic_core::timeline::node::StoryNode;
use eidetic_core::timeline::relationship::Relationship;
use rusqlite::Connection;

use crate::history_store::{self, RecordChangeOutcome};
use crate::timeline_command::TimelineCommandError;
use crate::timeline_command_history_codec::{
    encode_arc_ids, encode_beat_type, encode_content_status, encode_relationship_type,
    encode_story_level,
};

pub(crate) fn record_delete_timeline_node_history(
    conn: &mut Connection,
    project: &Project,
    command: &CommandEnvelope<DeleteTimelineNodeCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, TimelineCommandError> {
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "timeline.node_delete")?
    {
        return Ok(outcome);
    }

    let node = project.timeline.node(command.payload.node_id)?;
    let mut removed_nodes = vec![node];
    removed_nodes.extend(project.timeline.descendants_of(command.payload.node_id));
    let removed_node_ids: Vec<_> = removed_nodes.iter().map(|node| node.id).collect();
    let removed_relationships: Vec<_> = project
        .timeline
        .relationships
        .iter()
        .filter(|relationship| {
            removed_node_ids.contains(&relationship.from_node)
                || removed_node_ids.contains(&relationship.to_node)
        })
        .collect();

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("delete timeline node {}", node.name),
    )
    .with_created_at_ms(created_at_ms);
    let mut revisions = Vec::new();
    for node in removed_nodes {
        revisions.push(deleted_node_revision(
            node,
            project.timeline.arcs_for_node(node.id),
            event.id,
        )?);
    }
    for relationship in removed_relationships {
        revisions.push(deleted_relationship_revision(relationship, event.id)?);
    }

    Ok(history_store::record_change(
        conn,
        command,
        "timeline.node_delete",
        &event,
        &revisions,
    )?)
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

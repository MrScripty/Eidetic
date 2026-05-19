use eidetic_core::Template;
use eidetic_core::contracts::{
    ApplyTimelineChildCommand, ApplyTimelineChildrenCommand, CommandEnvelope, CommandId,
    CreateTimelineNodeCommand, CreateTimelineRelationshipCommand, DeleteTimelineNodeCommand,
    DeleteTimelineRelationshipCommand, SetTimelineNodeLockCommand, SetTimelineNodeNotesCommand,
    SetTimelineNodeRangeCommand, SplitTimelineNodeCommand,
};
use eidetic_core::timeline::node::{ContentStatus, NodeId};
use eidetic_core::timeline::relationship::{Relationship, RelationshipId, RelationshipType};

use crate::timeline_command::{
    apply_create_timeline_node, apply_create_timeline_relationship, apply_delete_timeline_node,
    apply_delete_timeline_relationship, apply_set_timeline_node_lock,
    apply_set_timeline_node_notes, apply_set_timeline_node_range, apply_split_timeline_node,
    apply_timeline_children,
};

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
    let left_node_id = NodeId::new();
    let right_node_id = NodeId::new();
    let command = CommandEnvelope {
        id: CommandId::new(),
        payload: SplitTimelineNodeCommand {
            node_id: node.id,
            at_ms: split_ms,
            left_node_id,
            right_node_id,
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
            .any(|clip| clip.node_id == left_node_id
                && clip.start_ms == node.time_range.start_ms
                && clip.end_ms == split_ms)
    );
    assert!(
        projection
            .payload
            .clips
            .iter()
            .any(|clip| clip.node_id == right_node_id
                && clip.start_ms == split_ms
                && clip.end_ms == node.time_range.end_ms)
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
            node_id: NodeId::new(),
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
            node_id: NodeId::new(),
            notes: "New outline".to_string(),
        },
    };

    assert!(apply_set_timeline_node_notes(&mut project, &command).is_err());
}

#[test]
fn create_timeline_node_returns_projection_with_new_node() {
    let mut project = Template::MultiCam.build_project("Timeline Command Test");
    let parent = project.timeline.nodes[0].clone();
    let node_id = NodeId::new();
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
    let first_child_id = NodeId::new();
    let second_child_id = NodeId::new();
    let command = CommandEnvelope {
        id: CommandId::new(),
        payload: ApplyTimelineChildrenCommand {
            parent_id: parent.id,
            children: vec![
                ApplyTimelineChildCommand {
                    node_id: first_child_id,
                    name: "First child".to_string(),
                    outline: "First outline".to_string(),
                    weight: 1.0,
                    beat_type: None,
                    characters: Vec::new(),
                    location: None,
                    props: Vec::new(),
                },
                ApplyTimelineChildCommand {
                    node_id: second_child_id,
                    name: "Second child".to_string(),
                    outline: "Second outline".to_string(),
                    weight: 1.0,
                    beat_type: None,
                    characters: Vec::new(),
                    location: None,
                    props: Vec::new(),
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
    assert!(projection.payload.clips.iter().any(|clip| {
        clip.node_id == first_child_id
            && clip.parent_id == Some(parent.id)
            && clip.name == "First child"
    }));
    assert!(projection.payload.clips.iter().any(|clip| {
        clip.node_id == second_child_id
            && clip.parent_id == Some(parent.id)
            && clip.name == "Second child"
    }));
}

#[test]
fn create_timeline_relationship_returns_projection_with_relationship() {
    let mut project = Template::MultiCam.build_project("Timeline Command Test");
    let from_node = project.timeline.nodes[0].id;
    let to_node = project.timeline.nodes[1].id;
    let relationship_id = RelationshipId::new();
    let command = CommandEnvelope {
        id: CommandId::new(),
        payload: CreateTimelineRelationshipCommand {
            relationship_id,
            from_node_id: from_node,
            to_node_id: to_node,
            relationship_type: RelationshipType::Thematic,
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
            relationship_id: RelationshipId::new(),
            from_node_id: NodeId::new(),
            to_node_id: to_node,
            relationship_type: RelationshipType::Causal,
        },
    };

    assert!(apply_create_timeline_relationship(&mut project, &command).is_err());
}

#[test]
fn delete_timeline_relationship_returns_projection_without_relationship() {
    let mut project = Template::MultiCam.build_project("Timeline Command Test");
    let from_node = project.timeline.nodes[0].id;
    let to_node = project.timeline.nodes[1].id;
    let mut relationship = Relationship::new(from_node, to_node, RelationshipType::Thematic);
    let relationship_id = RelationshipId::new();
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

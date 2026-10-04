use eidetic_core::contracts::{
    ApplyTimelineChildrenCommand, CommandEnvelope, CreateTimelineNodeCommand,
    CreateTimelineRelationshipCommand, DeleteTimelineNodeCommand,
    DeleteTimelineRelationshipCommand, SetTimelineNodeLockCommand, SetTimelineNodeNotesCommand,
    SetTimelineNodeRangeCommand, SplitTimelineNodeCommand,
};
use eidetic_core::timeline::node::NodeId;
use eidetic_core::timeline::relationship::{Relationship, RelationshipId, RelationshipType};
use eidetic_core::{Project, Template};
use rusqlite::Connection;

use super::validate_current_timeline;
use crate::history_store::{self, RecordChangeOutcome};
use crate::timeline_command::{
    TimelineCommandError, record_apply_timeline_children_history,
    record_create_timeline_node_history, record_create_timeline_relationship_history,
    record_delete_timeline_node_history, record_delete_timeline_relationship_history,
    record_set_timeline_node_lock_history, record_set_timeline_node_notes_history,
    record_set_timeline_node_range_history, record_split_timeline_node_history,
};
use crate::{timeline_node_store, timeline_relationship_store};

fn fixture() -> (Connection, Project) {
    let mut project = Template::MultiCam.build_project("Shared timeline");
    project.timeline.relationships.push(Relationship::new(
        project.timeline.nodes[0].id,
        project.timeline.nodes[1].id,
        RelationshipType::Thematic,
    ));
    let mut conn = Connection::open_in_memory().unwrap();
    history_store::create_schema(&conn).unwrap();
    conn.execute_batch("CREATE TABLE project (id INTEGER PRIMARY KEY, total_duration_ms INTEGER);")
        .unwrap();
    conn.execute(
        "INSERT INTO project VALUES (1, ?1)",
        [project.timeline.total_duration_ms],
    )
    .unwrap();
    let tx = conn.transaction().unwrap();
    timeline_node_store::upsert_nodes_in_transaction(&tx, &project.timeline.nodes).unwrap();
    timeline_node_store::replace_node_arcs_in_transaction(&tx, &project.timeline.node_arcs)
        .unwrap();
    timeline_relationship_store::upsert_relationships_in_transaction(
        &tx,
        &project.timeline.relationships,
    )
    .unwrap();
    tx.commit().unwrap();
    (conn, project)
}

fn notes_command(project: &Project) -> CommandEnvelope<SetTimelineNodeNotesCommand> {
    CommandEnvelope::new(SetTimelineNodeNotesCommand {
        node_id: project.timeline.nodes[0].id,
        notes: "Intervening human edit".into(),
    })
}

fn assert_conflict(result: Result<RecordChangeOutcome, TimelineCommandError>) {
    let error = result.expect_err("stale command must fail");
    assert!(error.to_string().contains("timeline changed"), "{error}");
}

fn counts(conn: &Connection) -> Vec<i64> {
    [
        "commands",
        "change_events",
        "object_revisions",
        "object_revision_fields",
    ]
    .into_iter()
    .map(|table| {
        conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
    })
    .collect()
}

#[test]
fn every_shared_timeline_writer_rejects_a_stale_snapshot_atomically() {
    let (mut conn, project) = fixture();
    record_set_timeline_node_notes_history(&mut conn, &project, &notes_command(&project), 1)
        .unwrap();
    let original_counts = counts(&conn);
    let original_nodes =
        serde_json::to_value(timeline_node_store::load_nodes(&conn).unwrap()).unwrap();
    let node = project.timeline.nodes.last().unwrap();
    let parent = project.timeline.node(node.parent_id.unwrap()).unwrap();
    assert_conflict(record_set_timeline_node_range_history(
        &mut conn,
        &project,
        &CommandEnvelope::new(SetTimelineNodeRangeCommand {
            node_id: node.id,
            start_ms: node.time_range.start_ms,
            end_ms: node.time_range.end_ms,
        }),
        2,
    ));
    assert_conflict(record_set_timeline_node_lock_history(
        &mut conn,
        &project,
        &CommandEnvelope::new(SetTimelineNodeLockCommand {
            node_id: node.id,
            locked: true,
        }),
        2,
    ));
    assert_conflict(record_set_timeline_node_notes_history(
        &mut conn,
        &project,
        &notes_command(&project),
        2,
    ));
    assert_conflict(record_delete_timeline_node_history(
        &mut conn,
        &project,
        &CommandEnvelope::new(DeleteTimelineNodeCommand { node_id: node.id }),
        2,
    ));
    assert_conflict(record_split_timeline_node_history(
        &mut conn,
        &project,
        &CommandEnvelope::new(SplitTimelineNodeCommand {
            node_id: node.id,
            at_ms: node.time_range.start_ms + node.time_range.duration_ms() / 2,
            left_node_id: NodeId::new(),
            right_node_id: NodeId::new(),
        }),
        2,
    ));
    assert_conflict(record_create_timeline_node_history(
        &mut conn,
        &project,
        &CommandEnvelope::new(CreateTimelineNodeCommand {
            node_id: NodeId::new(),
            parent_id: node.parent_id,
            level: parent.level.child_level().unwrap(),
            name: "New clip".into(),
            start_ms: node.time_range.start_ms,
            end_ms: node.time_range.end_ms,
            beat_type: None,
        }),
        2,
    ));
    assert_conflict(record_apply_timeline_children_history(
        &mut conn,
        &project,
        &CommandEnvelope::new(ApplyTimelineChildrenCommand {
            parent_id: parent.id,
            child_plan_id: None,
            children: vec![],
        }),
        2,
    ));
    assert_conflict(record_create_timeline_relationship_history(
        &mut conn,
        &project,
        &CommandEnvelope::new(CreateTimelineRelationshipCommand {
            relationship_id: RelationshipId::new(),
            from_node_id: node.id,
            to_node_id: parent.id,
            relationship_type: RelationshipType::Causal,
        }),
        2,
    ));
    assert_conflict(record_delete_timeline_relationship_history(
        &mut conn,
        &project,
        &CommandEnvelope::new(DeleteTimelineRelationshipCommand {
            relationship_id: project.timeline.relationships[0].id,
        }),
        2,
    ));
    assert_eq!(
        counts(&conn),
        original_counts,
        "rejected commands leave no history"
    );
    assert_eq!(
        serde_json::to_value(timeline_node_store::load_nodes(&conn).unwrap()).unwrap(),
        original_nodes
    );
}

#[test]
fn identical_replay_succeeds_even_when_its_original_snapshot_is_stale() {
    let (mut conn, project) = fixture();
    let command = notes_command(&project);
    assert_eq!(
        record_set_timeline_node_notes_history(&mut conn, &project, &command, 1).unwrap(),
        RecordChangeOutcome::Recorded
    );
    let before = counts(&conn);
    assert_eq!(
        record_set_timeline_node_notes_history(&mut conn, &project, &command, 2).unwrap(),
        RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(counts(&conn), before);
    let mut changed = command;
    changed.payload.notes = "Different payload".into();
    assert!(
        record_set_timeline_node_notes_history(&mut conn, &project, &changed, 3)
            .unwrap_err()
            .to_string()
            .contains("different payload")
    );
}

#[test]
fn reloaded_snapshot_can_apply_without_overwriting_the_prior_edit() {
    let (mut conn, mut project) = fixture();
    let command = notes_command(&project);
    record_set_timeline_node_notes_history(&mut conn, &project, &command, 1).unwrap();
    project.timeline.nodes = timeline_node_store::load_nodes(&conn).unwrap();
    project.timeline.nodes.reverse(); // SQL order is not semantic identity.
    record_set_timeline_node_lock_history(
        &mut conn,
        &project,
        &CommandEnvelope::new(SetTimelineNodeLockCommand {
            node_id: command.payload.node_id,
            locked: true,
        }),
        2,
    )
    .unwrap();
    let nodes = timeline_node_store::load_nodes(&conn).unwrap();
    let node = nodes
        .iter()
        .find(|n| n.id == command.payload.node_id)
        .unwrap();
    assert_eq!(node.content.notes, command.payload.notes);
    assert!(node.locked);
}

#[test]
fn removed_rows_and_membership_changes_invalidate_snapshots() {
    for mutation in [
        "DELETE FROM nodes",
        "DELETE FROM relationships",
        "DELETE FROM node_arcs",
        "UPDATE project SET total_duration_ms = 1",
    ] {
        let (mut conn, project) = fixture();
        // The template must contain arc memberships for that deletion case.
        if mutation == "DELETE FROM node_arcs" && project.timeline.node_arcs.is_empty() {
            conn.execute(
                "INSERT INTO node_arcs VALUES (?1, ?2)",
                [
                    project.timeline.nodes[0].id.0.to_string(),
                    uuid::Uuid::new_v4().to_string(),
                ],
            )
            .unwrap();
        } else {
            conn.execute(mutation, []).unwrap();
        }
        let before = counts(&conn);
        assert_conflict(record_set_timeline_node_notes_history(
            &mut conn,
            &project,
            &notes_command(&project),
            1,
        ));
        assert_eq!(counts(&conn), before);
    }
}

#[test]
fn current_rows_are_checked_inside_the_write_transaction() {
    let (mut conn, project) = fixture();
    let tx = conn.transaction().unwrap();
    validate_current_timeline(&tx, &project.timeline).unwrap();
    tx.execute("UPDATE nodes SET locked = 1", []).unwrap();
    assert!(validate_current_timeline(&tx, &project.timeline).is_err());
    tx.rollback().unwrap();
    let tx = conn.transaction().unwrap();
    validate_current_timeline(&tx, &project.timeline).unwrap();
}

#[test]
fn deleting_the_last_subtree_cannot_be_undone_by_a_stale_writer() {
    let (mut conn, project) = fixture();
    let root_id = project.timeline.nodes[0].id;
    record_delete_timeline_node_history(
        &mut conn,
        &project,
        &CommandEnvelope::new(DeleteTimelineNodeCommand { node_id: root_id }),
        1,
    )
    .unwrap();
    assert!(timeline_node_store::load_nodes(&conn).unwrap().is_empty());
    let before = counts(&conn);
    assert_conflict(record_set_timeline_node_notes_history(
        &mut conn,
        &project,
        &notes_command(&project),
        2,
    ));
    assert!(timeline_node_store::load_nodes(&conn).unwrap().is_empty());
    assert!(
        timeline_relationship_store::load_relationships(&conn)
            .unwrap()
            .is_empty()
    );
    assert_eq!(counts(&conn), before);
}

#[test]
fn structural_commands_accept_a_current_snapshot() {
    for operation in [
        "create",
        "split",
        "replace_children",
        "relationship_create",
        "relationship_delete",
    ] {
        let (mut conn, project) = fixture();
        let node = project.timeline.nodes.last().unwrap();
        let parent = project.timeline.node(node.parent_id.unwrap()).unwrap();
        let outcome = match operation {
            "create" => record_create_timeline_node_history(
                &mut conn,
                &project,
                &CommandEnvelope::new(CreateTimelineNodeCommand {
                    node_id: NodeId::new(),
                    parent_id: node.parent_id,
                    level: parent.level.child_level().unwrap(),
                    name: "New clip".into(),
                    start_ms: node.time_range.start_ms,
                    end_ms: node.time_range.end_ms,
                    beat_type: None,
                }),
                1,
            ),
            "split" => record_split_timeline_node_history(
                &mut conn,
                &project,
                &CommandEnvelope::new(SplitTimelineNodeCommand {
                    node_id: node.id,
                    at_ms: node.time_range.start_ms + node.time_range.duration_ms() / 2,
                    left_node_id: NodeId::new(),
                    right_node_id: NodeId::new(),
                }),
                1,
            ),
            "replace_children" => record_apply_timeline_children_history(
                &mut conn,
                &project,
                &CommandEnvelope::new(ApplyTimelineChildrenCommand {
                    parent_id: parent.id,
                    child_plan_id: None,
                    children: vec![],
                }),
                1,
            ),
            "relationship_create" => record_create_timeline_relationship_history(
                &mut conn,
                &project,
                &CommandEnvelope::new(CreateTimelineRelationshipCommand {
                    relationship_id: RelationshipId::new(),
                    from_node_id: node.id,
                    to_node_id: parent.id,
                    relationship_type: RelationshipType::Causal,
                }),
                1,
            ),
            "relationship_delete" => record_delete_timeline_relationship_history(
                &mut conn,
                &project,
                &CommandEnvelope::new(DeleteTimelineRelationshipCommand {
                    relationship_id: project.timeline.relationships[0].id,
                }),
                1,
            ),
            _ => unreachable!(),
        }
        .unwrap_or_else(|error| panic!("{operation}: {error}"));
        assert_eq!(outcome, RecordChangeOutcome::Recorded);
        assert_eq!(counts(&conn)[0], 1);
    }
}

#[tokio::test]
async fn real_project_load_then_command_preserves_database_text_and_ydoc_blob() {
    let path = std::env::temp_dir().join(format!("eidetic-guard-{}.db", uuid::Uuid::new_v4()));
    let mut project = Template::MultiCam.build_project("Durable timeline");
    let node_id = project.timeline.nodes[0].id;
    project.timeline.nodes[0].content.notes = "Canonical notes".into();
    project.timeline.nodes[0].content.content = "Canonical screenplay cache".into();
    // The persistence loader returns this separately; it must not overlay rows.
    let ydoc_blob = vec![0, 1, 2, 3];
    crate::persistence::save_project(&project, &path, Some(ydoc_blob.clone()))
        .await
        .unwrap();
    let (loaded, loaded_ydoc) = crate::persistence::load_project(&path).await.unwrap();
    assert_eq!(loaded_ydoc, Some(ydoc_blob.clone()));
    let mut conn = crate::sqlite::open_write_connection(&path).unwrap();
    record_set_timeline_node_lock_history(
        &mut conn,
        &loaded,
        &CommandEnvelope::new(SetTimelineNodeLockCommand {
            node_id,
            locked: true,
        }),
        1,
    )
    .unwrap();
    drop(conn);
    let (reloaded, reloaded_ydoc) = crate::persistence::load_project(&path).await.unwrap();
    let node = reloaded.timeline.node(node_id).unwrap();
    assert_eq!(node.content.notes, "Canonical notes");
    assert_eq!(node.content.content, "Canonical screenplay cache");
    assert!(node.locked);
    assert_eq!(reloaded_ydoc, Some(ydoc_blob));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn command_before_initial_save_fails_explicitly_without_seeding_or_history() {
    let mut conn = Connection::open_in_memory().unwrap();
    history_store::create_schema(&conn).unwrap();
    let project = Template::MultiCam.build_project("Not saved yet");
    let error =
        record_set_timeline_node_notes_history(&mut conn, &project, &notes_command(&project), 1)
            .unwrap_err();
    assert!(error.to_string().contains("not durably initialized"));
    assert_eq!(counts(&conn), vec![0, 0, 0, 0]);
    assert!(timeline_node_store::load_nodes(&conn).unwrap().is_empty());
}

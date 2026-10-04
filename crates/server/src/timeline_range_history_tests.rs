use eidetic_core::contracts::{
    CommandEnvelope, FieldValue, ObjectKind, SetTimelineNodeRangeCommand,
};
use eidetic_core::timeline::node::{NodeId, StoryLevel, StoryNode};
use eidetic_core::timeline::timing::TimeRange;
use eidetic_core::{Project, Template};
use rusqlite::Connection;

use super::record_set_timeline_node_range_history;
use crate::history_store::{self, RecordChangeOutcome};
use crate::{revision_projection, timeline_node_store, timeline_relationship_store};

fn fixture() -> (Connection, Project, Vec<NodeId>) {
    let mut project = Template::MultiCam.build_project("Range history");
    project.timeline.total_duration_ms = 1_000;
    project.timeline.nodes.clear();
    project.timeline.node_arcs.clear();
    project.timeline.relationships.clear();
    let mut ids = Vec::new();
    for (level, start, end) in [
        (StoryLevel::Premise, 0, 1_000),
        (StoryLevel::Act, 100, 900),
        (StoryLevel::Sequence, 200, 600),
        (StoryLevel::Scene, 250, 450),
        (StoryLevel::Beat, 300, 400),
    ] {
        let mut node = StoryNode::new(level.label(), level, TimeRange::new(start, end).unwrap());
        node.parent_id = ids.last().copied();
        ids.push(node.id);
        project.timeline.add_node(node).unwrap();
    }
    let sibling = StoryNode::new_child(
        "Unrelated act",
        StoryLevel::Act,
        TimeRange::new(900, 1_000).unwrap(),
        ids[0],
    );
    ids.push(sibling.id);
    project.timeline.add_node(sibling).unwrap();
    let mut conn = Connection::open_in_memory().unwrap();
    history_store::create_schema(&conn).unwrap();
    conn.execute_batch("CREATE TABLE project (id INTEGER PRIMARY KEY, total_duration_ms INTEGER); INSERT INTO project VALUES (1, 1000);").unwrap();
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
    (conn, project, ids)
}

fn command(
    node_id: NodeId,
    start_ms: u64,
    end_ms: u64,
) -> CommandEnvelope<SetTimelineNodeRangeCommand> {
    CommandEnvelope::new(SetTimelineNodeRangeCommand {
        node_id,
        start_ms,
        end_ms,
    })
}

#[test]
fn parent_resize_records_each_changed_descendant_and_no_unrelated_clip() {
    let (mut conn, project, ids) = fixture();
    let command = command(ids[1], 100, 500);
    record_set_timeline_node_range_history(&mut conn, &project, &command, 1).unwrap();
    let current = timeline_node_store::load_nodes(&conn).unwrap();
    let mut event_id = None;
    for old in &project.timeline.nodes {
        let new = current.iter().find(|node| node.id == old.id).unwrap();
        let revisions = history_store::load_revisions_for_object(
            &conn,
            ObjectKind::TimelineNode,
            &old.id.0.to_string(),
        )
        .unwrap();
        if old.time_range == new.time_range {
            assert!(revisions.is_empty(), "unchanged clips have no revisions");
            continue;
        }
        assert_eq!(revisions.len(), 1);
        let revision = &revisions[0];
        assert_eq!(
            *event_id.get_or_insert(revision.change_event_id),
            revision.change_event_id
        );
        assert_eq!(revision.fields.len(), 2);
        for (key, old_value, new_value) in [
            ("start_ms", old.time_range.start_ms, new.time_range.start_ms),
            ("end_ms", old.time_range.end_ms, new.time_range.end_ms),
        ] {
            let field = revision
                .fields
                .iter()
                .find(|field| field.field_key == key)
                .unwrap();
            assert_eq!(field.old_value, Some(FieldValue::Integer(old_value as i64)));
            assert_eq!(field.new_value, Some(FieldValue::Integer(new_value as i64)));
        }
    }
    let count: i64 = conn
        .query_row("SELECT count(*) FROM object_revisions", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 4, "act plus sequence, scene and beat");
    assert_eq!(
        record_set_timeline_node_range_history(&mut conn, &project, &command, 2).unwrap(),
        RecordChangeOutcome::AlreadyRecorded
    );
    let after: i64 = conn
        .query_row("SELECT count(*) FROM object_revisions", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(after, count);
}

#[test]
fn direct_child_edit_replays_after_earlier_parent_resize_despite_event_local_sort_order() {
    let (mut conn, mut project, ids) = fixture();
    record_set_timeline_node_range_history(&mut conn, &project, &command(ids[1], 100, 500), 10)
        .unwrap();
    project.timeline.nodes = timeline_node_store::load_nodes(&conn).unwrap();
    let child_command = command(ids[3], 180, 260);
    // Clock order is deliberately reversed: committed event order is authoritative.
    record_set_timeline_node_range_history(&mut conn, &project, &child_command, 5).unwrap();
    let revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &ids[3].0.to_string(),
    )
    .unwrap();
    assert_eq!(revisions.len(), 2);
    assert_eq!(
        revisions[0].fields[1].new_value,
        Some(FieldValue::Integer(275))
    );
    assert_eq!(
        revisions[1].fields[1].new_value,
        Some(FieldValue::Integer(260))
    );
    let projection = revision_projection::load_object_field_projection(
        &conn,
        ObjectKind::TimelineNode,
        &ids[3].0.to_string(),
    )
    .unwrap();
    assert_eq!(projection.fields["start_ms"], FieldValue::Integer(180));
    assert_eq!(projection.fields["end_ms"], FieldValue::Integer(260));
    let current = timeline_node_store::load_nodes(&conn).unwrap();
    assert_eq!(
        current
            .iter()
            .find(|node| node.id == ids[3])
            .unwrap()
            .time_range,
        TimeRange::new(180, 260).unwrap()
    );
}

#[test]
fn unchanged_descendants_do_not_gain_revisions_for_noop_parent_range() {
    let (mut conn, project, ids) = fixture();
    record_set_timeline_node_range_history(&mut conn, &project, &command(ids[1], 100, 900), 1)
        .unwrap();
    let count: i64 = conn
        .query_row("SELECT count(*) FROM object_revisions", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(
        count, 1,
        "retain existing target-command history, no unchanged descendants"
    );
}

#[test]
fn stale_parent_resize_leaves_no_partial_descendant_history() {
    let (mut conn, project, ids) = fixture();
    record_set_timeline_node_range_history(&mut conn, &project, &command(ids[3], 260, 460), 1)
        .unwrap();
    let before: i64 = conn
        .query_row("SELECT count(*) FROM object_revisions", [], |row| {
            row.get(0)
        })
        .unwrap();
    let error =
        record_set_timeline_node_range_history(&mut conn, &project, &command(ids[1], 100, 500), 2)
            .unwrap_err();
    assert!(error.to_string().contains("timeline changed"));
    let after: i64 = conn
        .query_row("SELECT count(*) FROM object_revisions", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(before, after);
    let revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &ids[1].0.to_string(),
    )
    .unwrap();
    assert!(revisions.is_empty());
}

#[tokio::test]
async fn broad_save_and_reopen_preserve_append_only_event_order() {
    let (_, project, ids) = fixture();
    let path =
        std::env::temp_dir().join(format!("eidetic-range-history-{}.db", uuid::Uuid::new_v4()));
    crate::persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    let mut conn = crate::sqlite::open_write_connection(&path).unwrap();
    record_set_timeline_node_range_history(&mut conn, &project, &command(ids[1], 100, 500), 10)
        .unwrap();
    let mut latest = project.clone();
    latest.timeline.nodes = timeline_node_store::load_nodes(&conn).unwrap();
    record_set_timeline_node_range_history(&mut conn, &latest, &command(ids[3], 180, 260), 5)
        .unwrap();
    let events_before = event_rows(&conn);
    drop(conn);
    // Broad save uses an intentionally stale mirror; history must be retained.
    crate::persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    let (loaded, _) = crate::persistence::load_project(&path).await.unwrap();
    assert_eq!(
        loaded.timeline.node(ids[3]).unwrap().time_range,
        TimeRange::new(180, 260).unwrap()
    );
    let conn = crate::sqlite::open_write_connection(&path).unwrap();
    assert_eq!(event_rows(&conn), events_before);
    let projection = revision_projection::load_object_field_projection(
        &conn,
        ObjectKind::TimelineNode,
        &ids[3].0.to_string(),
    )
    .unwrap();
    assert_eq!(projection.fields["end_ms"], FieldValue::Integer(260));
    drop(conn);
    std::fs::remove_file(path).unwrap();
}

fn event_rows(conn: &Connection) -> Vec<(i64, String)> {
    conn.prepare("SELECT rowid, id FROM change_events ORDER BY rowid")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

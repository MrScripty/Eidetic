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
        expected: None,
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

fn history_counts(conn: &Connection) -> Vec<i64> {
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

fn node_rows(conn: &Connection) -> serde_json::Value {
    serde_json::to_value(timeline_node_store::load_nodes(conn).unwrap()).unwrap()
}

#[tokio::test]
async fn wal_reader_cannot_promote_a_validated_snapshot_after_another_writer_commits() {
    let (_, project, ids) = fixture();
    let path =
        std::env::temp_dir().join(format!("eidetic-wal-snapshot-{}.db", uuid::Uuid::new_v4()));
    crate::persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    let mut reader = crate::sqlite::open_write_connection(&path).unwrap();
    let mut writer = crate::sqlite::open_write_connection(&path).unwrap();
    let mode: String = reader
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap();
    assert_eq!(mode, "wal");

    // Deterministic overlap: keep a validated read transaction alive while the
    // other connection commits. WAL permits this without sleeps or a scheduler race.
    let tx = reader.transaction().unwrap();
    crate::timeline_command_guard::validate_current_timeline(&tx, &project.timeline).unwrap();
    let before = node_rows(&tx);
    let edit = command(ids[1], 100, 500);
    record_set_timeline_node_range_history(&mut writer, &project, &edit, 10).unwrap();
    assert_eq!(
        node_rows(&tx),
        before,
        "reader retains its old WAL snapshot"
    );
    assert_eq!(history_counts(&tx), vec![0, 0, 0, 0]);
    let committed = node_rows(&writer);
    assert_ne!(committed, before);
    assert_eq!(history_counts(&writer), vec![1, 1, 4, 8]);

    // Snapshot equality alone cannot see the intervening commit; SQLite must
    // refuse to upgrade this transaction to a writer (SQLITE_BUSY_SNAPSHOT).
    crate::timeline_command_guard::validate_current_timeline(&tx, &project.timeline).unwrap();
    let error =
        timeline_node_store::upsert_nodes_in_transaction(&tx, &project.timeline.nodes).unwrap_err();
    match error {
        history_store::HistoryStoreError::Sqlite(rusqlite::Error::SqliteFailure(error, _)) => {
            assert_eq!(error.extended_code, rusqlite::ffi::SQLITE_BUSY_SNAPSHOT);
        }
        error => panic!("expected WAL snapshot conflict, got {error}"),
    }
    tx.rollback().unwrap();
    assert_eq!(node_rows(&reader), committed);
    assert_eq!(history_counts(&reader), vec![1, 1, 4, 8]);
    drop(reader);
    drop(writer);
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn wal_stale_edit_rejects_then_reload_and_replay_preserve_committed_history() {
    let (_, project, ids) = fixture();
    let path = std::env::temp_dir().join(format!("eidetic-wal-edit-{}.db", uuid::Uuid::new_v4()));
    let ydoc = vec![0, 1, 2, 3];
    crate::persistence::save_project(&project, &path, Some(ydoc.clone()))
        .await
        .unwrap();
    let (stale, _) = crate::persistence::load_project(&path).await.unwrap();
    let mut first = crate::sqlite::open_write_connection(&path).unwrap();
    let mut second = crate::sqlite::open_write_connection(&path).unwrap();
    let parent_edit = command(ids[1], 100, 500);
    record_set_timeline_node_range_history(&mut first, &project, &parent_edit, 10).unwrap();
    let child_edit = command(ids[3], 180, 260);
    let committed = node_rows(&first);
    let error =
        record_set_timeline_node_range_history(&mut second, &stale, &child_edit, 5).unwrap_err();
    assert!(error.to_string().contains("timeline changed"), "{error}");
    assert_eq!(node_rows(&second), committed);
    assert_eq!(history_counts(&second), vec![1, 1, 4, 8]);

    let (fresh, _) = crate::persistence::load_project(&path).await.unwrap();
    record_set_timeline_node_range_history(&mut second, &fresh, &child_edit, 5).unwrap();
    let counts = history_counts(&second);
    let events = event_rows(&second);
    // Replaying the same identity and payload remains valid after a later edit.
    assert_eq!(
        record_set_timeline_node_range_history(&mut first, &stale, &parent_edit, 20).unwrap(),
        RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(history_counts(&first), counts);
    assert_eq!(event_rows(&first), events);
    let projection = revision_projection::load_object_field_projection(
        &first,
        ObjectKind::TimelineNode,
        &ids[3].0.to_string(),
    )
    .unwrap();
    assert_eq!(projection.fields["end_ms"], FieldValue::Integer(260));
    drop(first);
    drop(second);
    let (reopened, blob) = crate::persistence::load_project(&path).await.unwrap();
    assert_eq!(
        reopened.timeline.node(ids[3]).unwrap().time_range,
        TimeRange::new(180, 260).unwrap()
    );
    assert_eq!(blob, Some(ydoc));
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn interrupted_descendant_write_rolls_back_history_and_current_state_on_reopen() {
    for action in ["ABORT", "ROLLBACK"] {
        let (_, project, ids) = fixture();
        let path =
            std::env::temp_dir().join(format!("eidetic-interrupted-{}.db", uuid::Uuid::new_v4()));
        let ydoc = vec![0, 1, 2, 3];
        crate::persistence::save_project(&project, &path, Some(ydoc.clone()))
            .await
            .unwrap();
        let mut writer = crate::sqlite::open_write_connection(&path).unwrap();
        let observer = crate::sqlite::open_write_connection(&path).unwrap();
        // Retain an existing event so rollback must preserve committed history,
        // not merely leave an initially empty history table empty.
        record_set_timeline_node_range_history(
            &mut writer,
            &project,
            &command(ids[5], 900, 1_000),
            0,
        )
        .unwrap();
        let baseline_events = event_rows(&observer);
        let before = node_rows(&observer);
        // Interrupt only after all history and an earlier ancestor row have
        // been written. ABORT leaves rollback to Transaction::drop; ROLLBACK
        // makes SQLite end the transaction itself. Neither may leak a prefix.
        writer
            .execute_batch(&format!(
                "CREATE TEMP TRIGGER interrupt_descendant AFTER UPDATE ON nodes
             WHEN NEW.id = '{}' AND NEW.end_ms = 275
               AND (SELECT end_ms FROM nodes WHERE id = '{}') = 500
               AND (SELECT count(*) FROM object_revisions) = 5
               AND (SELECT count(*) FROM object_revision_fields) = 10
             BEGIN SELECT RAISE({action}, 'injected descendant interruption'); END;",
                ids[3].0, ids[1].0,
            ))
            .unwrap();
        let edit = command(ids[1], 100, 500);
        let error =
            record_set_timeline_node_range_history(&mut writer, &project, &edit, 1).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("injected descendant interruption"),
            "{error}"
        );
        assert!(writer.is_autocommit());
        assert_eq!(node_rows(&writer), before);
        assert_eq!(node_rows(&observer), before);
        assert_eq!(history_counts(&writer), vec![1, 1, 1, 2]);
        assert_eq!(history_counts(&observer), vec![1, 1, 1, 2]);
        assert_eq!(event_rows(&observer), baseline_events);
        drop(writer);
        drop(observer);

        let (reopened, blob) = crate::persistence::load_project(&path).await.unwrap();
        assert_eq!(blob, Some(ydoc));
        let mut conn = crate::sqlite::open_write_connection(&path).unwrap();
        assert_eq!(node_rows(&conn), before);
        assert_eq!(history_counts(&conn), vec![1, 1, 1, 2]);
        assert_eq!(event_rows(&conn), baseline_events);
        // Failed attempts do not reserve an idempotency key. Explicitly submit
        // the reviewed command after reopening, then prove its replay is inert.
        assert_eq!(
            record_set_timeline_node_range_history(&mut conn, &reopened, &edit, 2).unwrap(),
            RecordChangeOutcome::Recorded
        );
        assert_eq!(history_counts(&conn), vec![2, 2, 5, 10]);
        assert_eq!(
            record_set_timeline_node_range_history(&mut conn, &reopened, &edit, 3).unwrap(),
            RecordChangeOutcome::AlreadyRecorded
        );
        assert_eq!(history_counts(&conn), vec![2, 2, 5, 10]);
        drop(conn);
        std::fs::remove_file(path).unwrap();
    }
}

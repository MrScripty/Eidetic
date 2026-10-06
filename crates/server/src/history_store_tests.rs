use super::*;
use eidetic_core::contracts::{ChangeEventId, ChangeEventKind, ObjectKind, RevisionOperation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TestCommand {
    label: String,
}

fn memory_connection() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    create_schema(&conn).unwrap();
    conn
}

fn command(label: &str) -> CommandEnvelope<TestCommand> {
    CommandEnvelope::new(TestCommand {
        label: label.to_string(),
    })
}

fn event(command_id: CommandId) -> ChangeEvent {
    ChangeEvent::new(command_id, ChangeEventKind::UserEdit, "edit weather").with_created_at_ms(42)
}

fn revision(event_id: ChangeEventId) -> ObjectRevision {
    ObjectRevision::new(
        ObjectKind::BiblePartField,
        "field-weather",
        event_id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "weather",
        Some(FieldValue::Text("sunny".to_string())),
        Some(FieldValue::Text("rainy".to_string())),
    ))
    .with_field(FieldDelta::new(
        "is_locked",
        Some(FieldValue::Bool(false)),
        Some(FieldValue::Bool(true)),
    ))
    .with_field(FieldDelta::new(
        "scene_ref",
        None,
        Some(FieldValue::ObjectRef {
            kind: ObjectKind::TimelineNode,
            id: "scene-1".to_string(),
        }),
    ))
}

#[test]
fn record_change_persists_event_and_sparse_revision_fields() {
    let mut conn = memory_connection();
    let command = command("update weather");
    let event = event(command.id);
    let revision = revision(event.id);

    let outcome = record_change(
        &mut conn,
        &command,
        "test.update_weather",
        &event,
        std::slice::from_ref(&revision),
    )
    .unwrap();
    assert_eq!(outcome, RecordChangeOutcome::Recorded);

    let decoded: CommandEnvelope<TestCommand> = load_command(&conn, command.id).unwrap().unwrap();
    assert_eq!(decoded, command);

    let revisions =
        load_revisions_for_object(&conn, ObjectKind::BiblePartField, "field-weather").unwrap();
    assert_eq!(revisions, vec![revision]);
}

#[test]
fn duplicate_command_id_is_idempotent() {
    let mut conn = memory_connection();
    let command = command("update weather");
    let event = event(command.id);
    let revision = revision(event.id);

    let first = record_change(
        &mut conn,
        &command,
        "test.update_weather",
        &event,
        std::slice::from_ref(&revision),
    )
    .unwrap();
    let second = record_change(
        &mut conn,
        &command,
        "test.update_weather",
        &event,
        &[revision],
    )
    .unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "change_events"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 1);
}

#[test]
fn duplicate_command_id_rejects_different_payload() {
    let mut conn = memory_connection();
    let mut command = command("update weather");
    let event = event(command.id);
    let revision = revision(event.id);
    record_change(
        &mut conn,
        &command,
        "test.update_weather",
        &event,
        std::slice::from_ref(&revision),
    )
    .unwrap();

    command.payload.label = "different".to_string();
    let error = record_change(
        &mut conn,
        &command,
        "test.update_weather",
        &event,
        &[revision],
    )
    .unwrap_err();

    assert!(matches!(error, HistoryStoreError::InvalidValue(_)));
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 1);
}

#[test]
fn failed_revision_rolls_back_command_and_event() {
    let mut conn = memory_connection();
    let command = command("broken update");
    let event = event(command.id);
    let revision = ObjectRevision::new(
        ObjectKind::BiblePartField,
        "",
        event.id,
        RevisionOperation::Update,
    );

    let error = record_change(
        &mut conn,
        &command,
        "test.update_weather",
        &event,
        &[revision],
    )
    .unwrap_err();

    assert!(matches!(error, HistoryStoreError::Sqlite(_)));
    assert_eq!(table_count(&conn, "commands"), 0);
    assert_eq!(table_count(&conn, "change_events"), 0);
    assert_eq!(table_count(&conn, "object_revisions"), 0);
}

#[test]
fn failed_current_state_write_rolls_back_history_rows() {
    let mut conn = memory_connection();
    let command = command("broken current state update");
    let event = event(command.id);
    let revision = revision(event.id);

    let error = record_change_with(
        &mut conn,
        &command,
        "test.update_weather",
        &event,
        &[revision],
        |_| {
            Err(HistoryStoreError::InvalidValue(
                "current state update failed".to_string(),
            ))
        },
    )
    .unwrap_err();

    assert!(matches!(error, HistoryStoreError::InvalidValue(_)));
    assert_eq!(table_count(&conn, "commands"), 0);
    assert_eq!(table_count(&conn, "change_events"), 0);
    assert_eq!(table_count(&conn, "object_revisions"), 0);
}

#[test]
fn revision_summary_reports_count_and_latest_event_for_kind() {
    let mut conn = memory_connection();
    let first_command = command("first update");
    let first_event = event(first_command.id);
    let first_revision = revision(first_event.id);
    record_change(
        &mut conn,
        &first_command,
        "test.update_weather",
        &first_event,
        &[first_revision],
    )
    .unwrap();

    let second_command = command("second update");
    let second_event = event(second_command.id);
    let second_revision = revision(second_event.id);
    record_change(
        &mut conn,
        &second_command,
        "test.update_weather",
        &second_event,
        &[second_revision],
    )
    .unwrap();

    let summary = load_revision_summary_for_kind(&conn, ObjectKind::BiblePartField).unwrap();

    assert_eq!(summary.revision_count, 2);
    assert_eq!(summary.latest_change_event_id, Some(second_event.id));
}

fn table_count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}

#[test]
fn committed_replay_remains_read_only_while_another_connection_owns_the_writer() {
    let directory =
        std::env::temp_dir().join(format!("eidetic-history-replay-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("project.db");
    let mut conn = crate::sqlite::open_write_connection(&path).unwrap();
    create_schema(&conn).unwrap();
    let mut command = command("immutable committed operation");
    let event = event(command.id);
    let revision = revision(event.id);
    record_change(
        &mut conn,
        &command,
        "test.read_only_replay",
        &event,
        std::slice::from_ref(&revision),
    )
    .unwrap();
    conn.pragma_update(None, "query_only", true).unwrap();
    let before = conn.total_changes();
    let other = Connection::open(&path).unwrap();
    other.execute_batch("BEGIN IMMEDIATE").unwrap();
    let outcome = record_change_with(
        &mut conn,
        &command,
        "test.read_only_replay",
        &event,
        &[revision],
        |_| panic!("committed replay must not apply current state twice"),
    )
    .unwrap();
    assert_eq!(outcome, RecordChangeOutcome::AlreadyRecorded);
    command.payload.label = "conflicting replay payload".into();
    assert!(matches!(
        record_change(&mut conn, &command, "test.read_only_replay", &event, &[]),
        Err(HistoryStoreError::InvalidValue(_))
    ));
    assert_eq!(conn.total_changes(), before);
    assert_eq!(table_count(&conn, "commands"), 1);
    assert_eq!(table_count(&conn, "change_events"), 1);
    assert_eq!(table_count(&conn, "object_revisions"), 1);
    other.execute_batch("ROLLBACK").unwrap();
    drop(other);
    drop(conn);
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn concurrent_fresh_command_id_rechecks_signature_after_writer_admission() {
    use crate::write_concurrency_probe::{self as probe, CommandProbe, CommandStage};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    for conflict in [false, true] {
        let directory = std::env::temp_dir().join(format!(
            "eidetic-history-admission-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("project.db");
        let initial = crate::sqlite::open_write_connection(&path).unwrap();
        create_schema(&initial).unwrap();
        drop(initial);
        let command = command("first payload");
        let first_event = event(command.id);
        let (first_stages, mut first_rx) = tokio::sync::mpsc::unbounded_channel();
        let (release_first, release) = std::sync::mpsc::channel();
        let _first_probe = probe::command(
            command.id,
            CommandProbe {
                stages: first_stages,
                release,
            },
        );
        let applications = Arc::new(AtomicUsize::new(0));
        let applied = applications.clone();
        let first_path = path.clone();
        let first_command = command.clone();
        let first = std::thread::spawn(move || {
            let mut conn = crate::sqlite::open_write_connection(&first_path).unwrap();
            record_change_with(
                &mut conn,
                &first_command,
                "test.concurrent_admission",
                &first_event,
                &[],
                |_| {
                    applied.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                },
            )
        });
        for expected_signature in [false, true] {
            let stage = tokio::time::timeout(std::time::Duration::from_secs(10), first_rx.recv())
                .await
                .unwrap()
                .unwrap();
            assert!(if expected_signature {
                matches!(stage, CommandStage::SignatureRead)
            } else {
                matches!(stage, CommandStage::BeforeAdmission)
            });
        }
        // The first owns the writer and has not inserted its command. Force a
        // second caller to observe the same ID missing before writer admission.
        let (second_stages, mut second_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_release_second, release) = std::sync::mpsc::channel();
        let _second_probe = probe::command(
            command.id,
            CommandProbe {
                stages: second_stages,
                release,
            },
        );
        let mut second_command = command;
        if conflict {
            second_command.payload.label = "conflicting payload".into();
        }
        let second_path = path.clone();
        let second = std::thread::spawn(move || {
            let mut conn = crate::sqlite::open_write_connection(&second_path).unwrap();
            let second_event = event(second_command.id);
            record_change_with(
                &mut conn,
                &second_command,
                "test.concurrent_admission",
                &second_event,
                &[],
                |_| panic!("second caller must not apply an already committed ID"),
            )
        });
        assert!(matches!(
            tokio::time::timeout(std::time::Duration::from_secs(10), second_rx.recv())
                .await
                .unwrap()
                .unwrap(),
            CommandStage::BeforeAdmission
        ));
        release_first.send(()).unwrap();
        assert_eq!(
            first.join().unwrap().unwrap(),
            RecordChangeOutcome::Recorded
        );
        let result = second.join().unwrap();
        if conflict {
            assert!(matches!(result, Err(HistoryStoreError::InvalidValue(_))));
        } else {
            assert_eq!(result.unwrap(), RecordChangeOutcome::AlreadyRecorded);
        }
        assert_eq!(applications.load(Ordering::SeqCst), 1);
        let conn = Connection::open(&path).unwrap();
        assert_eq!(table_count(&conn, "commands"), 1);
        assert_eq!(table_count(&conn, "change_events"), 1);
        drop(conn);
        std::fs::remove_dir_all(directory).unwrap();
    }
}

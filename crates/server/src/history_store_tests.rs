use super::*;
use eidetic_core::contracts::ChangeEventKind;
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
        &[revision.clone()],
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
        &[revision.clone()],
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

fn table_count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}

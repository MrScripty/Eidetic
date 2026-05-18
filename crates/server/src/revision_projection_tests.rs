use super::*;
use crate::history_store::{create_schema, record_change};
use eidetic_core::contracts::{
    ChangeEvent, ChangeEventId, ChangeEventKind, CommandEnvelope, FieldDelta, ObjectRevision,
};
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

fn event(command: &CommandEnvelope<TestCommand>, summary: &str) -> ChangeEvent {
    ChangeEvent::new(command.id, ChangeEventKind::UserEdit, summary)
}

fn base_revision(event_id: ChangeEventId, operation: RevisionOperation) -> ObjectRevision {
    ObjectRevision::new(
        ObjectKind::BiblePartField,
        "field-weather",
        event_id,
        operation,
    )
}

#[test]
fn projection_rebuilds_current_fields_from_revisions() {
    let mut conn = memory_connection();

    let create_command = command("create");
    let create_event = event(&create_command, "create field");
    let create_revision = base_revision(create_event.id, RevisionOperation::Create)
        .with_field(FieldDelta::new(
            "weather",
            None,
            Some(FieldValue::Text("sunny".to_string())),
        ))
        .with_field(FieldDelta::new(
            "locked",
            None,
            Some(FieldValue::Bool(false)),
        ));
    record_change(
        &mut conn,
        &create_command,
        "test.create_field",
        &create_event,
        &[create_revision],
    )
    .unwrap();

    let update_command = command("update");
    let update_event = event(&update_command, "update field");
    let update_revision = base_revision(update_event.id, RevisionOperation::Update)
        .with_field(FieldDelta::new(
            "weather",
            Some(FieldValue::Text("sunny".to_string())),
            Some(FieldValue::Text("rainy".to_string())),
        ))
        .with_field(FieldDelta::new(
            "locked",
            Some(FieldValue::Bool(false)),
            None,
        ));
    record_change(
        &mut conn,
        &update_command,
        "test.update_field",
        &update_event,
        &[update_revision],
    )
    .unwrap();

    let projection =
        load_object_field_projection(&conn, ObjectKind::BiblePartField, "field-weather").unwrap();

    assert!(!projection.deleted);
    assert_eq!(
        projection.fields.get("weather"),
        Some(&FieldValue::Text("rainy".to_string()))
    );
    assert!(!projection.fields.contains_key("locked"));
}

#[test]
fn projection_marks_object_deleted_after_delete_revision() {
    let mut conn = memory_connection();
    let command = command("delete");
    let event = event(&command, "delete field");
    let delete_revision = base_revision(event.id, RevisionOperation::Delete);

    record_change(
        &mut conn,
        &command,
        "test.delete_field",
        &event,
        &[delete_revision],
    )
    .unwrap();

    let projection =
        load_object_field_projection(&conn, ObjectKind::BiblePartField, "field-weather").unwrap();

    assert!(projection.deleted);
    assert!(projection.fields.is_empty());
}

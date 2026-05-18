use super::*;
use crate::history_store::create_schema;
use eidetic_core::contracts::{FieldValue, ObjectKind};

fn memory_connection() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    create_schema(&conn).unwrap();
    conn
}

fn set_weather_command(value: Option<FieldValue>) -> CommandEnvelope<SetObjectFieldCommand> {
    CommandEnvelope::new(SetObjectFieldCommand::new(
        ObjectKind::BiblePartField,
        "field-weather",
        "weather",
        value,
    ))
}

#[test]
fn applies_field_command_and_returns_rebuilt_projection() {
    let mut conn = memory_connection();
    let command = set_weather_command(Some(FieldValue::Text("rainy".to_string())));

    let (outcome, projection) = apply_set_object_field(&mut conn, &command, 100).unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(
        projection.fields.get("weather"),
        Some(&FieldValue::Text("rainy".to_string()))
    );
}

#[test]
fn duplicate_command_returns_current_projection_without_new_rows() {
    let mut conn = memory_connection();
    let command = set_weather_command(Some(FieldValue::Text("rainy".to_string())));

    let first = apply_set_object_field(&mut conn, &command, 100).unwrap();
    let second = apply_set_object_field(&mut conn, &command, 100).unwrap();

    assert_eq!(first.0, RecordChangeOutcome::Recorded);
    assert_eq!(second.0, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(
        second.1.fields.get("weather"),
        Some(&FieldValue::Text("rainy".to_string()))
    );
}

#[test]
fn clear_field_command_removes_value_from_projection() {
    let mut conn = memory_connection();
    let set_command = set_weather_command(Some(FieldValue::Text("rainy".to_string())));
    apply_set_object_field(&mut conn, &set_command, 100).unwrap();

    let clear_command = set_weather_command(None);
    let (_, projection) = apply_set_object_field(&mut conn, &clear_command, 101).unwrap();

    assert!(!projection.fields.contains_key("weather"));
}

#[test]
fn rejects_empty_field_key() {
    let mut conn = memory_connection();
    let command = CommandEnvelope::new(SetObjectFieldCommand::new(
        ObjectKind::BiblePartField,
        "field-weather",
        "",
        Some(FieldValue::Text("rainy".to_string())),
    ));

    let error = apply_set_object_field(&mut conn, &command, 100).unwrap_err();

    assert!(matches!(error, ObjectFieldCommandError::InvalidCommand(_)));
}

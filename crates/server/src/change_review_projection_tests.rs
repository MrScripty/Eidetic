use eidetic_core::contracts::{CommandEnvelope, FieldValue, ObjectKind, SetObjectFieldCommand};
use rusqlite::Connection;

use super::load_change_review_projection_envelope;

#[test]
fn change_review_projection_loads_event_revisions_and_field_deltas() {
    let mut conn = Connection::open_in_memory().unwrap();
    seed_weather_change(&mut conn, "rainy");

    let projection = load_change_review_projection_envelope(&conn).unwrap();

    assert_eq!(projection.version.0, 2);
    assert_eq!(projection.payload.changes.len(), 1);
    let change = &projection.payload.changes[0];
    assert_eq!(change.event.summary, "set weather");
    assert_eq!(change.revisions.len(), 1);
    assert_eq!(change.revisions[0].object_kind, ObjectKind::BiblePartField);
    assert_eq!(change.revisions[0].fields[0].field_key, "weather");
    assert_eq!(
        change.revisions[0].fields[0].new_value,
        Some(FieldValue::Text("rainy".to_string()))
    );
}

fn seed_weather_change(conn: &mut Connection, weather: &str) {
    crate::history_store::create_schema(conn).unwrap();
    let command = CommandEnvelope::new(SetObjectFieldCommand::new(
        ObjectKind::BiblePartField,
        "field-weather",
        "weather",
        Some(FieldValue::Text(weather.to_string())),
    ));
    crate::object_field_command::apply_set_object_field(conn, &command, 100).unwrap();
}

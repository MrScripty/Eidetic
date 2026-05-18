use super::*;

#[test]
fn write_connection_enables_foreign_keys() {
    let conn = Connection::open_in_memory().unwrap();
    configure_write_connection(&conn).unwrap();

    let enabled: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();

    assert_eq!(enabled, 1);
}

use std::path::Path;

use rusqlite::Connection;

const WRITE_CONNECTION_PRAGMAS: &str = "PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;";

pub(crate) fn open_write_connection(path: &Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(path)?;
    configure_write_connection(&conn)?;
    Ok(conn)
}

pub(crate) fn configure_write_connection(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(WRITE_CONNECTION_PRAGMAS)
}

#[cfg(test)]
#[path = "sqlite_tests.rs"]
mod tests;

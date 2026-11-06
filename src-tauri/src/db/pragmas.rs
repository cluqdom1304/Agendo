use std::time::Duration;

use rusqlite::Connection;

use super::DbError;

pub const BUSY_TIMEOUT_MS: u64 = 5000;

/// Pragmas for a read-only connection: it cannot change the journal mode or the schema, so
/// only the settings that affect reads apply, and `query_only` guards against accidental writes.
pub fn apply_read_only(conn: &Connection) -> Result<(), DbError> {
    conn.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))?;
    conn.execute_batch(
        "PRAGMA temp_store = MEMORY;
         PRAGMA query_only = ON;",
    )?;
    Ok(())
}

/// Applies the connection pragmas of SPEC §5.4.
///
/// `busy_timeout` goes first so the journal-mode switch waits for a lock instead of failing.
/// In-memory databases cannot use WAL; SQLite reports `memory` and that is accepted.
pub fn apply(conn: &Connection) -> Result<(), DbError> {
    conn.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))?;

    let mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    if !mode.eq_ignore_ascii_case("wal") && !mode.eq_ignore_ascii_case("memory") {
        return Err(DbError::Pragma(format!(
            "journal_mode is `{mode}`, expected `wal`"
        )));
    }

    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA synchronous = NORMAL;
         PRAGMA temp_store = MEMORY;",
    )?;

    let foreign_keys: i64 = conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
    if foreign_keys != 1 {
        return Err(DbError::Pragma(
            "foreign_keys could not be enabled".to_string(),
        ));
    }
    Ok(())
}

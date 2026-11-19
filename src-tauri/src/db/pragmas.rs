use std::time::Duration;

use rusqlite::Connection;

use super::DbError;

pub const BUSY_TIMEOUT_MS: u64 = 5000;

/// Pragmas for a read-only connection: it cannot change the journal mode or the schema, so
/// only the settings that affect reads apply, and `query_only` guards against accidental writes.
pub fn apply_read_only(conn: &Connection) -> Result<(), DbError> {
    conn.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))?;
    conn.execute_batch(&format!(
        "PRAGMA temp_store = MEMORY;
         PRAGMA cache_size = {CACHE_SIZE_KIB};
         PRAGMA mmap_size = {MMAP_SIZE_BYTES};
         PRAGMA query_only = ON;"
    ))?;
    Ok(())
}

/// Page cache per connection: negative values are KiB (16 MiB), so a few thousand notes stay
/// in memory instead of being re-read from disk.
const CACHE_SIZE_KIB: i64 = -16_384;
/// Memory-mapped reads (64 MiB): pages come straight from the OS cache.
const MMAP_SIZE_BYTES: i64 = 67_108_864;
/// Upper bound of the `-wal` file left after a checkpoint (64 MiB).
const JOURNAL_SIZE_LIMIT_BYTES: i64 = 67_108_864;
/// Pages the WAL may grow to before an automatic checkpoint (SQLite's own default, made
/// explicit so it does not depend on how SQLite was compiled).
const WAL_AUTOCHECKPOINT_PAGES: i64 = 1000;

/// Lets SQLite refresh the planner statistics it considers stale (cheap when nothing changed).
/// Failure is harmless: the statistics only steer the query planner.
pub fn optimize(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch("PRAGMA analysis_limit = 400; PRAGMA optimize;")?;
    Ok(())
}

/// Same, for a connection that has not run any query yet: `0x10002` also checks the tables
/// nothing has asked about, so a database that was never analysed gets its statistics once.
pub fn optimize_on_open(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch("PRAGMA analysis_limit = 400; PRAGMA optimize = 0x10002;")?;
    Ok(())
}

/// Recomputes the planner statistics of every table; for after a bulk import.
pub fn analyze(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch("ANALYZE")?;
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

    conn.execute_batch(&format!(
        "PRAGMA foreign_keys = ON;
         PRAGMA synchronous = NORMAL;
         PRAGMA temp_store = MEMORY;
         PRAGMA cache_size = {CACHE_SIZE_KIB};
         PRAGMA mmap_size = {MMAP_SIZE_BYTES};
         PRAGMA journal_size_limit = {JOURNAL_SIZE_LIMIT_BYTES};
         PRAGMA wal_autocheckpoint = {WAL_AUTOCHECKPOINT_PAGES};"
    ))?;

    let foreign_keys: i64 = conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
    if foreign_keys != 1 {
        return Err(DbError::Pragma(
            "foreign_keys could not be enabled".to_string(),
        ));
    }
    Ok(())
}

//! Focus sessions (D-97). Storage only: starting, stopping and the end-of-session notice live in
//! `services::focus`.

use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use super::model::{TaskFocusTotal, TimeEntry};

const COLUMNS: &str = "id, task_id, started_at, ended_at, planned_minutes";

fn map_entry(row: &Row<'_>) -> rusqlite::Result<TimeEntry> {
    Ok(TimeEntry {
        id: row.get(0)?,
        task_id: row.get(1)?,
        started_at: row.get(2)?,
        ended_at: row.get(3)?,
        planned_minutes: row.get(4)?,
    })
}

/// The session in progress, if any.
pub fn running(conn: &Connection) -> rusqlite::Result<Option<TimeEntry>> {
    conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM time_entries WHERE ended_at IS NULL"
    ))?
    .query_row([], map_entry)
    .optional()
}

pub fn start(
    tx: &Transaction<'_>,
    id: &str,
    task_id: &str,
    planned_minutes: i64,
    now: &str,
) -> rusqlite::Result<TimeEntry> {
    tx.prepare_cached(
        "INSERT INTO time_entries (id, task_id, started_at, planned_minutes) VALUES (?1, ?2, ?3, ?4)",
    )?
    .execute(params![id, task_id, now, planned_minutes])?;
    tx.prepare_cached(&format!("SELECT {COLUMNS} FROM time_entries WHERE id = ?1"))?
        .query_row([id], map_entry)
}

/// Ends the running session at `ended_at`. Returns it, or `None` if none was running.
pub fn finish(tx: &Transaction<'_>, ended_at: &str) -> rusqlite::Result<Option<TimeEntry>> {
    let Some(entry) = running(tx)? else {
        return Ok(None);
    };
    // A clock that went back must not end a session before it started.
    let ended_at = ended_at.max(entry.started_at.as_str());
    tx.prepare_cached("UPDATE time_entries SET ended_at = ?2 WHERE id = ?1")?
        .execute(params![entry.id, ended_at])?;
    Ok(Some(TimeEntry {
        ended_at: Some(ended_at.to_string()),
        ..entry
    }))
}

/// Sessions that started in `[from, to)` (UTC instants), oldest first.
pub fn between(conn: &Connection, from: &str, to: &str) -> rusqlite::Result<Vec<TimeEntry>> {
    conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM time_entries
         WHERE started_at >= ?1 AND started_at < ?2 ORDER BY started_at, id"
    ))?
    .query_map(params![from, to], map_entry)?
    .collect()
}

/// Title of a live task, for the end-of-session notice.
pub fn task_title(conn: &Connection, task_id: &str) -> rusqlite::Result<Option<String>> {
    conn.prepare_cached("SELECT title FROM tasks WHERE id = ?1 AND deleted_at IS NULL")?
        .query_row([task_id], |row| row.get(0))
        .optional()
}

/// Minutes spent per task, counting only closed sessions and manual entries, rounded to the
/// minute. Tasks without time do not appear.
pub fn totals(conn: &Connection) -> rusqlite::Result<Vec<TaskFocusTotal>> {
    conn.prepare_cached(
        "SELECT task_id,
                CAST(SUM(ROUND((julianday(ended_at) - julianday(started_at)) * 1440)) AS INTEGER)
         FROM time_entries WHERE ended_at IS NOT NULL GROUP BY task_id
         HAVING SUM(ROUND((julianday(ended_at) - julianday(started_at)) * 1440)) > 0",
    )?
    .query_map([], |row| {
        Ok(TaskFocusTotal {
            task_id: row.get(0)?,
            minutes: row.get(1)?,
        })
    })?
    .collect()
}

/// The closed sessions of one task, newest first.
pub fn of_task(conn: &Connection, task_id: &str) -> rusqlite::Result<Vec<TimeEntry>> {
    conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM time_entries
         WHERE task_id = ?1 AND ended_at IS NOT NULL ORDER BY started_at DESC, id"
    ))?
    .query_map([task_id], map_entry)?
    .collect()
}

/// Adds an already closed entry (a manual one).
pub fn insert_closed(
    tx: &Transaction<'_>,
    id: &str,
    task_id: &str,
    minutes: i64,
    started_at: &str,
    ended_at: &str,
) -> rusqlite::Result<TimeEntry> {
    tx.prepare_cached(
        "INSERT INTO time_entries (id, task_id, started_at, ended_at, planned_minutes)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?
    .execute(params![id, task_id, started_at, ended_at, minutes])?;
    tx.prepare_cached(&format!("SELECT {COLUMNS} FROM time_entries WHERE id = ?1"))?
        .query_row([id], map_entry)
}

/// Deletes a closed entry; `false` if there was none (the running one is not deleted here).
pub fn delete_closed(tx: &Transaction<'_>, id: &str) -> rusqlite::Result<bool> {
    let rows = tx
        .prepare_cached("DELETE FROM time_entries WHERE id = ?1 AND ended_at IS NOT NULL")?
        .execute([id])?;
    Ok(rows > 0)
}

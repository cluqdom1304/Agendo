//! Focus sessions (D-97). Storage only: starting, stopping and the end-of-session notice live in
//! `services::focus`.

use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use super::model::{TaskFocusTotal, TimeEntry};

// `spent_seconds` is what the session really lasted: end − start − pauses, `0` while it runs.
const COLUMNS: &str = "id, task_id, started_at, ended_at, planned_minutes, paused_seconds, paused_at,
    CASE WHEN ended_at IS NULL THEN 0 ELSE MAX(0,
      CAST(ROUND((julianday(ended_at) - julianday(started_at)) * 86400) AS INTEGER) - paused_seconds)
    END";

fn map_entry(row: &Row<'_>) -> rusqlite::Result<TimeEntry> {
    Ok(TimeEntry {
        id: row.get(0)?,
        task_id: row.get(1)?,
        started_at: row.get(2)?,
        ended_at: row.get(3)?,
        planned_minutes: row.get(4)?,
        paused_seconds: row.get(5)?,
        paused_at: row.get(6)?,
        spent_seconds: row.get(7)?,
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

/// Ends the running session at `ended_at`. A session in pause ends where the pause began, so the
/// pause never counts as time spent. Returns it, or `None` if none was running.
pub fn finish(tx: &Transaction<'_>, ended_at: &str) -> rusqlite::Result<Option<TimeEntry>> {
    let Some(entry) = running(tx)? else {
        return Ok(None);
    };
    let end = entry.paused_at.as_deref().unwrap_or(ended_at);
    // A clock that went back must not end a session before it started.
    let end = end.max(entry.started_at.as_str());
    tx.prepare_cached("UPDATE time_entries SET ended_at = ?2, paused_at = NULL WHERE id = ?1")?
        .execute(params![entry.id, end])?;
    tx.prepare_cached(&format!("SELECT {COLUMNS} FROM time_entries WHERE id = ?1"))?
        .query_row([&entry.id], map_entry)
        .map(Some)
}

/// Puts the running session in pause at `now`. `false` if there is none or it is already paused.
pub fn pause(tx: &Transaction<'_>, now: &str) -> rusqlite::Result<bool> {
    let rows = tx
        .prepare_cached(
            "UPDATE time_entries SET paused_at = ?1 WHERE ended_at IS NULL AND paused_at IS NULL",
        )?
        .execute([now])?;
    Ok(rows > 0)
}

/// Resumes the paused session: the pause so far (at least zero seconds) joins the total.
pub fn resume(tx: &Transaction<'_>, now: &str) -> rusqlite::Result<bool> {
    let rows = tx
        .prepare_cached(
            "UPDATE time_entries
             SET paused_seconds = paused_seconds
                   + MAX(0, CAST(ROUND((julianday(?1) - julianday(paused_at)) * 86400) AS INTEGER)),
                 paused_at = NULL
             WHERE ended_at IS NULL AND paused_at IS NOT NULL",
        )?
        .execute([now])?;
    Ok(rows > 0)
}

/// Deletes the running session without keeping any of its time; `false` if none was running.
pub fn discard_running(tx: &Transaction<'_>) -> rusqlite::Result<bool> {
    let rows = tx
        .prepare_cached("DELETE FROM time_entries WHERE ended_at IS NULL")?
        .execute([])?;
    Ok(rows > 0)
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

/// Sessions with some part in `[from, to)`, oldest first: one that started before the window and
/// ended inside it counts, and so does the running one.
pub fn overlapping(conn: &Connection, from: &str, to: &str) -> rusqlite::Result<Vec<TimeEntry>> {
    conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM time_entries
         WHERE started_at < ?2 AND (started_at >= ?1 OR ended_at IS NULL OR ended_at > ?1)
         ORDER BY started_at, id"
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

/// Seconds spent per task, counting only closed sessions and manual entries. Tasks without time
/// do not appear.
pub fn totals(conn: &Connection) -> rusqlite::Result<Vec<TaskFocusTotal>> {
    conn.prepare_cached(
        "SELECT task_id, SUM(spent) FROM (
           SELECT task_id, MAX(0,
             CAST(ROUND((julianday(ended_at) - julianday(started_at)) * 86400) AS INTEGER)
               - paused_seconds) AS spent
           FROM time_entries WHERE ended_at IS NOT NULL)
         GROUP BY task_id HAVING SUM(spent) > 0",
    )?
    .query_map([], |row| {
        Ok(TaskFocusTotal {
            task_id: row.get(0)?,
            seconds: row.get(1)?,
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

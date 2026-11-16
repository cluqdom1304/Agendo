//! Reminders of notes (F13-04, D-52). Only storage lives here: due, expiry and "sent" are in
//! `reminders.rs`, which the one scheduler already uses for tasks.

use rusqlite::{Connection, OptionalExtension, Transaction, params};

use super::model::NoteReminder;

/// Replaces the unsent reminder of a note with one at `notify_at` (UTC). Sent ones stay.
pub fn replace_unsent(
    tx: &Transaction<'_>,
    id: &str,
    note_id: &str,
    notify_at: &str,
    now: &str,
) -> rusqlite::Result<()> {
    clear_unsent(tx, note_id)?;
    tx.prepare_cached(
        "INSERT INTO note_reminders (id, note_id, notify_at, created_at) VALUES (?1, ?2, ?3, ?4)",
    )?
    .execute(params![id, note_id, notify_at, now])?;
    Ok(())
}

pub fn clear_unsent(tx: &Transaction<'_>, note_id: &str) -> rusqlite::Result<()> {
    tx.prepare_cached("DELETE FROM note_reminders WHERE note_id = ?1 AND is_sent = 0")?
        .execute([note_id])?;
    Ok(())
}

/// The reminder still waiting to fire for a note, if any.
pub fn pending(conn: &Connection, note_id: &str) -> rusqlite::Result<Option<NoteReminder>> {
    conn.prepare_cached(
        "SELECT id, note_id, notify_at FROM note_reminders WHERE note_id = ?1 AND is_sent = 0",
    )?
    .query_row([note_id], |row| {
        Ok(NoteReminder {
            id: row.get(0)?,
            note_id: row.get(1)?,
            notify_at: row.get(2)?,
        })
    })
    .optional()
}

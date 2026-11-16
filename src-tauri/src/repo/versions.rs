//! Note version history (F13-03, D-48): snapshots of the document a save is about to replace,
//! spaced in time and capped per note. Restoring never deletes: it snapshots the current
//! document first, so a restore can itself be undone.

use rusqlite::{Connection, OptionalExtension, Transaction, params};

use super::model::{EMPTY_DOC, Note, NoteVersion, NoteVersionSummary};

/// A save snapshots the old document only if the last snapshot is at least this old.
pub const SNAPSHOT_EVERY_MINUTES: i64 = 10;
/// Oldest snapshots beyond this many per note are dropped.
pub const MAX_VERSIONS: i64 = 30;

/// Stores the document of `note` as a version, before `new_json` replaces it. Nothing is stored
/// when the document does not change, is still empty, or (unless `force`) a snapshot newer than
/// `due_before` already exists. Returns whether a snapshot was taken.
pub fn capture(
    tx: &Transaction<'_>,
    new_id: &str,
    note: &Note,
    new_json: &str,
    now: &str,
    due_before: &str,
    force: bool,
) -> rusqlite::Result<bool> {
    if note.content_json == new_json || note.content_json == EMPTY_DOC {
        return Ok(false);
    }
    if !force {
        let latest: Option<String> = tx
            .prepare_cached("SELECT MAX(created_at) FROM note_versions WHERE note_id = ?1")?
            .query_row([&note.id], |row| row.get(0))?;
        if latest.is_some_and(|at| at.as_str() > due_before) {
            return Ok(false);
        }
    }
    tx.prepare_cached(
        "INSERT INTO note_versions (id, note_id, title, content_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?
    .execute(params![new_id, note.id, note.title, note.content_json, now])?;
    tx.prepare_cached(
        "DELETE FROM note_versions WHERE note_id = ?1 AND id NOT IN (
             SELECT id FROM note_versions WHERE note_id = ?1
             ORDER BY created_at DESC, id DESC LIMIT ?2)",
    )?
    .execute(params![note.id, MAX_VERSIONS])?;
    Ok(true)
}

/// Versions of a note, newest first, without their documents.
pub fn list(conn: &Connection, note_id: &str) -> rusqlite::Result<Vec<NoteVersionSummary>> {
    conn.prepare_cached(
        "SELECT id, note_id, title, length(content_json), created_at
         FROM note_versions WHERE note_id = ?1 ORDER BY created_at DESC, id DESC",
    )?
    .query_map([note_id], |row| {
        Ok(NoteVersionSummary {
            id: row.get(0)?,
            note_id: row.get(1)?,
            title: row.get(2)?,
            size: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?
    .collect()
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<NoteVersion>> {
    conn.prepare_cached(
        "SELECT id, note_id, title, content_json, created_at FROM note_versions WHERE id = ?1",
    )?
    .query_row([id], |row| {
        Ok(NoteVersion {
            id: row.get(0)?,
            note_id: row.get(1)?,
            title: row.get(2)?,
            content_json: row.get(3)?,
            created_at: row.get(4)?,
        })
    })
    .optional()
}

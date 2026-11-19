//! Note version history (F13-03, D-48): snapshots of the document a save is about to replace,
//! spaced in time and capped per note. Restoring never deletes: it snapshots the current
//! document first, so a restore can itself be undone.

use rusqlite::{Connection, OptionalExtension, Transaction, params};

use super::model::{EMPTY_DOC, Note, NoteVersion, NoteVersionSummary};

/// A save snapshots the old document only if the last snapshot is at least this old.
pub const SNAPSHOT_EVERY_MINUTES: i64 = 10;
/// Oldest snapshots beyond this many per note are dropped.
pub const MAX_VERSIONS: i64 = 30;
/// The snapshots of one note may weigh this much together (A4-07); older ones go first.
pub const MAX_BYTES_PER_NOTE: i64 = 16 * 1024 * 1024;
/// The newest snapshots always survive the size limit, however large they are.
pub const MIN_VERSIONS_KEPT: i64 = 3;

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
    trim_to_size(tx, &note.id, MAX_BYTES_PER_NOTE, MIN_VERSIONS_KEPT)?;
    Ok(true)
}

/// Drops the oldest snapshots of a note while the rest weigh more than `max_bytes`, always
/// keeping the `keep_newest` newest ones: a huge document cannot fill the disk with 30 copies.
fn trim_to_size(
    tx: &Transaction<'_>,
    note_id: &str,
    max_bytes: i64,
    keep_newest: i64,
) -> rusqlite::Result<usize> {
    tx.prepare_cached(
        "DELETE FROM note_versions WHERE id IN (
             SELECT id FROM (
                 SELECT id,
                        SUM(length(content_json)) OVER newest_first AS running,
                        ROW_NUMBER() OVER newest_first AS position
                 FROM note_versions WHERE note_id = ?1
                 WINDOW newest_first AS (ORDER BY created_at DESC, id DESC))
             WHERE running > ?2 AND position > ?3)",
    )?
    .execute(params![note_id, max_bytes, keep_newest])
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{MIGRATIONS, open_in_memory},
        repo::with_tx,
    };

    fn count(conn: &Connection) -> i64 {
        conn.query_row("SELECT COUNT(*) FROM note_versions", [], |row| row.get(0))
            .expect("count")
    }

    #[test]
    fn the_size_limit_drops_the_oldest_snapshots_but_keeps_the_newest_ones() {
        let mut conn = open_in_memory(&MIGRATIONS).expect("database");
        conn.execute_batch(
            "INSERT INTO notes (id, title, created_at, updated_at) VALUES ('n', 'N', 'x', 'x');",
        )
        .expect("note");
        for index in 0..6 {
            conn.execute(
                "INSERT INTO note_versions (id, note_id, title, content_json, created_at)
                 VALUES (?1, 'n', 'N', ?2, ?3)",
                params![
                    format!("v{index}"),
                    "x".repeat(1000),
                    format!("2026-03-0{}T10:00:00Z", index + 1)
                ],
            )
            .expect("version");
        }

        // 2 500 bytes allow two snapshots; the three newest are kept regardless.
        let dropped = with_tx(&mut conn, |tx| trim_to_size(tx, "n", 2_500, 3)).expect("trim");
        assert_eq!(dropped, 3);
        let kept: Vec<String> = conn
            .prepare("SELECT id FROM note_versions ORDER BY created_at DESC")
            .expect("prepare")
            .query_map([], |row| row.get(0))
            .expect("query")
            .collect::<Result<_, _>>()
            .expect("rows");
        assert_eq!(kept, ["v5", "v4", "v3"]);

        // Within the limit nothing is dropped.
        assert_eq!(
            with_tx(&mut conn, |tx| trim_to_size(tx, "n", 1 << 20, 3)).expect("trim"),
            0
        );
        assert_eq!(count(&conn), 3);
    }
}

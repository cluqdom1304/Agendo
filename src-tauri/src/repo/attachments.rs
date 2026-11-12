//! Attachments of the notes (D-47): one row per distinct file, and which notes use it.

use rusqlite::{Connection, OptionalExtension, Transaction, params};

/// What is known about a stored file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    pub id: String,
    pub mime: String,
    pub size_bytes: i64,
}

fn read_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Attachment> {
    Ok(Attachment {
        id: row.get(0)?,
        mime: row.get(1)?,
        size_bytes: row.get(2)?,
    })
}

/// Registers a file; the same content (same id) is only registered once.
pub fn insert_if_absent(
    tx: &Transaction<'_>,
    attachment: &Attachment,
    now: &str,
) -> rusqlite::Result<()> {
    tx.prepare_cached(
        "INSERT OR IGNORE INTO attachments (id, mime, size_bytes, created_at)
         VALUES (?1, ?2, ?3, ?4)",
    )?
    .execute(params![
        attachment.id,
        attachment.mime,
        attachment.size_bytes,
        now
    ])?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Attachment>> {
    conn.prepare_cached("SELECT id, mime, size_bytes FROM attachments WHERE id = ?1")?
        .query_row([id], read_row)
        .optional()
}

/// Every registered file, for a backup.
pub fn all(conn: &Connection) -> rusqlite::Result<Vec<Attachment>> {
    conn.prepare_cached("SELECT id, mime, size_bytes FROM attachments ORDER BY id")?
        .query_map([], read_row)?
        .collect()
}

/// Replaces the files a note uses. A file that was never registered is skipped: the document
/// may hold an image whose file is gone, and saving the note must not fail for that.
pub fn sync_note(
    tx: &Transaction<'_>,
    note_id: &str,
    attachment_ids: &[String],
) -> rusqlite::Result<()> {
    tx.prepare_cached("DELETE FROM note_attachments WHERE note_id = ?1")?
        .execute([note_id])?;
    let mut link = tx.prepare_cached(
        "INSERT OR IGNORE INTO note_attachments (note_id, attachment_id)
         SELECT ?1, id FROM attachments WHERE id = ?2",
    )?;
    for id in attachment_ids {
        link.execute(params![note_id, id])?;
    }
    Ok(())
}

/// Files no note uses that were registered more than `grace_hours` ago. The margin keeps a
/// file that was just pasted, or that an undo in the editor brings back, from being collected.
pub fn orphans(conn: &Connection, now: &str, grace_hours: u32) -> rusqlite::Result<Vec<String>> {
    conn.prepare_cached(
        "SELECT id FROM attachments a
         WHERE NOT EXISTS (SELECT 1 FROM note_attachments n WHERE n.attachment_id = a.id)
           AND a.created_at < strftime('%Y-%m-%dT%H:%M:%SZ', ?1, '-' || ?2 || ' hours')",
    )?
    .query_map(params![now, grace_hours], |row| row.get(0))?
    .collect()
}

pub fn delete(tx: &Transaction<'_>, id: &str) -> rusqlite::Result<bool> {
    Ok(tx
        .prepare_cached("DELETE FROM attachments WHERE id = ?1")?
        .execute([id])?
        > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{MIGRATIONS, open_in_memory},
        repo::{model::NewNote, notes, with_tx},
    };

    fn file(id: &str) -> Attachment {
        Attachment {
            id: id.into(),
            mime: "image/png".into(),
            size_bytes: 10,
        }
    }

    fn note(conn: &mut Connection, id: &str) {
        with_tx(conn, |tx| {
            notes::create(
                tx,
                &NewNote {
                    id: id.into(),
                    now: "2026-03-01T10:00:00Z".into(),
                    ..NewNote::default()
                },
            )
        })
        .expect("note");
    }

    #[test]
    fn a_file_is_registered_once_and_orphans_wait_for_the_grace_period() {
        let mut conn = open_in_memory(&MIGRATIONS).expect("db");
        note(&mut conn, "n1");
        with_tx(&mut conn, |tx| {
            insert_if_absent(tx, &file("used"), "2026-03-01T10:00:00Z")?;
            insert_if_absent(tx, &file("used"), "2026-03-09T10:00:00Z")?;
            insert_if_absent(tx, &file("lost"), "2026-03-01T10:00:00Z")?;
            insert_if_absent(tx, &file("fresh"), "2026-03-10T09:00:00Z")?;
            sync_note(tx, "n1", &["used".into(), "unknown".into()])
        })
        .expect("save");
        assert_eq!(all(&conn).expect("all").len(), 3);
        assert_eq!(
            orphans(&conn, "2026-03-10T10:00:00Z", 24).expect("orphans"),
            ["lost"]
        );
        with_tx(&mut conn, |tx| sync_note(tx, "n1", &[])).expect("empty");
        let mut found = orphans(&conn, "2026-03-10T10:00:00Z", 24).expect("orphans");
        found.sort();
        assert_eq!(found, ["lost", "used"]);
        with_tx(&mut conn, |tx| delete(tx, "lost")).expect("delete");
        assert!(get(&conn, "lost").expect("get").is_none());
    }

    #[test]
    fn deleting_a_note_for_good_releases_its_files() {
        let mut conn = open_in_memory(&MIGRATIONS).expect("db");
        note(&mut conn, "n1");
        with_tx(&mut conn, |tx| {
            insert_if_absent(tx, &file("a"), "2026-03-01T10:00:00Z")?;
            sync_note(tx, "n1", &["a".into()])
        })
        .expect("save");
        with_tx(&mut conn, |tx| {
            tx.execute("DELETE FROM notes WHERE id = 'n1'", [])
        })
        .expect("purge");
        assert_eq!(
            orphans(&conn, "2026-03-10T10:00:00Z", 24).expect("orphans"),
            ["a"]
        );
    }
}

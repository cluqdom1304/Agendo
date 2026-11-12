//! Tags of notes. They are the very same rows of `tags` that tasks use (D-44): this module only
//! manages the `note_tags` links.

use rusqlite::{Transaction, params};

/// Replaces the tag set of a live note and touches `updated_at`. Returns `false` if the note
/// does not exist or is in the trash. An unknown tag id fails with a foreign-key error.
pub fn set_note_tags(
    tx: &Transaction<'_>,
    note_id: &str,
    tag_ids: &[String],
    now: &str,
) -> rusqlite::Result<bool> {
    let touched = tx
        .prepare_cached("UPDATE notes SET updated_at = ?2 WHERE id = ?1 AND deleted_at IS NULL")?
        .execute(params![note_id, now])?;
    if touched == 0 {
        return Ok(false);
    }
    replace_links(tx, note_id, tag_ids)?;
    Ok(true)
}

/// Replaces the links of `note_id` without checking the note. Duplicated ids are ignored.
pub(super) fn replace_links(
    tx: &Transaction<'_>,
    note_id: &str,
    tag_ids: &[String],
) -> rusqlite::Result<()> {
    tx.prepare_cached("DELETE FROM note_tags WHERE note_id = ?1")?
        .execute([note_id])?;
    let mut insert =
        tx.prepare_cached("INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)")?;
    for tag_id in tag_ids {
        insert.execute(params![note_id, tag_id])?;
    }
    Ok(())
}

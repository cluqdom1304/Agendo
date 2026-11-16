//! Note templates (F13-02). Plain rows: no search index, no references.

use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use super::model::NoteTemplate;

fn row_to_template(row: &Row<'_>) -> rusqlite::Result<NoteTemplate> {
    Ok(NoteTemplate {
        id: row.get(0)?,
        name: row.get(1)?,
        content_json: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<NoteTemplate>> {
    conn.prepare_cached(
        "SELECT id, name, content_json, created_at, updated_at
         FROM note_templates ORDER BY name COLLATE NOCASE, id",
    )?
    .query_map([], row_to_template)?
    .collect()
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<NoteTemplate>> {
    conn.prepare_cached(
        "SELECT id, name, content_json, created_at, updated_at
         FROM note_templates WHERE id = ?1",
    )?
    .query_row([id], row_to_template)
    .optional()
}

pub fn insert(
    tx: &Transaction<'_>,
    id: &str,
    name: &str,
    content_json: &str,
    now: &str,
) -> rusqlite::Result<NoteTemplate> {
    tx.prepare_cached(
        "INSERT INTO note_templates (id, name, content_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)",
    )?
    .execute(params![id, name, content_json, now])?;
    get(tx, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

/// Renames a template and, when `content_json` is given, replaces its document.
pub fn update(
    tx: &Transaction<'_>,
    id: &str,
    name: &str,
    content_json: Option<&str>,
    now: &str,
) -> rusqlite::Result<Option<NoteTemplate>> {
    let changed = tx
        .prepare_cached(
            "UPDATE note_templates
             SET name = ?2, content_json = COALESCE(?3, content_json), updated_at = ?4
             WHERE id = ?1",
        )?
        .execute(params![id, name, content_json, now])?;
    if changed == 0 {
        return Ok(None);
    }
    get(tx, id)
}

pub fn delete(tx: &Transaction<'_>, id: &str) -> rusqlite::Result<bool> {
    Ok(tx
        .prepare_cached("DELETE FROM note_templates WHERE id = ?1")?
        .execute([id])?
        > 0)
}

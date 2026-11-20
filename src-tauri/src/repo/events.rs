//! Events and their reminders (D-94). Storage only: the rules about dates, repetition and when a
//! reminder sounds live in `services::events`.

use rusqlite::{Connection, OptionalExtension, Row, Transaction, params, types::Type};

use super::model::{Event, EventData};

/// Columns of [`map_event`]; tags come back as a JSON array from a correlated subquery.
macro_rules! event_sql {
    ($suffix:literal) => {
        concat!(
            "SELECT e.id, e.title, e.notes, e.start_date, e.start_time, e.end_date, e.end_time,
                    e.rrule, e.reminder_offsets, e.created_at, e.updated_at,
                    (SELECT json_group_array(et.tag_id ORDER BY et.tag_id)
                       FROM event_tags et WHERE et.event_id = e.id)
             FROM events e ",
            $suffix
        )
    };
}

fn json_column<T: serde::de::DeserializeOwned>(row: &Row<'_>, index: usize) -> rusqlite::Result<T> {
    let raw: String = row.get(index)?;
    serde_json::from_str(&raw).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(index, Type::Text, Box::new(error))
    })
}

fn map_event(row: &Row<'_>) -> rusqlite::Result<Event> {
    let start_time: Option<String> = row.get(4)?;
    Ok(Event {
        id: row.get(0)?,
        title: row.get(1)?,
        notes: row.get(2)?,
        start_date: row.get(3)?,
        all_day: start_time.is_none(),
        start_time,
        end_date: row.get(5)?,
        end_time: row.get(6)?,
        rrule: row.get(7)?,
        reminder_offsets: json_column(row, 8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        tag_ids: json_column(row, 11)?,
    })
}

/// A live (not trashed) event.
pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Event>> {
    conn.prepare_cached(event_sql!("WHERE e.id = ?1 AND e.deleted_at IS NULL"))?
        .query_row([id], map_event)
        .optional()
}

/// Live events that can show up between `from` and `to` (inclusive, `YYYY-MM-DD`): the ones that
/// overlap the window and every series that has started by `to` (its repetitions are expanded
/// by the caller).
pub fn candidates(conn: &Connection, from: &str, to: &str) -> rusqlite::Result<Vec<Event>> {
    conn.prepare_cached(event_sql!(
        "WHERE e.deleted_at IS NULL AND e.start_date <= ?2
           AND (e.rrule IS NOT NULL OR e.end_date >= ?1)
         ORDER BY e.start_date, e.start_time IS NOT NULL, e.start_time, e.id"
    ))?
    .query_map(params![from, to], map_event)?
    .collect()
}

fn write_links(tx: &Transaction<'_>, id: &str, tag_ids: &[String]) -> rusqlite::Result<()> {
    tx.prepare_cached("DELETE FROM event_tags WHERE event_id = ?1")?
        .execute([id])?;
    let mut insert =
        tx.prepare_cached("INSERT OR IGNORE INTO event_tags (event_id, tag_id) VALUES (?1, ?2)")?;
    for tag_id in tag_ids {
        insert.execute(params![id, tag_id])?;
    }
    Ok(())
}

fn offsets_json(offsets: &[i64]) -> String {
    serde_json::to_string(offsets).unwrap_or_else(|_| "[]".to_string())
}

pub fn create(tx: &Transaction<'_>, data: &EventData) -> rusqlite::Result<Event> {
    tx.prepare_cached(
        "INSERT INTO events (id, title, notes, start_date, start_time, end_date, end_time, rrule,
                             reminder_offsets, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
    )?
    .execute(params![
        data.id,
        data.title,
        data.notes,
        data.start_date,
        data.start_time,
        data.end_date,
        data.end_time,
        data.rrule,
        offsets_json(&data.reminder_offsets),
        data.now
    ])?;
    write_links(tx, &data.id, &data.tag_ids)?;
    get(tx, &data.id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

/// Replaces every editable field of a live event. `None` if it does not exist or is trashed.
pub fn update(tx: &Transaction<'_>, data: &EventData) -> rusqlite::Result<Option<Event>> {
    let changed = tx
        .prepare_cached(
            "UPDATE events SET title = ?2, notes = ?3, start_date = ?4, start_time = ?5,
                    end_date = ?6, end_time = ?7, rrule = ?8, reminder_offsets = ?9,
                    updated_at = ?10
             WHERE id = ?1 AND deleted_at IS NULL",
        )?
        .execute(params![
            data.id,
            data.title,
            data.notes,
            data.start_date,
            data.start_time,
            data.end_date,
            data.end_time,
            data.rrule,
            offsets_json(&data.reminder_offsets),
            data.now
        ])?;
    if changed == 0 {
        return Ok(None);
    }
    write_links(tx, &data.id, &data.tag_ids)?;
    get(tx, &data.id)
}

/// Moves a live event to the trash. `false` if there was nothing to delete.
pub fn soft_delete(tx: &Transaction<'_>, id: &str, now: &str) -> rusqlite::Result<bool> {
    let changed = tx
        .prepare_cached(
            "UPDATE events SET deleted_at = ?2, updated_at = ?2
             WHERE id = ?1 AND deleted_at IS NULL",
        )?
        .execute(params![id, now])?;
    Ok(changed > 0)
}

/// Takes an event out of the trash. `None` if it is not in the trash.
pub fn restore(tx: &Transaction<'_>, id: &str, now: &str) -> rusqlite::Result<Option<Event>> {
    let changed = tx
        .prepare_cached(
            "UPDATE events SET deleted_at = NULL, updated_at = ?2
             WHERE id = ?1 AND deleted_at IS NOT NULL",
        )?
        .execute(params![id, now])?;
    if changed == 0 {
        return Ok(None);
    }
    get(tx, id)
}

/// Removes the reminders of an event that have not sounded yet. Sounded ones stay as history.
pub fn clear_unsent(tx: &Transaction<'_>, event_id: &str) -> rusqlite::Result<()> {
    tx.prepare_cached("DELETE FROM event_reminders WHERE event_id = ?1 AND is_sent = 0")?
        .execute([event_id])?;
    Ok(())
}

/// Offsets of the reminders of an event that are still waiting.
pub fn pending_offsets(conn: &Connection, event_id: &str) -> rusqlite::Result<Vec<i64>> {
    conn.prepare_cached(
        "SELECT offset_minutes FROM event_reminders WHERE event_id = ?1 AND is_sent = 0
         ORDER BY offset_minutes",
    )?
    .query_map([event_id], |row| row.get(0))?
    .collect()
}

pub fn insert_reminder(
    tx: &Transaction<'_>,
    id: &str,
    event_id: &str,
    offset_minutes: i64,
    notify_at: &str,
    now: &str,
) -> rusqlite::Result<()> {
    tx.prepare_cached(
        "INSERT INTO event_reminders (id, event_id, offset_minutes, notify_at, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?
    .execute(params![id, event_id, offset_minutes, notify_at, now])?;
    Ok(())
}

/// The event a reminder belongs to, if the reminder is an event's.
pub fn event_of_reminder(conn: &Connection, reminder_id: &str) -> rusqlite::Result<Option<String>> {
    conn.prepare_cached("SELECT event_id FROM event_reminders WHERE id = ?1")?
        .query_row([reminder_id], |row| row.get(0))
        .optional()
}

/// Trashed events, most recently deleted first.
pub fn list_trash(conn: &Connection) -> rusqlite::Result<Vec<Event>> {
    conn.prepare_cached(event_sql!(
        "WHERE e.deleted_at IS NOT NULL ORDER BY e.deleted_at DESC, e.id"
    ))?
    .query_map([], map_event)?
    .collect()
}

/// Permanently removes events trashed more than `PURGE_AFTER_DAYS` days before `now`; their
/// tag links and reminders go with them. Returns how many were purged.
pub fn purge_deleted(tx: &Transaction<'_>, now: &str) -> rusqlite::Result<usize> {
    tx.prepare_cached(
        "DELETE FROM events
         WHERE deleted_at IS NOT NULL
           AND deleted_at < strftime('%Y-%m-%dT%H:%M:%SZ', ?1, '-' || ?2 || ' days')",
    )?
    .execute(params![now, super::tasks::PURGE_AFTER_DAYS])
}

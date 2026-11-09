//! Reminder rows (SPEC §5.2, §6). Instants are UTC `YYYY-MM-DDTHH:MM:SSZ`, so comparing them as
//! text is comparing them in time. A reminder only counts while its task is live and open.

use chrono::TimeZone;
use rusqlite::{Connection, Transaction, params, types::Type};

use super::{model::Reminder, tasks};
use crate::domain::{dates, reminders as rules};

/// A reminder ready to fire, with what the notification shows.
#[derive(Debug, Clone, PartialEq)]
pub struct DueReminder {
    pub id: String,
    pub task_id: String,
    pub title: String,
    pub due_time: Option<String>,
    /// Plain text of the task's notes (`content_text`), possibly empty.
    pub notes: String,
    pub tag_names: Vec<String>,
}

/// A reminder to store. `offset_minutes` is `None` for a custom time.
#[derive(Debug, Clone, PartialEq)]
pub struct NewReminder {
    pub id: String,
    pub notify_at: String,
    pub offset_minutes: Option<i64>,
}

/// Unsent reminders of live, open tasks whose time has come, oldest first.
pub fn due(conn: &Connection, now: &str) -> rusqlite::Result<Vec<DueReminder>> {
    conn.prepare_cached(
        "SELECT r.id, t.id, t.title, t.due_time, t.content_text,
                (SELECT json_group_array(g.name ORDER BY g.name)
                   FROM task_tags tt JOIN tags g ON g.id = tt.tag_id WHERE tt.task_id = t.id)
         FROM reminders r JOIN tasks t ON t.id = r.task_id
         WHERE r.is_sent = 0 AND r.notify_at <= ?1
           AND t.deleted_at IS NULL AND t.is_completed = 0
         ORDER BY r.notify_at, r.id",
    )?
    .query_map(params![now], |row| {
        let names: String = row.get(5)?;
        Ok(DueReminder {
            id: row.get(0)?,
            task_id: row.get(1)?,
            title: row.get(2)?,
            due_time: row.get(3)?,
            notes: row.get(4)?,
            tag_names: serde_json::from_str(&names).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(5, Type::Text, Box::new(error))
            })?,
        })
    })?
    .collect()
}

/// The earliest `notify_at` after `now` among reminders that can still fire.
pub fn next_after(conn: &Connection, now: &str) -> rusqlite::Result<Option<String>> {
    conn.prepare_cached(
        "SELECT MIN(r.notify_at)
         FROM reminders r JOIN tasks t ON t.id = r.task_id
         WHERE r.is_sent = 0 AND r.notify_at > ?1
           AND t.deleted_at IS NULL AND t.is_completed = 0",
    )?
    .query_row(params![now], |row| row.get(0))
}

pub fn mark_sent(tx: &Transaction<'_>, id: &str, now: &str) -> rusqlite::Result<bool> {
    let changed = tx
        .prepare_cached(
            "UPDATE reminders SET is_sent = 1, sent_at = ?2 WHERE id = ?1 AND is_sent = 0",
        )?
        .execute(params![id, now])?;
    Ok(changed > 0)
}

/// Marks as sent, without notifying, every unsent reminder due before `cutoff` (F-14).
/// Returns their ids.
pub fn expire_before(
    tx: &Transaction<'_>,
    cutoff: &str,
    now: &str,
) -> rusqlite::Result<Vec<String>> {
    let ids: Vec<String> = tx
        .prepare_cached(
            "SELECT id FROM reminders WHERE is_sent = 0 AND notify_at < ?1 ORDER BY id",
        )?
        .query_map(params![cutoff], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    tx.prepare_cached(
        "UPDATE reminders SET is_sent = 1, sent_at = ?2 WHERE is_sent = 0 AND notify_at < ?1",
    )?
    .execute(params![cutoff, now])?;
    Ok(ids)
}

/// Replaces the unsent reminders of a task. Sent ones stay as history.
pub fn replace_unsent(
    tx: &Transaction<'_>,
    task_id: &str,
    reminders: &[NewReminder],
    now: &str,
) -> rusqlite::Result<()> {
    tx.prepare_cached("DELETE FROM reminders WHERE task_id = ?1 AND is_sent = 0")?
        .execute(params![task_id])?;
    let mut insert = tx.prepare_cached(
        "INSERT INTO reminders (id, task_id, notify_at, offset_minutes, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    for reminder in reminders {
        insert.execute(params![
            reminder.id,
            task_id,
            reminder.notify_at,
            reminder.offset_minutes,
            now
        ])?;
    }
    Ok(())
}

pub fn for_task(conn: &Connection, task_id: &str) -> rusqlite::Result<Vec<Reminder>> {
    conn.prepare_cached(
        "SELECT id, notify_at, offset_minutes, is_sent FROM reminders
         WHERE task_id = ?1 ORDER BY notify_at, id",
    )?
    .query_map(params![task_id], |row| {
        Ok(Reminder {
            id: row.get(0)?,
            notify_at: row.get(1)?,
            offset_minutes: row.get(2)?,
            is_sent: row.get(3)?,
        })
    })?
    .collect()
}

/// Recomputes the unsent reminders that are relative to the task's date and time, after either
/// changed (SPEC §5.3). Without a date and a time they have no meaning and are deleted
/// (D-26). Custom reminders and sent ones are left alone.
pub fn recalculate<Tz: TimeZone>(
    tx: &Transaction<'_>,
    task_id: &str,
    zone: &Tz,
) -> rusqlite::Result<()> {
    let Some(task) = tasks::get(tx, task_id)? else {
        return Ok(());
    };
    let relative: Vec<(String, i64)> = tx
        .prepare_cached(
            "SELECT id, offset_minutes FROM reminders
             WHERE task_id = ?1 AND is_sent = 0 AND offset_minutes IS NOT NULL",
        )?
        .query_map(params![task_id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    for (id, offset) in relative {
        let notify_at = match (&task.due_date, &task.due_time) {
            (Some(date), Some(time)) => rules::offset_notify_at(zone, date, time, offset).ok(),
            _ => None,
        };
        match notify_at {
            Some(instant) => {
                tx.prepare_cached("UPDATE reminders SET notify_at = ?2 WHERE id = ?1")?
                    .execute(params![id, dates::format_instant(instant)])?;
            }
            None => {
                tx.prepare_cached("DELETE FROM reminders WHERE id = ?1")?
                    .execute(params![id])?;
            }
        }
    }
    Ok(())
}

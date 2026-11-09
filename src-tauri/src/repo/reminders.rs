//! Reminder rows (SPEC §5.2, §6). Instants are UTC `YYYY-MM-DDTHH:MM:SSZ`, so comparing them as
//! text is comparing them in time. A reminder only counts while its task is live and open.

use chrono::TimeZone;
use rusqlite::{Connection, Transaction, params, types::Type};

use super::{model::Reminder, settings, tasks};
use crate::domain::{
    dates,
    reminders::{self as rules, Repeat, RepeatUnit},
};

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
    /// Only for custom-time reminders.
    pub repeat: Option<Repeat>,
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
        "INSERT INTO reminders (id, task_id, notify_at, offset_minutes, created_at,
                                repeat_every, repeat_unit, repeat_count, repeat_until, occurrence)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )?;
    for reminder in reminders {
        let repeat = reminder.repeat.as_ref();
        insert.execute(params![
            reminder.id,
            task_id,
            reminder.notify_at,
            reminder.offset_minutes,
            now,
            repeat.map(|r| r.every),
            repeat.map(|r| r.unit.as_str()),
            repeat.and_then(|r| r.count),
            repeat.and_then(|r| r.until.as_deref()),
            repeat.map_or(1, |r| r.occurrence),
        ])?;
    }
    Ok(())
}

/// A reminder that belongs to a repeating series, as stored.
#[derive(Debug, Clone, PartialEq)]
pub struct SeriesMember {
    pub task_id: String,
    pub notify_at: String,
    pub repeat: Repeat,
}

/// The repeat rule of reminder `id`, or `None` if it does not repeat.
pub fn series_member(conn: &Connection, id: &str) -> rusqlite::Result<Option<SeriesMember>> {
    conn.prepare_cached(
        "SELECT task_id, notify_at, repeat_every, repeat_unit, repeat_count, repeat_until, occurrence
         FROM reminders WHERE id = ?1 AND repeat_unit IS NOT NULL",
    )?
    .query_map(params![id], |row| {
        let unit: String = row.get(3)?;
        let unit = RepeatUnit::parse(&unit).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                Type::Text,
                format!("unknown repeat unit {unit}").into(),
            )
        })?;
        Ok(SeriesMember {
            task_id: row.get(0)?,
            notify_at: row.get(1)?,
            repeat: Repeat {
                every: row.get(2)?,
                unit,
                count: row.get(4)?,
                until: row.get(5)?,
                occurrence: row.get(6)?,
            },
        })
    })?
    .next()
    .transpose()
}

/// Stores the next sounding of a series, unsent.
pub fn insert_next(
    tx: &Transaction<'_>,
    id: &str,
    member: &SeriesMember,
    notify_at: &str,
    occurrence: u32,
    now: &str,
) -> rusqlite::Result<()> {
    let next = Repeat {
        occurrence,
        ..member.repeat.clone()
    };
    insert_one(tx, id, &member.task_id, notify_at, &next, now)
}

fn insert_one(
    tx: &Transaction<'_>,
    id: &str,
    task_id: &str,
    notify_at: &str,
    repeat: &Repeat,
    now: &str,
) -> rusqlite::Result<()> {
    tx.prepare_cached(
        "INSERT INTO reminders (id, task_id, notify_at, offset_minutes, created_at,
                                repeat_every, repeat_unit, repeat_count, repeat_until, occurrence)
         VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, ?8, ?9)",
    )?
    .execute(params![
        id,
        task_id,
        notify_at,
        now,
        repeat.every,
        repeat.unit.as_str(),
        repeat.count,
        repeat.until.as_deref(),
        repeat.occurrence,
    ])?;
    Ok(())
}

pub fn for_task(conn: &Connection, task_id: &str) -> rusqlite::Result<Vec<Reminder>> {
    conn.prepare_cached(
        "SELECT id, notify_at, offset_minutes, is_sent,
                repeat_every, repeat_unit, repeat_count, repeat_until, occurrence
         FROM reminders WHERE task_id = ?1 ORDER BY notify_at, id",
    )?
    .query_map(params![task_id], |row| {
        let unit: Option<String> = row.get(5)?;
        let repeat = match unit.as_deref().map(RepeatUnit::parse) {
            None => None,
            Some(Some(unit)) => Some(Repeat {
                every: row.get(4)?,
                unit,
                count: row.get(6)?,
                until: row.get(7)?,
                occurrence: row.get(8)?,
            }),
            Some(None) => {
                return Err(rusqlite::Error::FromSqlConversionFailure(
                    5,
                    Type::Text,
                    "unknown repeat unit".into(),
                ));
            }
        };
        Ok(Reminder {
            id: row.get(0)?,
            notify_at: row.get(1)?,
            offset_minutes: row.get(2)?,
            is_sent: row.get(3)?,
            repeat,
        })
    })?
    .collect()
}

/// Recomputes the unsent reminders that are relative to the task's date and time, after either
/// changed (SPEC §5.3). A task with a date but no time uses the default notification time, and
/// then only whole-day offsets make sense. Without a date, or with an offset that does not fit,
/// they are deleted (D-26, D-35). Custom reminders and sent ones are left alone.
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
    if relative.is_empty() {
        return Ok(());
    }
    let default_time = settings::notify_time(tx)?;
    for (id, offset) in relative {
        let notify_at = match (&task.due_date, &task.due_time) {
            (Some(date), Some(time)) => rules::offset_notify_at(zone, date, time, offset).ok(),
            (Some(date), None) if rules::validate_untimed_offset(offset).is_ok() => {
                rules::offset_notify_at(zone, date, &default_time, offset).ok()
            }
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

/// After the default notification time changed: moves the unsent relative reminders of the
/// tasks that have a date but no time.
pub fn recalculate_untimed<Tz: TimeZone>(tx: &Transaction<'_>, zone: &Tz) -> rusqlite::Result<()> {
    let task_ids: Vec<String> = tx
        .prepare_cached(
            "SELECT DISTINCT r.task_id FROM reminders r JOIN tasks t ON t.id = r.task_id
             WHERE r.is_sent = 0 AND r.offset_minutes IS NOT NULL
               AND t.due_date IS NOT NULL AND t.due_time IS NULL",
        )?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for task_id in task_ids {
        recalculate(tx, &task_id, zone)?;
    }
    Ok(())
}

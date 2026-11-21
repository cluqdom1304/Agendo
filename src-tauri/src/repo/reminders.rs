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
    pub notify_at: String,
    /// Set for the reminder of a task; `note_id` for the reminder of a note (D-52).
    pub task_id: Option<String>,
    pub note_id: Option<String>,
    /// Set for the reminder of an event (D-94).
    pub event_id: Option<String>,
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

/// Unsent reminders of live, open tasks and of live notes whose time has come, oldest first.
pub fn due(conn: &Connection, now: &str) -> rusqlite::Result<Vec<DueReminder>> {
    let mut found = due_tasks(conn, now)?;
    found.extend(due_notes(conn, now)?);
    found.extend(due_events(conn, now)?);
    found.sort_by(|a, b| (&a.notify_at, &a.id).cmp(&(&b.notify_at, &b.id)));
    Ok(found)
}

fn tag_names(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<Vec<String>> {
    let names: String = row.get(index)?;
    serde_json::from_str(&names).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(index, Type::Text, Box::new(error))
    })
}

fn due_notes(conn: &Connection, now: &str) -> rusqlite::Result<Vec<DueReminder>> {
    conn.prepare_cached(
        "SELECT r.id, n.id, n.title, n.content_text, r.notify_at,
                (SELECT json_group_array(g.name ORDER BY g.name)
                   FROM note_tags nt JOIN tags g ON g.id = nt.tag_id WHERE nt.note_id = n.id)
         FROM note_reminders r JOIN notes n ON n.id = r.note_id
         WHERE r.is_sent = 0 AND r.notify_at <= ?1 AND n.deleted_at IS NULL
         ORDER BY r.notify_at, r.id",
    )?
    .query_map(params![now], |row| {
        Ok(DueReminder {
            id: row.get(0)?,
            notify_at: row.get(4)?,
            task_id: None,
            note_id: Some(row.get(1)?),
            event_id: None,
            title: row.get(2)?,
            due_time: None,
            notes: row.get(3)?,
            tag_names: tag_names(row, 5)?,
        })
    })?
    .collect()
}

fn due_events(conn: &Connection, now: &str) -> rusqlite::Result<Vec<DueReminder>> {
    conn.prepare_cached(
        "SELECT r.id, e.id, e.title, e.start_time, e.notes, r.notify_at,
                (SELECT json_group_array(g.name ORDER BY g.name)
                   FROM event_tags et JOIN tags g ON g.id = et.tag_id WHERE et.event_id = e.id)
         FROM event_reminders r JOIN events e ON e.id = r.event_id
         WHERE r.is_sent = 0 AND r.notify_at <= ?1 AND e.deleted_at IS NULL
         ORDER BY r.notify_at, r.id",
    )?
    .query_map(params![now], |row| {
        Ok(DueReminder {
            id: row.get(0)?,
            notify_at: row.get(5)?,
            task_id: None,
            note_id: None,
            event_id: Some(row.get(1)?),
            title: row.get(2)?,
            due_time: row.get(3)?,
            notes: row.get(4)?,
            tag_names: tag_names(row, 6)?,
        })
    })?
    .collect()
}

fn due_tasks(conn: &Connection, now: &str) -> rusqlite::Result<Vec<DueReminder>> {
    conn.prepare_cached(
        "SELECT r.id, t.id, t.title, t.due_time, t.content_text,
                (SELECT json_group_array(g.name ORDER BY g.name)
                   FROM task_tags tt JOIN tags g ON g.id = tt.tag_id WHERE tt.task_id = t.id),
                r.notify_at
         FROM reminders r JOIN tasks t ON t.id = r.task_id
         WHERE r.is_sent = 0 AND r.notify_at <= ?1
           AND t.deleted_at IS NULL AND t.is_completed = 0
         ORDER BY r.notify_at, r.id",
    )?
    .query_map(params![now], |row| {
        Ok(DueReminder {
            id: row.get(0)?,
            notify_at: row.get(6)?,
            task_id: Some(row.get(1)?),
            note_id: None,
            event_id: None,
            title: row.get(2)?,
            due_time: row.get(3)?,
            notes: row.get(4)?,
            tag_names: tag_names(row, 5)?,
        })
    })?
    .collect()
}

/// The earliest `notify_at` after `now` among reminders that can still fire.
pub fn next_after(conn: &Connection, now: &str) -> rusqlite::Result<Option<String>> {
    let task: Option<String> = conn
        .prepare_cached(
            "SELECT MIN(r.notify_at)
             FROM reminders r JOIN tasks t ON t.id = r.task_id
             WHERE r.is_sent = 0 AND r.notify_at > ?1
               AND t.deleted_at IS NULL AND t.is_completed = 0",
        )?
        .query_row(params![now], |row| row.get(0))?;
    let note: Option<String> = conn
        .prepare_cached(
            "SELECT MIN(r.notify_at)
             FROM note_reminders r JOIN notes n ON n.id = r.note_id
             WHERE r.is_sent = 0 AND r.notify_at > ?1 AND n.deleted_at IS NULL",
        )?
        .query_row(params![now], |row| row.get(0))?;
    let event: Option<String> = conn
        .prepare_cached(
            "SELECT MIN(r.notify_at)
             FROM event_reminders r JOIN events e ON e.id = r.event_id
             WHERE r.is_sent = 0 AND r.notify_at > ?1 AND e.deleted_at IS NULL",
        )?
        .query_row(params![now], |row| row.get(0))?;
    Ok([task, note, event].into_iter().flatten().min())
}

pub fn mark_sent(tx: &Transaction<'_>, id: &str, now: &str) -> rusqlite::Result<bool> {
    let changed = tx
        .prepare_cached(
            "UPDATE reminders SET is_sent = 1, sent_at = ?2 WHERE id = ?1 AND is_sent = 0",
        )?
        .execute(params![id, now])?;
    if changed > 0 {
        return Ok(true);
    }
    for sql in [
        "UPDATE note_reminders SET is_sent = 1, sent_at = ?2 WHERE id = ?1 AND is_sent = 0",
        "UPDATE event_reminders SET is_sent = 1, sent_at = ?2 WHERE id = ?1 AND is_sent = 0",
    ] {
        if tx.prepare_cached(sql)?.execute(params![id, now])? > 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Marks as sent, without notifying, every unsent reminder due before `cutoff` (F-14).
/// Returns their ids.
pub fn expire_before(
    tx: &Transaction<'_>,
    cutoff: &str,
    now: &str,
) -> rusqlite::Result<Vec<String>> {
    let mut ids: Vec<String> = Vec::new();
    for sql in [
        "UPDATE reminders SET is_sent = 1, sent_at = ?2
         WHERE is_sent = 0 AND notify_at < ?1 RETURNING id",
        "UPDATE note_reminders SET is_sent = 1, sent_at = ?2
         WHERE is_sent = 0 AND notify_at < ?1 RETURNING id",
        "UPDATE event_reminders SET is_sent = 1, sent_at = ?2
         WHERE is_sent = 0 AND notify_at < ?1 RETURNING id",
    ] {
        let mut expired: Vec<String> = tx
            .prepare_cached(sql)?
            .query_map(params![cutoff, now], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        expired.sort();
        ids.extend(expired);
    }
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

/// Deletes the unsent reminders relative to the task's time that are due at `instant` or before.
pub fn delete_unsent_relative_through(
    tx: &Transaction<'_>,
    task_id: &str,
    instant: &str,
) -> rusqlite::Result<()> {
    tx.prepare_cached(
        "DELETE FROM reminders
         WHERE task_id = ?1 AND is_sent = 0 AND offset_minutes IS NOT NULL AND notify_at <= ?2",
    )?
    .execute(params![task_id, instant])?;
    Ok(())
}

/// Whether the task already has an unsent reminder exactly at `instant`.
pub fn unsent_exists_at(
    tx: &Transaction<'_>,
    task_id: &str,
    instant: &str,
) -> rusqlite::Result<bool> {
    tx.prepare_cached(
        "SELECT EXISTS(SELECT 1 FROM reminders
                       WHERE task_id = ?1 AND is_sent = 0 AND notify_at = ?2)",
    )?
    .query_row(params![task_id, instant], |row| row.get(0))
}

/// Adds one unsent reminder at a custom time without touching the ones the task already has
/// (a snoozed notification, F17-02).
pub fn insert_custom(
    tx: &Transaction<'_>,
    id: &str,
    task_id: &str,
    notify_at: &str,
    now: &str,
) -> rusqlite::Result<()> {
    tx.prepare_cached(
        "INSERT INTO reminders (id, task_id, notify_at, offset_minutes, created_at)
         VALUES (?1, ?2, ?3, NULL, ?4)",
    )?
    .execute(params![id, task_id, notify_at, now])?;
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

#[cfg(test)]
mod expire_tests {
    use super::*;
    use crate::db::{MIGRATIONS, open_in_memory};

    #[test]
    fn expire_before_marks_only_overdue_unsent_reminders_of_both_kinds() {
        let mut conn = open_in_memory(&MIGRATIONS).expect("database");
        conn.execute_batch(
            "INSERT INTO tasks (id, title, created_at, updated_at) VALUES ('t', 'T', 'x', 'x');
             INSERT INTO notes (id, title, created_at, updated_at) VALUES ('n', 'N', 'x', 'x');
             INSERT INTO reminders (id, task_id, notify_at, is_sent, created_at) VALUES
               ('r-b', 't', '2026-03-01T08:00:00Z', 0, 'x'),
               ('r-a', 't', '2026-03-01T09:00:00Z', 0, 'x'),
               ('r-late', 't', '2026-03-05T09:00:00Z', 0, 'x'),
               ('r-done', 't', '2026-03-01T07:00:00Z', 1, 'x');
             INSERT INTO note_reminders (id, note_id, notify_at, created_at)
               VALUES ('nr', 'n', '2026-03-01T10:00:00Z', 'x');",
        )
        .expect("seed");

        let tx = conn.transaction().expect("tx");
        let ids =
            expire_before(&tx, "2026-03-02T00:00:00Z", "2026-03-02T00:00:01Z").expect("expire");
        tx.commit().expect("commit");

        assert_eq!(ids, ["r-a", "r-b", "nr"]);
        let sent: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM reminders WHERE is_sent = 1 AND sent_at IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .expect("count");
        assert_eq!(sent, 2, "the already sent one keeps its empty sent_at");
        let late: i64 = conn
            .query_row(
                "SELECT is_sent FROM reminders WHERE id = 'r-late'",
                [],
                |r| r.get(0),
            )
            .expect("late");
        assert_eq!(late, 0);
    }
}

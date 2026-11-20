//! Habits and their daily marks (D-95). Storage only: the days a habit is due and its streak are
//! `domain::habits`, applied by `services::habits`.

use std::collections::{BTreeSet, HashMap};

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use super::model::{HabitData, HabitLog, HabitRow};

const COLUMNS: &str =
    "id, title, days, time, tag_id, position, created_at, updated_at, archived_at";

fn map_habit(row: &Row<'_>) -> rusqlite::Result<HabitRow> {
    Ok(HabitRow {
        id: row.get(0)?,
        title: row.get(1)?,
        days: row.get(2)?,
        time: row.get(3)?,
        tag_id: row.get(4)?,
        position: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        archived_at: row.get(8)?,
    })
}

/// A live (not trashed) habit, archived or not.
pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<HabitRow>> {
    conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM habits WHERE id = ?1 AND deleted_at IS NULL"
    ))?
    .query_row([id], map_habit)
    .optional()
}

/// Live habits in display order; archived ones only when `include_archived`.
pub fn list(conn: &Connection, include_archived: bool) -> rusqlite::Result<Vec<HabitRow>> {
    conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM habits
         WHERE deleted_at IS NULL AND (?1 OR archived_at IS NULL)
         ORDER BY archived_at IS NOT NULL, position, id"
    ))?
    .query_map([include_archived], map_habit)?
    .collect()
}

pub fn create(tx: &Transaction<'_>, data: &HabitData) -> rusqlite::Result<HabitRow> {
    tx.prepare_cached(
        "INSERT INTO habits (id, title, days, time, tag_id, position, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5,
                 (SELECT COALESCE(MAX(position), -1) + 1 FROM habits WHERE deleted_at IS NULL),
                 ?6, ?6)",
    )?
    .execute(params![
        data.id,
        data.title,
        data.days,
        data.time,
        data.tag_id,
        data.now
    ])?;
    get(tx, &data.id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

/// Replaces title, days, time and tag of a live habit. `None` if it does not exist.
pub fn update(tx: &Transaction<'_>, data: &HabitData) -> rusqlite::Result<Option<HabitRow>> {
    let changed = tx
        .prepare_cached(
            "UPDATE habits SET title = ?2, days = ?3, time = ?4, tag_id = ?5, updated_at = ?6
             WHERE id = ?1 AND deleted_at IS NULL",
        )?
        .execute(params![
            data.id,
            data.title,
            data.days,
            data.time,
            data.tag_id,
            data.now
        ])?;
    if changed == 0 {
        return Ok(None);
    }
    get(tx, &data.id)
}

/// Archives (`Some(now)`) or unarchives (`None`) a live habit. `false` if it does not exist.
pub fn set_archived(
    tx: &Transaction<'_>,
    id: &str,
    archived_at: Option<&str>,
    now: &str,
) -> rusqlite::Result<bool> {
    let changed = tx
        .prepare_cached(
            "UPDATE habits SET archived_at = ?2, updated_at = ?3
             WHERE id = ?1 AND deleted_at IS NULL",
        )?
        .execute(params![id, archived_at, now])?;
    Ok(changed > 0)
}

pub fn soft_delete(tx: &Transaction<'_>, id: &str, now: &str) -> rusqlite::Result<bool> {
    let changed = tx
        .prepare_cached(
            "UPDATE habits SET deleted_at = ?2, updated_at = ?2
             WHERE id = ?1 AND deleted_at IS NULL",
        )?
        .execute(params![id, now])?;
    Ok(changed > 0)
}

pub fn restore(tx: &Transaction<'_>, id: &str, now: &str) -> rusqlite::Result<Option<HabitRow>> {
    let changed = tx
        .prepare_cached(
            "UPDATE habits SET deleted_at = NULL, updated_at = ?2
             WHERE id = ?1 AND deleted_at IS NOT NULL",
        )?
        .execute(params![id, now])?;
    if changed == 0 {
        return Ok(None);
    }
    get(tx, id)
}

/// Marks (`done`) or unmarks `date` for the habit. Only that habit's row is touched.
pub fn set_log(
    tx: &Transaction<'_>,
    habit_id: &str,
    date: &str,
    done: bool,
) -> rusqlite::Result<()> {
    if done {
        tx.prepare_cached("INSERT OR IGNORE INTO habit_logs (habit_id, date) VALUES (?1, ?2)")?
            .execute(params![habit_id, date])?;
    } else {
        tx.prepare_cached("DELETE FROM habit_logs WHERE habit_id = ?1 AND date = ?2")?
            .execute(params![habit_id, date])?;
    }
    Ok(())
}

/// Marks from `from` to `to` (inclusive) of the live habits, by habit and day.
pub fn logs_between(conn: &Connection, from: &str, to: &str) -> rusqlite::Result<Vec<HabitLog>> {
    conn.prepare_cached(
        "SELECT l.habit_id, l.date FROM habit_logs l JOIN habits h ON h.id = l.habit_id
         WHERE h.deleted_at IS NULL AND l.date BETWEEN ?1 AND ?2
         ORDER BY l.date, l.habit_id",
    )?
    .query_map(params![from, to], |row| {
        Ok(HabitLog {
            habit_id: row.get(0)?,
            date: row.get(1)?,
        })
    })?
    .collect()
}

/// Every marked day of the live habits, grouped by habit, for the streaks. Marks that are not a
/// valid date (a damaged file) are left out.
pub fn all_marks(conn: &Connection) -> rusqlite::Result<HashMap<String, BTreeSet<NaiveDate>>> {
    let mut statement = conn.prepare_cached(
        "SELECT l.habit_id, l.date FROM habit_logs l JOIN habits h ON h.id = l.habit_id
         WHERE h.deleted_at IS NULL",
    )?;
    let mut marks: HashMap<String, BTreeSet<NaiveDate>> = HashMap::new();
    for row in statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })? {
        let (habit_id, date) = row?;
        if let Ok(date) = NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
            marks.entry(habit_id).or_default().insert(date);
        }
    }
    Ok(marks)
}

/// Trashed habits, most recently deleted first.
pub fn list_trash(conn: &Connection) -> rusqlite::Result<Vec<HabitRow>> {
    conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM habits WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC, id"
    ))?
    .query_map([], map_habit)?
    .collect()
}

/// Permanently removes habits trashed more than `PURGE_AFTER_DAYS` days before `now`; their
/// marks go with them. Returns how many were purged.
pub fn purge_deleted(tx: &Transaction<'_>, now: &str) -> rusqlite::Result<usize> {
    tx.prepare_cached(
        "DELETE FROM habits
         WHERE deleted_at IS NOT NULL
           AND deleted_at < strftime('%Y-%m-%dT%H:%M:%SZ', ?1, '-' || ?2 || ' days')",
    )?
    .execute(params![now, super::tasks::PURGE_AFTER_DAYS])
}

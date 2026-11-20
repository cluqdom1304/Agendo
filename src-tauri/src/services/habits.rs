//! Habit use cases (D-95). The Tauri commands in `commands/habits.rs` only wrap these functions.

use chrono::{Local, NaiveDate, TimeZone};
use serde::{Deserialize, Deserializer};

use crate::{
    domain::{
        dates,
        habits::{self, EVERY_DAY},
        validation,
    },
    error::{AppError, AppResult},
    repo::{
        habits as repo,
        model::{Habit, HabitData, HabitLog, HabitRow},
        tags,
    },
    state::AppState,
};

/// Longest window `logs` answers for, in days.
pub const MAX_LOG_WINDOW_DAYS: i64 = 400;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitInput {
    pub title: String,
    /// Weekday mask, bit 0 = Monday; every day when absent.
    pub days: Option<u8>,
    pub time: Option<String>,
    pub tag_id: Option<String>,
}

/// Distinguishes an absent field (leave as is) from `null` (clear it).
fn present_or_null<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitPatchInput {
    pub title: Option<String>,
    pub days: Option<u8>,
    #[serde(default, deserialize_with = "present_or_null")]
    pub time: Option<Option<String>>,
    #[serde(default, deserialize_with = "present_or_null")]
    pub tag_id: Option<Option<String>>,
    pub archived: Option<bool>,
}

fn habit_not_found(id: &str) -> AppError {
    AppError::not_found("hábito", id)
}

fn normalize(data: HabitData) -> AppResult<HabitData> {
    let title = data.title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::Validation("el hábito necesita un nombre".into()));
    }
    validation::validate_title(&title)?;
    if title.chars().count() > 200 {
        return Err(AppError::Validation(
            "el nombre del hábito no puede superar 200 caracteres".into(),
        ));
    }
    if data.days == 0 || data.days > EVERY_DAY {
        return Err(AppError::Validation(
            "el hábito necesita al menos un día de la semana".into(),
        ));
    }
    if let Some(time) = &data.time {
        dates::parse_time(time)?;
    }
    Ok(HabitData { title, ..data })
}

fn today<Tz: TimeZone>(state: &AppState, zone: &Tz) -> NaiveDate {
    dates::utc_to_local(zone, state.clock.now()).date()
}

fn present(
    row: HabitRow,
    marks: Option<&std::collections::BTreeSet<NaiveDate>>,
    day: NaiveDate,
) -> Habit {
    let empty = std::collections::BTreeSet::new();
    let marks = marks.unwrap_or(&empty);
    let streak = habits::streak(row.days, marks, day);
    Habit {
        scheduled_today: habits::is_scheduled(row.days, day),
        done_today: marks.contains(&day),
        current_streak: streak.current,
        best_streak: streak.best,
        is_archived: row.archived_at.is_some(),
        id: row.id,
        title: row.title,
        days: row.days,
        time: row.time,
        tag_id: row.tag_id,
        position: row.position,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

/// Live habits with their streaks as of today in `zone`.
pub fn list_in<Tz: TimeZone>(
    state: &AppState,
    include_archived: bool,
    zone: &Tz,
) -> AppResult<Vec<Habit>> {
    let day = today(state, zone);
    state.db.read(|conn| {
        let marks = repo::all_marks(conn)?;
        Ok(repo::list(conn, include_archived)?
            .into_iter()
            .map(|row| {
                let own = marks.get(&row.id);
                present(row, own, day)
            })
            .collect())
    })
}

pub fn list(state: &AppState, include_archived: bool) -> AppResult<Vec<Habit>> {
    list_in(state, include_archived, &Local)
}

fn one<Tz: TimeZone>(
    state: &AppState,
    conn: &rusqlite::Connection,
    id: &str,
    zone: &Tz,
) -> AppResult<Habit> {
    let row = repo::get(conn, id)?.ok_or_else(|| habit_not_found(id))?;
    let marks = repo::all_marks(conn)?;
    Ok(present(row, marks.get(id), today(state, zone)))
}

pub fn create(state: &AppState, input: HabitInput) -> AppResult<Habit> {
    create_in(state, input, &Local)
}

pub fn create_in<Tz: TimeZone>(state: &AppState, input: HabitInput, zone: &Tz) -> AppResult<Habit> {
    let data = normalize(HabitData {
        id: state.ids.new_id(),
        title: input.title,
        days: input.days.unwrap_or(EVERY_DAY),
        time: input.time,
        tag_id: input.tag_id,
        now: state.clock.now_string(),
    })?;
    state.db.write(|tx| {
        check_tag(tx, data.tag_id.as_deref())?;
        repo::create(tx, &data)?;
        one(state, tx, &data.id, zone)
    })
}

pub fn update(state: &AppState, id: &str, patch: HabitPatchInput) -> AppResult<Habit> {
    update_in(state, id, patch, &Local)
}

pub fn update_in<Tz: TimeZone>(
    state: &AppState,
    id: &str,
    patch: HabitPatchInput,
    zone: &Tz,
) -> AppResult<Habit> {
    let now = state.clock.now_string();
    state.db.write(|tx| {
        let current = repo::get(tx, id)?.ok_or_else(|| habit_not_found(id))?;
        let data = normalize(HabitData {
            id: id.to_string(),
            title: patch.title.clone().unwrap_or(current.title.clone()),
            days: patch.days.unwrap_or(current.days),
            time: patch.time.clone().unwrap_or(current.time.clone()),
            tag_id: patch.tag_id.clone().unwrap_or(current.tag_id.clone()),
            now: now.clone(),
        })?;
        check_tag(tx, data.tag_id.as_deref())?;
        repo::update(tx, &data)?.ok_or_else(|| habit_not_found(id))?;
        if let Some(archived) = patch.archived {
            let at = archived.then_some(now.as_str());
            repo::set_archived(tx, id, at, &now)?;
        }
        one(state, tx, id, zone)
    })
}

fn check_tag(conn: &rusqlite::Connection, tag_id: Option<&str>) -> AppResult<()> {
    if let Some(id) = tag_id
        && tags::get(conn, id)?.is_none()
    {
        return Err(AppError::not_found("etiqueta", id));
    }
    Ok(())
}

pub fn delete(state: &AppState, id: &str) -> AppResult<()> {
    let now = state.clock.now_string();
    state.db.write(|tx| {
        if repo::soft_delete(tx, id, &now)? {
            Ok(())
        } else {
            Err(habit_not_found(id))
        }
    })
}

pub fn restore(state: &AppState, id: &str) -> AppResult<Habit> {
    restore_in(state, id, &Local)
}

pub fn restore_in<Tz: TimeZone>(state: &AppState, id: &str, zone: &Tz) -> AppResult<Habit> {
    let now = state.clock.now_string();
    state.db.write(|tx| {
        repo::restore(tx, id, &now)?.ok_or_else(|| habit_not_found(id))?;
        one(state, tx, id, zone)
    })
}

/// Trashed habits, most recently deleted first.
pub fn trash_list_in<Tz: TimeZone>(state: &AppState, zone: &Tz) -> AppResult<Vec<Habit>> {
    let day = today(state, zone);
    state.db.read(|conn| {
        let marks = repo::all_marks(conn)?;
        Ok(repo::list_trash(conn)?
            .into_iter()
            .map(|row| {
                let own = marks.get(&row.id);
                present(row, own, day)
            })
            .collect())
    })
}

/// Marks or unmarks `date` (local `YYYY-MM-DD`, not later than today) for a habit and returns it
/// with its streaks updated.
pub fn set_done<Tz: TimeZone>(
    state: &AppState,
    id: &str,
    date: &str,
    done: bool,
    zone: &Tz,
) -> AppResult<Habit> {
    let day = dates::parse_date(date)?;
    if day > today(state, zone) {
        return Err(AppError::Validation(
            "no se puede marcar un día que aún no ha llegado".into(),
        ));
    }
    state.db.write(|tx| {
        repo::get(tx, id)?.ok_or_else(|| habit_not_found(id))?;
        repo::set_log(tx, id, &dates::format_date(day), done)?;
        one(state, tx, id, zone)
    })
}

/// The days marked between `from` and `to` (inclusive) for the live habits.
pub fn logs(state: &AppState, from: &str, to: &str) -> AppResult<Vec<HabitLog>> {
    let first = dates::parse_date(from)?;
    let last = dates::parse_date(to)?;
    if last < first {
        return Err(AppError::Validation(
            "el rango termina antes de empezar".into(),
        ));
    }
    if last.signed_duration_since(first).num_days() > MAX_LOG_WINDOW_DAYS {
        return Err(AppError::Validation(
            "el rango de fechas es demasiado grande".into(),
        ));
    }
    state
        .db
        .read(|conn| Ok(repo::logs_between(conn, from, to)?))
}

#[cfg(test)]
mod tests;

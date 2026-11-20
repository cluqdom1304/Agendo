//! Focus sessions on a task (D-97): the backend marks the start and the end with its clock, so a
//! closed window cannot lose the time. The Tauri commands in `commands/focus.rs` only wrap these
//! functions; the scheduler calls [`finish_due`] and [`next_end`].

use chrono::{DateTime, Duration, Local, TimeZone, Utc};

use crate::{
    domain::dates,
    error::{AppError, AppResult},
    notify::{Notification, Notifier},
    repo::{
        model::{TaskFocusTotal, TimeEntry},
        time_entries as repo,
    },
    state::AppState,
};

/// Longest session that can be planned, in minutes.
pub const MAX_PLANNED_MINUTES: i64 = 600;

/// Longest window `list` answers for, in days.
pub const MAX_WINDOW_DAYS: i64 = 400;

fn planned_end(entry: &TimeEntry) -> AppResult<DateTime<Utc>> {
    Ok(dates::parse_instant(&entry.started_at)? + Duration::minutes(entry.planned_minutes))
}

pub fn current(state: &AppState) -> AppResult<Option<TimeEntry>> {
    state.db.read(|conn| Ok(repo::running(conn)?))
}

/// Starts a session of `planned_minutes` on a live task. Only one can run at a time.
pub fn start(state: &AppState, task_id: &str, planned_minutes: i64) -> AppResult<TimeEntry> {
    if !(1..=MAX_PLANNED_MINUTES).contains(&planned_minutes) {
        return Err(AppError::Validation(format!(
            "la sesión debe durar entre 1 y {MAX_PLANNED_MINUTES} minutos"
        )));
    }
    let now = state.clock.now_string();
    let entry = state.db.write(|tx| {
        if repo::task_title(tx, task_id)?.is_none() {
            return Err(AppError::not_found("tarea", task_id));
        }
        if repo::running(tx)?.is_some() {
            return Err(AppError::Conflict(
                "ya hay una sesión de enfoque en marcha".into(),
            ));
        }
        Ok(repo::start(
            tx,
            &state.ids.new_id(),
            task_id,
            planned_minutes,
            &now,
        )?)
    })?;
    state.wake.notify_one();
    Ok(entry)
}

/// Ends the session in progress now. `None` if there was none.
pub fn stop(state: &AppState) -> AppResult<Option<TimeEntry>> {
    let now = state.clock.now_string();
    let entry = state.db.write(|tx| Ok(repo::finish(tx, &now)?))?;
    state.wake.notify_one();
    Ok(entry)
}

/// The sessions that started on the local days `from` to `to` (inclusive, `YYYY-MM-DD`).
pub fn list<Tz: TimeZone>(
    state: &AppState,
    from: &str,
    to: &str,
    zone: &Tz,
) -> AppResult<Vec<TimeEntry>> {
    let first = dates::parse_date(from)?;
    let last = dates::parse_date(to)?;
    if last < first {
        return Err(AppError::Validation(
            "el rango termina antes de empezar".into(),
        ));
    }
    if last.signed_duration_since(first).num_days() > MAX_WINDOW_DAYS {
        return Err(AppError::Validation(
            "el rango de fechas es demasiado grande".into(),
        ));
    }
    let midnight = dates::parse_time("00:00")?;
    let start = dates::local_to_utc(zone, first, midnight)?;
    let end = dates::local_to_utc(zone, last + Duration::days(1), midnight)?;
    state.db.read(|conn| {
        Ok(repo::between(
            conn,
            &dates::format_instant(start),
            &dates::format_instant(end),
        )?)
    })
}

/// When the session in progress is due to end, if one is running.
pub fn next_end(state: &AppState) -> AppResult<Option<DateTime<Utc>>> {
    current(state)?.map(|entry| planned_end(&entry)).transpose()
}

/// Ends the session in progress if its planned time is over, at its planned end, and says so with
/// a notification. Returns the closed session. The session is closed even if the notice fails:
/// the time is already counted and a notice is not worth blocking the scheduler for.
pub fn finish_due(state: &AppState, notifier: &dyn Notifier) -> AppResult<Option<TimeEntry>> {
    let now = state.clock.now();
    let Some(entry) = current(state)? else {
        return Ok(None);
    };
    let end = planned_end(&entry)?;
    if end > now {
        return Ok(None);
    }
    let closed = state
        .db
        .write(|tx| Ok(repo::finish(tx, &dates::format_instant(end))?))?;
    let Some(closed) = closed else {
        return Ok(None);
    };
    let title = state
        .db
        .read(|conn| Ok(repo::task_title(conn, &closed.task_id)?))?
        .unwrap_or_default();
    let notice = Notification {
        title: "Sesión de enfoque terminada".into(),
        body: format!("{} min · {title}", closed.planned_minutes),
        task_id: None,
        note_id: None,
        event_id: None,
    };
    if let Err(error) = notifier.show(&notice) {
        crate::applog::warn(
            "focus",
            &format!("no se pudo avisar del fin de la sesión: {error}"),
        );
    }
    Ok(Some(closed))
}

/// At start-up a session without an end is closed: the app was closed while it ran. Its end is
/// the moment it was left, capped at the planned time (D-97). Returns the closed session.
pub fn recover_orphans(state: &AppState) -> AppResult<Option<TimeEntry>> {
    let now = state.clock.now();
    let Some(entry) = current(state)? else {
        return Ok(None);
    };
    let end = planned_end(&entry)?.min(now);
    state
        .db
        .write(|tx| Ok(repo::finish(tx, &dates::format_instant(end))?))
}

/// Minutes of focus per task, for the rows and the detail.
pub fn totals(state: &AppState) -> AppResult<Vec<TaskFocusTotal>> {
    state.db.read(|conn| Ok(repo::totals(conn)?))
}

/// The closed sessions and manual entries of one task, newest first.
pub fn of_task(state: &AppState, task_id: &str) -> AppResult<Vec<TimeEntry>> {
    state.db.read(|conn| Ok(repo::of_task(conn, task_id)?))
}

/// Records `minutes` already spent on a task, ending now. It does not touch a session in
/// progress.
pub fn add_manual(state: &AppState, task_id: &str, minutes: i64) -> AppResult<TimeEntry> {
    if !(1..=MAX_PLANNED_MINUTES).contains(&minutes) {
        return Err(AppError::Validation(format!(
            "el tiempo debe estar entre 1 y {MAX_PLANNED_MINUTES} minutos"
        )));
    }
    let now = state.clock.now();
    let started = dates::format_instant(now - Duration::minutes(minutes));
    let ended = dates::format_instant(now);
    state.db.write(|tx| {
        if repo::task_title(tx, task_id)?.is_none() {
            return Err(AppError::not_found("tarea", task_id));
        }
        Ok(repo::insert_closed(
            tx,
            &state.ids.new_id(),
            task_id,
            minutes,
            &started,
            &ended,
        )?)
    })
}

pub fn delete_entry(state: &AppState, id: &str) -> AppResult<()> {
    let deleted = state.db.write(|tx| Ok(repo::delete_closed(tx, id)?))?;
    if deleted {
        Ok(())
    } else {
        Err(AppError::not_found("entrada de tiempo", id))
    }
}

pub fn list_local(state: &AppState, from: &str, to: &str) -> AppResult<Vec<TimeEntry>> {
    list(state, from, to, &Local)
}

#[cfg(test)]
mod tests;

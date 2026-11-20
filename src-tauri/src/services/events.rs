//! Event use cases (D-94): dates and repetition rules, the expansion of a series into the days of
//! a window, and the reminders that warn before an occurrence. The Tauri commands in
//! `commands/events.rs` only wrap these functions.

use chrono::{Duration, Local, NaiveDate, TimeZone};
use rusqlite::Transaction;
use serde::{Deserialize, Deserializer};

use crate::{
    domain::{
        dates,
        recurrence::{MAX_SCANNED_OCCURRENCES, Rule},
        reminders as rules, validation,
    },
    error::{AppError, AppResult},
    repo::{
        events,
        model::{Event, EventData, EventOccurrence},
        settings,
    },
    state::AppState,
};

/// Most reminders one event may have.
pub const MAX_REMINDERS: usize = 10;
/// Longest window `list` answers for, in days.
pub const MAX_WINDOW_DAYS: i64 = 800;
/// Longest note of an event, in characters.
pub const MAX_NOTES_CHARS: usize = 20_000;
/// Minutes an event lasts when it has a start time and no end time.
const DEFAULT_MINUTES: i64 = 60;
/// Occurrences examined when looking for the next one that still has to warn.
const MAX_REMINDER_LOOKAHEAD: usize = 64;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventInput {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub start_date: String,
    pub start_time: Option<String>,
    /// Defaults to the start date.
    pub end_date: Option<String>,
    /// Defaults to an hour after the start time; absent for an all-day event.
    pub end_time: Option<String>,
    pub rrule: Option<String>,
    pub tag_ids: Option<Vec<String>>,
    pub reminder_offsets: Option<Vec<i64>>,
}

/// Distinguishes an absent field (leave as is) from `null` (clear it).
fn present_or_null<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(deserializer).map(Some)
}

/// Partial update of an event, as sent by the frontend.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventPatchInput {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub start_date: Option<String>,
    /// `null` makes it an all-day event.
    #[serde(default, deserialize_with = "present_or_null")]
    pub start_time: Option<Option<String>>,
    pub end_date: Option<String>,
    pub end_time: Option<String>,
    #[serde(default, deserialize_with = "present_or_null")]
    pub rrule: Option<Option<String>>,
    pub tag_ids: Option<Vec<String>>,
    pub reminder_offsets: Option<Vec<i64>>,
}

fn invalid(message: &str) -> AppError {
    AppError::Validation(message.to_string())
}

fn event_not_found(id: &str) -> AppError {
    AppError::not_found("evento", id)
}

fn minutes_of(time: &str) -> AppResult<i64> {
    let time = dates::parse_time(time)?;
    Ok(i64::from(chrono::Timelike::hour(&time)) * 60 + i64::from(chrono::Timelike::minute(&time)))
}

fn time_after(start: &str, minutes: i64) -> AppResult<String> {
    let total = (minutes_of(start)? + minutes).min(24 * 60 - 1);
    Ok(format!("{:02}:{:02}", total / 60, total % 60))
}

/// Checks every rule of an event and returns it ready to store: an all-day event has no times,
/// the end is not before the start, the repeat rule is anchored on the start date and the
/// reminders are distinct and fit the kind of event.
fn normalize(mut data: EventData) -> AppResult<EventData> {
    validation::validate_title(&data.title)?;
    if data.notes.chars().count() > MAX_NOTES_CHARS {
        return Err(AppError::Validation(format!(
            "las notas del evento no pueden superar {MAX_NOTES_CHARS} caracteres"
        )));
    }
    let start = dates::parse_date(&data.start_date)?;
    let end = dates::parse_date(&data.end_date)?;
    if end < start {
        return Err(invalid("el evento no puede terminar antes de empezar"));
    }
    match (&data.start_time, &data.end_time) {
        (None, None) => {}
        (Some(from), Some(to)) => {
            dates::parse_time(from)?;
            dates::parse_time(to)?;
            if end == start && minutes_of(to)? < minutes_of(from)? {
                return Err(invalid("el evento no puede terminar antes de empezar"));
            }
        }
        _ => {
            return Err(invalid(
                "un evento lleva hora de inicio y de fin, o ninguna",
            ));
        }
    }
    data.rrule = data
        .rrule
        .as_deref()
        .map(|rule| Rule::parse(rule)?.anchored(start).map(|rule| rule.format()))
        .transpose()?;

    if data.reminder_offsets.len() > MAX_REMINDERS {
        return Err(AppError::Validation(format!(
            "un evento admite como máximo {MAX_REMINDERS} recordatorios"
        )));
    }
    for offset in &data.reminder_offsets {
        rules::validate_offset(*offset)?;
        if data.start_time.is_none() {
            rules::validate_untimed_offset(*offset)?;
        }
    }
    data.reminder_offsets.sort_unstable();
    data.reminder_offsets.dedup();
    Ok(data)
}

pub fn create(state: &AppState, input: EventInput) -> AppResult<Event> {
    create_in(state, input, &Local)
}

pub fn create_in<Tz: TimeZone>(state: &AppState, input: EventInput, zone: &Tz) -> AppResult<Event> {
    let end_date = input.end_date.unwrap_or_else(|| input.start_date.clone());
    let end_time = match (&input.start_time, input.end_time) {
        (Some(start), None) => Some(time_after(start, DEFAULT_MINUTES)?),
        (None, _) => None,
        (Some(_), end) => end,
    };
    let data = normalize(EventData {
        id: state.ids.new_id(),
        title: input.title.unwrap_or_default(),
        notes: input.notes.unwrap_or_default(),
        start_date: input.start_date,
        start_time: input.start_time,
        end_date,
        end_time,
        rrule: input.rrule,
        tag_ids: input.tag_ids.unwrap_or_default(),
        reminder_offsets: input.reminder_offsets.unwrap_or_default(),
        now: state.clock.now_string(),
    })?;
    let now = state.clock.now();
    let event = state.db.write(|tx| {
        let event = events::create(tx, &data)?;
        schedule_reminders(state, tx, &event, now, zone, true)?;
        Ok(event)
    })?;
    state.wake.notify_one();
    Ok(event)
}

pub fn get(state: &AppState, id: &str) -> AppResult<Event> {
    state
        .db
        .read(|conn| Ok(events::get(conn, id)?))?
        .ok_or_else(|| event_not_found(id))
}

pub fn update(state: &AppState, id: &str, patch: EventPatchInput) -> AppResult<Event> {
    update_in(state, id, patch, &Local)
}

pub fn update_in<Tz: TimeZone>(
    state: &AppState,
    id: &str,
    patch: EventPatchInput,
    zone: &Tz,
) -> AppResult<Event> {
    let now = state.clock.now();
    let event = state.db.write(|tx| {
        let current = events::get(tx, id)?.ok_or_else(|| event_not_found(id))?;
        let start_date = patch
            .start_date
            .clone()
            .unwrap_or(current.start_date.clone());
        let start_time = patch
            .start_time
            .clone()
            .unwrap_or_else(|| current.start_time.clone());
        // Moving the start keeps the length of the event unless the patch sets the end too.
        let (end_date, end_time) = if patch.end_date.is_some() || patch.end_time.is_some() {
            (
                patch.end_date.clone().unwrap_or(current.end_date.clone()),
                patch.end_time.clone().or(current.end_time.clone()),
            )
        } else if patch.start_date.is_some() || patch.start_time.is_some() {
            shifted_end(&current, &start_date, start_time.as_deref())?
        } else {
            (current.end_date.clone(), current.end_time.clone())
        };
        let end_time = match (&start_time, end_time) {
            (None, _) => None,
            (Some(start), None) => Some(time_after(start, DEFAULT_MINUTES)?),
            (Some(_), end) => end,
        };
        let data = normalize(EventData {
            id: id.to_string(),
            title: patch.title.clone().unwrap_or(current.title.clone()),
            notes: patch.notes.clone().unwrap_or(current.notes.clone()),
            start_date,
            start_time,
            end_date,
            end_time,
            rrule: patch.rrule.clone().unwrap_or(current.rrule.clone()),
            tag_ids: patch.tag_ids.clone().unwrap_or(current.tag_ids.clone()),
            reminder_offsets: patch
                .reminder_offsets
                .clone()
                .unwrap_or(current.reminder_offsets.clone()),
            now: dates::format_instant(now),
        })?;
        let event = events::update(tx, &data)?.ok_or_else(|| event_not_found(id))?;
        schedule_reminders(state, tx, &event, now, zone, true)?;
        Ok(event)
    })?;
    state.wake.notify_one();
    Ok(event)
}

/// End of `current` after its start moves to `start_date` / `start_time`, keeping its length.
fn shifted_end(
    current: &Event,
    start_date: &str,
    start_time: Option<&str>,
) -> AppResult<(String, Option<String>)> {
    let old_start = dates::parse_date(&current.start_date)?;
    let old_end = dates::parse_date(&current.end_date)?;
    let new_start = dates::parse_date(start_date)?;
    let days = old_end.signed_duration_since(old_start);
    let end_date = dates::format_date(
        new_start
            .checked_add_signed(days)
            .ok_or_else(|| invalid("fecha fuera de rango"))?,
    );
    let end_time = match (start_time, &current.start_time, &current.end_time) {
        (Some(new), Some(old), Some(end)) => {
            let length = minutes_of(end)? - minutes_of(old)? + days.num_days() * 24 * 60;
            let total = minutes_of(new)? + length;
            if (0..24 * 60).contains(&total) && days.num_days() == 0 {
                Some(format!("{:02}:{:02}", total / 60, total % 60))
            } else {
                Some(end.clone())
            }
        }
        _ => None,
    };
    Ok((end_date, end_time))
}

pub fn delete(state: &AppState, id: &str) -> AppResult<()> {
    let now = state.clock.now_string();
    state.db.write(|tx| {
        if !events::soft_delete(tx, id, &now)? {
            return Err(event_not_found(id));
        }
        events::clear_unsent(tx, id)?;
        Ok(())
    })?;
    state.wake.notify_one();
    Ok(())
}

pub fn restore(state: &AppState, id: &str) -> AppResult<Event> {
    restore_in(state, id, &Local)
}

pub fn restore_in<Tz: TimeZone>(state: &AppState, id: &str, zone: &Tz) -> AppResult<Event> {
    let now = state.clock.now_string();
    let instant = state.clock.now();
    let event = state.db.write(|tx| {
        let event = events::restore(tx, id, &now)?.ok_or_else(|| event_not_found(id))?;
        schedule_reminders(state, tx, &event, instant, zone, true)?;
        Ok(event)
    })?;
    state.wake.notify_one();
    Ok(event)
}

/// What shows on the calendar between `from` and `to` (inclusive): single events that overlap
/// the window and every repetition of a series that starts or ends inside it.
pub fn list(state: &AppState, from: &str, to: &str) -> AppResult<Vec<EventOccurrence>> {
    let first = dates::parse_date(from)?;
    let last = dates::parse_date(to)?;
    if last < first {
        return Err(invalid("el rango termina antes de empezar"));
    }
    if last.signed_duration_since(first).num_days() > MAX_WINDOW_DAYS {
        return Err(invalid("el rango de fechas es demasiado grande"));
    }
    let found = state
        .db
        .read(|conn| Ok(events::candidates(conn, from, to)?))?;
    let mut occurrences = Vec::new();
    for event in &found {
        expand(event, first, last, &mut occurrences)?;
    }
    occurrences.sort_by(|a, b| {
        (&a.start_date, !a.all_day, &a.start_time, &a.event_id).cmp(&(
            &b.start_date,
            !b.all_day,
            &b.start_time,
            &b.event_id,
        ))
    });
    Ok(occurrences)
}

fn expand(
    event: &Event,
    first: NaiveDate,
    last: NaiveDate,
    out: &mut Vec<EventOccurrence>,
) -> AppResult<()> {
    let start = dates::parse_date(&event.start_date)?;
    let span = dates::parse_date(&event.end_date)?.signed_duration_since(start);
    let starts = match &event.rrule {
        // An occurrence that began before the window still shows if it lasts into it.
        Some(rule) => Rule::parse(rule)?.occurrences_in(start, first - span, last),
        None => vec![start],
    };
    for date in starts {
        let end = date + span;
        if end < first || date > last {
            continue;
        }
        out.push(EventOccurrence {
            event_id: event.id.clone(),
            title: event.title.clone(),
            start_date: dates::format_date(date),
            start_time: event.start_time.clone(),
            end_date: dates::format_date(end),
            end_time: event.end_time.clone(),
            all_day: event.all_day,
            recurring: event.rrule.is_some(),
            tag_ids: event.tag_ids.clone(),
        });
    }
    Ok(())
}

/// Leaves, for each reminder of the event, one unsent row at the first occurrence that still has
/// to warn (D-94). Rows that already sounded stay as history. With `replace` the rows that are
/// waiting are first thrown away (the event changed); without it the ones that wait are kept,
/// including those that are due but have not sounded yet.
fn schedule_reminders<Tz: TimeZone>(
    state: &AppState,
    tx: &Transaction<'_>,
    event: &Event,
    now: chrono::DateTime<chrono::Utc>,
    zone: &Tz,
    replace: bool,
) -> AppResult<()> {
    if replace {
        events::clear_unsent(tx, &event.id)?;
    }
    if event.reminder_offsets.is_empty() {
        return Ok(());
    }
    let waiting = events::pending_offsets(tx, &event.id)?;
    let start = dates::parse_date(&event.start_date)?;
    let time = match &event.start_time {
        Some(time) => time.clone(),
        None => settings::notify_time(tx)?,
    };
    let rule = event.rrule.as_deref().map(Rule::parse).transpose()?;
    // An occurrence before yesterday started before any reminder that has yet to sound.
    let from = dates::utc_to_local(zone, now).date() - Duration::days(1);
    let now_text = dates::format_instant(now);
    for offset in event
        .reminder_offsets
        .iter()
        .filter(|o| !waiting.contains(o))
    {
        let candidates: Vec<NaiveDate> = match &rule {
            Some(rule) => rule
                .occurrences(start)
                .take(MAX_SCANNED_OCCURRENCES)
                .filter(|date| *date >= from)
                .take(MAX_REMINDER_LOOKAHEAD)
                .collect(),
            None => vec![start],
        };
        for date in candidates {
            let notify_at =
                rules::offset_notify_at(zone, &dates::format_date(date), &time, *offset)?;
            if notify_at > now {
                events::insert_reminder(
                    tx,
                    &state.ids.new_id(),
                    &event.id,
                    *offset,
                    &dates::format_instant(notify_at),
                    &now_text,
                )?;
                break;
            }
        }
    }
    Ok(())
}

/// A reminder of the scheduler has just been closed (sounded, given up or expired): if it was an
/// event's, store the warning for the next occurrence. Runs in the same transaction as the
/// closing. Does nothing for the reminders of tasks and notes.
pub fn continue_after_reminder<Tz: TimeZone>(
    state: &AppState,
    tx: &Transaction<'_>,
    reminder_id: &str,
    now: chrono::DateTime<chrono::Utc>,
    zone: &Tz,
) -> AppResult<()> {
    let Some(event_id) = events::event_of_reminder(tx, reminder_id)? else {
        return Ok(());
    };
    if let Some(event) = events::get(tx, &event_id)?
        && event.rrule.is_some()
    {
        schedule_reminders(state, tx, &event, now, zone, false)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;

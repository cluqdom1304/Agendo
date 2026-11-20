//! Weekly summary (D-100, F26-01): what was completed, left open or late, which events there were,
//! how well each habit was kept and how much focus time was recorded.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{Duration, Local, NaiveDate, NaiveTime, TimeZone};

use crate::{
    domain::{dates, habits},
    error::{AppError, AppResult},
    repo::{
        habits as habit_repo,
        model::{HabitWeek, TagTally, WeekSummary},
        summary as repo, time_entries,
    },
    services::events,
    state::AppState,
};

/// The summary of the week that contains `date` (local `YYYY-MM-DD`); `first_day` is the first
/// day of the week, 0 for Sunday to 6 for Saturday.
pub fn week(state: &AppState, date: &str, first_day: u8) -> AppResult<WeekSummary> {
    week_in(state, date, first_day, &Local)
}

pub fn week_in<Tz: TimeZone>(
    state: &AppState,
    date: &str,
    first_day: u8,
    zone: &Tz,
) -> AppResult<WeekSummary> {
    if first_day > 6 {
        return Err(AppError::Validation(
            "el primer día de la semana no es válido".into(),
        ));
    }
    let start = dates::start_of_week(dates::parse_date(date)?, first_day);
    let end = start + Duration::days(6);
    let days: Vec<NaiveDate> = (0..7)
        .map(|offset| start + Duration::days(offset))
        .collect();
    let today = dates::utc_to_local(zone, state.clock.now()).date();
    let (start_text, end_text) = (dates::format_date(start), dates::format_date(end));
    let midnight = NaiveTime::MIN;
    let from_utc = dates::local_to_utc(zone, start, midnight)?;
    let to_utc = dates::local_to_utc(zone, end + Duration::days(1), midnight)?;
    let (from_text, to_text) = (
        dates::format_instant(from_utc),
        dates::format_instant(to_utc),
    );

    let (facts, habit_rows, marks, sessions) = state.db.read(|conn| {
        Ok((
            repo::task_facts(conn, &start_text, &end_text, &from_text, &to_text)?,
            habit_repo::list(conn, false)?,
            habit_repo::all_marks(conn)?,
            time_entries::between(conn, &from_text, &to_text)?,
        ))
    })?;

    let (mut completed, mut pending, mut overdue) = (0, 0, 0);
    let mut tallies: BTreeMap<Option<String>, TagTally> = BTreeMap::new();
    for fact in &facts {
        let late = !fact.completed
            && fact
                .due_date
                .as_deref()
                .is_some_and(|due| due < dates::format_date(today).as_str());
        let tags: Vec<Option<String>> = if fact.tag_ids.is_empty() {
            vec![None]
        } else {
            fact.tag_ids.iter().cloned().map(Some).collect()
        };
        for tag in tags {
            let tally = tallies.entry(tag.clone()).or_insert(TagTally {
                tag_id: tag,
                completed: 0,
                pending: 0,
                overdue: 0,
            });
            match (fact.completed, late) {
                (true, _) => tally.completed += 1,
                (false, true) => tally.overdue += 1,
                (false, false) => tally.pending += 1,
            }
        }
        match (fact.completed, late) {
            (true, _) => completed += 1,
            (false, true) => overdue += 1,
            (false, false) => pending += 1,
        }
    }

    let occurrences = events::list(state, &start_text, &end_text)?;
    let event_minutes = occurrences.iter().map(event_minutes).sum();

    let no_marks = BTreeSet::new();
    let habits = habit_rows
        .into_iter()
        .filter_map(|row| {
            let created = dates::parse_instant(&row.created_at)
                .ok()
                .map(|at| dates::utc_to_local(zone, at).date());
            let own = marks.get(&row.id).unwrap_or(&no_marks);
            let kept: Vec<&NaiveDate> = days
                .iter()
                .filter(|day| **day <= today)
                .filter(|day| created.is_none_or(|first| **day >= first))
                .filter(|day| habits::is_scheduled(row.days, **day))
                .collect();
            (!kept.is_empty()).then(|| HabitWeek {
                habit_id: row.id.clone(),
                title: row.title.clone(),
                scheduled: kept.len() as i64,
                done: kept.iter().filter(|day| own.contains(**day)).count() as i64,
            })
        })
        .collect();

    let mut focus_by_day = vec![0_i64; 7];
    let mut focus_sessions = 0;
    for session in sessions.iter().filter(|entry| entry.spent_seconds > 0) {
        let began = dates::parse_instant(&session.started_at)?;
        let day = dates::utc_to_local(zone, began).date();
        if let Some(slot) = days.iter().position(|candidate| *candidate == day) {
            focus_by_day[slot] += session.spent_seconds;
            focus_sessions += 1;
        }
    }

    Ok(WeekSummary {
        start: start_text,
        end: end_text,
        completed,
        pending,
        overdue,
        by_tag: tallies.into_values().collect(),
        events: occurrences.len() as i64,
        event_minutes,
        habits,
        focus_seconds: focus_by_day.iter().sum(),
        focus_sessions,
        focus_by_day,
    })
}

/// Length of a timed occurrence that starts and ends the same day; all-day and multi-day ones
/// have no meaningful length and count as zero.
fn event_minutes(occurrence: &crate::repo::model::EventOccurrence) -> i64 {
    if occurrence.all_day || occurrence.start_date != occurrence.end_date {
        return 0;
    }
    match (&occurrence.start_time, &occurrence.end_time) {
        (Some(start), Some(end)) => match (dates::parse_time(start), dates::parse_time(end)) {
            (Ok(start), Ok(end)) => (end - start).num_minutes().max(0),
            _ => 0,
        },
        _ => 0,
    }
}

#[cfg(test)]
mod tests;

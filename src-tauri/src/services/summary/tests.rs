use chrono::{Duration, FixedOffset};

use super::*;
use crate::{
    services::{
        events::{self, EventInput},
        focus,
        habits::{self as habit_service, HabitInput},
        tags::{self, TagUpsertInput},
        tasks::{self, TaskCreateInput},
    },
    state::test_state,
};

fn zone() -> FixedOffset {
    FixedOffset::east_opt(3600).expect("zone")
}

fn task(state: &AppState, title: &str, due: Option<&str>, tag_ids: Vec<String>) -> String {
    tasks::create(
        state,
        TaskCreateInput {
            title: Some(title.into()),
            due_date: due.map(str::to_string),
            tag_ids: Some(tag_ids),
            ..Default::default()
        },
    )
    .expect("task")
    .id
}

fn tag(state: &AppState, name: &str) -> String {
    tags::upsert(
        state,
        TagUpsertInput {
            id: None,
            name: name.into(),
            color_hex: "#3366FF".into(),
            parent_id: None,
        },
    )
    .expect("tag")
    .id
}

// The test clock starts on Tuesday 2026-03-10 at 10:00 UTC, 11:00 in `zone()`.

#[test]
fn tasks_are_counted_as_completed_pending_or_overdue_by_tag() {
    let (state, _) = test_state();
    let work = tag(&state, "Trabajo");
    task(&state, "Tarde", Some("2026-03-09"), vec![work.clone()]);
    task(&state, "Viernes", Some("2026-03-13"), vec![work.clone()]);
    task(&state, "Otra semana", Some("2026-03-20"), vec![]);
    let done = task(&state, "Hecha", Some("2026-03-10"), vec![work.clone()]);
    let loose = task(&state, "Suelta", None, vec![]);
    for id in [&done, &loose] {
        tasks::set_completed(&state, id, true).expect("complete");
    }

    let summary = week_in(&state, "2026-03-12", 1, &zone()).expect("week");

    assert_eq!(
        (summary.start.as_str(), summary.end.as_str()),
        ("2026-03-09", "2026-03-15")
    );
    assert_eq!(
        (summary.completed, summary.pending, summary.overdue),
        (2, 1, 1)
    );
    let of = |tag_id: Option<&str>| {
        summary
            .by_tag
            .iter()
            .find(|tally| tally.tag_id.as_deref() == tag_id)
            .map(|tally| (tally.completed, tally.pending, tally.overdue))
    };
    assert_eq!(of(Some(&work)), Some((1, 1, 1)));
    assert_eq!(of(None), Some((1, 0, 0)));
}

#[test]
fn the_week_follows_the_first_day_and_other_weeks_are_empty() {
    let (state, _) = test_state();
    task(&state, "Lunes siguiente", Some("2026-03-16"), vec![]);

    let sunday_first = week_in(&state, "2026-03-10", 0, &zone()).expect("week");
    assert_eq!(sunday_first.start, "2026-03-08");
    assert_eq!(sunday_first.pending, 0);

    let next = week_in(&state, "2026-03-16", 1, &zone()).expect("week");
    assert_eq!(
        (next.start.as_str(), next.pending, next.overdue),
        ("2026-03-16", 1, 0)
    );

    let bad = week_in(&state, "2026-03-10", 7, &zone()).expect_err("invalid");
    assert!(bad.to_string().starts_with("VALIDATION:"), "{bad}");
    let bad = week_in(&state, "no", 1, &zone()).expect_err("invalid");
    assert!(bad.to_string().starts_with("VALIDATION:"), "{bad}");
}

#[test]
fn events_are_counted_with_the_length_of_the_timed_ones() {
    let (state, _) = test_state();
    for (start_time, end_time, date) in [
        (Some("10:00"), Some("11:30"), "2026-03-11"),
        (None, None, "2026-03-12"),
        (Some("10:00"), Some("11:00"), "2026-03-23"),
    ] {
        events::create(
            &state,
            EventInput {
                title: Some("Evento".into()),
                start_date: date.into(),
                start_time: start_time.map(str::to_string),
                end_time: end_time.map(str::to_string),
                ..Default::default()
            },
        )
        .expect("event");
    }

    let summary = week_in(&state, "2026-03-10", 1, &zone()).expect("week");
    assert_eq!((summary.events, summary.event_minutes), (2, 90));
}

#[test]
fn habits_count_scheduled_days_up_to_today_since_they_were_created() {
    let (state, _) = test_state();
    let daily = habit_service::create_in(
        &state,
        HabitInput {
            title: "Leer".into(),
            ..Default::default()
        },
        &zone(),
    )
    .expect("habit");
    // Monday only: the week's Monday is before the habit existed, so it has nothing to keep yet.
    habit_service::create_in(
        &state,
        HabitInput {
            title: "Correr".into(),
            days: Some(1),
            ..Default::default()
        },
        &zone(),
    )
    .expect("habit");
    habit_service::set_done(&state, &daily.id, "2026-03-10", true, &zone()).expect("mark");

    let summary = week_in(&state, "2026-03-10", 1, &zone()).expect("week");
    assert_eq!(summary.habits.len(), 1);
    assert_eq!(summary.habits[0].title, "Leer");
    assert_eq!(
        (summary.habits[0].scheduled, summary.habits[0].done),
        (1, 1)
    );

    let future = week_in(&state, "2026-03-17", 1, &zone()).expect("week");
    assert!(future.habits.is_empty());
}

#[test]
fn focus_time_is_added_up_by_day_without_the_pauses() {
    let (state, clock) = test_state();
    let id = task(&state, "Estudiar", None, vec![]);
    focus::start(&state, &id, 25).expect("start");
    clock.advance(Duration::minutes(20));
    focus::pause(&state).expect("pause");
    clock.advance(Duration::minutes(5));
    focus::resume(&state).expect("resume");
    clock.advance(Duration::minutes(10));
    focus::stop(&state).expect("stop");

    let summary = week_in(&state, "2026-03-10", 1, &zone()).expect("week");
    assert_eq!(summary.focus_seconds, 30 * 60);
    assert_eq!(summary.focus_sessions, 1);
    assert_eq!(summary.focus_by_day, vec![0, 1800, 0, 0, 0, 0, 0]);
}

use chrono::{Duration, FixedOffset};

use super::*;
use crate::state::test_state;

fn zone() -> FixedOffset {
    FixedOffset::east_opt(3600).expect("zone")
}

fn habit(state: &AppState, title: &str, days: Option<u8>) -> Habit {
    create_in(
        state,
        HabitInput {
            title: title.into(),
            days,
            ..HabitInput::default()
        },
        &zone(),
    )
    .expect("create")
}

fn mark(state: &AppState, id: &str, date: &str, done: bool) -> Habit {
    set_done(state, id, date, done, &zone()).expect("mark")
}

// The test clock starts on 2026-03-10 at 10:00 UTC, a Tuesday.

#[test]
fn a_new_habit_is_daily_and_starts_without_a_streak() {
    let (state, _) = test_state();
    let created = habit(&state, "  Leer  ", None);

    assert_eq!(created.title, "Leer");
    assert_eq!(created.days, 127);
    assert!(created.scheduled_today);
    assert!(!created.done_today);
    assert_eq!((created.current_streak, created.best_streak), (0, 0));
    assert_eq!(habit(&state, "Correr", None).position, 1);
}

#[test]
fn invalid_habits_are_rejected() {
    let (state, _) = test_state();
    for bad in [
        HabitInput {
            title: "   ".into(),
            ..HabitInput::default()
        },
        HabitInput {
            title: "Leer".into(),
            days: Some(0),
            ..HabitInput::default()
        },
        HabitInput {
            title: "Leer".into(),
            days: Some(128),
            ..HabitInput::default()
        },
        HabitInput {
            title: "Leer".into(),
            time: Some("25:00".into()),
            ..HabitInput::default()
        },
        HabitInput {
            title: "x".repeat(201),
            ..HabitInput::default()
        },
    ] {
        let error = create_in(&state, bad, &zone()).expect_err("rejected");
        assert!(error.to_string().starts_with("VALIDATION:"), "{error}");
    }
    let unknown_tag = create_in(
        &state,
        HabitInput {
            title: "Leer".into(),
            tag_id: Some("ghost".into()),
            ..HabitInput::default()
        },
        &zone(),
    )
    .expect_err("tag");
    assert!(unknown_tag.to_string().starts_with("NOT_FOUND:"));
}

#[test]
fn marking_days_builds_the_streak_and_unmarking_only_touches_that_habit() {
    let (state, _) = test_state();
    let reading = habit(&state, "Leer", None);
    let running = habit(&state, "Correr", None);

    mark(&state, &reading.id, "2026-03-08", true);
    mark(&state, &reading.id, "2026-03-09", true);
    mark(&state, &running.id, "2026-03-09", true);
    let today = mark(&state, &reading.id, "2026-03-10", true);
    assert!(today.done_today);
    assert_eq!((today.current_streak, today.best_streak), (3, 3));

    // Marking twice changes nothing.
    assert_eq!(
        mark(&state, &reading.id, "2026-03-10", true).current_streak,
        3
    );

    // Unmarking a day in the middle cuts the streak but keeps the best of the rest.
    let cut = mark(&state, &reading.id, "2026-03-09", false);
    assert_eq!((cut.current_streak, cut.best_streak), (1, 1));
    let other = list(&state, false)
        .expect("list")
        .into_iter()
        .find(|h| h.id == running.id)
        .expect("running");
    assert_eq!(other.current_streak, 1, "the other habit keeps its mark");
}

#[test]
fn the_streak_follows_the_clock() {
    let (state, clock) = test_state();
    let reading = habit(&state, "Leer", None);
    mark(&state, &reading.id, "2026-03-09", true);
    mark(&state, &reading.id, "2026-03-10", true);

    // Tomorrow is still to do: the streak holds.
    clock.advance(Duration::days(1));
    let tomorrow = &list_in(&state, false, &zone()).expect("list")[0];
    assert_eq!((tomorrow.current_streak, tomorrow.done_today), (2, false));

    // A whole day without marking breaks it, the best remains.
    clock.advance(Duration::days(1));
    let later = &list_in(&state, false, &zone()).expect("list")[0];
    assert_eq!((later.current_streak, later.best_streak), (0, 2));
}

#[test]
fn days_off_do_not_break_a_weekday_habit() {
    let (state, clock) = test_state();
    let weekdays = habit(&state, "Estudiar", Some(0b001_1111));
    // Friday 6th, then Monday 9th to Friday 13th, skipping the weekend between them.
    clock.set(dates::parse_instant("2026-03-13T10:00:00Z").expect("friday"));
    for day in [
        "2026-03-06",
        "2026-03-09",
        "2026-03-10",
        "2026-03-11",
        "2026-03-12",
    ] {
        mark(&state, &weekdays.id, day, true);
    }
    let friday = mark(&state, &weekdays.id, "2026-03-13", true);
    assert_eq!((friday.current_streak, friday.best_streak), (6, 6));

    // On a Saturday nothing is scheduled and the streak is kept.
    clock.set(dates::parse_instant("2026-03-14T10:00:00Z").expect("saturday"));
    let saturday = &list_in(&state, false, &zone()).expect("list")[0];
    assert!(!saturday.scheduled_today);
    assert_eq!(saturday.current_streak, 6);

    // Missing a Monday breaks it once the day is over.
    clock.set(dates::parse_instant("2026-03-17T10:00:00Z").expect("tuesday"));
    let tuesday = &list_in(&state, false, &zone()).expect("list")[0];
    assert_eq!((tuesday.current_streak, tuesday.best_streak), (0, 6));
}

#[test]
fn future_days_and_unknown_habits_are_rejected() {
    let (state, _) = test_state();
    let reading = habit(&state, "Leer", None);
    let future = set_done(&state, &reading.id, "2026-03-11", true, &zone()).expect_err("future");
    assert!(future.to_string().starts_with("VALIDATION:"));
    let missing = set_done(&state, "ghost", "2026-03-10", true, &zone()).expect_err("missing");
    assert!(missing.to_string().starts_with("NOT_FOUND:"));
    assert!(set_done(&state, &reading.id, "ayer", true, &zone()).is_err());
}

#[test]
fn update_archive_delete_and_restore() {
    let (state, _) = test_state();
    let reading = habit(&state, "Leer", None);
    mark(&state, &reading.id, "2026-03-10", true);

    let renamed = update_in(
        &state,
        &reading.id,
        HabitPatchInput {
            title: Some("Leer 20 min".into()),
            days: Some(0b000_0111),
            time: Some(Some("21:30".into())),
            ..HabitPatchInput::default()
        },
        &zone(),
    )
    .expect("update");
    assert_eq!(renamed.title, "Leer 20 min");
    assert_eq!(renamed.time.as_deref(), Some("21:30"));
    assert_eq!(renamed.days, 7);

    let cleared = update_in(
        &state,
        &reading.id,
        HabitPatchInput {
            time: Some(None),
            ..HabitPatchInput::default()
        },
        &zone(),
    )
    .expect("clear time");
    assert_eq!(cleared.time, None);
    assert_eq!(cleared.title, "Leer 20 min");

    let archived = update_in(
        &state,
        &reading.id,
        HabitPatchInput {
            archived: Some(true),
            ..HabitPatchInput::default()
        },
        &zone(),
    )
    .expect("archive");
    assert!(archived.is_archived);
    assert!(list(&state, false).expect("list").is_empty());
    assert_eq!(list(&state, true).expect("list").len(), 1);

    delete(&state, &reading.id).expect("delete");
    assert!(list(&state, true).expect("list").is_empty());
    assert!(delete(&state, &reading.id).is_err());
    assert!(
        logs(&state, "2026-03-01", "2026-03-31")
            .expect("logs")
            .is_empty()
    );

    let back = restore_in(&state, &reading.id, &zone()).expect("restore");
    assert_eq!(back.id, reading.id);
    assert_eq!(
        logs(&state, "2026-03-01", "2026-03-31").expect("logs"),
        [HabitLog {
            habit_id: reading.id.clone(),
            date: "2026-03-10".into()
        }]
    );
    assert!(restore_in(&state, &reading.id, &zone()).is_err());
}

#[test]
fn logs_are_limited_to_a_window() {
    let (state, _) = test_state();
    assert!(logs(&state, "2026-03-05", "2026-03-01").is_err());
    assert!(logs(&state, "2024-01-01", "2026-03-01").is_err());
}

#[test]
fn deleting_a_tag_keeps_the_habit_without_it() {
    let (state, _) = test_state();
    state
        .db
        .write(|tx| {
            tx.execute(
                "INSERT INTO tags (id, name, color_hex, position, created_at)
                 VALUES ('a', 'A', '#0A84FF', 0, 'x')",
                [],
            )?;
            Ok(())
        })
        .expect("tag");
    let tagged = create_in(
        &state,
        HabitInput {
            title: "Leer".into(),
            tag_id: Some("a".into()),
            ..HabitInput::default()
        },
        &zone(),
    )
    .expect("create");
    assert_eq!(tagged.tag_id.as_deref(), Some("a"));

    state
        .db
        .write(|tx| {
            tx.execute("DELETE FROM tags WHERE id = 'a'", [])?;
            Ok(())
        })
        .expect("delete tag");
    let found = &list(&state, false).expect("list")[0];
    assert_eq!(found.tag_id, None);
}

#[test]
fn trashed_habits_are_listed_and_purged_with_their_marks_after_thirty_days() {
    let (state, clock) = test_state();
    let old = habit(&state, "Antiguo", None);
    let recent = habit(&state, "Reciente", None);
    mark(&state, &old.id, "2026-03-10", true);
    delete(&state, &old.id).expect("trash old");
    clock.advance(Duration::days(20));
    delete(&state, &recent.id).expect("trash recent");

    let ids: Vec<String> = trash_list_in(&state, &zone())
        .expect("trash")
        .into_iter()
        .map(|h| h.id)
        .collect();
    assert_eq!(ids, [recent.id.clone(), old.id.clone()]);

    clock.advance(Duration::days(11));
    state.purge_trash().expect("purge");
    let ids: Vec<String> = trash_list_in(&state, &zone())
        .expect("trash")
        .into_iter()
        .map(|h| h.id)
        .collect();
    assert_eq!(ids, [recent.id]);
    let marks: i64 = state
        .db
        .read(|conn| Ok(conn.query_row("SELECT COUNT(*) FROM habit_logs", [], |r| r.get(0))?))
        .expect("count");
    assert_eq!(marks, 0);
}

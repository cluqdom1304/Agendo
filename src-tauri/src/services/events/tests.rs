use chrono::{Duration, FixedOffset};

use super::*;
use crate::state::test_state;

/// UTC+1, so a local 09:00 is 08:00 UTC.
fn zone() -> FixedOffset {
    FixedOffset::east_opt(3600).expect("zone")
}

fn input(start_date: &str, start_time: Option<&str>) -> EventInput {
    EventInput {
        title: Some("Clase".into()),
        start_date: start_date.into(),
        start_time: start_time.map(str::to_string),
        ..EventInput::default()
    }
}

fn make(state: &AppState, input: EventInput) -> Event {
    create_in(state, input, &zone()).expect("create")
}

fn waiting(state: &AppState, id: &str) -> Vec<(i64, String)> {
    state
        .db
        .read(|conn| {
            Ok(conn
                .prepare(
                    "SELECT offset_minutes, notify_at FROM event_reminders
                     WHERE event_id = ?1 AND is_sent = 0 ORDER BY offset_minutes",
                )?
                .query_map([id], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<rusqlite::Result<_>>()?)
        })
        .expect("rows")
}

fn dates_of(found: &[EventOccurrence]) -> Vec<&str> {
    found.iter().map(|o| o.start_date.as_str()).collect()
}

fn reminder_ids(state: &AppState) -> Vec<String> {
    state
        .db
        .read(|conn| {
            Ok(conn
                .prepare("SELECT id FROM event_reminders ORDER BY offset_minutes, id")?
                .query_map([], |r| r.get(0))?
                .collect::<rusqlite::Result<_>>()?)
        })
        .expect("ids")
}

#[test]
fn a_timed_event_defaults_to_an_hour_and_an_all_day_event_has_no_times() {
    let (state, _) = test_state();
    let timed = make(&state, input("2026-03-12", Some("09:30")));
    assert_eq!(timed.end_date, "2026-03-12");
    assert_eq!(timed.end_time.as_deref(), Some("10:30"));
    assert!(!timed.all_day);

    let late = make(&state, input("2026-03-12", Some("23:30")));
    assert_eq!(late.end_time.as_deref(), Some("23:59"));

    let whole_day = make(
        &state,
        EventInput {
            end_time: Some("11:00".into()),
            ..input("2026-03-12", None)
        },
    );
    assert!(whole_day.all_day);
    assert_eq!((whole_day.start_time, whole_day.end_time), (None, None));
}

#[test]
fn invalid_events_are_rejected_with_a_validation_error() {
    let (state, _) = test_state();
    let cases = [
        EventInput {
            end_date: Some("2026-03-11".into()),
            ..input("2026-03-12", None)
        },
        EventInput {
            end_time: Some("08:00".into()),
            ..input("2026-03-12", Some("09:00"))
        },
        EventInput {
            rrule: Some("FREQ=HOURLY".into()),
            ..input("2026-03-12", None)
        },
        EventInput {
            reminder_offsets: Some(vec![60]),
            ..input("2026-03-12", None)
        },
        EventInput {
            reminder_offsets: Some(vec![-5]),
            ..input("2026-03-12", Some("09:00"))
        },
        EventInput {
            reminder_offsets: Some((0..11).collect()),
            ..input("2026-03-12", Some("09:00"))
        },
        EventInput {
            title: Some("x".repeat(600)),
            ..input("2026-03-12", None)
        },
        input("12/03/2026", None),
    ];
    for case in cases {
        let error = create_in(&state, case.clone(), &zone()).expect_err("rejected");
        assert!(
            error.to_string().starts_with("VALIDATION:"),
            "{case:?} {error}"
        );
    }
    // A multi-day event may end earlier in the day than it started.
    let overnight = EventInput {
        end_date: Some("2026-03-13".into()),
        end_time: Some("08:00".into()),
        ..input("2026-03-12", Some("22:00"))
    };
    assert!(create_in(&state, overnight, &zone()).is_ok());
}

#[test]
fn list_returns_what_overlaps_the_window_in_calendar_order() {
    let (state, _) = test_state();
    make(&state, input("2026-03-20", Some("10:00")));
    make(&state, input("2026-03-20", None));
    make(&state, input("2026-03-12", Some("09:00")));
    make(
        &state,
        EventInput {
            end_date: Some("2026-03-14".into()),
            ..input("2026-03-09", None)
        },
    );
    make(&state, input("2026-04-01", None));

    let found = list(&state, "2026-03-11", "2026-03-31").expect("list");

    assert_eq!(
        dates_of(&found),
        ["2026-03-09", "2026-03-12", "2026-03-20", "2026-03-20"]
    );
    assert!(found[2].all_day, "all-day events come before timed ones");
    assert!(list(&state, "2026-03-05", "2026-03-01").is_err());
    assert!(list(&state, "2020-01-01", "2026-03-01").is_err());
}

#[test]
fn a_series_is_expanded_by_query_without_storing_occurrences() {
    let (state, _) = test_state();
    let event = make(
        &state,
        EventInput {
            rrule: Some("FREQ=WEEKLY;INTERVAL=1".into()),
            ..input("2026-03-02", Some("09:00"))
        },
    );
    assert_eq!(event.rrule.as_deref(), Some("FREQ=WEEKLY;INTERVAL=1"));

    let found = list(&state, "2026-03-10", "2026-03-31").expect("list");
    assert_eq!(dates_of(&found), ["2026-03-16", "2026-03-23", "2026-03-30"]);
    assert!(found.iter().all(|o| o.recurring && o.event_id == event.id));
    let rows: i64 = state
        .db
        .read(|conn| Ok(conn.query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))?))
        .expect("count");
    assert_eq!(rows, 1);

    let counted = make(
        &state,
        EventInput {
            rrule: Some("FREQ=DAILY;INTERVAL=1;COUNT=2".into()),
            ..input("2026-05-01", None)
        },
    );
    let found = list(&state, "2026-05-01", "2026-05-31").expect("list");
    assert_eq!(found.iter().filter(|o| o.event_id == counted.id).count(), 2);
}

#[test]
fn a_repetition_that_began_before_the_window_shows_if_it_lasts_into_it() {
    let (state, _) = test_state();
    make(
        &state,
        EventInput {
            end_date: Some("2026-03-04".into()),
            rrule: Some("FREQ=WEEKLY;INTERVAL=1".into()),
            ..input("2026-03-02", None)
        },
    );
    // The repetition of 9 to 11 March is under way on the 10th.
    let found = list(&state, "2026-03-10", "2026-03-10").expect("list");
    assert_eq!(dates_of(&found), ["2026-03-09"]);
    assert_eq!(found[0].end_date, "2026-03-11");
}

#[test]
fn reminders_are_computed_from_the_local_start_and_wait_for_the_first_future_one() {
    let (state, _) = test_state();
    let event = make(
        &state,
        EventInput {
            reminder_offsets: Some(vec![15, 0, 15]),
            ..input("2026-03-12", Some("09:00"))
        },
    );
    assert_eq!(event.reminder_offsets, [0, 15]);
    assert_eq!(
        waiting(&state, &event.id),
        [
            (0, "2026-03-12T08:00:00Z".to_string()),
            (15, "2026-03-12T07:45:00Z".to_string())
        ]
    );

    // All-day events warn at the notification time of whole days before.
    let day = make(
        &state,
        EventInput {
            reminder_offsets: Some(vec![1440]),
            ..input("2026-03-15", None)
        },
    );
    assert_eq!(
        waiting(&state, &day.id),
        [(1440, "2026-03-14T08:00:00Z".to_string())]
    );

    // Past reminders are not stored.
    let soon = make(
        &state,
        EventInput {
            reminder_offsets: Some(vec![120]),
            ..input("2026-03-10", Some("11:30"))
        },
    );
    assert!(waiting(&state, &soon.id).is_empty());
}

#[test]
fn a_series_warns_for_the_next_occurrence_and_continues_after_each_one() {
    let (state, clock) = test_state();
    let event = make(
        &state,
        EventInput {
            reminder_offsets: Some(vec![30]),
            rrule: Some("FREQ=WEEKLY;INTERVAL=1".into()),
            ..input("2026-03-02", Some("09:00"))
        },
    );
    // The series began last week: the first warning is for the 16th.
    assert_eq!(
        waiting(&state, &event.id),
        [(30, "2026-03-16T07:30:00Z".to_string())]
    );

    clock.set(dates::parse_instant("2026-03-16T07:30:00Z").expect("instant"));
    let now = state.clock.now();
    let id = reminder_ids(&state).remove(0);
    state
        .db
        .write(|tx| {
            crate::repo::reminders::mark_sent(tx, &id, &dates::format_instant(now))?;
            continue_after_reminder(&state, tx, &id, now, &zone())
        })
        .expect("continue");

    assert_eq!(
        waiting(&state, &event.id),
        [(30, "2026-03-23T07:30:00Z".to_string())]
    );
}

#[test]
fn continuing_after_one_reminder_keeps_the_others_that_are_waiting() {
    let (state, clock) = test_state();
    let event = make(
        &state,
        EventInput {
            reminder_offsets: Some(vec![0, 10]),
            rrule: Some("FREQ=DAILY;INTERVAL=1".into()),
            ..input("2026-03-12", Some("09:00"))
        },
    );
    clock.set(dates::parse_instant("2026-03-12T08:00:00Z").expect("instant"));
    let now = state.clock.now();
    let ids = reminder_ids(&state);
    state
        .db
        .write(|tx| {
            crate::repo::reminders::mark_sent(tx, &ids[0], &dates::format_instant(now))?;
            continue_after_reminder(&state, tx, &ids[0], now, &zone())
        })
        .expect("continue");

    let rows = waiting(&state, &event.id);
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert!(rows.contains(&(0, "2026-03-13T08:00:00Z".to_string())));
    assert!(
        rows.iter().any(|(offset, _)| *offset == 10),
        "the 10-minute warning stays: {rows:?}"
    );
}

#[test]
fn update_moves_the_event_keeps_its_length_and_recalculates_reminders() {
    let (state, _) = test_state();
    let event = make(
        &state,
        EventInput {
            end_time: Some("10:30".into()),
            reminder_offsets: Some(vec![15]),
            ..input("2026-03-12", Some("09:00"))
        },
    );

    let moved = update_in(
        &state,
        &event.id,
        EventPatchInput {
            start_date: Some("2026-03-14".into()),
            start_time: Some(Some("14:00".into())),
            ..EventPatchInput::default()
        },
        &zone(),
    )
    .expect("update");

    assert_eq!(moved.start_date, "2026-03-14");
    assert_eq!(moved.end_date, "2026-03-14");
    assert_eq!(moved.end_time.as_deref(), Some("15:30"));
    assert_eq!(
        waiting(&state, &event.id),
        [(15, "2026-03-14T12:45:00Z".to_string())]
    );

    let whole_day = update_in(
        &state,
        &event.id,
        EventPatchInput {
            start_time: Some(None),
            reminder_offsets: Some(vec![]),
            ..EventPatchInput::default()
        },
        &zone(),
    )
    .expect("all day");
    assert!(whole_day.all_day);
    assert_eq!(whole_day.end_time, None);
    assert!(waiting(&state, &event.id).is_empty());

    let error =
        update_in(&state, "ghost", EventPatchInput::default(), &zone()).expect_err("unknown");
    assert!(error.to_string().starts_with("NOT_FOUND:"));
}

#[test]
fn deleting_clears_the_warnings_and_restoring_brings_them_back() {
    let (state, clock) = test_state();
    let event = make(
        &state,
        EventInput {
            reminder_offsets: Some(vec![15]),
            ..input("2026-03-12", Some("09:00"))
        },
    );
    delete(&state, &event.id).expect("delete");
    assert!(waiting(&state, &event.id).is_empty());
    assert!(get(&state, &event.id).is_err());
    assert!(
        list(&state, "2026-03-01", "2026-03-31")
            .expect("list")
            .is_empty()
    );
    assert!(delete(&state, &event.id).is_err());

    clock.advance(Duration::hours(1));
    let back = restore_in(&state, &event.id, &zone()).expect("restore");
    assert_eq!(back.id, event.id);
    assert_eq!(waiting(&state, &event.id).len(), 1);
    assert!(restore_in(&state, &event.id, &zone()).is_err());
}

#[test]
fn tags_are_linked_and_replaced() {
    let (state, _) = test_state();
    state
        .db
        .write(|tx| {
            tx.execute_batch(
                "INSERT INTO tags (id, name, color_hex, position, created_at)
                 VALUES ('a', 'A', '#0A84FF', 0, 'x'), ('b', 'B', '#0A84FF', 1, 'x');",
            )?;
            Ok(())
        })
        .expect("tags");
    let event = make(
        &state,
        EventInput {
            tag_ids: Some(vec!["a".into()]),
            ..input("2026-03-12", None)
        },
    );
    assert_eq!(event.tag_ids, ["a"]);
    let updated = update_in(
        &state,
        &event.id,
        EventPatchInput {
            tag_ids: Some(vec!["b".into()]),
            ..EventPatchInput::default()
        },
        &zone(),
    )
    .expect("update");
    assert_eq!(updated.tag_ids, ["b"]);
}

#[test]
fn trashed_events_are_listed_and_purged_after_thirty_days() {
    let (state, clock) = test_state();
    let kept = make(&state, input("2026-03-12", None));
    let gone = make(&state, input("2026-03-13", None));
    let old = make(&state, input("2026-03-14", Some("09:00")));
    delete(&state, &old.id).expect("trash old");
    clock.advance(Duration::days(20));
    delete(&state, &gone.id).expect("trash recent");

    let trash: Vec<String> = trash_list(&state)
        .expect("trash")
        .into_iter()
        .map(|event| event.id)
        .collect();
    assert_eq!(trash, [gone.id.clone(), old.id.clone()], "latest first");

    clock.advance(Duration::days(11));
    state.purge_trash().expect("purge");
    let left: Vec<String> = trash_list(&state)
        .expect("trash")
        .into_iter()
        .map(|event| event.id)
        .collect();
    assert_eq!(
        left,
        [gone.id],
        "only the one trashed 31 days ago is purged"
    );
    assert!(get(&state, &kept.id).is_ok());
}

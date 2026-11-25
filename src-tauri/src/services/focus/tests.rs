use std::sync::Mutex;

use chrono::FixedOffset;

use super::*;
use crate::state::test_state;

#[derive(Default)]
struct FakeNotifier {
    shown: Mutex<Vec<Notification>>,
    fail: bool,
}

impl Notifier for FakeNotifier {
    fn show(&self, notification: &Notification) -> Result<(), AppError> {
        if self.fail {
            return Err(AppError::Notify("sin servicio".into()));
        }
        self.shown.lock().expect("lock").push(notification.clone());
        Ok(())
    }
}

fn zone() -> FixedOffset {
    FixedOffset::east_opt(3600).expect("zone")
}

fn task(state: &AppState, title: &str) -> String {
    crate::services::tasks::create(
        state,
        crate::services::tasks::TaskCreateInput {
            title: Some(title.into()),
            ..Default::default()
        },
    )
    .expect("task")
    .id
}

// The test clock starts on 2026-03-10 at 10:00 UTC.

#[test]
fn a_session_starts_with_the_backend_clock_and_only_one_runs_at_a_time() {
    let (state, _) = test_state();
    let id = task(&state, "Estudiar");

    let entry = start(&state, &id, 25).expect("start");
    assert_eq!(entry.started_at, "2026-03-10T10:00:00Z");
    assert_eq!((entry.ended_at, entry.planned_minutes), (None, 25));
    assert_eq!(
        current(&state).expect("current").map(|e| e.id),
        Some(entry.id)
    );

    let again = start(&state, &id, 10).expect_err("already running");
    assert!(again.to_string().starts_with("CONFLICT:"), "{again}");
}

#[test]
fn start_validates_the_length_and_the_task() {
    let (state, _) = test_state();
    let id = task(&state, "Estudiar");
    for minutes in [0, -5, 601] {
        let error = start(&state, &id, minutes).expect_err("length");
        assert!(error.to_string().starts_with("VALIDATION:"), "{error}");
    }
    let error = start(&state, "ghost", 25).expect_err("task");
    assert!(error.to_string().starts_with("NOT_FOUND:"));
    crate::services::tasks::delete(&state, &id).expect("trash");
    assert!(start(&state, &id, 25).is_err());
}

#[test]
fn stopping_ends_it_now_and_allows_a_new_one() {
    let (state, clock) = test_state();
    let id = task(&state, "Estudiar");
    start(&state, &id, 25).expect("start");

    clock.advance(Duration::minutes(7));
    let stopped = stop(&state).expect("stop").expect("entry");
    assert_eq!(stopped.ended_at.as_deref(), Some("2026-03-10T10:07:00Z"));
    assert_eq!(current(&state).expect("current"), None);
    assert_eq!(stop(&state).expect("stop twice"), None);

    assert!(start(&state, &id, 15).is_ok());
}

#[test]
fn the_scheduler_ends_a_session_at_its_planned_time_and_notifies_once() {
    let (state, clock) = test_state();
    let notifier = FakeNotifier::default();
    let id = task(&state, "Estudiar");
    let entry = start(&state, &id, 25).expect("start");
    assert_eq!(
        next_end(&state).expect("end").map(dates::format_instant),
        Some("2026-03-10T10:25:00Z".to_string())
    );

    clock.advance(Duration::minutes(24));
    assert_eq!(finish_due(&state, &notifier).expect("early"), None);
    assert!(notifier.shown.lock().expect("lock").is_empty());

    // A late tick (the computer slept) still ends it at the planned time, not at the late one.
    clock.advance(Duration::minutes(40));
    let closed = finish_due(&state, &notifier).expect("due").expect("closed");
    assert_eq!(closed.id, entry.id);
    assert_eq!(closed.ended_at.as_deref(), Some("2026-03-10T10:25:00Z"));
    let shown = notifier.shown.lock().expect("lock");
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].body, "25 min · Estudiar");
    drop(shown);

    assert_eq!(finish_due(&state, &notifier).expect("again"), None);
    assert_eq!(notifier.shown.lock().expect("lock").len(), 1);
    assert_eq!(next_end(&state).expect("end"), None);
}

#[test]
fn a_failing_notification_does_not_leave_the_session_open() {
    let (state, clock) = test_state();
    let notifier = FakeNotifier {
        fail: true,
        ..FakeNotifier::default()
    };
    let id = task(&state, "Estudiar");
    start(&state, &id, 5).expect("start");
    clock.advance(Duration::minutes(5));

    assert!(finish_due(&state, &notifier).expect("due").is_some());
    assert_eq!(current(&state).expect("current"), None);
}

#[test]
fn an_orphan_session_is_closed_at_start_up_capped_at_its_planned_time() {
    let (state, clock) = test_state();
    let id = task(&state, "Estudiar");
    start(&state, &id, 25).expect("start");

    // The app was closed after 10 minutes and opened 3 hours later.
    clock.advance(Duration::hours(3));
    let closed = recover_orphans(&state).expect("recover").expect("closed");
    assert_eq!(closed.ended_at.as_deref(), Some("2026-03-10T10:25:00Z"));
    assert_eq!(recover_orphans(&state).expect("none"), None);

    // Reopened before the planned end, it ends where it was.
    start(&state, &id, 60).expect("start");
    clock.advance(Duration::minutes(12));
    let closed = recover_orphans(&state).expect("recover").expect("closed");
    assert_eq!(closed.ended_at.as_deref(), Some("2026-03-10T13:12:00Z"));
}

#[test]
fn list_returns_the_sessions_of_the_local_days_asked_for() {
    let (state, clock) = test_state();
    let id = task(&state, "Estudiar");
    start(&state, &id, 25).expect("start"); // 10 March, 11:00 local
    stop(&state).expect("stop");
    clock.advance(Duration::days(1));
    start(&state, &id, 25).expect("start"); // 11 March
    stop(&state).expect("stop");
    clock.set(dates::parse_instant("2026-03-11T23:30:00Z").expect("late"));
    start(&state, &id, 25).expect("start"); // 12 March 00:30 local
    stop(&state).expect("stop");

    let eleventh = list(&state, "2026-03-11", "2026-03-11", &zone()).expect("list");
    assert_eq!(eleventh.len(), 1);
    assert_eq!(eleventh[0].started_at, "2026-03-11T10:00:00Z");
    assert_eq!(
        list(&state, "2026-03-10", "2026-03-12", &zone())
            .expect("list")
            .len(),
        3
    );
    assert_eq!(
        list(&state, "2026-03-12", "2026-03-12", &zone())
            .expect("list")
            .len(),
        1
    );
    assert!(list(&state, "2026-03-12", "2026-03-10", &zone()).is_err());
    assert!(list(&state, "2024-01-01", "2026-03-10", &zone()).is_err());
}

#[test]
fn list_includes_a_session_that_crosses_midnight_into_the_range() {
    let (state, clock) = test_state();
    let id = task(&state, "Estudiar");
    clock.set(dates::parse_instant("2026-03-10T22:50:00Z").expect("late")); // 23:50 local
    start(&state, &id, 25).expect("start");

    // Still running: it may already reach the next day.
    let running = list(&state, "2026-03-11", "2026-03-11", &zone()).expect("list");
    assert_eq!(running.len(), 1);

    clock.advance(Duration::minutes(25));
    stop(&state).expect("stop");
    for day in ["2026-03-10", "2026-03-11"] {
        let found = list(&state, day, day, &zone()).expect("list");
        assert_eq!(found.len(), 1, "{day}");
    }
    let after = list(&state, "2026-03-12", "2026-03-12", &zone()).expect("list");
    assert!(after.is_empty());
}

#[test]
fn deleting_the_task_for_good_removes_its_sessions() {
    let (state, _) = test_state();
    let id = task(&state, "Estudiar");
    start(&state, &id, 25).expect("start");
    stop(&state).expect("stop");

    state
        .db
        .write(|tx| {
            tx.execute("DELETE FROM tasks WHERE id = ?1", [&id])?;
            Ok(())
        })
        .expect("delete");

    let rows: i64 = state
        .db
        .read(|conn| Ok(conn.query_row("SELECT COUNT(*) FROM time_entries", [], |r| r.get(0))?))
        .expect("count");
    assert_eq!(rows, 0);
}

#[test]
fn totals_add_closed_sessions_and_manual_entries_per_task() {
    let (state, _) = test_state();
    let a = task(&state, "A");
    let b = task(&state, "B");

    let manual = add_manual(&state, &a, 40).expect("manual");
    assert_eq!(manual.started_at, "2026-03-10T09:20:00Z");
    assert_eq!(manual.ended_at.as_deref(), Some("2026-03-10T10:00:00Z"));

    start(&state, &a, 25).expect("start");
    // A running session does not count yet.
    assert_eq!(
        totals(&state).expect("totals"),
        vec![TaskFocusTotal {
            task_id: a.clone(),
            seconds: 2400
        }]
    );
    stop(&state).expect("stop");
    add_manual(&state, &b, 5).expect("manual b");

    let mut all = totals(&state).expect("totals");
    all.sort_by(|x, y| x.task_id.cmp(&y.task_id));
    let seconds: i64 = all.iter().map(|t| t.seconds).sum();
    assert_eq!(all.len(), 2);
    assert_eq!(seconds, 2700);
    assert_eq!(of_task(&state, &b).expect("of task").len(), 1);
}

#[test]
fn manual_entries_are_validated_and_can_be_deleted() {
    let (state, _) = test_state();
    let id = task(&state, "A");
    assert!(
        add_manual(&state, &id, 0)
            .expect_err("zero")
            .to_string()
            .starts_with("VALIDATION:")
    );
    assert!(add_manual(&state, &id, 601).is_err());
    assert!(
        add_manual(&state, "nope", 10)
            .expect_err("task")
            .to_string()
            .starts_with("NOT_FOUND:")
    );

    let entry = add_manual(&state, &id, 10).expect("manual");
    delete_entry(&state, &entry.id).expect("delete");
    assert!(totals(&state).expect("totals").is_empty());
    assert!(
        delete_entry(&state, &entry.id)
            .expect_err("twice")
            .to_string()
            .starts_with("NOT_FOUND:")
    );

    let running = start(&state, &id, 25).expect("start");
    assert!(delete_entry(&state, &running.id).is_err());
}

#[test]
fn a_session_of_seconds_is_recorded_with_its_exact_range() {
    let (state, clock) = test_state();
    let id = task(&state, "A");
    start(&state, &id, 25).expect("start");
    clock.advance(Duration::seconds(23));
    let closed = stop(&state).expect("stop").expect("closed");
    assert_eq!(closed.started_at, "2026-03-10T10:00:00Z");
    assert_eq!(closed.ended_at.as_deref(), Some("2026-03-10T10:00:23Z"));
    assert_eq!(closed.spent_seconds, 23);
    assert_eq!(totals(&state).expect("totals")[0].seconds, 23);
    assert_eq!(of_task(&state, &id).expect("of task").len(), 1);
}

#[test]
fn a_pause_does_not_count_and_delays_the_end() {
    let (state, clock) = test_state();
    let id = task(&state, "A");
    let notifier = FakeNotifier::default();
    start(&state, &id, 10).expect("start");
    clock.advance(Duration::minutes(4));
    let paused = pause(&state).expect("pause");
    assert_eq!(paused.paused_at.as_deref(), Some("2026-03-10T10:04:00Z"));
    assert!(pause(&state).is_err());
    assert_eq!(next_end(&state).expect("next"), None);

    // Nothing ends while it is paused, however long it lasts.
    clock.advance(Duration::minutes(30));
    assert!(finish_due(&state, &notifier).expect("due").is_none());

    let resumed = resume(&state).expect("resume");
    assert_eq!((resumed.paused_seconds, resumed.paused_at), (1800, None));
    assert!(resume(&state).is_err());
    // 6 minutes were left; the end is 6 minutes after resuming.
    clock.advance(Duration::minutes(5));
    assert!(finish_due(&state, &notifier).expect("due").is_none());
    clock.advance(Duration::minutes(1));
    let closed = finish_due(&state, &notifier).expect("due").expect("closed");
    assert_eq!(closed.spent_seconds, 600);
    assert_eq!(closed.ended_at.as_deref(), Some("2026-03-10T10:40:00Z"));
}

#[test]
fn stopping_while_paused_ends_where_the_pause_began() {
    let (state, clock) = test_state();
    let id = task(&state, "A");
    start(&state, &id, 25).expect("start");
    clock.advance(Duration::seconds(90));
    pause(&state).expect("pause");
    clock.advance(Duration::minutes(10));
    let closed = stop(&state).expect("stop").expect("closed");
    assert_eq!(closed.ended_at.as_deref(), Some("2026-03-10T10:01:30Z"));
    assert_eq!(closed.spent_seconds, 90);
}

#[test]
fn discarding_removes_the_session_without_keeping_time() {
    let (state, clock) = test_state();
    let id = task(&state, "A");
    start(&state, &id, 25).expect("start");
    clock.advance(Duration::minutes(3));
    discard(&state).expect("discard");
    assert!(current(&state).expect("current").is_none());
    assert!(totals(&state).expect("totals").is_empty());
    assert!(discard(&state).is_err());
}

#[test]
fn switching_keeps_the_time_on_the_old_task_and_carries_the_rest() {
    let (state, clock) = test_state();
    let a = task(&state, "A");
    let b = task(&state, "B");
    start(&state, &a, 25).expect("start");
    clock.advance(Duration::minutes(10));
    let next = switch_task(&state, &b).expect("switch");
    assert_eq!(
        (next.task_id.as_str(), next.planned_minutes),
        (b.as_str(), 15)
    );
    assert_eq!(next.started_at, "2026-03-10T10:10:00Z");
    assert_eq!(
        totals(&state).expect("totals"),
        vec![TaskFocusTotal {
            task_id: a.clone(),
            seconds: 600
        }]
    );

    assert!(switch_task(&state, &b).is_err());
    assert!(
        switch_task(&state, "nope")
            .expect_err("task")
            .to_string()
            .starts_with("NOT_FOUND:")
    );
}

#[test]
fn an_orphan_session_in_pause_is_closed_where_it_was_paused() {
    let (state, clock) = test_state();
    let id = task(&state, "A");
    start(&state, &id, 25).expect("start");
    clock.advance(Duration::minutes(2));
    pause(&state).expect("pause");
    clock.advance(Duration::hours(5));
    let closed = recover_orphans(&state).expect("recover").expect("closed");
    assert_eq!(closed.spent_seconds, 120);
}

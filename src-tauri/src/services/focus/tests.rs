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

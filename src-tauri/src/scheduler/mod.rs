//! Reminder scheduling (SPEC §6, D-04, D-26).
//!
//! [`Engine::tick`] does one pass: it expires reminders that are more than 24 hours overdue,
//! then fires the rest that are due. [`Engine::wait`] says how long to sleep afterwards.
//! [`run`] is the loop around both. Everything takes a [`Clock`], so tests move time by hand.

pub mod clock;

use std::{collections::HashMap, sync::Arc, time::Duration as StdDuration};

use chrono::{DateTime, Duration, Utc};

use crate::{
    domain::{dates, reminders as rules},
    error::AppResult,
    notify::{Fired, FiredSink, Notification, Notifier},
    repo::reminders,
    state::AppState,
};

/// The sleep never exceeds this, so clock jumps, suspension and time-zone changes are noticed
/// without listening to system events (SPEC §6.1).
pub const MAX_SLEEP: Duration = Duration::seconds(60);

/// Delivery attempts before a reminder is given up and marked as sent (SPEC §6.3).
pub const MAX_ATTEMPTS: u32 = 5;

/// Seconds to wait after the `attempt`-th failure (1-based): 30, 60, 120, 120.
pub fn backoff_seconds(attempt: u32) -> i64 {
    30 * (1 << attempt.saturating_sub(1).min(2))
}

#[derive(Debug, Clone, Copy)]
struct Retry {
    attempts: u32,
    not_before: DateTime<Utc>,
}

/// What one tick did, for tests and the log.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TickReport {
    pub fired: Vec<String>,
    pub expired: Vec<String>,
    pub failed: Vec<String>,
    pub given_up: Vec<String>,
}

/// In-memory retry state; the database holds everything else.
#[derive(Debug, Default)]
pub struct Engine {
    retries: HashMap<String, Retry>,
}

fn log(message: &str) {
    eprintln!("[scheduler] {message}");
}

impl Engine {
    pub fn tick(
        &mut self,
        state: &AppState,
        notifier: &dyn Notifier,
        sink: &dyn FiredSink,
    ) -> AppResult<TickReport> {
        let now = state.clock.now();
        let now_text = dates::format_instant(now);
        let mut report = TickReport::default();

        let cutoff = dates::format_instant(rules::stale_cutoff(now));
        report.expired = state
            .db
            .write(|tx| Ok(reminders::expire_before(tx, &cutoff, &now_text)?))?;
        for id in &report.expired {
            log(&format!(
                "recordatorio {id} caducado sin avisar (más de 24 h)"
            ));
        }

        let due = state.db.read(|conn| Ok(reminders::due(conn, &now_text)?))?;
        self.retries
            .retain(|id, _| due.iter().any(|reminder| &reminder.id == id));

        for reminder in due {
            if self
                .retries
                .get(&reminder.id)
                .is_some_and(|retry| retry.not_before > now)
            {
                continue;
            }
            let notification = Notification::for_reminder(&reminder);
            // The reminder is marked and notified in one transaction: if the notification fails
            // (or the process dies) the mark is rolled back and it will be tried again.
            let delivery = state.db.write(|tx| {
                reminders::mark_sent(tx, &reminder.id, &now_text)?;
                notifier.show(&notification)
            });
            match delivery {
                Ok(()) => {
                    self.retries.remove(&reminder.id);
                    sink.fired(&Fired {
                        task_id: reminder.task_id.clone(),
                        reminder_id: reminder.id.clone(),
                    });
                    report.fired.push(reminder.id);
                }
                Err(error) => {
                    let attempts = self.retries.get(&reminder.id).map_or(0, |r| r.attempts) + 1;
                    if attempts >= MAX_ATTEMPTS {
                        log(&format!(
                            "recordatorio {} descartado tras {attempts} intentos: {error}",
                            reminder.id
                        ));
                        self.retries.remove(&reminder.id);
                        state
                            .db
                            .write(|tx| Ok(reminders::mark_sent(tx, &reminder.id, &now_text)?))?;
                        report.given_up.push(reminder.id);
                    } else {
                        log(&format!(
                            "recordatorio {} falló (intento {attempts}): {error}",
                            reminder.id
                        ));
                        self.retries.insert(
                            reminder.id.clone(),
                            Retry {
                                attempts,
                                not_before: now + Duration::seconds(backoff_seconds(attempts)),
                            },
                        );
                        report.failed.push(reminder.id);
                    }
                }
            }
        }
        Ok(report)
    }

    /// How long to sleep after a tick: until the next reminder or retry, at most [`MAX_SLEEP`].
    pub fn wait(&self, state: &AppState) -> AppResult<StdDuration> {
        let now = state.clock.now();
        let mut wake = now + MAX_SLEEP;
        let next = state
            .db
            .read(|conn| Ok(reminders::next_after(conn, &dates::format_instant(now))?))?;
        if let Some(instant) = next.and_then(|text| dates::parse_instant(&text).ok()) {
            wake = wake.min(instant);
        }
        if let Some(retry) = self.retries.values().map(|retry| retry.not_before).min() {
            wake = wake.min(retry);
        }
        Ok((wake - now).to_std().unwrap_or(StdDuration::ZERO))
    }
}

/// The scheduler loop (SPEC §6.1): tick, then sleep until the next reminder, the 60 s cap or a
/// wake-up from `state.wake`, whichever comes first.
pub async fn run(state: Arc<AppState>, notifier: Arc<dyn Notifier>, sink: Arc<dyn FiredSink>) {
    let mut engine = Engine::default();
    loop {
        let pass = {
            let state = Arc::clone(&state);
            let notifier = Arc::clone(&notifier);
            let sink = Arc::clone(&sink);
            tokio::task::spawn_blocking(move || {
                if let Err(error) = engine.tick(&state, notifier.as_ref(), sink.as_ref()) {
                    log(&format!("error en la vuelta: {error}"));
                }
                let wait = engine.wait(&state).unwrap_or_else(|error| {
                    log(&format!("no se pudo calcular la espera: {error}"));
                    StdDuration::from_secs(60)
                });
                (engine, wait)
            })
            .await
        };
        let Ok((returned, wait)) = pass else {
            log("la vuelta se interrumpió; el scheduler se detiene");
            return;
        };
        engine = returned;
        tokio::select! {
            () = tokio::time::sleep(wait) => {}
            () = state.wake.notified() => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::FixedOffset;

    use super::*;
    use crate::{
        error::AppError,
        repo::{model::NewTask, reminders::NewReminder, tasks},
        state::test_state,
    };

    #[derive(Default)]
    struct FakeNotifier {
        shown: Mutex<Vec<Notification>>,
        failures_left: Mutex<u32>,
    }

    impl FakeNotifier {
        fn failing(times: u32) -> Self {
            Self {
                shown: Mutex::default(),
                failures_left: Mutex::new(times),
            }
        }
        fn titles(&self) -> Vec<String> {
            self.shown
                .lock()
                .expect("lock")
                .iter()
                .map(|n| n.title.clone())
                .collect()
        }
    }

    impl Notifier for FakeNotifier {
        fn show(&self, notification: &Notification) -> Result<(), AppError> {
            let mut left = self.failures_left.lock().expect("lock");
            if *left > 0 {
                *left -= 1;
                return Err(AppError::Notify("sin servicio".into()));
            }
            self.shown.lock().expect("lock").push(notification.clone());
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeSink(Mutex<Vec<Fired>>);

    impl FiredSink for FakeSink {
        fn fired(&self, fired: &Fired) {
            self.0.lock().expect("lock").push(fired.clone());
        }
    }

    fn add_task(state: &AppState, id: &str, title: &str) {
        let now = state.clock.now_string();
        state
            .db
            .write(|tx| {
                Ok(tasks::create(
                    tx,
                    &NewTask {
                        id: id.into(),
                        title: title.into(),
                        now,
                        ..NewTask::default()
                    },
                )?)
            })
            .expect("task");
    }

    fn add_reminder(state: &AppState, id: &str, task: &str, notify_at: &str) {
        let now = state.clock.now_string();
        state
            .db
            .write(|tx| {
                Ok(reminders::replace_unsent(
                    tx,
                    task,
                    &[NewReminder {
                        id: id.into(),
                        notify_at: notify_at.into(),
                        offset_minutes: None,
                    }],
                    &now,
                )?)
            })
            .expect("reminder");
    }

    fn is_sent(state: &AppState, id: &str) -> bool {
        state
            .db
            .read(|conn| {
                Ok(
                    conn.query_row("SELECT is_sent FROM reminders WHERE id = ?1", [id], |r| {
                        r.get(0)
                    })?,
                )
            })
            .expect("row")
    }

    fn wait_seconds(engine: &Engine, state: &AppState) -> u64 {
        engine.wait(state).expect("wait").as_secs()
    }

    #[test]
    fn fires_exactly_when_the_time_comes_and_only_once() {
        let (state, clock) = test_state();
        let (notifier, sink) = (FakeNotifier::default(), FakeSink::default());
        add_task(&state, "t1", "Entregar informe");
        add_reminder(&state, "r1", "t1", "2026-03-10T10:30:00Z");
        let mut engine = Engine::default();

        clock.advance(Duration::minutes(29) + Duration::seconds(59));
        assert!(
            engine
                .tick(&state, &notifier, &sink)
                .expect("tick")
                .fired
                .is_empty()
        );

        clock.advance(Duration::seconds(1));
        let report = engine.tick(&state, &notifier, &sink).expect("tick");
        assert_eq!(report.fired, ["r1"]);
        assert_eq!(notifier.titles(), ["Entregar informe"]);
        assert!(is_sent(&state, "r1"));

        clock.advance(Duration::hours(1));
        assert!(
            engine
                .tick(&state, &notifier, &sink)
                .expect("tick")
                .fired
                .is_empty()
        );
        assert_eq!(notifier.titles().len(), 1);
        assert_eq!(
            *sink.0.lock().expect("lock"),
            [Fired {
                task_id: "t1".into(),
                reminder_id: "r1".into()
            }]
        );
    }

    #[test]
    fn a_failed_notification_is_rolled_back_and_retried_with_backoff() {
        let (state, clock) = test_state();
        let (notifier, sink) = (FakeNotifier::failing(2), FakeSink::default());
        add_task(&state, "t1", "A");
        add_reminder(&state, "r1", "t1", "2026-03-10T10:00:00Z");
        let mut engine = Engine::default();

        let first = engine.tick(&state, &notifier, &sink).expect("tick");
        assert_eq!(first.failed, ["r1"]);
        assert!(!is_sent(&state, "r1"), "the mark is rolled back");
        assert!(sink.0.lock().expect("lock").is_empty());
        assert_eq!(wait_seconds(&engine, &state), 30);

        clock.advance(Duration::seconds(29));
        let early = engine.tick(&state, &notifier, &sink).expect("tick");
        assert_eq!(
            early,
            TickReport::default(),
            "no retry before the backoff ends"
        );

        clock.advance(Duration::seconds(1));
        assert_eq!(
            engine.tick(&state, &notifier, &sink).expect("tick").failed,
            ["r1"]
        );
        assert_eq!(wait_seconds(&engine, &state), 60);

        clock.advance(Duration::seconds(60));
        let done = engine.tick(&state, &notifier, &sink).expect("tick");
        assert_eq!(done.fired, ["r1"]);
        assert!(is_sent(&state, "r1"));
        assert_eq!(notifier.titles(), ["A"]);
    }

    #[test]
    fn it_gives_up_after_five_attempts_and_marks_the_reminder_as_sent() {
        let (state, clock) = test_state();
        let (notifier, sink) = (FakeNotifier::failing(100), FakeSink::default());
        add_task(&state, "t1", "A");
        add_reminder(&state, "r1", "t1", "2026-03-10T10:00:00Z");
        let mut engine = Engine::default();

        let mut last = TickReport::default();
        for _ in 0..MAX_ATTEMPTS {
            last = engine.tick(&state, &notifier, &sink).expect("tick");
            clock.advance(Duration::seconds(120));
        }

        assert_eq!(last.given_up, ["r1"]);
        assert!(is_sent(&state, "r1"));
        assert!(notifier.titles().is_empty());
        assert_eq!(
            engine.tick(&state, &notifier, &sink).expect("tick"),
            TickReport::default()
        );
    }

    #[test]
    fn backoff_follows_the_specification() {
        let waits: Vec<i64> = (1..=4).map(backoff_seconds).collect();
        assert_eq!(waits, [30, 60, 120, 120]);
    }

    #[test]
    fn reminders_missed_for_less_than_24_hours_fire_on_the_first_tick() {
        let (state, clock) = test_state();
        let (notifier, sink) = (FakeNotifier::default(), FakeSink::default());
        add_task(&state, "t1", "Perdido");
        add_reminder(&state, "r1", "t1", "2026-03-10T12:00:00Z");

        clock.advance(Duration::hours(20));
        let report = Engine::default()
            .tick(&state, &notifier, &sink)
            .expect("tick");

        assert_eq!(report.fired, ["r1"]);
        assert!(report.expired.is_empty());
    }

    #[test]
    fn reminders_missed_for_more_than_24_hours_are_expired_without_notifying() {
        let (state, clock) = test_state();
        let (notifier, sink) = (FakeNotifier::default(), FakeSink::default());
        add_task(&state, "t1", "Antiguo");
        add_reminder(&state, "r1", "t1", "2026-03-10T10:30:00Z");

        clock.advance(Duration::hours(24) + Duration::minutes(31));
        let report = Engine::default()
            .tick(&state, &notifier, &sink)
            .expect("tick");

        assert_eq!(report.expired, ["r1"]);
        assert!(report.fired.is_empty());
        assert!(notifier.titles().is_empty() && sink.0.lock().expect("lock").is_empty());
        assert!(is_sent(&state, "r1"));
    }

    #[test]
    fn exactly_24_hours_overdue_still_fires() {
        let (state, clock) = test_state();
        let (notifier, sink) = (FakeNotifier::default(), FakeSink::default());
        add_task(&state, "t1", "Justo");
        add_reminder(&state, "r1", "t1", "2026-03-10T10:00:00Z");

        clock.advance(Duration::hours(24));
        let report = Engine::default()
            .tick(&state, &notifier, &sink)
            .expect("tick");
        assert_eq!(report.fired, ["r1"]);
    }

    #[test]
    fn completed_and_trashed_tasks_do_not_fire_and_do_not_keep_the_loop_busy() {
        let (state, clock) = test_state();
        let (notifier, sink) = (FakeNotifier::default(), FakeSink::default());
        add_task(&state, "done", "Hecha");
        add_task(&state, "gone", "Borrada");
        add_reminder(&state, "r1", "done", "2026-03-10T10:05:00Z");
        add_reminder(&state, "r2", "gone", "2026-03-10T10:05:00Z");
        let now = state.clock.now_string();
        state
            .db
            .write(|tx| {
                tasks::set_completed(tx, "done", true, &now)?;
                Ok(tasks::soft_delete(tx, "gone", &now)?)
            })
            .expect("close tasks");

        clock.advance(Duration::minutes(10));
        let mut engine = Engine::default();
        assert_eq!(
            engine.tick(&state, &notifier, &sink).expect("tick"),
            TickReport::default()
        );
        assert_eq!(wait_seconds(&engine, &state), 60);
    }

    #[test]
    fn reopening_a_task_makes_its_pending_reminder_fire() {
        let (state, clock) = test_state();
        let (notifier, sink) = (FakeNotifier::default(), FakeSink::default());
        add_task(&state, "t1", "A");
        add_reminder(&state, "r1", "t1", "2026-03-10T10:05:00Z");
        let now = state.clock.now_string();
        state
            .db
            .write(|tx| Ok(tasks::set_completed(tx, "t1", true, &now)?))
            .expect("complete");
        clock.advance(Duration::minutes(10));
        let mut engine = Engine::default();
        assert!(
            engine
                .tick(&state, &notifier, &sink)
                .expect("tick")
                .fired
                .is_empty()
        );

        let later = state.clock.now_string();
        state
            .db
            .write(|tx| Ok(tasks::set_completed(tx, "t1", false, &later)?))
            .expect("reopen");

        assert_eq!(
            engine.tick(&state, &notifier, &sink).expect("tick").fired,
            ["r1"]
        );
    }

    #[test]
    fn the_wait_is_capped_at_sixty_seconds_and_shortened_by_the_next_reminder() {
        let (state, _clock) = test_state();
        let engine = Engine::default();
        assert_eq!(wait_seconds(&engine, &state), 60);

        add_task(&state, "t1", "A");
        add_reminder(&state, "r1", "t1", "2026-03-10T11:00:00Z");
        assert_eq!(wait_seconds(&engine, &state), 60);

        add_reminder(&state, "r1", "t1", "2026-03-10T10:00:20Z");
        assert_eq!(wait_seconds(&engine, &state), 20);
    }

    #[test]
    fn a_new_zone_offset_is_seen_by_recalculation() {
        let (state, clock) = test_state();
        let (notifier, sink) = (FakeNotifier::default(), FakeSink::default());
        add_task(&state, "t1", "A");
        let now = state.clock.now_string();
        let zone = FixedOffset::east_opt(3600).expect("zone");
        state
            .db
            .write(|tx| {
                tx.execute(
                    "UPDATE tasks SET due_date = '2026-03-10', due_time = '12:00' WHERE id = 't1'",
                    [],
                )?;
                reminders::replace_unsent(
                    tx,
                    "t1",
                    &[NewReminder {
                        id: "r1".into(),
                        notify_at: "2026-03-10T10:45:00Z".into(),
                        offset_minutes: Some(15),
                    }],
                    &now,
                )?;
                // The user moves the task two hours later: the reminder follows.
                tx.execute("UPDATE tasks SET due_time = '14:00' WHERE id = 't1'", [])?;
                Ok(reminders::recalculate(tx, "t1", &zone)?)
            })
            .expect("recalculate");

        clock.advance(Duration::hours(2));
        assert!(
            Engine::default()
                .tick(&state, &notifier, &sink)
                .expect("tick")
                .fired
                .is_empty()
        );
        clock.advance(Duration::minutes(50));
        assert_eq!(
            Engine::default()
                .tick(&state, &notifier, &sink)
                .expect("tick")
                .fired,
            ["r1"]
        );
    }

    #[tokio::test]
    async fn the_loop_fires_after_a_wake_up_without_waiting_for_the_cap() {
        let (state, clock) = test_state();
        let state = Arc::new(state);
        let notifier = Arc::new(FakeNotifier::default());
        let sink = Arc::new(FakeSink::default());
        add_task(&state, "t1", "Aviso");
        add_reminder(&state, "r1", "t1", "2026-03-10T10:30:00Z");

        let handle = tokio::spawn(run(
            Arc::clone(&state),
            Arc::clone(&notifier) as Arc<dyn Notifier>,
            Arc::clone(&sink) as Arc<dyn FiredSink>,
        ));
        tokio::time::sleep(StdDuration::from_millis(100)).await;
        assert!(notifier.titles().is_empty());

        clock.advance(Duration::minutes(31));
        state.wake.notify_one();
        for _ in 0..100 {
            if !notifier.titles().is_empty() {
                break;
            }
            tokio::time::sleep(StdDuration::from_millis(20)).await;
        }
        handle.abort();

        assert_eq!(notifier.titles(), ["Aviso"]);
    }
}

//! Reminder scheduling (SPEC §6, D-04, D-26).
//!
//! [`Engine::tick`] does one pass: it expires reminders that are more than 24 hours overdue,
//! then fires the rest that are due. [`Engine::wait`] says how long to sleep afterwards.
//! [`run`] is the loop around both. Everything takes a [`Clock`], so tests move time by hand.

pub mod clock;

use std::{collections::HashMap, sync::Arc, time::Duration as StdDuration};

use chrono::{DateTime, Duration, Local, Utc};
use rusqlite::Transaction;

use crate::{
    backoff,
    domain::{dates, reminders as rules},
    error::AppResult,
    notify::{Fired, FiredSink, Notification, Notifier},
    repo::reminders,
    services::events,
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
    crate::applog::warn("scheduler", message);
}

/// A reminder of a repeating series has just been closed (sounded, given up or expired): store
/// the next sounding, if the series goes on (D-32). Runs in the same transaction as the closing.
fn continue_series(
    state: &AppState,
    tx: &Transaction<'_>,
    id: &str,
    now: DateTime<Utc>,
) -> AppResult<()> {
    let Some(member) = reminders::series_member(tx, id)? else {
        return events::continue_after_reminder(state, tx, id, now, &Local);
    };
    let current = dates::parse_instant(&member.notify_at)?;
    if let Some((at, occurrence)) = rules::next_occurrence(&Local, current, &member.repeat, now)? {
        reminders::insert_next(
            tx,
            &state.ids.new_id(),
            &member,
            &dates::format_instant(at),
            occurrence,
            &dates::format_instant(now),
        )?;
    }
    Ok(())
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
        report.expired = state.db.write(|tx| {
            let ids = reminders::expire_before(tx, &cutoff, &now_text)?;
            for id in &ids {
                continue_series(state, tx, id, now)?;
            }
            Ok(ids)
        })?;
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
                if reminders::mark_sent(tx, &reminder.id, &now_text)? {
                    continue_series(state, tx, &reminder.id, now)?;
                }
                notifier.show(&notification)
            });
            match delivery {
                Ok(()) => {
                    self.retries.remove(&reminder.id);
                    sink.fired(&Fired {
                        task_id: reminder.task_id.clone(),
                        note_id: reminder.note_id.clone(),
                        event_id: reminder.event_id.clone(),
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
                        state.db.write(|tx| {
                            if reminders::mark_sent(tx, &reminder.id, &now_text)? {
                                continue_series(state, tx, &reminder.id, now)?;
                            }
                            Ok(())
                        })?;
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
    let mut failures = 0_u32;
    while !state.is_shutting_down() {
        let pass = {
            let state = Arc::clone(&state);
            let notifier = Arc::clone(&notifier);
            let sink = Arc::clone(&sink);
            tokio::task::spawn_blocking(move || {
                let failed = match engine.tick(&state, notifier.as_ref(), sink.as_ref()) {
                    Ok(_) => false,
                    Err(error) => {
                        log(&format!("error en la vuelta: {error}"));
                        true
                    }
                };
                let wait = engine.wait(&state).unwrap_or_else(|error| {
                    log(&format!("no se pudo calcular la espera: {error}"));
                    StdDuration::from_secs(60)
                });
                (engine, wait, failed)
            })
            .await
        };
        let Ok((returned, wait, failed)) = pass else {
            log("la vuelta se interrumpió; el scheduler se detiene");
            return;
        };
        engine = returned;
        // Consecutive failing rounds wait longer (5 s, 10 s, ... up to a minute), and a wake-up
        // does not cut that wait short: a broken database is not retried on every change.
        failures = if failed { failures + 1 } else { 0 };
        let backoff_wait = backoff::delay(
            failures,
            StdDuration::from_secs(5),
            StdDuration::from_secs(60),
        );
        let wait = wait.max(backoff_wait);
        tokio::select! {
            () = tokio::time::sleep(wait) => {}
            () = state.wake.notified(), if failures == 0 => {}
            () = state.shut_down() => return,
        }
    }
}

#[cfg(test)]
mod tests;

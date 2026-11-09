//! `reminders_set` (SPEC §7.3): replaces the unsent reminders of a task.

use std::sync::Arc;

use chrono::{Local, TimeZone};
use serde::Deserialize;
use tauri::State;

use crate::{
    domain::{
        dates,
        reminders::{self as rules, Repeat},
    },
    error::{AppError, AppResult},
    repo::{
        model::Task,
        reminders::{self, NewReminder},
        tasks,
    },
    state::{AppState, run_blocking},
};

/// Most reminders one task may have.
pub const MAX_PER_TASK: usize = 10;

/// Either a number of minutes before the task's date and time, or an absolute UTC instant.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderInput {
    pub offset_minutes: Option<i64>,
    pub notify_at: Option<String>,
    /// Only with `notifyAt`: an offset reminder follows the task's date, so it cannot repeat.
    #[serde(default)]
    pub repeat: Option<Repeat>,
}

pub fn set<Tz: TimeZone>(
    state: &AppState,
    task_id: &str,
    inputs: &[ReminderInput],
    zone: &Tz,
) -> AppResult<Task> {
    if inputs.len() > MAX_PER_TASK {
        return Err(AppError::Validation(format!(
            "una tarea admite como máximo {MAX_PER_TASK} recordatorios"
        )));
    }
    let now = state.clock.now();
    let now_text = dates::format_instant(now);

    let task = state.db.write(|tx| {
        let task = tasks::get(tx, task_id)?
            .ok_or_else(|| AppError::NotFound(format!("tarea {task_id} no encontrada")))?;

        let mut rows: Vec<NewReminder> = Vec::new();
        for input in inputs {
            if input.repeat.is_some() && input.offset_minutes.is_some() {
                return Err(AppError::Validation(
                    "solo los avisos a una hora concreta pueden repetirse".into(),
                ));
            }
            let (notify_at, offset_minutes) =
                match (input.offset_minutes, input.notify_at.as_deref()) {
                    (Some(offset), None) => {
                        rules::validate_offset(offset)?;
                        let (Some(date), Some(time)) = (&task.due_date, &task.due_time) else {
                            return Err(AppError::Validation(
                                "para avisar con antelación la tarea necesita fecha y hora".into(),
                            ));
                        };
                        (
                            rules::offset_notify_at(zone, date, time, offset)?,
                            Some(offset),
                        )
                    }
                    (None, Some(text)) => (dates::parse_instant(text)?, None),
                    _ => {
                        return Err(AppError::Validation(
                            "un recordatorio lleva «offsetMinutes» o «notifyAt», uno de los dos"
                                .into(),
                        ));
                    }
                };
            if notify_at <= now {
                return Err(AppError::Validation(
                    "ese recordatorio ya habría pasado".into(),
                ));
            }
            // The occurrence travels with the rule so that editing another reminder of the task
            // does not restart a series that is half way through.
            if let Some(repeat) = &input.repeat {
                rules::validate_repeat(zone, repeat, notify_at)?;
            }
            let repeat = input.repeat.clone();
            let notify_at = dates::format_instant(notify_at);
            if rows.iter().any(|row| row.notify_at == notify_at) {
                continue;
            }
            rows.push(NewReminder {
                id: state.ids.new_id(),
                notify_at,
                offset_minutes,
                repeat,
            });
        }

        reminders::replace_unsent(tx, task_id, &rows, &now_text)?;
        tasks::get(tx, task_id)?
            .ok_or_else(|| AppError::NotFound(format!("tarea {task_id} no encontrada")))
    })?;
    state.wake.notify_one();
    Ok(task)
}

#[tauri::command]
pub async fn reminders_set(
    state: State<'_, Arc<AppState>>,
    task_id: String,
    reminders: Vec<ReminderInput>,
) -> Result<Task, String> {
    run_blocking(state.inner().clone(), move |state| {
        set(state, &task_id, &reminders, &Local)
    })
    .await
}

#[cfg(test)]
mod tests {
    use chrono::FixedOffset;

    use super::*;
    use crate::{
        commands::tasks::{TaskCreateInput, create},
        state::test_state,
    };

    fn zone() -> FixedOffset {
        FixedOffset::east_opt(3600).expect("zone")
    }

    fn task(state: &AppState, date: Option<&str>, time: Option<&str>) -> Task {
        create(
            state,
            TaskCreateInput {
                title: Some("Clase".into()),
                due_date: date.map(str::to_string),
                due_time: time.map(str::to_string),
                ..TaskCreateInput::default()
            },
        )
        .expect("task")
    }

    fn offset(minutes: i64) -> ReminderInput {
        ReminderInput {
            offset_minutes: Some(minutes),
            notify_at: None,
            repeat: None,
        }
    }

    fn custom(at: &str) -> ReminderInput {
        ReminderInput {
            offset_minutes: None,
            notify_at: Some(at.into()),
            repeat: None,
        }
    }

    fn notify_ats(task: &Task) -> Vec<&str> {
        task.reminders
            .iter()
            .map(|r| r.notify_at.as_str())
            .collect()
    }

    #[test]
    fn offsets_are_computed_from_the_local_due_time() {
        let (state, _) = test_state();
        let created = task(&state, Some("2026-03-12"), Some("09:00"));

        let updated = set(
            &state,
            &created.id,
            &[offset(0), offset(15), offset(1440)],
            &zone(),
        )
        .expect("set");

        assert_eq!(
            notify_ats(&updated),
            [
                "2026-03-11T08:00:00Z",
                "2026-03-12T07:45:00Z",
                "2026-03-12T08:00:00Z"
            ]
        );
        assert_eq!(updated.reminders[1].offset_minutes, Some(15));
        assert!(updated.reminders.iter().all(|r| !r.is_sent));
    }

    #[test]
    fn a_custom_reminder_keeps_its_instant_and_has_no_offset() {
        let (state, _) = test_state();
        let created = task(&state, None, None);

        let updated = set(
            &state,
            &created.id,
            &[custom("2026-03-12T07:30:00Z")],
            &zone(),
        )
        .expect("set");

        assert_eq!(notify_ats(&updated), ["2026-03-12T07:30:00Z"]);
        assert_eq!(updated.reminders[0].offset_minutes, None);
    }

    #[test]
    fn setting_replaces_the_previous_unsent_set_and_keeps_sent_ones() {
        let (state, _) = test_state();
        let created = task(&state, Some("2026-03-11"), Some("09:00"));
        let first = set(&state, &created.id, &[offset(0), offset(60)], &zone()).expect("first");
        let sent_id = first.reminders[0].id.clone();
        let now = state.clock.now_string();
        state
            .db
            .write(|tx| Ok(reminders::mark_sent(tx, &sent_id, &now)?))
            .expect("mark");

        let second = set(&state, &created.id, &[offset(15)], &zone()).expect("second");

        assert_eq!(second.reminders.len(), 2);
        assert!(
            second
                .reminders
                .iter()
                .any(|r| r.id == sent_id && r.is_sent)
        );
        assert_eq!(second.reminders.iter().filter(|r| !r.is_sent).count(), 1);
        let cleared = set(&state, &created.id, &[], &zone()).expect("clear");
        assert_eq!(cleared.reminders.len(), 1);
    }

    #[test]
    fn validation_errors_use_the_validation_code() {
        let (state, _) = test_state();
        let undated = task(&state, Some("2026-03-11"), None);
        let timed = task(&state, Some("2026-03-11"), Some("09:00"));
        let code = |result: AppResult<Task>| result.expect_err("error").code();

        assert_eq!(
            code(set(&state, &undated.id, &[offset(15)], &zone())),
            "VALIDATION"
        );
        assert_eq!(
            code(set(&state, &timed.id, &[offset(-5)], &zone())),
            "VALIDATION"
        );
        assert_eq!(
            code(set(
                &state,
                &timed.id,
                &[ReminderInput {
                    offset_minutes: None,
                    notify_at: None,
                    repeat: None
                }],
                &zone()
            )),
            "VALIDATION"
        );
        assert_eq!(
            code(set(
                &state,
                &timed.id,
                &[ReminderInput {
                    offset_minutes: Some(5),
                    notify_at: Some("2026-03-12T00:00:00Z".into()),
                    repeat: None
                }],
                &zone()
            )),
            "VALIDATION"
        );
        assert_eq!(
            code(set(&state, &timed.id, &[custom("mañana")], &zone())),
            "VALIDATION"
        );
        assert_eq!(
            code(set(
                &state,
                &timed.id,
                &[custom("2026-03-10T09:59:59Z")],
                &zone()
            )),
            "VALIDATION",
            "in the past"
        );
        let too_many: Vec<_> = (0..=MAX_PER_TASK as i64).map(offset).collect();
        assert_eq!(
            code(set(&state, &timed.id, &too_many, &zone())),
            "VALIDATION"
        );
        assert_eq!(code(set(&state, "missing", &[], &zone())), "NOT_FOUND");
    }

    #[test]
    fn a_rejected_set_leaves_the_existing_reminders_untouched() {
        let (state, _) = test_state();
        let created = task(&state, Some("2026-03-11"), Some("09:00"));
        set(&state, &created.id, &[offset(15)], &zone()).expect("first");

        assert!(set(&state, &created.id, &[offset(30), offset(-1)], &zone()).is_err());

        let kept = state
            .db
            .read(|conn| Ok(reminders::for_task(conn, &created.id)?))
            .expect("read");
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].offset_minutes, Some(15));
    }

    #[test]
    fn duplicate_instants_are_stored_once() {
        let (state, _) = test_state();
        let created = task(&state, Some("2026-03-11"), Some("09:00"));
        let updated = set(
            &state,
            &created.id,
            &[offset(15), custom("2026-03-11T07:45:00Z")],
            &zone(),
        )
        .expect("set");
        assert_eq!(updated.reminders.len(), 1);
    }

    #[test]
    fn setting_wakes_the_scheduler() {
        let (state, _) = test_state();
        let created = task(&state, None, None);
        set(
            &state,
            &created.id,
            &[custom("2026-03-12T07:30:00Z")],
            &zone(),
        )
        .expect("set");

        let woken = tauri::async_runtime::block_on(async {
            tokio::time::timeout(std::time::Duration::from_millis(200), state.wake.notified())
                .await
                .is_ok()
        });
        assert!(woken);
    }

    fn repeating(at: &str, repeat: Repeat) -> ReminderInput {
        ReminderInput {
            repeat: Some(repeat),
            ..custom(at)
        }
    }

    fn daily(count: Option<u32>) -> Repeat {
        Repeat {
            every: 1,
            unit: rules::RepeatUnit::Day,
            count,
            until: None,
            occurrence: 1,
        }
    }

    #[test]
    fn a_custom_reminder_can_repeat_and_the_rule_is_returned() {
        let (state, _) = test_state();
        let plain = task(&state, None, None);

        let saved = set(
            &state,
            &plain.id,
            &[repeating("2026-03-12T08:00:00Z", daily(Some(3)))],
            &zone(),
        )
        .expect("set");

        assert_eq!(saved.reminders.len(), 1);
        assert_eq!(saved.reminders[0].repeat, Some(daily(Some(3))));
    }

    #[test]
    fn the_occurrence_of_a_series_is_kept_and_checked_against_its_count() {
        let (state, _) = test_state();
        let plain = task(&state, None, None);
        let half_way = Repeat {
            occurrence: 2,
            ..daily(Some(3))
        };

        let saved = set(
            &state,
            &plain.id,
            &[repeating("2026-03-12T08:00:00Z", half_way.clone())],
            &zone(),
        )
        .expect("set");
        assert_eq!(saved.reminders[0].repeat, Some(half_way));

        for occurrence in [0, 4] {
            let bad = Repeat {
                occurrence,
                ..daily(Some(3))
            };
            let error = set(
                &state,
                &plain.id,
                &[repeating("2026-03-12T08:00:00Z", bad)],
                &zone(),
            )
            .expect_err("occurrence out of range");
            assert_eq!(error.code(), "VALIDATION");
        }
    }

    #[test]
    fn invalid_repeats_and_repeating_offsets_are_refused() {
        let (state, _) = test_state();
        let code = |result: AppResult<Task>| result.expect_err("error").code();
        let timed = task(&state, Some("2026-03-12"), Some("10:00"));
        let bad = Repeat {
            every: 0,
            ..daily(None)
        };
        assert_eq!(
            code(set(
                &state,
                &timed.id,
                &[repeating("2026-03-12T08:00:00Z", bad)],
                &zone()
            )),
            "VALIDATION"
        );
        let offset_with_repeat = ReminderInput {
            repeat: Some(daily(None)),
            ..offset(15)
        };
        assert_eq!(
            code(set(&state, &timed.id, &[offset_with_repeat], &zone())),
            "VALIDATION"
        );
    }
}

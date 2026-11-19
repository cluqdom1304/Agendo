//! Reminder use cases (SPEC §7.3). The Tauri command in `commands/reminders.rs` only wraps these functions.

use chrono::TimeZone;
use serde::Deserialize;

use crate::{
    domain::{
        dates,
        reminders::{self as rules, Repeat},
    },
    error::{AppError, AppResult},
    repo::{
        model::Task,
        reminders::{self, NewReminder},
        settings, tasks,
    },
    state::AppState,
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
        let task = tasks::get(tx, task_id)?.ok_or_else(|| AppError::not_found("tarea", task_id))?;
        if task.is_completed && !inputs.is_empty() {
            return Err(AppError::Validation(
                "una tarea hecha no avisa: reábrela para añadir recordatorios".into(),
            ));
        }

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
                        let Some(date) = &task.due_date else {
                            return Err(AppError::Validation(
                                "para avisar con antelación la tarea necesita fecha".into(),
                            ));
                        };
                        let time = match &task.due_time {
                            Some(time) => time.clone(),
                            None => {
                                rules::validate_untimed_offset(offset)?;
                                settings::notify_time(tx)?
                            }
                        };
                        (
                            rules::offset_notify_at(zone, date, &time, offset)?,
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
        tasks::get(tx, task_id)?.ok_or_else(|| AppError::not_found("tarea", task_id))
    })?;
    state.wake.notify_one();
    Ok(task)
}

#[cfg(test)]
mod tests;

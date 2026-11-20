//! Native notifications and the in-app event (SPEC §6.3, §6.5). The scheduler talks to the traits;
//! the Tauri implementations are the only code that touches the plugin or the event bus.

use std::sync::Arc;

use chrono::{Duration, Local, NaiveTime, TimeZone};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_notification::{Action, ActionType, NotificationExt};

use crate::{
    domain::dates,
    error::AppError,
    repo::{
        self,
        reminders::{self as reminder_repo, DueReminder},
    },
    services::tasks,
    state::AppState,
};

/// Longest piece of the notes shown in a notification, in characters.
const SNIPPET_CHARS: usize = 100;

/// Shown when the task has no title.
const UNTITLED: &str = "Tarea sin título";

pub const FIRED_EVENT: &str = "reminder:fired";
/// Tells the interface that tasks changed behind its back (SPEC §7.4).
pub const TASKS_CHANGED_EVENT: &str = "tasks:changed";
/// The user clicked a notification: the interface shows the task or the note.
pub const OPEN_EVENT: &str = "reminder:open";

/// Action type of the notifications of tasks, and the id and title of its one button (F13-06).
const TASK_ACTIONS: &str = "task-reminder";
const ACTION_DONE: &str = "done";
const ACTION_DONE_TITLE: &str = "Marcar como hecha";
/// Snooze buttons (F17-02): the id says for how long.
const ACTION_SNOOZE_10: &str = "snooze-10m";
const ACTION_SNOOZE_HOUR: &str = "snooze-1h";
const ACTION_SNOOZE_TOMORROW: &str = "snooze-tomorrow";
const SNOOZE_BUTTONS: [(&str, &str); 3] = [
    (ACTION_SNOOZE_10, "Posponer 10 min"),
    (ACTION_SNOOZE_HOUR, "Posponer 1 h"),
    (ACTION_SNOOZE_TOMORROW, "Posponer a mañana"),
];
/// Local time at which a task snoozed to tomorrow sounds again.
const TOMORROW_AT: (u32, u32) = (9, 0);
/// Keys of what a notification carries so that an action knows what it belongs to.
const EXTRA_TASK: &str = "taskId";
const EXTRA_NOTE: &str = "noteId";
const EXTRA_EVENT: &str = "eventId";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub title: String,
    pub body: String,
    /// What the notification is about; actions and clicks refer to it (F13-06).
    pub task_id: Option<String>,
    pub note_id: Option<String>,
    pub event_id: Option<String>,
}

impl Notification {
    /// Title: the task's. Body: its tags and its time, when it has them (SPEC §6.5).
    pub fn for_reminder(reminder: &DueReminder) -> Self {
        let title = if reminder.title.trim().is_empty() {
            UNTITLED.to_string()
        } else {
            reminder.title.clone()
        };
        let mut parts = Vec::new();
        if !reminder.tag_names.is_empty() {
            parts.push(reminder.tag_names.join(", "));
        }
        if let Some(time) = &reminder.due_time {
            parts.push(time.clone());
        }
        let mut lines = Vec::new();
        if !parts.is_empty() {
            lines.push(parts.join(" · "));
        }
        if let Some(snippet) = snippet_of(&reminder.notes) {
            lines.push(snippet);
        }
        Self {
            title,
            task_id: reminder.task_id.clone(),
            note_id: reminder.note_id.clone(),
            event_id: reminder.event_id.clone(),
            body: lines.join(
                "
",
            ),
        }
    }
}

/// The start of the notes on one line, cut at a word boundary with «…» if it is long.
fn snippet_of(notes: &str) -> Option<String> {
    let flat = notes.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.is_empty() {
        return None;
    }
    if flat.chars().count() <= SNIPPET_CHARS {
        return Some(flat);
    }
    let cut: String = flat.chars().take(SNIPPET_CHARS).collect();
    let end = cut
        .rfind(' ')
        .filter(|at| *at > SNIPPET_CHARS / 2)
        .unwrap_or(cut.len());
    Some(format!("{}…", cut[..end].trim_end()))
}

pub trait Notifier: Send + Sync {
    fn show(&self, notification: &Notification) -> Result<(), AppError>;
}

/// Payload of the `reminder:fired` event (SPEC §7.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fired {
    /// The task of the reminder, or the note (`note_id`) when it is a note's (D-52).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_id: Option<String>,
    /// The event (D-94) when the reminder is an event's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    pub reminder_id: String,
}

pub trait FiredSink: Send + Sync {
    fn fired(&self, fired: &Fired);
}

pub struct TauriNotifier<R: Runtime>(pub AppHandle<R>);

impl<R: Runtime> Notifier for TauriNotifier<R> {
    fn show(&self, notification: &Notification) -> Result<(), AppError> {
        let mut builder = self
            .0
            .notification()
            .builder()
            .title(&notification.title)
            .body(&notification.body);
        // Only tasks have the button: a note has nothing to mark as done.
        if let Some(id) = &notification.task_id {
            builder = builder.action_type_id(TASK_ACTIONS).extra(EXTRA_TASK, id);
        }
        if let Some(id) = &notification.note_id {
            builder = builder.extra(EXTRA_NOTE, id);
        }
        if let Some(id) = &notification.event_id {
            builder = builder.extra(EXTRA_EVENT, id);
        }
        builder
            .show()
            .map_err(|error| AppError::Notify(error.to_string()))
    }
}

/// What to do with a click on a notification or on its button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionOutcome {
    /// The task was marked as done.
    Completed(String),
    /// The notification was postponed: a new reminder of the task will sound later.
    Snoozed(String),
    /// Bring the app forward on this task, note or event.
    Open {
        task_id: Option<String>,
        note_id: Option<String>,
        event_id: Option<String>,
    },
    /// Nothing to do (the platform did not say which notification, or the task is gone).
    Ignored,
}

/// Decides and does what an action asks. `done` completes the task of the notification; any
/// other action (a click, `tap`) opens the app on it. Without knowing the notification it never
/// guesses: completing the wrong task is worse than doing nothing.
pub fn perform_action(
    state: &AppState,
    action_id: &str,
    task_id: Option<&str>,
    note_id: Option<&str>,
    event_id: Option<&str>,
) -> ActionOutcome {
    if action_id == ACTION_DONE {
        let Some(id) = task_id else {
            return ActionOutcome::Ignored;
        };
        return match tasks::complete(state, id, true, None) {
            Ok(_) => ActionOutcome::Completed(id.to_string()),
            Err(_) => ActionOutcome::Ignored,
        };
    }
    if SNOOZE_BUTTONS.iter().any(|(id, _)| *id == action_id) {
        let Some(id) = task_id else {
            return ActionOutcome::Ignored;
        };
        return match snooze(state, id, action_id, &Local) {
            Ok(()) => ActionOutcome::Snoozed(id.to_string()),
            Err(_) => ActionOutcome::Ignored,
        };
    }
    if task_id.is_none() && note_id.is_none() && event_id.is_none() {
        return ActionOutcome::Ignored;
    }
    ActionOutcome::Open {
        task_id: task_id.map(str::to_string),
        note_id: note_id.map(str::to_string),
        event_id: event_id.map(str::to_string),
    }
}

/// Adds a reminder of the task 10 minutes or an hour from now, or at `TOMORROW_AT` tomorrow in
/// `zone`. The reminders the task already has stay as they are.
fn snooze<Tz: TimeZone>(
    state: &AppState,
    task_id: &str,
    action_id: &str,
    zone: &Tz,
) -> Result<(), AppError> {
    let now = state.clock.now();
    let at = match action_id {
        ACTION_SNOOZE_10 => now + Duration::minutes(10),
        ACTION_SNOOZE_HOUR => now + Duration::hours(1),
        ACTION_SNOOZE_TOMORROW => {
            let tomorrow = dates::utc_to_local(zone, now).date() + Duration::days(1);
            let time = NaiveTime::from_hms_opt(TOMORROW_AT.0, TOMORROW_AT.1, 0)
                .ok_or_else(|| AppError::Internal("hora de «mañana» no válida".into()))?;
            dates::local_to_utc(zone, tomorrow, time)?
        }
        _ => return Err(AppError::Internal("acción de posponer desconocida".into())),
    };
    let notify_at = dates::format_instant(at);
    let created = state.clock.now_string();
    state.db.write(|tx| {
        let task = repo::tasks::get(tx, task_id)?
            .filter(|task| !task.is_completed)
            .ok_or_else(|| AppError::NotFound("tarea no encontrada".into()))?;
        reminder_repo::insert_custom(tx, &state.ids.new_id(), &task.id, &notify_at, &created)?;
        Ok(())
    })?;
    state.wake.notify_one();
    Ok(())
}

/// Registers the button of task notifications and reacts to it (F13-06, B-01). What Windows
/// reports for a click depends on the toast, so this is validated by hand (`[H]`).
pub fn install_actions<R: Runtime>(app: &AppHandle<R>, state: Arc<AppState>) {
    let mut buttons = vec![Action::builder(ACTION_DONE, ACTION_DONE_TITLE).build()];
    buttons.extend(
        SNOOZE_BUTTONS
            .iter()
            .map(|(id, title)| Action::builder(*id, *title).build()),
    );
    let types = vec![ActionType::builder(TASK_ACTIONS).actions(buttons).build()];
    if let Err(error) = app.notification().register_action_types(types) {
        crate::applog::error(
            "notify",
            &format!("no se pudieron registrar las acciones: {error}"),
        );
        return;
    }
    let handle = app.clone();
    let registered = app.notification().on_action(move |performed| {
        let extra = performed.notification().map(|shown| shown.extra());
        let text = |key: &str| {
            extra
                .and_then(|extra| extra.get(key))
                .and_then(Value::as_str)
        };
        let outcome = perform_action(
            &state,
            performed.action_id(),
            text(EXTRA_TASK),
            text(EXTRA_NOTE),
            text(EXTRA_EVENT),
        );
        match outcome {
            ActionOutcome::Completed(id) | ActionOutcome::Snoozed(id) => {
                let _ = handle.emit(TASKS_CHANGED_EVENT, serde_json::json!({ "ids": [id] }));
            }
            ActionOutcome::Open {
                task_id,
                note_id,
                event_id,
            } => {
                if let Some(window) = handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
                let _ = handle.emit(
                    OPEN_EVENT,
                    serde_json::json!({ "taskId": task_id, "noteId": note_id, "eventId": event_id }),
                );
            }
            ActionOutcome::Ignored => {}
        }
    });
    if let Err(error) = registered {
        crate::applog::error(
            "notify",
            &format!("no se pudo escuchar las acciones: {error}"),
        );
    }
}

pub struct TauriFiredSink<R: Runtime>(pub AppHandle<R>);

impl<R: Runtime> FiredSink for TauriFiredSink<R> {
    fn fired(&self, fired: &Fired) {
        if let Err(error) = self.0.emit(FIRED_EVENT, fired) {
            crate::applog::error(
                "scheduler",
                &format!("no se pudo emitir {FIRED_EVENT}: {error}"),
            );
        }
    }
}

#[cfg(test)]
mod tests;

//! Native notifications and the in-app event (SPEC §6.3, §6.5). The scheduler talks to the traits;
//! the Tauri implementations are the only code that touches the plugin or the event bus.

use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_notification::{Action, ActionType, NotificationExt};

use crate::{commands::tasks, error::AppError, repo::reminders::DueReminder, state::AppState};

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
/// Keys of what a notification carries so that an action knows what it belongs to.
const EXTRA_TASK: &str = "taskId";
const EXTRA_NOTE: &str = "noteId";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub title: String,
    pub body: String,
    /// What the notification is about; actions and clicks refer to it (F13-06).
    pub task_id: Option<String>,
    pub note_id: Option<String>,
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
    /// Bring the app forward on this task or note.
    Open {
        task_id: Option<String>,
        note_id: Option<String>,
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
    if task_id.is_none() && note_id.is_none() {
        return ActionOutcome::Ignored;
    }
    ActionOutcome::Open {
        task_id: task_id.map(str::to_string),
        note_id: note_id.map(str::to_string),
    }
}

/// Registers the button of task notifications and reacts to it (F13-06, B-01). What Windows
/// reports for a click depends on the toast, so this is validated by hand (`[H]`).
pub fn install_actions<R: Runtime>(app: &AppHandle<R>, state: Arc<AppState>) {
    let button = Action::builder(ACTION_DONE, ACTION_DONE_TITLE).build();
    let types = vec![
        ActionType::builder(TASK_ACTIONS)
            .actions(vec![button])
            .build(),
    ];
    if let Err(error) = app.notification().register_action_types(types) {
        eprintln!("[notify] no se pudieron registrar las acciones: {error}");
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
        );
        match outcome {
            ActionOutcome::Completed(id) => {
                let _ = handle.emit(TASKS_CHANGED_EVENT, serde_json::json!({ "ids": [id] }));
            }
            ActionOutcome::Open { task_id, note_id } => {
                if let Some(window) = handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
                let _ = handle.emit(
                    OPEN_EVENT,
                    serde_json::json!({ "taskId": task_id, "noteId": note_id }),
                );
            }
            ActionOutcome::Ignored => {}
        }
    });
    if let Err(error) = registered {
        eprintln!("[notify] no se pudo escuchar las acciones: {error}");
    }
}

pub struct TauriFiredSink<R: Runtime>(pub AppHandle<R>);

impl<R: Runtime> FiredSink for TauriFiredSink<R> {
    fn fired(&self, fired: &Fired) {
        if let Err(error) = self.0.emit(FIRED_EVENT, fired) {
            eprintln!("[scheduler] no se pudo emitir {FIRED_EVENT}: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reminder(title: &str, tags: &[&str], time: Option<&str>) -> DueReminder {
        DueReminder {
            id: "r".into(),
            notify_at: "2026-03-10T10:00:00Z".into(),
            task_id: Some("t".into()),
            note_id: None,
            title: title.into(),
            due_time: time.map(str::to_string),
            notes: String::new(),
            tag_names: tags.iter().map(|tag| (*tag).to_string()).collect(),
        }
    }

    #[test]
    fn the_body_lists_tags_and_time() {
        let n =
            Notification::for_reminder(&reminder("Entregar", &["Mates", "Examen"], Some("09:30")));
        assert_eq!(n.title, "Entregar");
        assert_eq!(n.body, "Mates, Examen · 09:30");
    }

    #[test]
    fn the_body_leaves_out_what_is_missing() {
        assert_eq!(
            Notification::for_reminder(&reminder("A", &[], Some("08:00"))).body,
            "08:00"
        );
        assert_eq!(
            Notification::for_reminder(&reminder("A", &["Mates"], None)).body,
            "Mates"
        );
        assert_eq!(
            Notification::for_reminder(&reminder("A", &[], None)).body,
            ""
        );
    }

    #[test]
    fn an_untitled_task_still_has_a_title() {
        assert_eq!(
            Notification::for_reminder(&reminder("  ", &[], None)).title,
            UNTITLED
        );
    }

    #[test]
    fn notes_add_a_second_line_flattened_to_one_line() {
        let mut r = reminder("A", &["Mates"], Some("09:00"));
        r.notes = "Primera línea

  segunda   línea"
            .into();
        assert_eq!(
            Notification::for_reminder(&r).body,
            "Mates · 09:00
Primera línea segunda línea"
        );
        r.tag_names.clear();
        r.due_time = None;
        assert_eq!(
            Notification::for_reminder(&r).body,
            "Primera línea segunda línea"
        );
    }

    #[test]
    fn long_notes_are_cut_at_a_word_with_an_ellipsis() {
        let mut r = reminder("A", &[], None);
        r.notes = "palabra ".repeat(40);
        let body = Notification::for_reminder(&r).body;
        assert!(body.ends_with("palabra…"), "{body}");
        assert!(body.chars().count() <= SNIPPET_CHARS + 1);
    }

    #[test]
    fn the_fired_payload_is_camel_case() {
        let json = serde_json::to_value(Fired {
            task_id: Some("t".into()),
            note_id: None,
            reminder_id: "r".into(),
        })
        .expect("json");
        assert_eq!(
            json,
            serde_json::json!({ "taskId": "t", "reminderId": "r" })
        );
    }
    fn task_in(state: &AppState, title: &str) -> String {
        tasks::create(
            state,
            tasks::TaskCreateInput {
                title: Some(title.into()),
                ..tasks::TaskCreateInput::default()
            },
        )
        .expect("task")
        .id
    }

    #[test]
    fn the_notification_remembers_what_it_is_about() {
        let task = Notification::for_reminder(&reminder("Entregar", &[], None));
        assert_eq!(task.task_id.as_deref(), Some("t"));
        assert_eq!(task.note_id, None);
        let mut for_note = reminder("Reunión", &[], None);
        for_note.task_id = None;
        for_note.note_id = Some("n".into());
        let note = Notification::for_reminder(&for_note);
        assert_eq!((note.task_id, note.note_id.as_deref()), (None, Some("n")));
    }

    #[test]
    fn the_done_button_completes_the_task_of_the_notification() {
        let (state, _) = crate::state::test_state();
        let id = task_in(&state, "Entregar");
        assert_eq!(
            perform_action(&state, ACTION_DONE, Some(&id), None),
            ActionOutcome::Completed(id.clone())
        );
        let done = state
            .db
            .read(|conn| Ok(crate::repo::tasks::get(conn, &id)?))
            .expect("read")
            .expect("task");
        assert!(done.is_completed);
    }

    #[test]
    fn the_done_button_never_guesses_which_task() {
        let (state, _) = crate::state::test_state();
        let id = task_in(&state, "Entregar");
        assert_eq!(
            perform_action(&state, ACTION_DONE, None, None),
            ActionOutcome::Ignored
        );
        assert_eq!(
            perform_action(&state, ACTION_DONE, Some("no-existe"), None),
            ActionOutcome::Ignored
        );
        let open = state
            .db
            .read(|conn| Ok(crate::repo::tasks::get(conn, &id)?))
            .expect("read")
            .expect("task");
        assert!(!open.is_completed);
    }

    #[test]
    fn a_click_opens_the_task_or_the_note_and_does_not_complete_anything() {
        let (state, _) = crate::state::test_state();
        let id = task_in(&state, "Entregar");
        assert_eq!(
            perform_action(&state, "tap", Some(&id), None),
            ActionOutcome::Open {
                task_id: Some(id.clone()),
                note_id: None
            }
        );
        assert_eq!(
            perform_action(&state, "tap", None, Some("n1")),
            ActionOutcome::Open {
                task_id: None,
                note_id: Some("n1".into())
            }
        );
        assert_eq!(
            perform_action(&state, "tap", None, None),
            ActionOutcome::Ignored
        );
        let still_open = state
            .db
            .read(|conn| Ok(crate::repo::tasks::get(conn, &id)?))
            .expect("read")
            .expect("task");
        assert!(!still_open.is_completed);
    }
}

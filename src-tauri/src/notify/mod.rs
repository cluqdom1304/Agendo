//! Native notifications and the in-app event (SPEC §6.3, §6.5). The scheduler talks to the traits;
//! the Tauri implementations are the only code that touches the plugin or the event bus.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_notification::NotificationExt;

use crate::{error::AppError, repo::reminders::DueReminder};

/// Shown when the task has no title.
const UNTITLED: &str = "Tarea sin título";

pub const FIRED_EVENT: &str = "reminder:fired";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub title: String,
    pub body: String,
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
        Self {
            title,
            body: parts.join(" · "),
        }
    }
}

pub trait Notifier: Send + Sync {
    fn show(&self, notification: &Notification) -> Result<(), AppError>;
}

/// Payload of the `reminder:fired` event (SPEC §7.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fired {
    pub task_id: String,
    pub reminder_id: String,
}

pub trait FiredSink: Send + Sync {
    fn fired(&self, fired: &Fired);
}

pub struct TauriNotifier<R: Runtime>(pub AppHandle<R>);

impl<R: Runtime> Notifier for TauriNotifier<R> {
    fn show(&self, notification: &Notification) -> Result<(), AppError> {
        self.0
            .notification()
            .builder()
            .title(&notification.title)
            .body(&notification.body)
            .show()
            .map_err(|error| AppError::Notify(error.to_string()))
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
            task_id: "t".into(),
            title: title.into(),
            due_time: time.map(str::to_string),
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
    fn the_fired_payload_is_camel_case() {
        let json = serde_json::to_value(Fired {
            task_id: "t".into(),
            reminder_id: "r".into(),
        })
        .expect("json");
        assert_eq!(
            json,
            serde_json::json!({ "taskId": "t", "reminderId": "r" })
        );
    }
}

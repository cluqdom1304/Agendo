use serde::{Deserialize, Serialize};

use crate::domain::{ordering::Sortable, reminders::Repeat};

/// `content_json` of a task without notes.
pub const EMPTY_DOC: &str = r#"{"type":"doc","content":[]}"#;

/// Task as exposed over IPC (SPEC §7.2).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    pub content_json: String,
    pub due_date: Option<String>,
    pub due_time: Option<String>,
    /// Minutes the task lasts from `due_time`, within the same day (D-33).
    pub duration_minutes: Option<i64>,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub position: i64,
    pub tag_ids: Vec<String>,
    pub reminders: Vec<Reminder>,
    pub created_at: String,
    pub updated_at: String,
}

impl Sortable for Task {
    fn id(&self) -> &str {
        &self.id
    }
    fn due_date(&self) -> Option<&str> {
        self.due_date.as_deref()
    }
    fn due_time(&self) -> Option<&str> {
        self.due_time.as_deref()
    }
    fn is_completed(&self) -> bool {
        self.is_completed
    }
    fn completed_at(&self) -> Option<&str> {
        self.completed_at.as_deref()
    }
    fn position(&self) -> i64 {
        self.position
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: String,
    pub notify_at: String,
    pub offset_minutes: Option<i64>,
    pub is_sent: bool,
    /// How the reminder repeats, if it does (custom-time reminders only, D-32).
    pub repeat: Option<Repeat>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color_hex: String,
    pub parent_id: Option<String>,
    pub position: i64,
}

/// A tag with the number of its pending (not completed, not deleted) tasks.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagWithCount {
    #[serde(flatten)]
    pub tag: Tag,
    pub pending_count: i64,
}

/// Data to create a task. The caller supplies the id and the UTC instant `now`, so the
/// repository never reads the clock.
#[derive(Debug, Clone, Default)]
pub struct NewTask {
    pub id: String,
    pub title: String,
    /// `None` stores [`EMPTY_DOC`].
    pub content_json: Option<String>,
    pub content_text: String,
    pub due_date: Option<String>,
    pub due_time: Option<String>,
    pub duration_minutes: Option<i64>,
    pub tag_ids: Vec<String>,
    pub now: String,
}

/// Partial update. `None` leaves a field untouched; for nullable fields `Some(None)` clears it.
#[derive(Debug, Clone, Default)]
pub struct TaskPatch {
    pub title: Option<String>,
    pub content_json: Option<String>,
    pub content_text: Option<String>,
    pub due_date: Option<Option<String>>,
    pub due_time: Option<Option<String>>,
    pub duration_minutes: Option<Option<i64>>,
}

/// Data to create or update a tag. `now` is used only when the tag is created.
#[derive(Debug, Clone)]
pub struct TagInput {
    pub id: String,
    pub name: String,
    pub color_hex: String,
    pub parent_id: Option<String>,
    pub now: String,
}

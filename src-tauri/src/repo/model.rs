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
    /// Note this task was created from (D-45).
    pub source_note_id: Option<String>,
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
    /// Live notes carrying the tag (tags are shared between tasks and notes, D-44).
    pub note_count: i64,
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
    pub source_note_id: Option<String>,
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

/// Folder as exposed over IPC (D-43).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub position: i64,
}

/// A folder with the number of live notes directly inside it (not in its subfolders).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderWithCount {
    #[serde(flatten)]
    pub folder: Folder,
    pub note_count: i64,
}

/// Data to create or rename/move a folder. `now` stamps `created_at` and `updated_at`.
#[derive(Debug, Clone)]
pub struct FolderInput {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub now: String,
}

/// A note with its content, as opened in the editor.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    /// `None` is the inbox.
    pub folder_id: Option<String>,
    pub title: String,
    pub content_json: String,
    pub is_pinned: bool,
    pub tag_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A note as listed: no document, only a short plain-text excerpt.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSummary {
    pub id: String,
    pub folder_id: Option<String>,
    pub title: String,
    pub excerpt: String,
    pub is_pinned: bool,
    pub tag_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Data to create a note; like [`NewTask`], the caller supplies the id and `now`.
#[derive(Debug, Clone, Default)]
pub struct NewNote {
    pub id: String,
    pub folder_id: Option<String>,
    pub title: String,
    /// `None` stores [`EMPTY_DOC`].
    pub content_json: Option<String>,
    pub content_text: String,
    pub task_ids: Vec<String>,
    pub note_ids: Vec<String>,
    pub tag_ids: Vec<String>,
    pub now: String,
}

/// Partial update of a note. `content_*` and the references travel together: they are all
/// derived from one document.
#[derive(Debug, Clone, Default)]
pub struct NotePatch {
    pub title: Option<String>,
    pub content: Option<NoteContent>,
}

/// A saved document with what was derived from it.
#[derive(Debug, Clone)]
pub struct NoteContent {
    pub json: String,
    pub text: String,
    pub task_ids: Vec<String>,
    pub note_ids: Vec<String>,
}

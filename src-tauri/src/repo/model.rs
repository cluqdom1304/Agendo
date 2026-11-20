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
    /// How the task repeats, as a rule of `domain::recurrence` (F13-05).
    pub rrule: Option<String>,
    /// 0 none, 1 medium, 2 high, 3 urgent (D-93).
    pub priority: i64,
    /// Checked and total checklist items of `content_json`, stored on save (D-93).
    pub checklist_done: i64,
    pub checklist_total: i64,
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
    pub rrule: Option<String>,
    pub priority: i64,
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
    pub priority: Option<i64>,
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

/// The time a note will remind its owner (F13-04, D-52).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteReminder {
    pub id: String,
    pub note_id: String,
    pub notify_at: String,
}

/// A saved version of a note, as listed: no document (F13-03).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteVersionSummary {
    pub id: String,
    pub note_id: String,
    pub title: String,
    /// Length of the stored document, to tell versions apart at a glance.
    pub size: i64,
    pub created_at: String,
}

/// A saved version of a note with its document.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteVersion {
    pub id: String,
    pub note_id: String,
    pub title: String,
    pub content_json: String,
    pub created_at: String,
}

/// A named document new notes can start from (F13-02).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteTemplate {
    pub id: String,
    pub name: String,
    pub content_json: String,
    pub created_at: String,
    pub updated_at: String,
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

/// A window over an ordered list: `limit` rows after skipping `offset`. [`Page::ALL`] is the
/// whole list, which is what a caller that does not paginate gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    pub limit: Option<u32>,
    pub offset: u32,
}

impl Page {
    pub const ALL: Page = Page {
        limit: None,
        offset: 0,
    };

    /// SQLite's `LIMIT -1` means no limit.
    pub fn sql_limit(self) -> i64 {
        self.limit.map_or(-1, i64::from)
    }
}

/// Just the title of a live note, for labels such as the origin of a task (`note_titles`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NoteTitle {
    pub id: String,
    pub title: String,
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

/// An appointment with a start and an end (D-94). It has no completion state. Dates and times
/// are local; an event without times lasts whole days.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub title: String,
    pub notes: String,
    pub start_date: String,
    pub start_time: Option<String>,
    pub end_date: String,
    pub end_time: Option<String>,
    pub all_day: bool,
    /// Repeat rule of `domain::recurrence`, anchored on `start_date`.
    pub rrule: Option<String>,
    pub tag_ids: Vec<String>,
    /// Minutes before the start at which the event warns; one entry per reminder.
    pub reminder_offsets: Vec<i64>,
    pub created_at: String,
    pub updated_at: String,
}

/// One appearance of an event on the calendar: the event itself or one repetition of it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventOccurrence {
    pub event_id: String,
    pub title: String,
    pub start_date: String,
    pub start_time: Option<String>,
    pub end_date: String,
    pub end_time: Option<String>,
    pub all_day: bool,
    pub recurring: bool,
    pub tag_ids: Vec<String>,
}

/// Every editable field of an event, to create it or to replace what it has. The caller supplies
/// the id and the UTC instant `now`.
#[derive(Debug, Clone, Default)]
pub struct EventData {
    pub id: String,
    pub title: String,
    pub notes: String,
    pub start_date: String,
    pub start_time: Option<String>,
    pub end_date: String,
    pub end_time: Option<String>,
    pub rrule: Option<String>,
    pub tag_ids: Vec<String>,
    pub reminder_offsets: Vec<i64>,
    pub now: String,
}

/// A habit as stored (D-95). `services::habits` adds the streaks to make a `Habit`.
#[derive(Debug, Clone, PartialEq)]
pub struct HabitRow {
    pub id: String,
    pub title: String,
    /// Scheduled weekdays, bit 0 = Monday ... bit 6 = Sunday.
    pub days: u8,
    pub time: Option<String>,
    pub tag_id: Option<String>,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

/// Editable fields of a habit, to create it or replace what it has.
#[derive(Debug, Clone, Default)]
pub struct HabitData {
    pub id: String,
    pub title: String,
    pub days: u8,
    pub time: Option<String>,
    pub tag_id: Option<String>,
    pub now: String,
}

/// A habit as exposed over IPC, with what depends on the day (D-95).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Habit {
    pub id: String,
    pub title: String,
    pub days: u8,
    pub time: Option<String>,
    pub tag_id: Option<String>,
    pub position: i64,
    pub is_archived: bool,
    pub scheduled_today: bool,
    pub done_today: bool,
    pub current_streak: u32,
    pub best_streak: u32,
    pub created_at: String,
    pub updated_at: String,
}

/// A day on which a habit was marked as done.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitLog {
    pub habit_id: String,
    pub date: String,
}

/// A focus session on a task (D-97). Without `ended_at` it is still running.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeEntry {
    pub id: String,
    pub task_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub planned_minutes: i64,
}

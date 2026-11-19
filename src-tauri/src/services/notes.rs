//! Note use cases (SPEC §7.3). The Tauri commands in `commands/notes.rs` only wrap these functions.

use serde::Deserialize;

use crate::{
    domain::{content, dates, validation},
    error::{AppError, AppResult},
    repo::{
        attachments,
        model::{NewNote, Note, NoteContent, NotePatch, NoteSummary, NoteTitle, Page},
        note_tags,
        notes::{self, Scope, Sort},
        versions,
    },
    state::AppState,
};

fn note_not_found(id: &str) -> AppError {
    AppError::not_found("nota", id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotesScope {
    All,
    Inbox,
    Pinned,
    Folder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotesSort {
    #[default]
    Updated,
    Created,
    Title,
}

#[derive(Debug, Clone)]
pub struct NotesListInput {
    pub scope: NotesScope,
    /// Required (and only used) with `NotesScope::Folder`.
    pub folder_id: Option<String>,
    pub tag_id: Option<String>,
    pub sort: NotesSort,
}

#[derive(Debug, Clone, Default)]
pub struct NoteCreateInput {
    pub folder_id: Option<String>,
    pub title: Option<String>,
    pub content_json: Option<String>,
    pub tag_ids: Option<Vec<String>>,
}

/// Partial update of a note, as sent by the frontend.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotePatchInput {
    pub title: Option<String>,
    pub content_json: Option<String>,
}

/// Validates a document and derives what is stored with it.
pub(crate) fn derive(content_json: String) -> AppResult<(NoteContent, Vec<String>)> {
    let text = content::text_of(&content_json)?;
    let references = content::references_of(&content_json)?;
    let attachment_ids = content::attachment_ids_of(&content_json)?;
    Ok((
        NoteContent {
            json: content_json,
            text,
            task_ids: references.task_ids,
            note_ids: references.note_ids,
        },
        attachment_ids,
    ))
}

#[cfg(test)]
pub fn list(state: &AppState, input: NotesListInput) -> AppResult<Vec<NoteSummary>> {
    list_page(state, input, Page::ALL)
}

/// One window of [`list`] (`limit` / `offset` of the command; the order is stable).
pub fn list_page(
    state: &AppState,
    input: NotesListInput,
    page: Page,
) -> AppResult<Vec<NoteSummary>> {
    let scope = match (input.scope, input.folder_id.as_deref()) {
        (NotesScope::All, _) => Scope::All,
        (NotesScope::Inbox, _) => Scope::Inbox,
        (NotesScope::Pinned, _) => Scope::Pinned,
        (NotesScope::Folder, Some(id)) => Scope::Folder(id),
        (NotesScope::Folder, None) => {
            return Err(AppError::Validation(
                "la vista de carpeta necesita «folderId»".into(),
            ));
        }
    };
    let sort = match input.sort {
        NotesSort::Updated => Sort::Updated,
        NotesSort::Created => Sort::Created,
        NotesSort::Title => Sort::Title,
    };
    state.db.read(|conn| {
        Ok(notes::list_page(
            conn,
            scope,
            input.tag_id.as_deref(),
            sort,
            page,
        )?)
    })
}

/// Most ids `note_titles` accepts in one call.
pub const MAX_TITLE_IDS: usize = 1000;

/// Titles of the live notes among `ids` (unknown and trashed ones are left out), in one query.
pub fn titles(state: &AppState, ids: &[String]) -> AppResult<Vec<NoteTitle>> {
    if ids.len() > MAX_TITLE_IDS {
        return Err(AppError::Validation(format!(
            "se piden demasiados títulos de nota a la vez (máximo {MAX_TITLE_IDS})"
        )));
    }
    state.db.read(|conn| Ok(notes::titles(conn, ids)?))
}

pub fn get(state: &AppState, id: &str) -> AppResult<Note> {
    state
        .db
        .read(|conn| notes::get(conn, id)?.ok_or_else(|| note_not_found(id)))
}

pub fn create(state: &AppState, input: NoteCreateInput) -> AppResult<Note> {
    let title = input.title.unwrap_or_default();
    validation::validate_title(&title)?;
    let derived = input.content_json.map(derive).transpose()?;
    let (content_json, content_text, task_ids, note_ids, attachment_ids) = match derived {
        Some((content, attachments)) => (
            Some(content.json),
            content.text,
            content.task_ids,
            content.note_ids,
            attachments,
        ),
        None => (None, String::new(), Vec::new(), Vec::new(), Vec::new()),
    };
    let new = NewNote {
        id: state.ids.new_id(),
        folder_id: input.folder_id,
        title,
        content_json,
        content_text,
        task_ids,
        note_ids,
        tag_ids: input.tag_ids.unwrap_or_default(),
        now: state.clock.now_string(),
    };
    state.db.write(|tx| {
        let note = notes::create(tx, &new)?;
        attachments::sync_note(tx, &note.id, &attachment_ids)?;
        Ok(note)
    })
}

/// The note of a day: found, brought back from the trash, or created with `title` (F13-01).
pub fn daily_open(state: &AppState, date: &str, title: &str) -> AppResult<Note> {
    dates::parse_date(date)?;
    validation::validate_title(title)?;
    let id = state.ids.new_id();
    let now = state.clock.now_string();
    state.db.write(|tx| {
        if let Some((existing, trashed)) = notes::daily_id(tx, date)? {
            let note = if trashed {
                notes::restore(tx, &existing, &now)?
            } else {
                notes::get(tx, &existing)?
            };
            return note.ok_or_else(|| note_not_found(&existing));
        }
        let new = NewNote {
            id,
            title: title.to_string(),
            now,
            ..NewNote::default()
        };
        let note = notes::create(tx, &new)?;
        notes::set_daily_date(tx, &note.id, date)?;
        Ok(note)
    })
}

pub fn update(state: &AppState, id: &str, patch: NotePatchInput) -> AppResult<Note> {
    update_with(state, id, patch, false)
}

/// `update`, where `force_snapshot` keeps the replaced document as a version even if a recent
/// one exists (restoring a version, F13-03).
pub fn update_with(
    state: &AppState,
    id: &str,
    patch: NotePatchInput,
    force_snapshot: bool,
) -> AppResult<Note> {
    if let Some(title) = &patch.title {
        validation::validate_title(title)?;
    }
    let (content, attachment_ids) = match patch.content_json.map(derive).transpose()? {
        Some((content, attachments)) => (Some(content), Some(attachments)),
        None => (None, None),
    };
    let repo_patch = NotePatch {
        title: patch.title,
        content,
    };
    let now = state.clock.now_string();
    let due_before = dates::format_instant(
        state.clock.now() - chrono::Duration::minutes(versions::SNAPSHOT_EVERY_MINUTES),
    );
    let version_id = state.ids.new_id();
    state.db.write(|tx| {
        if repo_patch.title.is_none() && repo_patch.content.is_none() {
            return notes::get(tx, id)?.ok_or_else(|| note_not_found(id));
        }
        if let Some(content) = &repo_patch.content {
            let current = notes::get(tx, id)?.ok_or_else(|| note_not_found(id))?;
            versions::capture(
                tx,
                &version_id,
                &current,
                &content.json,
                &now,
                &due_before,
                force_snapshot,
            )?;
        }
        let note = notes::update(tx, id, &repo_patch, &now)?.ok_or_else(|| note_not_found(id))?;
        if let Some(ids) = &attachment_ids {
            attachments::sync_note(tx, id, ids)?;
        }
        Ok(note)
    })
}

pub fn set_pinned(state: &AppState, id: &str, pinned: bool) -> AppResult<Note> {
    state
        .db
        .write(|tx| notes::set_pinned(tx, id, pinned)?.ok_or_else(|| note_not_found(id)))
}

/// Moves the note to a folder, or to the inbox with `None`.
pub fn move_to_folder(state: &AppState, id: &str, folder_id: Option<&str>) -> AppResult<Note> {
    let now = state.clock.now_string();
    state
        .db
        .write(|tx| notes::move_note(tx, id, folder_id, &now)?.ok_or_else(|| note_not_found(id)))
}

/// Replaces the tags of a note with exactly `tag_ids`.
pub fn set_note_tags(state: &AppState, note_id: &str, tag_ids: &[String]) -> AppResult<Note> {
    let now = state.clock.now_string();
    state.db.write(|tx| {
        if !note_tags::set_note_tags(tx, note_id, tag_ids, &now)? {
            return Err(note_not_found(note_id));
        }
        notes::get(tx, note_id)?.ok_or_else(|| note_not_found(note_id))
    })
}

/// Soft delete: the note goes to the trash and can be restored for 30 days.
pub fn delete(state: &AppState, id: &str) -> AppResult<()> {
    let now = state.clock.now_string();
    state.db.write(|tx| {
        if notes::soft_delete(tx, id, &now)? {
            Ok(())
        } else {
            Err(note_not_found(id))
        }
    })
}

pub fn restore(state: &AppState, id: &str) -> AppResult<Note> {
    let now = state.clock.now_string();
    state
        .db
        .write(|tx| notes::restore(tx, id, &now)?.ok_or_else(|| note_not_found(id)))
}

#[cfg(test)]
pub fn trash(state: &AppState) -> AppResult<Vec<NoteSummary>> {
    trash_page(state, Page::ALL)
}

pub fn trash_page(state: &AppState, page: Page) -> AppResult<Vec<NoteSummary>> {
    state
        .db
        .read(|conn| Ok(notes::list_trash_page(conn, page)?))
}

/// A note of the trash with its document, to read it before restoring it.
pub fn trashed(state: &AppState, id: &str) -> AppResult<Note> {
    state
        .db
        .read(|conn| notes::get_trashed(conn, id)?.ok_or_else(|| note_not_found(id)))
}

/// Notes that link to `id` (D-46).
pub fn backlinks(state: &AppState, id: &str) -> AppResult<Vec<NoteSummary>> {
    state.db.read(|conn| {
        notes::get(conn, id)?.ok_or_else(|| note_not_found(id))?;
        Ok(notes::backlinks(conn, id)?)
    })
}

/// Notes without links to or from any other note.
pub fn orphans(state: &AppState) -> AppResult<Vec<NoteSummary>> {
    state.db.read(|conn| Ok(notes::orphans(conn)?))
}

#[cfg(test)]
mod tests;

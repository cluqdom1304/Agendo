//! Note commands (D-43 to D-46). Same shape as `tasks.rs`: thin async wrappers over plain, tested
//! functions. Saving a document derives everything from it in one place: the search text and the
//! references to tasks and notes.

use std::sync::Arc;

use serde::Deserialize;
use tauri::State;

use crate::{
    domain::{content, dates, validation},
    error::{AppError, AppResult},
    repo::{
        attachments,
        model::{NewNote, Note, NoteContent, NotePatch, NoteSummary, NoteTitle},
        note_tags,
        notes::{self, Scope, Sort},
        versions,
    },
    state::{AppState, run_blocking},
};

fn note_not_found(id: &str) -> AppError {
    AppError::NotFound(format!("nota {id} no encontrada"))
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

pub fn list(state: &AppState, input: NotesListInput) -> AppResult<Vec<NoteSummary>> {
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
    state
        .db
        .read(|conn| Ok(notes::list(conn, scope, input.tag_id.as_deref(), sort)?))
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

pub fn trash(state: &AppState) -> AppResult<Vec<NoteSummary>> {
    state.db.read(|conn| Ok(notes::list_trash(conn)?))
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

#[tauri::command]
pub async fn note_backlinks(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Vec<NoteSummary>, String> {
    run_blocking(state.inner().clone(), move |state| backlinks(state, &id)).await
}

#[tauri::command]
pub async fn notes_orphans(state: State<'_, Arc<AppState>>) -> Result<Vec<NoteSummary>, String> {
    run_blocking(state.inner().clone(), orphans).await
}

#[tauri::command]
pub async fn notes_list(
    state: State<'_, Arc<AppState>>,
    scope: NotesScope,
    folder_id: Option<String>,
    tag_id: Option<String>,
    sort: Option<NotesSort>,
) -> Result<Vec<NoteSummary>, String> {
    let input = NotesListInput {
        scope,
        folder_id,
        tag_id,
        sort: sort.unwrap_or_default(),
    };
    run_blocking(state.inner().clone(), move |state| list(state, input)).await
}

#[tauri::command]
pub async fn note_titles(
    state: State<'_, Arc<AppState>>,
    ids: Vec<String>,
) -> Result<Vec<NoteTitle>, String> {
    run_blocking(state.inner().clone(), move |state| titles(state, &ids)).await
}

#[tauri::command]
pub async fn note_get(state: State<'_, Arc<AppState>>, id: String) -> Result<Note, String> {
    run_blocking(state.inner().clone(), move |state| get(state, &id)).await
}

#[tauri::command]
pub async fn note_create(
    state: State<'_, Arc<AppState>>,
    folder_id: Option<String>,
    title: Option<String>,
    content_json: Option<String>,
    tag_ids: Option<Vec<String>>,
) -> Result<Note, String> {
    let input = NoteCreateInput {
        folder_id,
        title,
        content_json,
        tag_ids,
    };
    run_blocking(state.inner().clone(), move |state| create(state, input)).await
}

#[tauri::command]
pub async fn note_daily_open(
    state: State<'_, Arc<AppState>>,
    date: String,
    title: String,
) -> Result<Note, String> {
    run_blocking(state.inner().clone(), move |state| {
        daily_open(state, &date, &title)
    })
    .await
}

#[tauri::command]
pub async fn note_update(
    state: State<'_, Arc<AppState>>,
    id: String,
    patch: NotePatchInput,
) -> Result<Note, String> {
    run_blocking(state.inner().clone(), move |state| {
        update(state, &id, patch)
    })
    .await
}

#[tauri::command]
pub async fn note_set_pinned(
    state: State<'_, Arc<AppState>>,
    id: String,
    pinned: bool,
) -> Result<Note, String> {
    run_blocking(state.inner().clone(), move |state| {
        set_pinned(state, &id, pinned)
    })
    .await
}

#[tauri::command]
pub async fn note_move(
    state: State<'_, Arc<AppState>>,
    id: String,
    folder_id: Option<String>,
) -> Result<Note, String> {
    run_blocking(state.inner().clone(), move |state| {
        move_to_folder(state, &id, folder_id.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn note_tags_set(
    state: State<'_, Arc<AppState>>,
    note_id: String,
    tag_ids: Vec<String>,
) -> Result<Note, String> {
    run_blocking(state.inner().clone(), move |state| {
        set_note_tags(state, &note_id, &tag_ids)
    })
    .await
}

#[tauri::command]
pub async fn note_delete(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    run_blocking(state.inner().clone(), move |state| delete(state, &id)).await
}

#[tauri::command]
pub async fn note_restore(state: State<'_, Arc<AppState>>, id: String) -> Result<Note, String> {
    run_blocking(state.inner().clone(), move |state| restore(state, &id)).await
}

#[tauri::command]
pub async fn note_trash_get(state: State<'_, Arc<AppState>>, id: String) -> Result<Note, String> {
    run_blocking(state.inner().clone(), move |state| trashed(state, &id)).await
}

#[tauri::command]
pub async fn notes_trash_list(state: State<'_, Arc<AppState>>) -> Result<Vec<NoteSummary>, String> {
    run_blocking(state.inner().clone(), trash).await
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::{
        commands::{
            folders::{self, FolderUpsertInput},
            tags::{self, TagUpsertInput},
            tasks::{self, TaskCreateInput},
        },
        repo::model::EMPTY_DOC,
        state::test_state,
    };

    fn code_of<T: std::fmt::Debug>(result: AppResult<T>) -> String {
        result.expect_err("must fail").to_string()
    }

    fn make(state: &AppState, title: &str) -> Note {
        create(
            state,
            NoteCreateInput {
                title: Some(title.into()),
                ..NoteCreateInput::default()
            },
        )
        .expect("note")
    }

    fn doc(body: &str) -> String {
        format!(r#"{{"type":"doc","content":[{body}]}}"#)
    }

    #[test]
    fn titles_returns_live_notes_only_in_one_call() {
        let state = test_state().0;
        let a = make(&state, "Álgebra");
        let b = make(&state, "Cálculo");
        let gone = make(&state, "Borrada");
        delete(&state, &gone.id).expect("trash");

        let ids = vec![
            b.id.clone(),
            a.id.clone(),
            gone.id.clone(),
            "unknown".into(),
        ];
        let found = titles(&state, &ids).expect("titles");
        let mut expected = vec![
            NoteTitle {
                id: a.id.clone(),
                title: "Álgebra".into(),
            },
            NoteTitle {
                id: b.id.clone(),
                title: "Cálculo".into(),
            },
        ];
        expected.sort_by(|x, y| x.id.cmp(&y.id));
        assert_eq!(found, expected);
        assert!(titles(&state, &[]).expect("empty").is_empty());
    }

    #[test]
    fn titles_rejects_more_ids_than_the_limit() {
        let state = test_state().0;
        let ids: Vec<String> = (0..=MAX_TITLE_IDS).map(|i| format!("n{i}")).collect();
        assert!(code_of(titles(&state, &ids)).starts_with("VALIDATION:"));
    }

    fn all(state: &AppState) -> Vec<NoteSummary> {
        list(
            state,
            NotesListInput {
                scope: NotesScope::All,
                folder_id: None,
                tag_id: None,
                sort: NotesSort::Updated,
            },
        )
        .expect("list")
    }

    #[test]
    fn create_uses_defaults_and_a_generated_id() {
        let (state, _) = test_state();
        let note = create(&state, NoteCreateInput::default()).expect("create");
        assert_eq!(note.id, "id-1");
        assert_eq!(note.title, "");
        assert_eq!(note.content_json, EMPTY_DOC);
        assert_eq!(note.folder_id, None);
        assert!(!note.is_pinned);
        assert_eq!(note.created_at, "2026-03-10T10:00:00Z");
    }

    #[test]
    fn create_validates_title_document_and_known_ids() {
        let (state, _) = test_state();
        let long = "x".repeat(501);
        assert!(
            code_of(create(
                &state,
                NoteCreateInput {
                    title: Some(long),
                    ..NoteCreateInput::default()
                }
            ))
            .starts_with("VALIDATION: el título no puede superar 500")
        );
        assert!(
            code_of(create(
                &state,
                NoteCreateInput {
                    content_json: Some("not json".into()),
                    ..NoteCreateInput::default()
                }
            ))
            .starts_with("VALIDATION:")
        );
        assert_eq!(
            code_of(create(
                &state,
                NoteCreateInput {
                    folder_id: Some("ghost".into()),
                    ..NoteCreateInput::default()
                }
            )),
            "NOT_FOUND: el elemento referenciado no existe"
        );
        assert!(all(&state).is_empty(), "nothing was stored");
    }

    #[test]
    fn the_search_text_is_derived_from_the_document() {
        let (state, _) = test_state();
        let note = create(
            &state,
            NoteCreateInput {
                content_json: Some(doc(
                    r#"{"type":"paragraph","content":[{"type":"text","text":"Preparar el calendario"}]}"#,
                )),
                ..NoteCreateInput::default()
            },
        )
        .expect("create");
        let found: Vec<String> = state
            .db
            .read(|conn| {
                Ok(conn
                    .prepare("SELECT note_id FROM notes_fts WHERE notes_fts MATCH 'calendario'")?
                    .query_map([], |row| row.get(0))?
                    .collect::<Result<_, _>>()?)
            })
            .expect("search");
        assert_eq!(found, [note.id]);
    }

    #[test]
    fn update_changes_fields_and_rejects_bad_input_or_unknown_notes() {
        let (state, clock) = test_state();
        let note = make(&state, "Antes");
        clock.advance(Duration::minutes(5));

        let updated = update(
            &state,
            &note.id,
            NotePatchInput {
                title: Some("Después".into()),
                content_json: Some(doc(r#"{"type":"paragraph"}"#)),
            },
        )
        .expect("update");
        assert_eq!(updated.title, "Después");
        assert!(updated.content_json.contains("paragraph"));
        assert_eq!(updated.updated_at, "2026-03-10T10:05:00Z");

        let untouched = update(&state, &note.id, NotePatchInput::default()).expect("noop");
        assert_eq!(untouched.updated_at, "2026-03-10T10:05:00Z");
        assert!(
            code_of(update(
                &state,
                &note.id,
                NotePatchInput {
                    content_json: Some("{}".into()),
                    ..NotePatchInput::default()
                }
            ))
            .starts_with("VALIDATION:")
        );
        assert_eq!(
            code_of(update(&state, "ghost", NotePatchInput::default())),
            "NOT_FOUND: nota ghost no encontrada"
        );
    }

    #[test]
    fn references_to_tasks_and_notes_are_recorded_and_refreshed() {
        let (state, _) = test_state();
        let target = make(&state, "Destino");
        let task = tasks::create(&state, TaskCreateInput::default()).expect("task");
        let source = make(&state, "Origen");
        let refs = |state: &AppState| -> (i64, i64) {
            state
                .db
                .read(|conn| {
                    Ok((
                        conn.query_row("SELECT COUNT(*) FROM note_tasks", [], |r| r.get(0))?,
                        conn.query_row("SELECT COUNT(*) FROM note_links", [], |r| r.get(0))?,
                    ))
                })
                .expect("counts")
        };

        let body = doc(&format!(
            r#"{{"type":"taskRef","attrs":{{"taskId":"{}"}}}},{{"type":"paragraph","content":[{{"type":"noteLink","attrs":{{"noteId":"{}"}}}}]}}"#,
            task.id, target.id
        ));
        update(
            &state,
            &source.id,
            NotePatchInput {
                content_json: Some(body),
                ..NotePatchInput::default()
            },
        )
        .expect("save");
        assert_eq!(refs(&state), (1, 1));

        update(
            &state,
            &source.id,
            NotePatchInput {
                content_json: Some(doc(r#"{"type":"paragraph"}"#)),
                ..NotePatchInput::default()
            },
        )
        .expect("save");
        assert_eq!(refs(&state), (0, 0));
    }

    #[test]
    fn a_task_created_from_a_note_keeps_its_origin_and_outlives_the_note() {
        let (state, _) = test_state();
        let note = make(&state, "Reunión");
        let task = tasks::create(
            &state,
            TaskCreateInput {
                title: Some("Enviar acta".into()),
                source_note_id: Some(note.id.clone()),
                ..TaskCreateInput::default()
            },
        )
        .expect("task");
        assert_eq!(task.source_note_id.as_deref(), Some(note.id.as_str()));

        state
            .db
            .write(|tx| Ok(tx.execute("DELETE FROM notes WHERE id = ?1", [&note.id])?))
            .expect("hard delete");
        let after = state
            .db
            .read(|conn| Ok(crate::repo::tasks::get(conn, &task.id)?))
            .expect("get")
            .expect("task survives");
        assert_eq!(after.source_note_id, None);

        assert_eq!(
            code_of(tasks::create(
                &state,
                TaskCreateInput {
                    source_note_id: Some("ghost".into()),
                    ..TaskCreateInput::default()
                }
            )),
            "NOT_FOUND: el elemento referenciado no existe"
        );
    }

    #[test]
    fn notes_and_tasks_share_the_same_tags() {
        let (state, _) = test_state();
        let tag = tags::upsert(
            &state,
            TagUpsertInput {
                id: None,
                name: "Física".into(),
                color_hex: "#0A84FF".into(),
                parent_id: None,
            },
        )
        .expect("tag");
        let note = make(&state, "Apuntes");
        let task = tasks::create(&state, TaskCreateInput::default()).expect("task");

        let tagged = set_note_tags(&state, &note.id, std::slice::from_ref(&tag.id)).expect("tag");
        tags::set_task_tags(&state, &task.id, std::slice::from_ref(&tag.id)).expect("tag");
        assert_eq!(tagged.tag_ids, std::slice::from_ref(&tag.id));
        assert_eq!(
            code_of(set_note_tags(&state, &note.id, &["nope".to_string()])),
            "NOT_FOUND: el elemento referenciado no existe"
        );
        assert_eq!(
            code_of(set_note_tags(&state, "ghost", &[])),
            "NOT_FOUND: nota ghost no encontrada"
        );

        tags::delete(&state, &tag.id).expect("delete tag");
        assert!(get(&state, &note.id).expect("note").tag_ids.is_empty());
    }

    #[test]
    fn list_scopes_sorting_and_the_folder_requirement() {
        let (state, clock) = test_state();
        let folder = folders::upsert(
            &state,
            FolderUpsertInput {
                id: None,
                name: "Trabajo".into(),
                parent_id: None,
            },
        )
        .expect("folder");
        let inbox = make(&state, "b inbox");
        clock.advance(Duration::minutes(1));
        let filed = create(
            &state,
            NoteCreateInput {
                title: Some("a filed".into()),
                folder_id: Some(folder.id.clone()),
                ..NoteCreateInput::default()
            },
        )
        .expect("note");
        let list_of = |scope, folder_id: Option<&str>, sort| {
            list(
                &state,
                NotesListInput {
                    scope,
                    folder_id: folder_id.map(str::to_string),
                    tag_id: None,
                    sort,
                },
            )
            .expect("list")
            .into_iter()
            .map(|note| note.id)
            .collect::<Vec<_>>()
        };

        assert_eq!(
            list_of(NotesScope::All, None, NotesSort::Updated),
            [filed.id.clone(), inbox.id.clone()]
        );
        assert_eq!(
            list_of(NotesScope::All, None, NotesSort::Title),
            [filed.id.clone(), inbox.id.clone()]
        );
        assert_eq!(
            list_of(NotesScope::Inbox, None, NotesSort::Updated),
            [inbox.id]
        );
        assert_eq!(
            list_of(NotesScope::Folder, Some(&folder.id), NotesSort::Updated),
            std::slice::from_ref(&filed.id)
        );
        assert!(list_of(NotesScope::Pinned, None, NotesSort::Updated).is_empty());
        set_pinned(&state, &filed.id, true).expect("pin");
        assert_eq!(
            list_of(NotesScope::Pinned, None, NotesSort::Updated),
            [filed.id]
        );
        assert_eq!(
            code_of(list(
                &state,
                NotesListInput {
                    scope: NotesScope::Folder,
                    folder_id: None,
                    tag_id: None,
                    sort: NotesSort::Updated,
                }
            )),
            "VALIDATION: la vista de carpeta necesita «folderId»"
        );
    }

    #[test]
    fn move_to_folder_and_back_to_the_inbox() {
        let (state, _) = test_state();
        let folder = folders::upsert(
            &state,
            FolderUpsertInput {
                id: None,
                name: "Casa".into(),
                parent_id: None,
            },
        )
        .expect("folder");
        let note = make(&state, "Una");

        let moved = move_to_folder(&state, &note.id, Some(&folder.id)).expect("move");
        assert_eq!(moved.folder_id.as_deref(), Some(folder.id.as_str()));
        assert_eq!(
            move_to_folder(&state, &note.id, None)
                .expect("inbox")
                .folder_id,
            None
        );
        assert_eq!(
            code_of(move_to_folder(&state, &note.id, Some("ghost"))),
            "NOT_FOUND: el elemento referenciado no existe"
        );
        assert_eq!(
            code_of(move_to_folder(&state, "ghost", None)),
            "NOT_FOUND: nota ghost no encontrada"
        );
    }

    #[test]
    fn delete_and_restore_go_through_the_trash() {
        let (state, clock) = test_state();
        let note = make(&state, "Una");
        delete(&state, &note.id).expect("delete");
        assert!(all(&state).is_empty());
        assert_eq!(trash(&state).expect("trash").len(), 1);
        assert_eq!(
            code_of(get(&state, &note.id)),
            format!("NOT_FOUND: nota {} no encontrada", note.id)
        );
        assert_eq!(trashed(&state, &note.id).expect("trashed").title, "Una");
        assert_eq!(
            code_of(delete(&state, &note.id)),
            format!("NOT_FOUND: nota {} no encontrada", note.id)
        );

        clock.advance(Duration::minutes(1));
        let restored = restore(&state, &note.id).expect("restore");
        assert_eq!(restored.updated_at, "2026-03-10T10:01:00Z");
        assert_eq!(all(&state).len(), 1);
        assert!(trash(&state).expect("trash").is_empty());
        assert_eq!(
            code_of(trashed(&state, &note.id)),
            format!("NOT_FOUND: nota {} no encontrada", note.id)
        );
        assert_eq!(
            code_of(restore(&state, &note.id)),
            format!("NOT_FOUND: nota {} no encontrada", note.id)
        );
    }

    #[test]
    fn the_trash_is_purged_after_thirty_days_together_with_tasks() {
        let (state, clock) = test_state();
        let note = make(&state, "Vieja");
        delete(&state, &note.id).expect("delete");
        clock.advance(Duration::days(31));
        assert_eq!(state.purge_trash().expect("purge"), 1);
        assert!(trash(&state).expect("trash").is_empty());
    }
    #[test]
    fn the_daily_note_is_created_once_per_day() {
        let (state, _) = test_state();
        let first = daily_open(&state, "2026-03-05", "5 de marzo").expect("first");
        let again = daily_open(&state, "2026-03-05", "otro título").expect("again");
        assert_eq!(first.id, again.id);
        assert_eq!(again.title, "5 de marzo");
        let other = daily_open(&state, "2026-03-06", "6 de marzo").expect("other");
        assert_ne!(first.id, other.id);
        assert_eq!(all(&state).len(), 2);
    }

    #[test]
    fn a_trashed_daily_note_comes_back_instead_of_a_second_one() {
        let (state, _) = test_state();
        let note = daily_open(&state, "2026-03-05", "5 de marzo").expect("note");
        delete(&state, &note.id).expect("delete");
        let back = daily_open(&state, "2026-03-05", "5 de marzo").expect("back");
        assert_eq!(back.id, note.id);
        assert_eq!(all(&state).len(), 1);
    }

    #[test]
    fn the_daily_note_rejects_a_bad_date_or_title() {
        let (state, _) = test_state();
        assert!(code_of(daily_open(&state, "05/03/2026", "x")).starts_with("VALIDATION"));
        assert!(
            code_of(daily_open(&state, "2026-03-05", &"a".repeat(501))).starts_with("VALIDATION")
        );
    }
}

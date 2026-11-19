//! Tauri commands of `services::notes` (SPEC §7.3): they take the IPC arguments and run the
//! use case on a blocking thread.

use std::sync::Arc;

use tauri::State;

use crate::{
    commands::page_of,
    repo::model::{Note, NoteSummary, NoteTitle},
    services::notes::*,
    state::{AppState, run_blocking},
};

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
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<NoteSummary>, String> {
    let input = NotesListInput {
        scope,
        folder_id,
        tag_id,
        sort: sort.unwrap_or_default(),
    };
    run_blocking(state.inner().clone(), move |state| {
        list_page(state, input, page_of(limit, offset)?)
    })
    .await
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
pub async fn notes_trash_list(
    state: State<'_, Arc<AppState>>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<NoteSummary>, String> {
    run_blocking(state.inner().clone(), move |state| {
        trash_page(state, page_of(limit, offset)?)
    })
    .await
}

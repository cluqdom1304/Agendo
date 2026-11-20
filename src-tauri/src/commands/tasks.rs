//! Tauri commands of `services::tasks` (SPEC §7.3): they take the IPC arguments and run the
//! use case on a blocking thread.

use std::sync::Arc;

use tauri::State;

use crate::{
    commands::page_of,
    repo::model::Task,
    services::tasks::*,
    state::{AppState, run_blocking},
};

#[tauri::command]
pub async fn tasks_list(
    state: State<'_, Arc<AppState>>,
    view: ListView,
    from: Option<String>,
    to: Option<String>,
    tag_id: Option<String>,
) -> Result<TasksListOutput, String> {
    let input = TasksListInput {
        view,
        from,
        to,
        tag_id,
    };
    run_blocking(state.inner().clone(), move |state| list(state, input)).await
}

#[tauri::command]
pub async fn tasks_list_all(
    state: State<'_, Arc<AppState>>,
    tag_id: Option<String>,
) -> Result<Sections, String> {
    run_blocking(state.inner().clone(), move |state| {
        list_all(state, tag_id.as_deref())
    })
    .await
}

// The IPC contract passes each field as its own argument (SPEC §7.3).
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn task_create(
    state: State<'_, Arc<AppState>>,
    title: Option<String>,
    due_date: Option<String>,
    due_time: Option<String>,
    duration_minutes: Option<i64>,
    tag_ids: Option<Vec<String>>,
    content_json: Option<String>,
    source_note_id: Option<String>,
    rrule: Option<String>,
    priority: Option<i64>,
) -> Result<Task, String> {
    let input = TaskCreateInput {
        title,
        due_date,
        due_time,
        duration_minutes,
        tag_ids,
        content_json,
        source_note_id,
        rrule,
        priority,
    };
    run_blocking(state.inner().clone(), move |state| create(state, input)).await
}

#[tauri::command]
pub async fn task_update(
    state: State<'_, Arc<AppState>>,
    id: String,
    patch: TaskPatchInput,
) -> Result<Task, String> {
    run_blocking(state.inner().clone(), move |state| {
        update(state, &id, patch)
    })
    .await
}

#[tauri::command]
pub async fn task_set_completed(
    state: State<'_, Arc<AppState>>,
    id: String,
    completed: bool,
    completed_date: Option<String>,
) -> Result<Task, String> {
    run_blocking(state.inner().clone(), move |state| {
        complete(state, &id, completed, completed_date.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn task_move(
    state: State<'_, Arc<AppState>>,
    id: String,
    to_section: Section,
    date: Option<String>,
    before_id: Option<String>,
) -> Result<Task, String> {
    run_blocking(state.inner().clone(), move |state| {
        move_task(state, &id, to_section, date, before_id)
    })
    .await
}

#[tauri::command]
pub async fn task_delete(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    run_blocking(state.inner().clone(), move |state| delete(state, &id)).await
}

#[tauri::command]
pub async fn task_restore(state: State<'_, Arc<AppState>>, id: String) -> Result<Task, String> {
    run_blocking(state.inner().clone(), move |state| restore(state, &id)).await
}

#[tauri::command]
pub async fn trash_list(
    state: State<'_, Arc<AppState>>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<Task>, String> {
    run_blocking(state.inner().clone(), move |state| {
        trash_page(state, page_of(limit, offset)?)
    })
    .await
}

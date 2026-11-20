//! Tauri commands of `services::focus` (D-97): they take the IPC arguments and run the use case on
//! a blocking thread.

use std::sync::Arc;

use tauri::State;

use crate::{
    repo::model::{TaskFocusTotal, TimeEntry},
    services::focus,
    state::{AppState, run_blocking},
};

#[tauri::command]
pub async fn focus_current(state: State<'_, Arc<AppState>>) -> Result<Option<TimeEntry>, String> {
    run_blocking(state.inner().clone(), focus::current).await
}

#[tauri::command]
pub async fn focus_start(
    state: State<'_, Arc<AppState>>,
    task_id: String,
    planned_minutes: i64,
) -> Result<TimeEntry, String> {
    run_blocking(state.inner().clone(), move |state| {
        focus::start(state, &task_id, planned_minutes)
    })
    .await
}

#[tauri::command]
pub async fn focus_stop(state: State<'_, Arc<AppState>>) -> Result<Option<TimeEntry>, String> {
    run_blocking(state.inner().clone(), focus::stop).await
}

#[tauri::command]
pub async fn focus_list(
    state: State<'_, Arc<AppState>>,
    from: String,
    to: String,
) -> Result<Vec<TimeEntry>, String> {
    run_blocking(state.inner().clone(), move |state| {
        focus::list_local(state, &from, &to)
    })
    .await
}

#[tauri::command]
pub async fn focus_totals(state: State<'_, Arc<AppState>>) -> Result<Vec<TaskFocusTotal>, String> {
    run_blocking(state.inner().clone(), focus::totals).await
}

#[tauri::command]
pub async fn focus_task_entries(
    state: State<'_, Arc<AppState>>,
    task_id: String,
) -> Result<Vec<TimeEntry>, String> {
    run_blocking(state.inner().clone(), move |state| {
        focus::of_task(state, &task_id)
    })
    .await
}

#[tauri::command]
pub async fn focus_add_manual(
    state: State<'_, Arc<AppState>>,
    task_id: String,
    minutes: i64,
) -> Result<TimeEntry, String> {
    run_blocking(state.inner().clone(), move |state| {
        focus::add_manual(state, &task_id, minutes)
    })
    .await
}

#[tauri::command]
pub async fn focus_delete_entry(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    run_blocking(state.inner().clone(), move |state| {
        focus::delete_entry(state, &id)
    })
    .await
}

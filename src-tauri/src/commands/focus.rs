//! Tauri commands of `services::focus` (D-97): they take the IPC arguments and run the use case on
//! a blocking thread.

use std::sync::Arc;

use tauri::State;

use crate::{
    repo::model::TimeEntry,
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

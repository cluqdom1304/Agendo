//! Tauri commands of `services::habits` (D-95): they take the IPC arguments and run the use case
//! on a blocking thread.

use std::sync::Arc;

use chrono::Local;
use tauri::State;

use crate::{
    repo::model::{Habit, HabitLog},
    services::habits::*,
    state::{AppState, run_blocking},
};

#[tauri::command]
pub async fn habits_list(
    state: State<'_, Arc<AppState>>,
    include_archived: Option<bool>,
) -> Result<Vec<Habit>, String> {
    run_blocking(state.inner().clone(), move |state| {
        list(state, include_archived.unwrap_or(false))
    })
    .await
}

#[tauri::command]
pub async fn habits_trash_list(state: State<'_, Arc<AppState>>) -> Result<Vec<Habit>, String> {
    run_blocking(state.inner().clone(), |state| trash_list_in(state, &Local)).await
}

#[tauri::command]
pub async fn habit_create(
    state: State<'_, Arc<AppState>>,
    input: HabitInput,
) -> Result<Habit, String> {
    run_blocking(state.inner().clone(), move |state| create(state, input)).await
}

#[tauri::command]
pub async fn habit_update(
    state: State<'_, Arc<AppState>>,
    id: String,
    patch: HabitPatchInput,
) -> Result<Habit, String> {
    run_blocking(state.inner().clone(), move |state| {
        update(state, &id, patch)
    })
    .await
}

#[tauri::command]
pub async fn habit_delete(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    run_blocking(state.inner().clone(), move |state| delete(state, &id)).await
}

#[tauri::command]
pub async fn habit_restore(state: State<'_, Arc<AppState>>, id: String) -> Result<Habit, String> {
    run_blocking(state.inner().clone(), move |state| restore(state, &id)).await
}

#[tauri::command]
pub async fn habit_set_done(
    state: State<'_, Arc<AppState>>,
    id: String,
    date: String,
    done: bool,
) -> Result<Habit, String> {
    run_blocking(state.inner().clone(), move |state| {
        set_done(state, &id, &date, done, &Local)
    })
    .await
}

#[tauri::command]
pub async fn habit_logs(
    state: State<'_, Arc<AppState>>,
    from: String,
    to: String,
) -> Result<Vec<HabitLog>, String> {
    run_blocking(state.inner().clone(), move |state| logs(state, &from, &to)).await
}

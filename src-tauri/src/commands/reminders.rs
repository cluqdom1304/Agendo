//! Tauri commands of `services::reminders` (SPEC §7.3): they take the IPC arguments and run the
//! use case on a blocking thread.

use std::sync::Arc;

use chrono::Local;
use tauri::State;

use crate::{
    repo::model::Task,
    services::reminders::*,
    state::{AppState, run_blocking},
};

#[tauri::command]
pub async fn reminders_set(
    state: State<'_, Arc<AppState>>,
    task_id: String,
    reminders: Vec<ReminderInput>,
) -> Result<Task, String> {
    run_blocking(state.inner().clone(), move |state| {
        set(state, &task_id, &reminders, &Local)
    })
    .await
}

//! Tauri commands of `services::events` (D-94): they take the IPC arguments and run the use case
//! on a blocking thread.

use std::sync::Arc;

use tauri::State;

use crate::{
    repo::model::{Event, EventOccurrence},
    services::events::*,
    state::{AppState, run_blocking},
};

#[tauri::command]
pub async fn events_list(
    state: State<'_, Arc<AppState>>,
    from: String,
    to: String,
) -> Result<Vec<EventOccurrence>, String> {
    run_blocking(state.inner().clone(), move |state| list(state, &from, &to)).await
}

#[tauri::command]
pub async fn events_trash_list(state: State<'_, Arc<AppState>>) -> Result<Vec<Event>, String> {
    run_blocking(state.inner().clone(), trash_list).await
}

#[tauri::command]
pub async fn event_get(state: State<'_, Arc<AppState>>, id: String) -> Result<Event, String> {
    run_blocking(state.inner().clone(), move |state| get(state, &id)).await
}

#[tauri::command]
pub async fn event_create(
    state: State<'_, Arc<AppState>>,
    input: EventInput,
) -> Result<Event, String> {
    run_blocking(state.inner().clone(), move |state| create(state, input)).await
}

#[tauri::command]
pub async fn event_update(
    state: State<'_, Arc<AppState>>,
    id: String,
    patch: EventPatchInput,
) -> Result<Event, String> {
    run_blocking(state.inner().clone(), move |state| {
        update(state, &id, patch)
    })
    .await
}

#[tauri::command]
pub async fn event_delete(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    run_blocking(state.inner().clone(), move |state| delete(state, &id)).await
}

#[tauri::command]
pub async fn event_restore(state: State<'_, Arc<AppState>>, id: String) -> Result<Event, String> {
    run_blocking(state.inner().clone(), move |state| restore(state, &id)).await
}

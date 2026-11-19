//! Tauri commands of `services::tags` (SPEC §7.3): they take the IPC arguments and run the
//! use case on a blocking thread.

use std::sync::Arc;

use tauri::State;

use crate::{
    repo::model::{Tag, TagWithCount, Task},
    services::tags::*,
    state::{AppState, run_blocking},
};

#[tauri::command]
pub async fn tag_list(state: State<'_, Arc<AppState>>) -> Result<Vec<TagWithCount>, String> {
    run_blocking(state.inner().clone(), list).await
}

#[tauri::command]
pub async fn tag_upsert(
    state: State<'_, Arc<AppState>>,
    id: Option<String>,
    name: String,
    color_hex: String,
    parent_id: Option<String>,
) -> Result<Tag, String> {
    let input = TagUpsertInput {
        id,
        name,
        color_hex,
        parent_id,
    };
    run_blocking(state.inner().clone(), move |state| upsert(state, input)).await
}

#[tauri::command]
pub async fn tag_delete(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    run_blocking(state.inner().clone(), move |state| delete(state, &id)).await
}

#[tauri::command]
pub async fn tag_merge(
    state: State<'_, Arc<AppState>>,
    source_id: String,
    target_id: String,
) -> Result<(), String> {
    run_blocking(state.inner().clone(), move |state| {
        merge(state, &source_id, &target_id)
    })
    .await
}

#[tauri::command]
pub async fn task_tags_set(
    state: State<'_, Arc<AppState>>,
    task_id: String,
    tag_ids: Vec<String>,
) -> Result<Task, String> {
    run_blocking(state.inner().clone(), move |state| {
        set_task_tags(state, &task_id, &tag_ids)
    })
    .await
}

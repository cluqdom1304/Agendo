//! Commands of the weekly summary (D-100): the figures and their Markdown export. The file dialog
//! lives in the frontend, so the export receives the chosen path.

use std::{path::PathBuf, sync::Arc};

use tauri::{AppHandle, State};

use crate::{
    DATABASE_FILE,
    commands::backup::protected_files,
    error::AppError,
    exchange,
    repo::model::WeekSummary,
    services::summary,
    state::{AppState, run_blocking},
};

/// The summary of the week that contains `date` (`YYYY-MM-DD`); `first_day` is 0 for Sunday to 6
/// for Saturday.
#[tauri::command]
pub async fn summary_week(
    state: State<'_, Arc<AppState>>,
    date: String,
    first_day: u8,
) -> Result<WeekSummary, String> {
    run_blocking(state.inner().clone(), move |state| {
        summary::week(state, &date, first_day)
    })
    .await
}

#[tauri::command]
pub async fn summary_export_markdown(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    date: String,
    first_day: u8,
    path: String,
) -> Result<(), String> {
    let database = crate::data_dir(&app)
        .map_err(|error| AppError::Io(format!("no se encontró la carpeta de datos: {error}")))?
        .join(DATABASE_FILE);
    let target = PathBuf::from(path);
    if protected_files(&database)
        .iter()
        .any(|file| file == &target)
    {
        return Err(AppError::Validation(
            "no se puede guardar la exportación sobre la base de datos en uso".into(),
        )
        .to_string());
    }
    run_blocking(state.inner().clone(), move |state| {
        exchange::export_summary(state, &date, first_day, &target)
    })
    .await
}

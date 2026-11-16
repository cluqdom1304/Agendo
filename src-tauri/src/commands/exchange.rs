//! Export and import commands (F14). The file dialogs live in the frontend; these commands
//! receive the chosen paths and run the work off the UI thread.

use std::{path::PathBuf, sync::Arc};

use tauri::{AppHandle, State};

use crate::{
    DATABASE_FILE,
    commands::{attachments::dir, backup::protected_files},
    error::{AppError, AppResult},
    exchange::{self, ExportReport, Format, ImportReport, Selection},
    state::{AppState, run_blocking},
};

fn format_of(text: &str) -> AppResult<Format> {
    match text {
        "markdown" => Ok(Format::Markdown),
        "html" => Ok(Format::Html),
        _ => Err(AppError::Validation(format!("formato desconocido: {text}"))),
    }
}

fn selection_of(kind: &str, id: Option<String>) -> AppResult<Selection> {
    match (kind, id) {
        ("all", _) => Ok(Selection::All),
        ("folder", Some(id)) => Ok(Selection::Folder(id)),
        ("note", Some(id)) => Ok(Selection::Note(id)),
        _ => Err(AppError::Validation(
            "la selección a exportar no es válida".into(),
        )),
    }
}

/// `format` is `markdown` or `html`; `kind` is `all`, `folder` or `note` (the last two need
/// `id`); `path` is the folder the files are written into.
#[tauri::command]
pub async fn export_notes(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    format: String,
    kind: String,
    id: Option<String>,
    path: String,
) -> Result<ExportReport, String> {
    let format = format_of(&format)?;
    let selection = selection_of(&kind, id)?;
    let attachments = dir(&app)?;
    run_blocking(state.inner().clone(), move |state| {
        exchange::export_notes(
            state,
            &attachments,
            &PathBuf::from(path),
            format,
            &selection,
        )
    })
    .await
}

#[tauri::command]
pub async fn export_json(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    path: String,
) -> Result<ExportReport, String> {
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
        exchange::export_json(state, &target)
    })
    .await
}

/// `path` is a folder, or one `.md` / `.markdown` / `.txt` file.
#[tauri::command]
pub async fn import_markdown(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    path: String,
) -> Result<ImportReport, String> {
    let attachments = dir(&app)?;
    run_blocking(state.inner().clone(), move |state| {
        exchange::import_text_files(state, &attachments, &PathBuf::from(path))
    })
    .await
}

#[tauri::command]
pub async fn import_enex(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    paths: Vec<String>,
) -> Result<ImportReport, String> {
    let attachments = dir(&app)?;
    let files: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    run_blocking(state.inner().clone(), move |state| {
        exchange::import_enex_files(state, &attachments, &files)
    })
    .await
}

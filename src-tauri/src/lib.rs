mod commands;
#[cfg(test)]
mod contract_tests;
pub mod db;
pub mod domain;
pub mod error;
pub mod ids;
pub mod notify;
pub mod repo;
pub mod scheduler;
mod startup;
pub mod state;
#[cfg(test)]
mod test_support;
mod window;

use std::sync::Arc;

use tauri::{Manager, webview::PageLoadEvent};

use crate::{
    notify::{TauriFiredSink, TauriNotifier},
    scheduler::clock::SystemClock,
    state::AppState,
};

/// File name of the SQLite database inside the app data directory.
const DATABASE_FILE: &str = "agenda.sqlite";

/// When set, the app keeps its data in this directory instead of the user's profile. The E2E
/// suite uses it so that tests never touch real data.
const DATA_DIR_ENV: &str = "MIAGENDA_DATA_DIR";

/// The directory for the database and the safety copy of a restore.
pub(crate) fn data_dir<R: tauri::Runtime>(
    app: &impl Manager<R>,
) -> Result<std::path::PathBuf, tauri::Error> {
    match std::env::var_os(DATA_DIR_ENV).filter(|value| !value.is_empty()) {
        Some(dir) => Ok(dir.into()),
        None => app.path().app_data_dir(),
    }
}

// Si Tauri no puede arrancar no hay estado recuperable: abortar con mensaje es correcto.
#[allow(clippy::expect_used)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    startup::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let path = data_dir(app)?.join(DATABASE_FILE);
            let state = AppState::open(&path, Arc::new(SystemClock))?;
            let state = Arc::new(state);
            app.manage(Arc::clone(&state));
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(scheduler::run(
                state,
                Arc::new(TauriNotifier(handle.clone())),
                Arc::new(TauriFiredSink(handle)),
            ));
            startup::mark("setup_done");
            Ok(())
        })
        .on_page_load(|_webview, payload| match payload.event() {
            PageLoadEvent::Started => startup::mark("page_load_started"),
            PageLoadEvent::Finished => startup::mark("page_load_finished"),
        })
        .invoke_handler(tauri::generate_handler![
            commands::startup::startup_mark,
            commands::window::window_set_effect,
            commands::tasks::tasks_list,
            commands::tasks::task_create,
            commands::tasks::task_update,
            commands::tasks::task_set_completed,
            commands::tasks::task_move,
            commands::tasks::task_delete,
            commands::tasks::task_restore,
            commands::tasks::trash_list,
            commands::search::search,
            commands::tags::tag_list,
            commands::tags::tag_upsert,
            commands::tags::tag_delete,
            commands::tags::task_tags_set,
            commands::reminders::reminders_set,
            commands::settings::settings_get,
            commands::settings::settings_set,
            commands::backup::backup_create,
            commands::backup::backup_restore,
            commands::autostart::autostart_get,
            commands::autostart::autostart_set
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

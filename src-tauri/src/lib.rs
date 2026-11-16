pub mod archive;
pub mod attachments;
mod commands;
#[cfg(test)]
mod contract_tests;
pub mod db;
pub mod domain;
pub mod error;
pub mod exchange;
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

/// When set, the app keeps its data in this directory instead of the profile folder. The E2E
/// suite uses it so that tests never touch real data.
const DATA_DIR_ENV: &str = "MIAGENDA_DATA_DIR";

/// Suffix added to the profile folder in debug builds (`tauri dev`), so development never opens or
/// migrates the database of the installed app (D-53).
const DEV_DIR_SUFFIX: &str = ".dev";

/// Picks the data directory: an explicit override wins, then a separate folder for debug builds,
/// and only release builds use the real profile folder.
fn resolve_data_dir(
    override_dir: Option<std::ffi::OsString>,
    profile_dir: std::path::PathBuf,
    debug_build: bool,
) -> std::path::PathBuf {
    if let Some(dir) = override_dir.filter(|value| !value.is_empty()) {
        return dir.into();
    }
    if !debug_build {
        return profile_dir;
    }
    let mut name = profile_dir.file_name().unwrap_or_default().to_os_string();
    name.push(DEV_DIR_SUFFIX);
    profile_dir.with_file_name(name)
}

/// The directory for the database and the safety copy of a restore.
pub(crate) fn data_dir<R: tauri::Runtime>(
    app: &impl Manager<R>,
) -> Result<std::path::PathBuf, tauri::Error> {
    Ok(resolve_data_dir(
        std::env::var_os(DATA_DIR_ENV),
        app.path().app_data_dir()?,
        cfg!(debug_assertions),
    ))
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
            // Files that no note uses any more (past a grace period) are removed at start-up; a
            // failure here is not worth stopping the app for, the next start tries again.
            if let Ok(dir) = commands::attachments::dir(app.handle()) {
                let _ = attachments::collect_orphans(&state, &dir);
            }
            app.manage(Arc::clone(&state));
            let handle = app.handle().clone();
            notify::install_actions(&handle, Arc::clone(&state));
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
            commands::search::search_all,
            commands::tags::tag_list,
            commands::tags::tag_upsert,
            commands::tags::tag_delete,
            commands::tags::tag_merge,
            commands::tags::task_tags_set,
            commands::folders::folder_list,
            commands::folders::folder_upsert,
            commands::folders::folder_delete,
            commands::notes::notes_list,
            commands::notes::note_get,
            commands::notes::note_backlinks,
            commands::notes::notes_orphans,
            commands::notes::note_create,
            commands::note_reminders::note_reminder_get,
            commands::note_reminders::note_reminder_set,
            commands::versions::note_versions_list,
            commands::versions::note_version_get,
            commands::versions::note_version_restore,
            commands::templates::template_list,
            commands::templates::template_save,
            commands::templates::template_delete,
            commands::notes::note_daily_open,
            commands::notes::note_update,
            commands::notes::note_set_pinned,
            commands::notes::note_move,
            commands::notes::note_tags_set,
            commands::notes::note_delete,
            commands::notes::note_restore,
            commands::notes::note_trash_get,
            commands::notes::notes_trash_list,
            commands::reminders::reminders_set,
            commands::settings::settings_get,
            commands::settings::settings_set,
            commands::attachments::attachment_add,
            commands::attachments::attachment_read,
            commands::backup::backup_create,
            commands::backup::backup_restore,
            commands::exchange::export_notes,
            commands::exchange::export_json,
            commands::exchange::import_markdown,
            commands::exchange::import_enex,
            commands::autostart::autostart_get,
            commands::autostart::autostart_set
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod data_dir_tests {
    use super::resolve_data_dir;
    use std::path::PathBuf;

    fn profile() -> PathBuf {
        PathBuf::from("AppData").join("com.cristian.agendo")
    }

    #[test]
    fn release_builds_use_the_profile_folder() {
        assert_eq!(resolve_data_dir(None, profile(), false), profile());
    }

    #[test]
    fn debug_builds_use_a_separate_folder() {
        let dir = resolve_data_dir(None, profile(), true);
        assert_eq!(
            dir,
            PathBuf::from("AppData").join("com.cristian.agendo.dev")
        );
        assert_ne!(dir, profile());
    }

    #[test]
    fn an_explicit_override_wins_in_both_builds() {
        for debug in [true, false] {
            let dir = resolve_data_dir(Some("scratch".into()), profile(), debug);
            assert_eq!(dir, PathBuf::from("scratch"));
        }
    }

    #[test]
    fn an_empty_override_is_ignored() {
        assert_eq!(
            resolve_data_dir(Some("".into()), profile(), false),
            profile()
        );
    }
}

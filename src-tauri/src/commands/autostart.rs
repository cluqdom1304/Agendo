//! Start with Windows (F6-05). Off by default: the app only registers
//! itself when the user asks. The state lives in the OS, not in the database.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::error::AppError;

fn failure(error: impl std::fmt::Display) -> String {
    AppError::Io(format!(
        "no se pudo consultar o cambiar el inicio con Windows: {error}"
    ))
    .to_string()
}

#[tauri::command]
pub fn autostart_get(app: AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(failure)
}

#[tauri::command]
pub fn autostart_set(app: AppHandle, enabled: bool) -> Result<(), String> {
    let launcher = app.autolaunch();
    if enabled {
        launcher.enable()
    } else {
        launcher.disable()
    }
    .map_err(failure)
}

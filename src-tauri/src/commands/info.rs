//! `app_info`: public facts about the installation for the «Información» box of the settings.
//! Everything is read locally; nothing leaves the machine.

use std::{path::Path, sync::Arc};

use rusqlite::Connection;
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::{
    DATABASE_FILE,
    error::{AppError, AppResult},
    state::{AppState, run_blocking},
};

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub tauri_version: String,
    /// `None` when the web view cannot report its version.
    pub webview_version: Option<String>,
    pub sqlite_version: String,
    pub schema_version: i64,
    pub os: String,
    pub arch: String,
    /// `release` for the installed app, `debug` for `tauri dev` (which uses its own data folder).
    pub build: String,
    pub data_dir: String,
    pub database_bytes: u64,
    pub tasks: i64,
    pub notes: i64,
    pub tags: i64,
    pub attachments: i64,
}

fn count(conn: &Connection, sql: &str) -> AppResult<i64> {
    Ok(conn.query_row(sql, [], |row| row.get(0))?)
}

/// Sums up the database; `database` is its file (its size is 0 if it cannot be read).
pub fn collect(conn: &Connection, database: &Path) -> AppResult<AppInfo> {
    let database_bytes = std::fs::metadata(database).map(|m| m.len()).unwrap_or(0);
    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        tauri_version: tauri::VERSION.to_string(),
        webview_version: tauri::webview_version().ok(),
        sqlite_version: rusqlite::version().to_string(),
        schema_version: count(conn, "PRAGMA user_version")?,
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        build: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
        .to_string(),
        data_dir: database
            .parent()
            .map(|dir| dir.display().to_string())
            .unwrap_or_default(),
        database_bytes,
        tasks: count(conn, "SELECT COUNT(*) FROM tasks WHERE deleted_at IS NULL")?,
        notes: count(conn, "SELECT COUNT(*) FROM notes WHERE deleted_at IS NULL")?,
        tags: count(conn, "SELECT COUNT(*) FROM tags")?,
        attachments: count(conn, "SELECT COUNT(*) FROM attachments")?,
    })
}

#[tauri::command]
pub async fn app_info(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<AppInfo, String> {
    let database = crate::data_dir(&app)
        .map_err(|error| {
            AppError::Io(format!("no se encontró la carpeta de datos: {error}")).to_string()
        })?
        .join(DATABASE_FILE);
    run_blocking(state.inner().clone(), move |state| {
        state.db.read(|conn| collect(conn, &database))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::test_state;

    #[test]
    fn counts_only_live_rows_and_reports_the_schema() {
        let (state, _) = test_state();
        let info = state
            .db
            .read(|conn| collect(conn, Path::new("no-such-dir/agenda.sqlite")))
            .expect("la base en memoria responde");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(info.schema_version >= 10);
        assert_eq!((info.tasks, info.notes, info.tags), (0, 0, 0));
        assert_eq!(info.database_bytes, 0);
        assert!(!info.sqlite_version.is_empty());
    }
}

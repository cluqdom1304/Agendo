//! `backup_create` / `backup_restore` (SPEC §7.3, F-16, D-09). The file dialogs live in the
//! frontend; these commands receive the chosen path.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use tauri::{AppHandle, State};

use crate::{
    DATABASE_FILE,
    db::{self, MIGRATIONS},
    error::{AppError, AppResult},
    state::{AppState, run_blocking},
};

/// Where the data that a restore replaces is kept, so a mistaken restore can be undone by hand.
pub const SAFETY_COPY_FILE: &str = "agenda.before-restore.sqlite";

/// The live database and its sidecar files, which a backup must never overwrite or read from.
pub fn protected_files(database: &Path) -> Vec<PathBuf> {
    ["", "-wal", "-shm"]
        .iter()
        .map(|suffix| {
            let mut name = database.as_os_str().to_os_string();
            name.push(suffix);
            PathBuf::from(name)
        })
        .collect()
}

pub fn create(state: &AppState, database: &Path, target: &Path) -> AppResult<()> {
    // The writer, not the read-only connection: `VACUUM INTO` takes a consistent snapshot there.
    state
        .db
        .with_writer(|conn| db::backup::create(conn, target, &protected_files(database)))
}

pub fn restore(
    state: &AppState,
    database: &Path,
    safety_copy: &Path,
    source: &Path,
) -> AppResult<()> {
    state.db.with_writer(|conn| {
        db::backup::restore(
            conn,
            source,
            safety_copy,
            &protected_files(database),
            &MIGRATIONS,
        )
    })?;
    // Reminders came back with the data: the scheduler must look at them again.
    state.wake.notify_one();
    Ok(())
}

fn data_dir(app: &AppHandle) -> AppResult<PathBuf> {
    crate::data_dir(app)
        .map_err(|error| AppError::Io(format!("no se encontró la carpeta de datos: {error}")))
}

#[tauri::command]
pub async fn backup_create(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    path: String,
) -> Result<(), String> {
    let database = data_dir(&app)?.join(DATABASE_FILE);
    run_blocking(state.inner().clone(), move |state| {
        create(state, &database, Path::new(&path))
    })
    .await
}

#[tauri::command]
pub async fn backup_restore(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    path: String,
) -> Result<(), String> {
    let dir = data_dir(&app)?;
    run_blocking(state.inner().clone(), move |state| {
        restore(
            state,
            &dir.join(DATABASE_FILE),
            &dir.join(SAFETY_COPY_FILE),
            Path::new(&path),
        )
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        repo::{
            model::NewTask,
            tasks::{self, Query},
        },
        state::test_state,
        test_support::TempDir,
    };

    fn add(state: &AppState, id: &str) {
        let now = state.clock.now_string();
        let task = NewTask {
            id: id.into(),
            title: id.into(),
            now,
            ..NewTask::default()
        };
        state
            .db
            .write(|tx| Ok(tasks::create(tx, &task)?))
            .expect("create");
    }

    fn titles(state: &AppState) -> Vec<String> {
        state
            .db
            .read(|conn| Ok(tasks::list(conn, Query::Todo, None)?))
            .expect("list")
            .into_iter()
            .map(|task| task.id)
            .collect()
    }

    #[test]
    fn creating_and_restoring_round_trips_through_the_app_state() {
        let (state, _) = test_state();
        let dir = TempDir::new();
        let database = dir.database();
        let copy = database.with_file_name("copy.sqlite");
        let safety = database.with_file_name(SAFETY_COPY_FILE);
        std::fs::create_dir_all(database.parent().expect("parent")).expect("dir");
        add(&state, "one");

        create(&state, &database, &copy).expect("create");
        add(&state, "two");
        restore(&state, &database, &safety, &copy).expect("restore");

        assert_eq!(titles(&state), ["one"]);
        assert!(safety.exists());
    }

    #[test]
    fn the_live_database_and_its_sidecars_are_protected() {
        let files = protected_files(Path::new("data/agenda.sqlite"));
        assert_eq!(
            files,
            [
                PathBuf::from("data/agenda.sqlite"),
                PathBuf::from("data/agenda.sqlite-wal"),
                PathBuf::from("data/agenda.sqlite-shm"),
            ]
        );
    }
}

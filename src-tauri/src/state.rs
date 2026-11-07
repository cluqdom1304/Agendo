//! Shared application state: the database handle, the clock and the id generator.

use std::{
    path::Path,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use rusqlite::{Connection, Transaction};
use tokio::sync::Notify;

use crate::{
    db::{self, DbError, MIGRATIONS, Migration},
    error::{AppError, AppResult},
    ids::{IdGenerator, UuidV7Generator},
    repo,
    scheduler::clock::Clock,
};

/// The single writer connection (SQLite admits one writer, SPEC §4.3) plus, for file databases,
/// a separate read-only connection so reads do not queue behind writes. In-memory databases
/// cannot be shared between connections, so there reads use the writer.
///
/// A panic while a connection is locked does not poison it for good: an open transaction is
/// rolled back when the panic unwinds, so the connection is still consistent.
pub struct Db {
    writer: Mutex<Connection>,
    reader: Option<Mutex<Connection>>,
}

fn lock(connection: &Mutex<Connection>) -> MutexGuard<'_, Connection> {
    connection.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Db {
    pub fn open(path: &Path, migrations: &[Migration]) -> Result<Self, DbError> {
        let writer = db::open(path, migrations)?;
        let reader = db::open_read_only(path)?;
        Ok(Self {
            writer: Mutex::new(writer),
            reader: Some(Mutex::new(reader)),
        })
    }

    pub fn open_in_memory(migrations: &[Migration]) -> Result<Self, DbError> {
        Ok(Self {
            writer: Mutex::new(db::open_in_memory(migrations)?),
            reader: None,
        })
    }

    pub fn read<T>(&self, operation: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let connection = lock(self.reader.as_ref().unwrap_or(&self.writer));
        operation(&connection)
    }

    /// One logical write: a single transaction, committed on `Ok` and rolled back on `Err`.
    pub fn write<T>(
        &self,
        operation: impl FnOnce(&Transaction<'_>) -> AppResult<T>,
    ) -> AppResult<T> {
        let mut connection = lock(&self.writer);
        repo::with_tx(&mut connection, operation)
    }
}

pub struct AppState {
    pub db: Db,
    pub clock: Arc<dyn Clock>,
    pub ids: Arc<dyn IdGenerator>,
    /// Wakes the reminder scheduler after anything that can change what fires next (SPEC §6.1).
    pub wake: Arc<Notify>,
}

impl AppState {
    pub fn new(db: Db, clock: Arc<dyn Clock>, ids: Arc<dyn IdGenerator>) -> Self {
        Self {
            db,
            clock,
            ids,
            wake: Arc::new(Notify::new()),
        }
    }

    /// Opens (and migrates) the database file, then permanently removes tasks that have been in
    /// the trash for more than 30 days (SPEC §5.3).
    pub fn open(path: &Path, clock: Arc<dyn Clock>) -> AppResult<Self> {
        let db = Db::open(path, &MIGRATIONS)?;
        let ids = Arc::new(UuidV7Generator::new(Arc::clone(&clock)));
        let state = Self::new(db, clock, ids);
        state.purge_trash()?;
        Ok(state)
    }

    /// Returns how many tasks were purged.
    pub fn purge_trash(&self) -> AppResult<usize> {
        let now = self.clock.now_string();
        self.db
            .write(|tx| repo::tasks::purge_deleted(tx, &now).map_err(AppError::from))
    }
}

/// Runs `work` on a blocking thread so commands never stall the async runtime (SPEC §4.3).
pub async fn run_blocking<T, F>(state: Arc<AppState>, work: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&AppState) -> AppResult<T> + Send + 'static,
{
    let outcome = tauri::async_runtime::spawn_blocking(move || work(&state))
        .await
        .map_err(|error| AppError::Internal(format!("la tarea se interrumpió: {error}")))?;
    Ok(outcome?)
}

/// An in-memory state with a manual clock fixed at 2026-03-10T10:00:00Z and ids `id-1`, `id-2`…
#[cfg(test)]
pub fn test_state() -> (AppState, Arc<crate::scheduler::clock::ManualClock>) {
    use crate::{domain::dates, ids::SequentialIds, scheduler::clock::ManualClock};

    let start = dates::parse_instant("2026-03-10T10:00:00Z").expect("start instant");
    let clock = Arc::new(ManualClock::new(start));
    let db = Db::open_in_memory(&MIGRATIONS).expect("database");
    let state = AppState::new(
        db,
        Arc::clone(&clock) as Arc<dyn Clock>,
        Arc::new(SequentialIds::default()),
    );
    (state, clock)
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::{
        domain::dates,
        repo::{
            model::NewTask,
            tasks::{self, Query},
        },
        scheduler::clock::ManualClock,
        test_support::TempDir,
    };

    fn new_task(id: &str, now: &str) -> NewTask {
        NewTask {
            id: id.to_string(),
            title: id.to_string(),
            now: now.to_string(),
            ..NewTask::default()
        }
    }

    #[test]
    fn write_commits_on_success_and_rolls_back_on_error() {
        let (state, _) = test_state();
        let now = state.clock.now_string();

        state
            .db
            .write(|tx| Ok(tasks::create(tx, &new_task("kept", &now))?))
            .expect("commit");
        let failed: AppResult<()> = state.db.write(|tx| {
            tasks::create(tx, &new_task("discarded", &now))?;
            Err(AppError::Internal("boom".into()))
        });

        assert_eq!(failed, Err(AppError::Internal("boom".into())));
        let titles = state
            .db
            .read(|conn| Ok(tasks::list(conn, Query::Todo, None)?))
            .expect("list");
        assert_eq!(titles.len(), 1);
        assert_eq!(titles[0].id, "kept");
    }

    #[test]
    fn write_reports_database_errors_with_stable_codes() {
        let (state, _) = test_state();
        let now = state.clock.now_string();
        let duplicate = state.db.write(|tx| {
            tasks::create(tx, &new_task("same", &now))?;
            Ok(tasks::create(tx, &new_task("same", &now))?)
        });
        assert_eq!(duplicate.expect_err("duplicate").code(), "CONFLICT");
    }

    #[test]
    fn file_database_reads_through_its_own_connection() {
        let dir = TempDir::new();
        let (clock, _) = manual_clock();
        let state = AppState::open(&dir.database(), clock).expect("open");
        let now = state.clock.now_string();
        assert!(state.db.reader.is_some());

        state
            .db
            .write(|tx| Ok(tasks::create(tx, &new_task("t1", &now))?))
            .expect("write");

        let seen = state
            .db
            .read(|conn| Ok(tasks::list(conn, Query::Todo, None)?.len()))
            .expect("read");
        assert_eq!(seen, 1);
        let refused = state.db.read(|conn| {
            conn.execute("DELETE FROM tasks", [])?;
            Ok(())
        });
        assert!(refused.is_err(), "the reader must not accept writes");
    }

    #[test]
    fn a_panic_inside_a_write_leaves_the_database_usable_and_unchanged() {
        let (state, _) = test_state();
        let state = Arc::new(state);
        let now = state.clock.now_string();

        let worker = {
            let state = Arc::clone(&state);
            let now = now.clone();
            std::thread::spawn(move || {
                let _ = state.db.write(|tx| -> AppResult<()> {
                    tasks::create(tx, &new_task("lost", &now))?;
                    panic!("simulated crash inside a command");
                });
            })
        };
        assert!(worker.join().is_err());

        state
            .db
            .write(|tx| Ok(tasks::create(tx, &new_task("after", &now))?))
            .expect("write after panic");
        let ids: Vec<String> = state
            .db
            .read(|conn| Ok(tasks::list(conn, Query::Todo, None)?))
            .expect("list")
            .into_iter()
            .map(|task| task.id)
            .collect();
        assert_eq!(ids, ["after"]);
    }

    fn manual_clock() -> (Arc<dyn Clock>, Arc<ManualClock>) {
        let start = dates::parse_instant("2026-03-10T10:00:00Z").expect("instant");
        let manual = Arc::new(ManualClock::new(start));
        (Arc::clone(&manual) as Arc<dyn Clock>, manual)
    }

    #[test]
    fn opening_purges_tasks_trashed_more_than_thirty_days_ago() {
        let dir = TempDir::new();
        let (clock, manual) = manual_clock();
        {
            let state = AppState::open(&dir.database(), Arc::clone(&clock)).expect("first open");
            let now = state.clock.now_string();
            for id in ["old", "recent", "live"] {
                state
                    .db
                    .write(|tx| Ok(tasks::create(tx, &new_task(id, &now))?))
                    .expect("create");
            }
            state
                .db
                .write(|tx| Ok(tasks::soft_delete(tx, "old", &now)?))
                .expect("trash old");
            manual.advance(Duration::days(20));
            let later = state.clock.now_string();
            state
                .db
                .write(|tx| Ok(tasks::soft_delete(tx, "recent", &later)?))
                .expect("trash recent");
        }

        manual.advance(Duration::days(11));
        let state = AppState::open(&dir.database(), clock).expect("second open");

        let trashed: Vec<String> = state
            .db
            .read(|conn| Ok(tasks::list_trash(conn)?))
            .expect("trash")
            .into_iter()
            .map(|task| task.id)
            .collect();
        assert_eq!(trashed, ["recent"]);
        let live = state
            .db
            .read(|conn| Ok(tasks::list(conn, Query::Todo, None)?.len()))
            .expect("live");
        assert_eq!(live, 1);
    }

    #[test]
    fn purge_trash_reports_how_many_tasks_it_removed() {
        let (state, clock) = test_state();
        let now = state.clock.now_string();
        state
            .db
            .write(|tx| Ok(tasks::create(tx, &new_task("a", &now))?))
            .expect("create");
        state
            .db
            .write(|tx| Ok(tasks::soft_delete(tx, "a", &now)?))
            .expect("trash");

        assert_eq!(state.purge_trash().expect("nothing yet"), 0);
        clock.advance(Duration::days(31));
        assert_eq!(state.purge_trash().expect("purge"), 1);
    }

    #[test]
    fn run_blocking_returns_values_and_ipc_errors() {
        let (state, _) = test_state();
        let state = Arc::new(state);

        let value = tauri::async_runtime::block_on(run_blocking(Arc::clone(&state), |state| {
            Ok(state.clock.now_string())
        }));
        assert_eq!(value, Ok("2026-03-10T10:00:00Z".to_string()));

        let error = tauri::async_runtime::block_on(run_blocking(state, |_| -> AppResult<()> {
            Err(AppError::NotFound("tarea 1".into()))
        }));
        assert_eq!(error, Err("NOT_FOUND: tarea 1".to_string()));
    }
}

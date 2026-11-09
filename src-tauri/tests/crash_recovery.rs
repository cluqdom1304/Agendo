#![allow(clippy::expect_used)]

//! N-06: a process killed in the middle of a write leaves the database intact and without the
//! half-written data. The test re-runs its own executable as the child that dies.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use miagendaapp_lib::{
    repo::{
        model::NewTask,
        reminders::{self, NewReminder},
        tasks::{self, Query},
    },
    scheduler::clock::SystemClock,
    state::AppState,
};

const CHILD_ENV: &str = "MIAGENDA_CRASH_DB";

fn new_task(id: &str) -> NewTask {
    NewTask {
        id: id.to_string(),
        title: id.to_string(),
        now: "2030-01-01T00:00:00Z".to_string(),
        ..NewTask::default()
    }
}

fn open(path: &Path) -> AppState {
    AppState::open(path, Arc::new(SystemClock)).expect("open database")
}

/// Runs only in the child: commits one task, then dies half-way through a second write.
fn die_during_write(path: &Path) {
    let state = open(path);
    state
        .db
        .write(|tx| Ok(tasks::create(tx, &new_task("committed"))?))
        .expect("committed write");
    let _ = state
        .db
        .write(|tx| -> miagendaapp_lib::error::AppResult<()> {
            tasks::create(tx, &new_task("half-written"))?;
            reminders::replace_unsent(
                tx,
                "half-written",
                &[NewReminder {
                    id: "r1".into(),
                    notify_at: "2030-01-01T01:00:00Z".into(),
                    offset_minutes: None,
                    repeat: None,
                }],
                "2030-01-01T00:00:00Z",
            )?;
            std::process::abort()
        });
}

fn scratch_dir() -> PathBuf {
    env::temp_dir().join(format!("miagendaapp-crash-{}", std::process::id()))
}

#[test]
fn a_killed_process_leaves_an_intact_database_without_the_unfinished_write() {
    if let Ok(path) = env::var(CHILD_ENV) {
        die_during_write(Path::new(&path));
        return;
    }

    let dir = scratch_dir();
    fs::create_dir_all(&dir).expect("scratch dir");
    let database = dir.join("agenda.sqlite");

    let status = Command::new(env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "a_killed_process_leaves_an_intact_database_without_the_unfinished_write",
            "--nocapture",
        ])
        .env(CHILD_ENV, &database)
        .status()
        .expect("spawn child");
    assert!(!status.success(), "the child must have been killed");

    let state = open(&database);
    let ids: Vec<String> = state
        .db
        .read(|conn| Ok(tasks::list(conn, Query::Todo, None)?))
        .expect("list")
        .into_iter()
        .map(|task| task.id)
        .collect();
    assert_eq!(ids, ["committed"]);
    let check: String = state
        .db
        .read(|conn| Ok(conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?))
        .expect("integrity_check");
    assert_eq!(check, "ok");
    let reminders: i64 = state
        .db
        .read(|conn| Ok(conn.query_row("SELECT COUNT(*) FROM reminders", [], |row| row.get(0))?))
        .expect("count");
    assert_eq!(reminders, 0);

    drop(state);
    let _ = fs::remove_dir_all(&dir);
}

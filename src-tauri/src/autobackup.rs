//! Scheduled backups with rotation (F17-09, D-61). While the app runs it checks once an hour
//! whether a copy is due; the copy is a `.agendo` archive (D-47) in the `backups` folder next to
//! the database, and only the newest N are kept. What is due is decided from the time of the
//! last copy, stored in the settings, so closing and opening the app does not skip or repeat one.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, PoisonError},
    time::Duration as StdDuration,
};

use chrono::Duration;

use crate::{
    DATABASE_FILE, archive, attachments, backoff,
    domain::dates,
    error::{AppError, AppResult},
    repo::settings,
    state::AppState,
};

pub const DIR_NAME: &str = "backups";
/// Settings: how often (`off`, `daily`, `weekly`), how many copies to keep, and when the last one
/// was made (UTC instant, written by the app).
pub const MODE_KEY: &str = "auto_backup";
pub const KEEP_KEY: &str = "auto_backup_keep";
pub const LAST_KEY: &str = "auto_backup_last";

pub const MODES: [&str; 3] = ["off", "daily", "weekly"];
pub const DEFAULT_KEEP: usize = 7;
pub const MAX_KEEP: usize = 30;

const FILE_PREFIX: &str = "agendo-auto-";
const FILE_EXTENSION: &str = "agendo";
/// A copy is due a little before a whole day or week has passed, so a daily start-up routine
/// that lands a few minutes early is not postponed by a day.
const TOLERANCE_MINUTES: i64 = 60;
/// How long the app waits after starting before it looks, and between looks.
const FIRST_LOOK: StdDuration = StdDuration::from_secs(45);
const LOOK_EVERY: StdDuration = StdDuration::from_secs(3600);
/// First wait after a failed copy; it doubles with each further failure up to `LOOK_EVERY`.
const RETRY_FIRST: StdDuration = StdDuration::from_secs(300);

fn interval(mode: &str) -> Option<Duration> {
    match mode {
        "daily" => Some(Duration::days(1)),
        "weekly" => Some(Duration::days(7)),
        _ => None,
    }
}

/// How many copies to keep, from the stored text (a missing or invalid value means the default).
pub fn keep_from(text: Option<&str>) -> usize {
    text.and_then(|value| value.parse::<usize>().ok())
        .filter(|count| (1..=MAX_KEEP).contains(count))
        .unwrap_or(DEFAULT_KEEP)
}

/// Makes a copy if the schedule says one is due and removes the oldest beyond the limit.
/// Returns the file made, or `None` when nothing was due.
pub fn run_if_due(state: &AppState, database: &Path, dir: &Path) -> AppResult<Option<PathBuf>> {
    let (mode, keep, last) = state.db.read(|conn| {
        Ok((
            settings::get(conn, MODE_KEY)?,
            settings::get(conn, KEEP_KEY)?,
            settings::get(conn, LAST_KEY)?,
        ))
    })?;
    let Some(every) = mode.as_deref().and_then(interval) else {
        return Ok(None);
    };
    let now = state.clock.now();
    let due = match last.as_deref().map(dates::parse_instant) {
        Some(Ok(last)) => now - last >= every - Duration::minutes(TOLERANCE_MINUTES),
        _ => true,
    };
    if !due {
        return Ok(None);
    }

    fs::create_dir_all(dir)
        .map_err(|error| AppError::Io(format!("no se pudo crear la carpeta de copias: {error}")))?;
    let name = format!(
        "{FILE_PREFIX}{}.{FILE_EXTENSION}",
        now.format("%Y%m%d-%H%M%S")
    );
    let target = dir.join(name);
    let files = database.with_file_name(attachments::DIR_NAME);
    archive::create(state, database, &files, &target)?;

    let stamp = state.clock.now_string();
    state
        .db
        .write(|tx| Ok(settings::set(tx, LAST_KEY, &stamp)?))?;
    rotate(dir, keep_from(keep.as_deref()))?;
    Ok(Some(target))
}

/// The automatic copies in `dir`, newest first (their names carry the time they were made).
fn automatic_copies(dir: &Path) -> AppResult<Vec<PathBuf>> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(AppError::Io(format!(
                "no se pudo leer la carpeta de copias: {error}"
            )));
        }
    };
    let mut copies: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path.extension().is_some_and(|e| e == FILE_EXTENSION)
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(FILE_PREFIX))
        })
        .collect();
    copies.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    Ok(copies)
}

/// Deletes all but the newest `keep` automatic copies. Files the user put there are not touched.
fn rotate(dir: &Path, keep: usize) -> AppResult<()> {
    for old in automatic_copies(dir)?.into_iter().skip(keep) {
        fs::remove_file(&old).map_err(|error| {
            AppError::Io(format!("no se pudo borrar una copia antigua: {error}"))
        })?;
    }
    Ok(())
}

/// Background loop: looks shortly after start-up and then every hour. A failed copy is logged and
/// tried again at the next look; it never stops the app.
pub async fn run(state: Arc<AppState>, data_dir: PathBuf) {
    if sleep_unless_closing(&state, FIRST_LOOK).await {
        return;
    }
    let mut failures = 0_u32;
    loop {
        let pass = {
            let state = Arc::clone(&state);
            let database = data_dir.join(DATABASE_FILE);
            let dir = data_dir.join(DIR_NAME);
            tokio::task::spawn_blocking(move || {
                // Closing the app waits for this guard, so a copy is not cut off half way.
                let _busy = state
                    .background_work
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner);
                run_if_due(&state, &database, &dir)
            })
            .await
        };
        match pass {
            Ok(Ok(_)) => failures = 0,
            Ok(Err(error)) => {
                failures += 1;
                crate::applog::error("autobackup", &format!("la copia falló: {error}"));
            }
            Err(error) => {
                failures += 1;
                crate::applog::error("autobackup", &format!("la tarea se interrumpió: {error}"));
            }
        }
        // After a failure look again sooner (5 min, 10, 20, ...) but never less often than usual.
        let wait = if failures == 0 {
            LOOK_EVERY
        } else {
            backoff::delay(failures, RETRY_FIRST, LOOK_EVERY)
        };
        if sleep_unless_closing(&state, wait).await {
            return;
        }
    }
}

/// Sleeps `wait`; returns `true` if the app began closing first (or already had).
async fn sleep_unless_closing(state: &AppState, wait: StdDuration) -> bool {
    tokio::select! {
        () = tokio::time::sleep(wait) => state.is_shutting_down(),
        () = state.shut_down() => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{commands::settings as setting_commands, state::test_state, test_support::TempDir};

    fn files_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }

    fn paths() -> (TempDir, PathBuf, PathBuf) {
        let temp = TempDir::new();
        let database = temp.database();
        let dir = database.with_file_name(DIR_NAME);
        (temp, database, dir)
    }

    #[tokio::test]
    async fn the_loop_ends_when_the_app_begins_to_close_and_waits_for_a_copy_in_progress() {
        let (state, _) = test_state();
        let state = Arc::new(state);
        let (_temp, _database, _dir) = paths();
        let handle = tokio::spawn(run(
            Arc::clone(&state),
            _database.parent().expect("dir").into(),
        ));
        tokio::time::sleep(StdDuration::from_millis(50)).await;
        assert!(!handle.is_finished(), "it is waiting for its first look");

        state.begin_shutdown();
        let ended = tokio::time::timeout(StdDuration::from_secs(5), handle).await;
        assert!(ended.is_ok(), "the loop did not stop");

        // A pass that holds the guard makes `wait_for_background_work` wait, and it gives up
        // at the limit instead of hanging.
        let guard = state.background_work.lock().expect("lock");
        let started = std::time::Instant::now();
        state.wait_for_background_work(StdDuration::from_millis(120));
        assert!(started.elapsed() >= StdDuration::from_millis(100));
        drop(guard);
        let started = std::time::Instant::now();
        state.wait_for_background_work(StdDuration::from_secs(5));
        assert!(started.elapsed() < StdDuration::from_secs(1));
    }

    #[test]
    fn keep_falls_back_to_the_default_for_missing_or_absurd_values() {
        assert_eq!(keep_from(None), DEFAULT_KEEP);
        assert_eq!(keep_from(Some("5")), 5);
        assert_eq!(keep_from(Some("0")), DEFAULT_KEEP);
        assert_eq!(keep_from(Some("31")), DEFAULT_KEEP);
        assert_eq!(keep_from(Some("muchas")), DEFAULT_KEEP);
    }

    #[test]
    fn nothing_happens_while_it_is_off() {
        let (state, _) = test_state();
        let (_temp, database, dir) = paths();
        assert_eq!(run_if_due(&state, &database, &dir).expect("off"), None);
        setting_commands::set(&state, MODE_KEY, "off").expect("set");
        assert_eq!(run_if_due(&state, &database, &dir).expect("off"), None);
        assert!(!dir.exists());
    }

    #[test]
    fn a_daily_copy_is_made_once_a_day_and_the_oldest_are_rotated_out() {
        let (state, clock) = test_state();
        let (_temp, database, dir) = paths();
        setting_commands::set(&state, MODE_KEY, "daily").expect("mode");
        setting_commands::set(&state, KEEP_KEY, "2").expect("keep");

        let first = run_if_due(&state, &database, &dir)
            .expect("first")
            .expect("made");
        assert_eq!(
            first.file_name().and_then(|n| n.to_str()),
            Some("agendo-auto-20260310-100000.agendo")
        );
        assert!(archive::is_archive(&first));
        // Not due again the same day.
        clock.advance(Duration::hours(5));
        assert_eq!(run_if_due(&state, &database, &dir).expect("same day"), None);

        for _ in 0..3 {
            clock.advance(Duration::days(1));
            assert!(run_if_due(&state, &database, &dir).expect("next").is_some());
        }
        // Four copies were made; only the two newest remain.
        assert_eq!(
            files_in(&dir),
            [
                "agendo-auto-20260312-150000.agendo",
                "agendo-auto-20260313-150000.agendo"
            ]
        );
    }

    #[test]
    fn weekly_waits_a_week_and_a_little_early_is_close_enough() {
        let (state, clock) = test_state();
        let (_temp, database, dir) = paths();
        setting_commands::set(&state, MODE_KEY, "weekly").expect("mode");
        assert!(
            run_if_due(&state, &database, &dir)
                .expect("first")
                .is_some()
        );
        clock.advance(Duration::days(6));
        assert_eq!(run_if_due(&state, &database, &dir).expect("day 6"), None);
        clock.advance(Duration::days(1) - Duration::minutes(30));
        assert!(
            run_if_due(&state, &database, &dir)
                .expect("day 7")
                .is_some()
        );
    }

    #[test]
    fn rotation_leaves_files_the_user_put_in_the_folder() {
        let (state, clock) = test_state();
        let (_temp, database, dir) = paths();
        fs::create_dir_all(&dir).expect("dir");
        fs::write(dir.join("mis-notas.agendo"), b"x").expect("own file");
        fs::write(dir.join("agendo-auto-20200101-000000.txt"), b"x").expect("other kind");
        setting_commands::set(&state, MODE_KEY, "daily").expect("mode");
        setting_commands::set(&state, KEEP_KEY, "1").expect("keep");
        for _ in 0..3 {
            run_if_due(&state, &database, &dir).expect("copy");
            clock.advance(Duration::days(1));
        }
        let names = files_in(&dir);
        assert!(names.contains(&"mis-notas.agendo".to_string()));
        assert!(names.contains(&"agendo-auto-20200101-000000.txt".to_string()));
        assert_eq!(
            names
                .iter()
                .filter(|n| n.starts_with(FILE_PREFIX) && n.ends_with(".agendo"))
                .count(),
            1
        );
    }
}

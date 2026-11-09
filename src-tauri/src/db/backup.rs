//! Backups and restores of the database file (SPEC §5, F-16, D-09).
//!
//! `create` uses `VACUUM INTO`, which writes a consistent copy even while the WAL is active.
//! `restore` never trusts the file it is given: it is validated first, the current data is
//! copied aside, and only then are its pages copied into the live connection.

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

use rusqlite::{Connection, OpenFlags, backup::Backup};

use super::{Migration, migrate, pragmas};
use crate::error::{AppError, AppResult};

const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\0";
const REQUIRED_TABLES: [&str; 6] = [
    "tags",
    "tasks",
    "task_tags",
    "reminders",
    "settings",
    "tasks_fts",
];

fn io_error(action: &str, error: std::io::Error) -> AppError {
    AppError::Io(format!("{action}: {error}"))
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn temp_sibling(target: &Path) -> PathBuf {
    let mut name = target.as_os_str().to_os_string();
    name.push(".tmp");
    PathBuf::from(name)
}

/// Writes a consistent copy of `conn` to `target`, replacing a file that is already there.
///
/// `protected` lists files that must never be overwritten (the live database and its sidecars).
/// The copy is written next to `target` and renamed, so a failure leaves an older file intact.
pub fn create(conn: &Connection, target: &Path, protected: &[PathBuf]) -> AppResult<()> {
    if protected.iter().any(|path| same_file(path, target)) {
        return Err(AppError::Validation(
            "no se puede guardar la copia sobre la base de datos en uso".into(),
        ));
    }
    let temp = temp_sibling(target);
    if temp.exists() {
        fs::remove_file(&temp).map_err(|error| io_error("no se pudo preparar la copia", error))?;
    }
    let temp_text = temp
        .to_str()
        .ok_or_else(|| AppError::Validation("la ruta de la copia no es válida".into()))?;
    let written = conn.execute("VACUUM INTO ?1", [temp_text]);
    if let Err(error) = written {
        let _ = fs::remove_file(&temp);
        return Err(AppError::from(error));
    }
    fs::rename(&temp, target).map_err(|error| {
        let _ = fs::remove_file(&temp);
        io_error("no se pudo guardar la copia", error)
    })
}

fn invalid(reason: &str) -> AppError {
    AppError::Validation(format!("el archivo no es una copia válida: {reason}"))
}

fn has_sqlite_header(path: &Path) -> AppResult<bool> {
    let mut file =
        fs::File::open(path).map_err(|error| io_error("no se pudo abrir el archivo", error))?;
    let mut header = [0_u8; 16];
    match file.read_exact(&mut header) {
        Ok(()) => Ok(&header == SQLITE_HEADER),
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => Ok(false),
        Err(error) => Err(io_error("no se pudo leer el archivo", error)),
    }
}

fn open_source(path: &Path) -> AppResult<Connection> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn =
        Connection::open_with_flags(path, flags).map_err(|_| invalid("no se puede abrir"))?;
    conn.busy_timeout(Duration::from_millis(pragmas::BUSY_TIMEOUT_MS))
        .map_err(AppError::from)?;
    Ok(conn)
}

/// Checks that `source` is an intact database of this app that the current build can use.
/// Returns its schema version. Nothing is written.
fn validate(source: &Connection, expected_page_size: i64, supported: u32) -> AppResult<u32> {
    let integrity: Vec<String> = source
        .prepare("PRAGMA integrity_check")
        .and_then(|mut stmt| stmt.query_map([], |row| row.get(0))?.collect())
        .map_err(|_| invalid("está dañado"))?;
    if integrity != ["ok"] {
        return Err(invalid("falló la comprobación de integridad"));
    }

    let version = migrate::current_version(source).map_err(|_| invalid("versión ilegible"))?;
    if version == 0 {
        return Err(invalid("no pertenece a esta aplicación"));
    }
    if version > supported {
        return Err(AppError::Validation(
            "la copia se hizo con una versión más reciente de la aplicación".into(),
        ));
    }

    for table in REQUIRED_TABLES {
        let found: bool = source
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name = ?1)",
                [table],
                |row| row.get(0),
            )
            .map_err(AppError::from)?;
        if !found {
            return Err(invalid("faltan datos de la aplicación"));
        }
    }

    let broken_references: bool = source
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_check)",
            [],
            |row| row.get(0),
        )
        .map_err(AppError::from)?;
    if broken_references {
        return Err(invalid("hay referencias rotas entre sus datos"));
    }

    let page_size: i64 = source
        .query_row("PRAGMA page_size", [], |row| row.get(0))
        .map_err(AppError::from)?;
    if page_size != expected_page_size {
        return Err(invalid("su formato interno no es compatible"));
    }
    Ok(version)
}

/// Replaces the contents of `live` with those of the backup at `source`.
///
/// 1. The file is validated; any problem returns `VALIDATION` and nothing changes.
/// 2. The current data is saved to `safety_copy` (replacing an older one).
/// 3. The backup's pages are copied into `live` inside SQLite's own backup transaction, so a
///    failure leaves the current data in place.
/// 4. The schema is migrated if the backup is older, and the connection pragmas re-applied.
pub fn restore(
    live: &mut Connection,
    source: &Path,
    safety_copy: &Path,
    protected: &[PathBuf],
    migrations: &[Migration],
) -> AppResult<()> {
    if protected.iter().any(|path| same_file(path, source)) {
        return Err(AppError::Validation(
            "no se puede restaurar desde la base de datos en uso".into(),
        ));
    }
    if !source.is_file() {
        return Err(AppError::NotFound(
            "el archivo de la copia no existe".into(),
        ));
    }
    if !has_sqlite_header(source)? {
        return Err(invalid("no es una base de datos SQLite"));
    }

    let page_size: i64 = live
        .query_row("PRAGMA page_size", [], |row| row.get(0))
        .map_err(AppError::from)?;
    let origin = open_source(source)?;
    validate(&origin, page_size, migrate::latest_version(migrations))?;

    create(live, safety_copy, protected)?;

    {
        let backup = Backup::new(&origin, live).map_err(AppError::from)?;
        backup
            .run_to_completion(100, Duration::from_millis(50), None)
            .map_err(AppError::from)?;
    }
    drop(origin);

    migrate::migrate(live, migrations).map_err(AppError::from)?;
    pragmas::apply(live).map_err(AppError::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::{db, test_support::TempDir};

    fn insert_task(conn: &Connection, id: &str) {
        conn.execute(
            "INSERT INTO tasks (id, title, position, created_at, updated_at)
             VALUES (?1, ?1, 1, '2026-03-10T10:00:00Z', '2026-03-10T10:00:00Z')",
            [id],
        )
        .expect("insert task");
    }

    fn task_ids(conn: &Connection) -> Vec<String> {
        conn.prepare("SELECT id FROM tasks ORDER BY id")
            .and_then(|mut stmt| stmt.query_map([], |row| row.get(0))?.collect())
            .expect("ids")
    }

    struct Fixture {
        dir: TempDir,
        live_path: PathBuf,
        live: Connection,
    }

    impl Fixture {
        fn new() -> Self {
            let dir = TempDir::new();
            let live_path = dir.database();
            let live = db::open(&live_path, &db::MIGRATIONS).expect("open");
            Self {
                dir,
                live_path,
                live,
            }
        }

        fn sibling(&self, name: &str) -> PathBuf {
            self.live_path.with_file_name(name)
        }

        fn protected(&self) -> Vec<PathBuf> {
            vec![self.live_path.clone()]
        }
    }

    #[test]
    fn create_writes_a_readable_copy_with_the_current_data() {
        let fx = Fixture::new();
        insert_task(&fx.live, "a");
        let target = fx.sibling("copy.sqlite");

        create(&fx.live, &target, &fx.protected()).expect("create");

        let copy = Connection::open(&target).expect("open copy");
        assert_eq!(task_ids(&copy), ["a"]);
        let check: String = copy
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .expect("check");
        assert_eq!(check, "ok");
        assert!(!temp_sibling(&target).exists());
    }

    #[test]
    fn create_replaces_an_existing_file_and_refuses_the_live_database() {
        let fx = Fixture::new();
        let target = fx.sibling("copy.sqlite");
        fs::write(&target, b"old").expect("old file");
        insert_task(&fx.live, "a");

        create(&fx.live, &target, &fx.protected()).expect("replace");
        assert_eq!(task_ids(&Connection::open(&target).expect("open")), ["a"]);

        let error = create(&fx.live, &fx.live_path, &fx.protected()).expect_err("live file");
        assert_eq!(error.code(), "VALIDATION");
        assert_eq!(task_ids(&fx.live), ["a"], "the live database is untouched");
    }

    #[test]
    fn restore_brings_back_the_state_of_the_copy_and_keeps_the_previous_data_aside() {
        let mut fx = Fixture::new();
        insert_task(&fx.live, "before");
        let copy = fx.sibling("copy.sqlite");
        create(&fx.live, &copy, &fx.protected()).expect("create");
        insert_task(&fx.live, "after");
        let safety = fx.sibling("before-restore.sqlite");

        let protected = fx.protected();
        restore(&mut fx.live, &copy, &safety, &protected, &db::MIGRATIONS).expect("restore");

        assert_eq!(task_ids(&fx.live), ["before"]);
        assert_eq!(
            task_ids(&Connection::open(&safety).expect("safety")),
            ["after", "before"]
        );
        let mode: String = fx
            .live
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("mode");
        assert_eq!(mode, "wal");
        insert_task(&fx.live, "later");
        assert_eq!(task_ids(&fx.live), ["before", "later"]);
    }

    #[test]
    fn restore_rejects_bad_files_without_touching_the_data() {
        let mut fx = Fixture::new();
        insert_task(&fx.live, "kept");
        let safety = fx.sibling("before-restore.sqlite");
        let protected = fx.protected();

        let text = fx.sibling("notes.sqlite");
        fs::write(&text, b"this is not a database at all").expect("text");
        let empty = fx.sibling("empty.sqlite");
        fs::write(&empty, b"").expect("empty");
        let foreign = fx.sibling("foreign.sqlite");
        Connection::open(&foreign)
            .and_then(|c| c.execute_batch("CREATE TABLE other (x); INSERT INTO other VALUES (1);"))
            .expect("foreign db");
        let truncated = fx.sibling("truncated.sqlite");
        let good = fx.sibling("good.sqlite");
        create(&fx.live, &good, &protected).expect("good copy");
        let bytes = fs::read(&good).expect("read good");
        fs::write(&truncated, &bytes[..bytes.len() / 2]).expect("truncated");
        let corrupt = fx.sibling("corrupt.sqlite");
        let mut damaged = bytes.clone();
        let middle = damaged.len() / 2;
        for byte in &mut damaged[middle..middle + 2048] {
            *byte = 0xAB;
        }
        fs::write(&corrupt, damaged).expect("corrupt");

        for (path, expected) in [
            (&text, "VALIDATION"),
            (&empty, "VALIDATION"),
            (&foreign, "VALIDATION"),
            (&truncated, "VALIDATION"),
            (&corrupt, "VALIDATION"),
            (&fx.sibling("missing.sqlite"), "NOT_FOUND"),
        ] {
            let error = restore(&mut fx.live, path, &safety, &protected, &db::MIGRATIONS)
                .expect_err("must be rejected");
            assert_eq!(error.code(), expected, "{path:?}: {error}");
        }

        assert_eq!(task_ids(&fx.live), ["kept"]);
        assert!(
            !safety.exists(),
            "nothing was saved aside for a rejected file"
        );
        let _ = &fx.dir;
    }

    #[test]
    fn restore_rejects_a_copy_from_a_newer_schema() {
        let mut fx = Fixture::new();
        let copy = fx.sibling("newer.sqlite");
        create(&fx.live, &copy, &fx.protected()).expect("create");
        Connection::open(&copy)
            .and_then(|c| c.execute_batch("PRAGMA user_version = 99;"))
            .expect("bump version");

        let safety = fx.sibling("safety.sqlite");
        let protected = fx.protected();
        let error =
            restore(&mut fx.live, &copy, &safety, &protected, &db::MIGRATIONS).expect_err("newer");

        assert_eq!(error.code(), "VALIDATION");
        assert!(error.message().contains("más reciente"));
    }

    #[test]
    fn restore_migrates_an_older_copy_to_the_current_schema() {
        const OLD: [Migration; 1] = [Migration {
            version: 1,
            sql: db::MIGRATIONS[0].sql,
        }];
        const NEW: [Migration; 2] = [
            OLD[0],
            Migration {
                version: 2,
                sql: "CREATE TABLE extra (id INTEGER PRIMARY KEY);",
            },
        ];
        let dir = TempDir::new();
        let live_path = dir.database();
        let old_path = live_path.with_file_name("old.sqlite");
        let old = db::open(&old_path, &OLD).expect("old db");
        insert_task(&old, "from-old");
        drop(old);
        let mut live = db::open(&live_path, &NEW).expect("new db");
        assert_eq!(migrate::current_version(&live).expect("version"), 2);

        restore(
            &mut live,
            &old_path,
            &live_path.with_file_name("safety.sqlite"),
            std::slice::from_ref(&live_path),
            &NEW,
        )
        .expect("restore");

        assert_eq!(task_ids(&live), ["from-old"]);
        assert_eq!(migrate::current_version(&live).expect("version"), 2);
        live.execute("INSERT INTO extra DEFAULT VALUES", [])
            .expect("migrated table exists");
    }
}

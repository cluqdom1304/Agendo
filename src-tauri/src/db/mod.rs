//! SQLite connection, pragmas and schema migrations (SPEC §5).

pub mod migrate;
pub mod pragmas;

use std::{fmt, fs, io, path::Path};

use rusqlite::Connection;

pub use migrate::Migration;

#[derive(Debug)]
pub enum DbError {
    Sqlite(rusqlite::Error),
    Io(io::Error),
    /// A migration failed; its transaction was rolled back.
    Migration {
        version: u32,
        source: rusqlite::Error,
    },
    /// The migration list is not numbered 1, 2, 3, … without gaps.
    InvalidMigrations(String),
    /// The database was written by a newer version of the app.
    SchemaTooNew {
        found: u32,
        supported: u32,
    },
    Pragma(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "database error: {error}"),
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Migration { version, source } => {
                write!(f, "migration {version} failed: {source}")
            }
            Self::InvalidMigrations(reason) => write!(f, "invalid migration list: {reason}"),
            Self::SchemaTooNew { found, supported } => write!(
                f,
                "database schema version {found} is newer than the supported version {supported}"
            ),
            Self::Pragma(reason) => write!(f, "connection setup failed: {reason}"),
        }
    }
}

impl std::error::Error for DbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sqlite(error) | Self::Migration { source: error, .. } => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

impl From<io::Error> for DbError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Opens (creating it and its parent directory if needed) the database at `path`, applies the
/// pragmas of SPEC §5.4 and migrates it to the latest version.
pub fn open(path: &Path, migrations: &[Migration]) -> Result<Connection, DbError> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    prepare(Connection::open(path)?, migrations)
}

pub fn open_in_memory(migrations: &[Migration]) -> Result<Connection, DbError> {
    prepare(Connection::open_in_memory()?, migrations)
}

fn prepare(mut conn: Connection, migrations: &[Migration]) -> Result<Connection, DbError> {
    pragmas::apply(&conn)?;
    migrate::migrate(&mut conn, migrations)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU32, Ordering},
    };

    use super::*;

    const SCHEMA: [Migration; 2] = [
        Migration {
            version: 1,
            sql: "CREATE TABLE parents (id INTEGER PRIMARY KEY);",
        },
        Migration {
            version: 2,
            sql: "CREATE TABLE children (
                      id INTEGER PRIMARY KEY,
                      parent_id INTEGER NOT NULL REFERENCES parents(id)
                  );",
        },
    ];

    static NEXT_DIR: AtomicU32 = AtomicU32::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("miagendaapp-db-test-{}-{id}", std::process::id()));
            Self(path)
        }

        fn database(&self) -> PathBuf {
            self.0.join("nested").join("agenda.sqlite")
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn pragma(conn: &Connection, name: &str) -> i64 {
        conn.query_row(&format!("PRAGMA {name}"), [], |row| row.get(0))
            .expect("pragma query")
    }

    #[test]
    fn file_database_is_created_migrated_and_uses_spec_pragmas() {
        let dir = TempDir::new();
        let conn = open(&dir.database(), &SCHEMA).expect("open");

        assert_eq!(migrate::current_version(&conn).expect("version"), 2);
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("journal_mode");
        assert_eq!(mode, "wal");
        assert_eq!(pragma(&conn, "foreign_keys"), 1);
        assert_eq!(pragma(&conn, "synchronous"), 1, "NORMAL");
        assert_eq!(pragma(&conn, "busy_timeout"), 5000);
        assert_eq!(pragma(&conn, "temp_store"), 2, "MEMORY");
    }

    #[test]
    fn reopening_keeps_data_and_does_not_reapply_migrations() {
        let dir = TempDir::new();
        {
            let conn = open(&dir.database(), &SCHEMA).expect("first open");
            conn.execute("INSERT INTO parents (id) VALUES (7)", [])
                .expect("insert");
        }

        let conn = open(&dir.database(), &SCHEMA).expect("second open");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM parents WHERE id = 7", [], |r| {
                r.get(0)
            })
            .expect("count");
        assert_eq!(count, 1);
        assert_eq!(migrate::current_version(&conn).expect("version"), 2);
    }

    #[test]
    fn foreign_keys_are_enforced() {
        let conn = open_in_memory(&SCHEMA).expect("open");
        let orphan = conn.execute("INSERT INTO children (parent_id) VALUES (99)", []);
        assert!(orphan.is_err());
    }

    #[test]
    fn in_memory_database_is_migrated() {
        let conn = open_in_memory(&SCHEMA).expect("open");
        assert_eq!(migrate::current_version(&conn).expect("version"), 2);
        assert_eq!(pragma(&conn, "foreign_keys"), 1);
    }

    #[test]
    fn failed_migration_on_disk_leaves_previous_schema_intact() {
        let dir = TempDir::new();
        let broken = [
            SCHEMA[0],
            Migration {
                version: 2,
                sql: "CREATE TABLE half_done (id INTEGER PRIMARY KEY); SELECT * FROM missing;",
            },
        ];
        let error = open(&dir.database(), &broken).expect_err("must fail");
        assert!(matches!(error, DbError::Migration { version: 2, .. }));

        let conn = open(&dir.database(), &SCHEMA[..1]).expect("reopen at v1");
        assert_eq!(migrate::current_version(&conn).expect("version"), 1);
    }

    #[test]
    fn errors_display_a_readable_message() {
        let error = DbError::SchemaTooNew {
            found: 5,
            supported: 2,
        };
        assert_eq!(
            error.to_string(),
            "database schema version 5 is newer than the supported version 2"
        );
    }
}

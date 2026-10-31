//! Application error and its IPC form (D-08, SPEC §7.1).
//!
//! Commands return `Result<T, String>`; the string is `CODE: human message`, with a code from a
//! closed set the frontend can rely on. The code is never shown to the user, only the message.

use std::{fmt, io};

use rusqlite::{ErrorCode, ffi};

use crate::{
    db::DbError,
    domain::{dates::DateError, validation::ValidationError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    NotFound(String),
    Validation(String),
    Conflict(String),
    Db(String),
    Io(String),
    Notify(String),
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    /// Stable code used as the prefix of the IPC string.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::Validation(_) => "VALIDATION",
            Self::Conflict(_) => "CONFLICT",
            Self::Db(_) => "DB",
            Self::Io(_) => "IO",
            Self::Notify(_) => "NOTIFY",
            Self::Internal(_) => "INTERNAL",
        }
    }

    /// The human-readable part, without the code.
    pub fn message(&self) -> &str {
        match self {
            Self::NotFound(message)
            | Self::Validation(message)
            | Self::Conflict(message)
            | Self::Db(message)
            | Self::Io(message)
            | Self::Notify(message)
            | Self::Internal(message) => message,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}

impl std::error::Error for AppError {}

/// The IPC form: lets command bodies use `?` on an [`AppResult`].
impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        match &error {
            rusqlite::Error::QueryReturnedNoRows => Self::NotFound("el elemento no existe".into()),
            rusqlite::Error::SqliteFailure(failure, _)
                if failure.code == ErrorCode::ConstraintViolation =>
            {
                match failure.extended_code {
                    ffi::SQLITE_CONSTRAINT_UNIQUE | ffi::SQLITE_CONSTRAINT_PRIMARYKEY => {
                        Self::Conflict("ya existe un elemento con esos datos".into())
                    }
                    ffi::SQLITE_CONSTRAINT_FOREIGNKEY => {
                        Self::NotFound("el elemento referenciado no existe".into())
                    }
                    _ => Self::Validation("los datos no cumplen las restricciones".into()),
                }
            }
            _ => Self::Db(format!("error de base de datos: {error}")),
        }
    }
}

impl From<DbError> for AppError {
    fn from(error: DbError) -> Self {
        match error {
            DbError::Io(error) => error.into(),
            DbError::Sqlite(error) => error.into(),
            DbError::InvalidMigrations(_) => Self::Internal(error.to_string()),
            DbError::Migration { .. } | DbError::SchemaTooNew { .. } | DbError::Pragma(_) => {
                Self::Db(error.to_string())
            }
        }
    }
}

impl From<ValidationError> for AppError {
    fn from(error: ValidationError) -> Self {
        Self::Validation(error.message().to_string())
    }
}

impl From<DateError> for AppError {
    fn from(error: DateError) -> Self {
        Self::Validation(error.to_string())
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::Io(format!("error de entrada/salida: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::{Connection, params};

    use super::*;
    use crate::db::{MIGRATIONS, open_in_memory};

    const CODES: [&str; 7] = [
        "NOT_FOUND",
        "VALIDATION",
        "CONFLICT",
        "DB",
        "IO",
        "NOTIFY",
        "INTERNAL",
    ];

    fn database() -> Connection {
        open_in_memory(&MIGRATIONS).expect("database")
    }

    fn failure_of(conn: &Connection, sql: &str) -> AppError {
        conn.execute(sql, params![])
            .expect_err("statement must fail")
            .into()
    }

    #[test]
    fn every_variant_formats_as_code_colon_message() {
        let all = [
            AppError::NotFound("a".into()),
            AppError::Validation("b".into()),
            AppError::Conflict("c".into()),
            AppError::Db("d".into()),
            AppError::Io("e".into()),
            AppError::Notify("f".into()),
            AppError::Internal("g".into()),
        ];
        for (error, code) in all.iter().zip(CODES) {
            assert_eq!(error.code(), code);
            assert_eq!(error.to_string(), format!("{code}: {}", error.message()));
            assert_eq!(String::from(error.clone()), error.to_string());
        }
    }

    #[test]
    fn question_mark_converts_to_the_ipc_string() {
        fn command() -> Result<(), String> {
            let failing: AppResult<()> = Err(AppError::NotFound("tarea 42".into()));
            failing?;
            Ok(())
        }
        assert_eq!(command(), Err("NOT_FOUND: tarea 42".to_string()));
    }

    #[test]
    fn constraint_violations_map_to_stable_codes() {
        let conn = database();
        conn.execute_batch(
            "INSERT INTO tags (id, name, color_hex, created_at)
                 VALUES ('g1', 'Grado', '#112233', 'now');
             INSERT INTO tasks (id, created_at, updated_at) VALUES ('t1', 'now', 'now');",
        )
        .expect("seed");

        let cases = [
            (
                "duplicate name",
                "INSERT INTO tags (id, name, color_hex, created_at) VALUES ('g2', 'GRADO', '#000000', 'now')",
                "CONFLICT",
            ),
            (
                "duplicate primary key",
                "INSERT INTO tasks (id, created_at, updated_at) VALUES ('t1', 'now', 'now')",
                "CONFLICT",
            ),
            (
                "check constraint",
                "INSERT INTO tasks (id, due_date, created_at, updated_at) VALUES ('t2', 'hoy', 'now', 'now')",
                "VALIDATION",
            ),
            (
                "not null",
                "INSERT INTO tasks (id, created_at, updated_at) VALUES ('t3', NULL, 'now')",
                "VALIDATION",
            ),
            (
                "foreign key",
                "INSERT INTO task_tags (task_id, tag_id) VALUES ('t1', 'missing')",
                "NOT_FOUND",
            ),
        ];
        for (label, sql, code) in cases {
            assert_eq!(failure_of(&conn, sql).code(), code, "{label}");
        }
    }

    #[test]
    fn missing_row_is_not_found() {
        let conn = database();
        let error: AppError = conn
            .query_row("SELECT 1 FROM tasks WHERE id = 'nope'", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect_err("no rows")
            .into();
        assert_eq!(error.to_string(), "NOT_FOUND: el elemento no existe");
    }

    #[test]
    fn other_sqlite_errors_are_db_errors_with_detail() {
        let conn = database();
        let error = failure_of(&conn, "INSERT INTO no_such_table VALUES (1)");
        assert_eq!(error.code(), "DB");
        assert!(error.message().contains("no_such_table"), "{error}");
    }

    #[test]
    fn domain_errors_are_validation_errors() {
        let validation: AppError = ValidationError::new("el título es largo").into();
        assert_eq!(validation.to_string(), "VALIDATION: el título es largo");

        let date: AppError = DateError::InvalidDate("hoy".into()).into();
        assert_eq!(
            date.to_string(),
            "VALIDATION: fecha no válida «hoy»: se espera AAAA-MM-DD"
        );
    }

    #[test]
    fn io_errors_keep_their_detail() {
        let error: AppError = io::Error::new(io::ErrorKind::NotFound, "no hay archivo").into();
        assert_eq!(error.code(), "IO");
        assert!(error.message().contains("no hay archivo"));
    }

    #[test]
    fn db_errors_are_classified() {
        let io: AppError = DbError::Io(io::Error::other("disco lleno")).into();
        assert_eq!(io.code(), "IO");

        let sqlite: AppError = DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows).into();
        assert_eq!(sqlite.code(), "NOT_FOUND");

        let too_new: AppError = DbError::SchemaTooNew {
            found: 3,
            supported: 1,
        }
        .into();
        assert_eq!(too_new.code(), "DB");
        assert!(too_new.message().contains("newer"));

        let migration: AppError = DbError::Migration {
            version: 2,
            source: rusqlite::Error::InvalidQuery,
        }
        .into();
        assert_eq!(migration.code(), "DB");

        let pragma: AppError = DbError::Pragma("x".into()).into();
        assert_eq!(pragma.code(), "DB");

        let invalid: AppError = DbError::InvalidMigrations("hueco".into()).into();
        assert_eq!(invalid.code(), "INTERNAL");
    }
}

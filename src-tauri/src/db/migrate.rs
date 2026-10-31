use rusqlite::{Connection, TransactionBehavior};

use super::DbError;

/// A schema migration. `sql` must not contain its own `BEGIN`/`COMMIT`: the runner wraps it in
/// a transaction and bumps `PRAGMA user_version` in that same transaction.
#[derive(Debug, Clone, Copy)]
pub struct Migration {
    pub version: u32,
    pub sql: &'static str,
}

/// Highest version defined by `migrations` (0 when the list is empty).
pub fn latest_version(migrations: &[Migration]) -> u32 {
    migrations.last().map_or(0, |migration| migration.version)
}

pub fn current_version(conn: &Connection) -> Result<u32, DbError> {
    let raw: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    u32::try_from(raw).map_err(|_| DbError::Pragma(format!("invalid user_version {raw}")))
}

/// Brings the database to the latest version and returns it.
///
/// Migrations run in order, each in its own `IMMEDIATE` transaction. If one fails, its changes
/// are rolled back, earlier ones stay applied and the error names the failing version.
pub fn migrate(conn: &mut Connection, migrations: &[Migration]) -> Result<u32, DbError> {
    validate(migrations)?;

    let latest = latest_version(migrations);
    let found = current_version(conn)?;
    if found > latest {
        return Err(DbError::SchemaTooNew {
            found,
            supported: latest,
        });
    }

    for migration in migrations {
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        // Re-read under the write lock: another process may have migrated in the meantime.
        if current_version(&tx)? >= migration.version {
            continue;
        }
        let apply = |error| DbError::Migration {
            version: migration.version,
            source: error,
        };
        tx.execute_batch(migration.sql).map_err(apply)?;
        // PRAGMA does not accept bound parameters; `version` is a validated u32, not user data.
        tx.execute_batch(&format!("PRAGMA user_version = {};", migration.version))
            .map_err(apply)?;
        tx.commit().map_err(apply)?;
    }
    current_version(conn)
}

fn validate(migrations: &[Migration]) -> Result<(), DbError> {
    for (index, migration) in migrations.iter().enumerate() {
        let expected = u32::try_from(index + 1)
            .map_err(|_| DbError::InvalidMigrations("too many migrations".to_string()))?;
        if migration.version != expected {
            return Err(DbError::InvalidMigrations(format!(
                "expected version {expected} at position {index}, found {}",
                migration.version
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CREATE_PARENTS: Migration = Migration {
        version: 1,
        sql: "CREATE TABLE parents (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
    };
    const CREATE_CHILDREN: Migration = Migration {
        version: 2,
        sql: "CREATE TABLE children (
                  id INTEGER PRIMARY KEY,
                  parent_id INTEGER NOT NULL REFERENCES parents(id)
              );
              CREATE INDEX ix_children_parent ON children(parent_id);",
    };
    const ADD_COLUMN: Migration = Migration {
        version: 3,
        sql: "ALTER TABLE parents ADD COLUMN note TEXT NULL;",
    };
    const BROKEN_SECOND_STATEMENT: Migration = Migration {
        version: 2,
        sql: "CREATE TABLE half_done (id INTEGER PRIMARY KEY);
              INSERT INTO table_that_does_not_exist VALUES (1);",
    };

    fn memory() -> Connection {
        Connection::open_in_memory().expect("in-memory database")
    }

    fn table_exists(conn: &Connection, name: &str) -> bool {
        conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [name],
            |row| row.get::<_, i64>(0),
        )
        .expect("sqlite_master query")
            == 1
    }

    #[test]
    fn empty_database_starts_at_version_zero() {
        assert_eq!(current_version(&memory()).expect("version"), 0);
    }

    #[test]
    fn migrates_from_empty_to_latest() {
        let mut conn = memory();
        let version = migrate(&mut conn, &[CREATE_PARENTS, CREATE_CHILDREN]).expect("migrate");

        assert_eq!(version, 2);
        assert_eq!(current_version(&conn).expect("version"), 2);
        assert!(table_exists(&conn, "parents"));
        assert!(table_exists(&conn, "children"));
    }

    #[test]
    fn empty_migration_list_is_a_no_op() {
        let mut conn = memory();
        assert_eq!(migrate(&mut conn, &[]).expect("migrate"), 0);
    }

    #[test]
    fn running_twice_does_not_reapply() {
        let mut conn = memory();
        let all = [CREATE_PARENTS, CREATE_CHILDREN];
        migrate(&mut conn, &all).expect("first run");
        // Re-applying `CREATE TABLE` would fail, so success proves nothing was re-run.
        assert_eq!(migrate(&mut conn, &all).expect("second run"), 2);
    }

    #[test]
    fn only_pending_migrations_are_applied() {
        let mut conn = memory();
        migrate(&mut conn, &[CREATE_PARENTS]).expect("v1");
        conn.execute("INSERT INTO parents (name) VALUES ('keep me')", [])
            .expect("insert");

        let version =
            migrate(&mut conn, &[CREATE_PARENTS, CREATE_CHILDREN, ADD_COLUMN]).expect("upgrade");

        assert_eq!(version, 3);
        let kept: i64 = conn
            .query_row("SELECT COUNT(*) FROM parents WHERE note IS NULL", [], |r| {
                r.get(0)
            })
            .expect("count");
        assert_eq!(kept, 1);
    }

    #[test]
    fn failing_migration_rolls_back_and_keeps_previous_version() {
        let mut conn = memory();
        let error =
            migrate(&mut conn, &[CREATE_PARENTS, BROKEN_SECOND_STATEMENT]).expect_err("must fail");

        assert!(matches!(error, DbError::Migration { version: 2, .. }));
        assert_eq!(current_version(&conn).expect("version"), 1);
        assert!(table_exists(&conn, "parents"));
        assert!(!table_exists(&conn, "half_done"));
    }

    #[test]
    fn failing_first_migration_leaves_database_untouched() {
        let mut conn = memory();
        let broken = Migration {
            version: 1,
            ..BROKEN_SECOND_STATEMENT
        };
        assert!(migrate(&mut conn, &[broken]).is_err());
        assert_eq!(current_version(&conn).expect("version"), 0);
        assert!(!table_exists(&conn, "half_done"));
    }

    #[test]
    fn database_newer_than_the_app_is_rejected() {
        let mut conn = memory();
        migrate(&mut conn, &[CREATE_PARENTS, CREATE_CHILDREN]).expect("v2");

        let error = migrate(&mut conn, &[CREATE_PARENTS]).expect_err("must fail");

        assert!(matches!(
            error,
            DbError::SchemaTooNew {
                found: 2,
                supported: 1
            }
        ));
        assert_eq!(current_version(&conn).expect("version"), 2);
    }

    #[test]
    fn non_consecutive_versions_are_rejected_before_touching_the_database() {
        let mut conn = memory();
        for list in [
            vec![CREATE_CHILDREN],
            vec![CREATE_PARENTS, ADD_COLUMN],
            vec![CREATE_PARENTS, CREATE_PARENTS],
        ] {
            let error = migrate(&mut conn, &list).expect_err("must fail");
            assert!(matches!(error, DbError::InvalidMigrations(_)));
        }
        assert_eq!(current_version(&conn).expect("version"), 0);
        assert!(!table_exists(&conn, "parents"));
    }

    #[test]
    fn latest_version_is_the_last_defined() {
        assert_eq!(latest_version(&[]), 0);
        assert_eq!(latest_version(&[CREATE_PARENTS, CREATE_CHILDREN]), 2);
    }
}

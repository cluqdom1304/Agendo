//! Key/value settings (SPEC §5.2 `settings`). Which keys and values are allowed is decided by
//! `commands/settings.rs`; this layer only stores text.

use rusqlite::{Connection, OptionalExtension, Transaction, params};

pub fn get(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
}

pub fn set(tx: &Transaction<'_>, key: &str, value: &str) -> rusqlite::Result<()> {
    tx.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, repo::with_tx};

    #[test]
    fn a_missing_key_is_none_and_set_overwrites() {
        let mut conn = db::open_in_memory(&db::MIGRATIONS).expect("open");
        assert_eq!(get(&conn, "view_mode").expect("get"), None);

        with_tx(&mut conn, |tx| set(tx, "view_mode", "week")).expect("set");
        with_tx(&mut conn, |tx| set(tx, "view_mode", "month")).expect("overwrite");

        assert_eq!(
            get(&conn, "view_mode").expect("get").as_deref(),
            Some("month")
        );
    }
}

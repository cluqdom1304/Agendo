//! Keeps `tasks_fts` and `notes_fts` in step with their tables (SPEC §5.3, D-11, D-62). Callers
//! pass the transaction that writes the row, so both changes commit or roll back together.
//!
//! The id column of an index is `UNINDEXED`, so deleting by id would scan the whole index.
//! Instead each index has a small map (`*_fts_map`, id -> FTS rowid) and rows are deleted by
//! rowid. The map and the index are always written together here; `rebuild*` restores both.

use rusqlite::{Transaction, params};

/// Which index a function works on: the SQL differs only in table and column names.
struct Index {
    fts: &'static str,
    map: &'static str,
    key: &'static str,
    base: &'static str,
}

const TASKS: Index = Index {
    fts: "tasks_fts",
    map: "tasks_fts_map",
    key: "task_id",
    base: "tasks",
};
const NOTES: Index = Index {
    fts: "notes_fts",
    map: "notes_fts_map",
    key: "note_id",
    base: "notes",
};

impl Index {
    /// Removes the FTS row of `id` (and its map entry), if any. The `key = ?1` check keeps a
    /// stale map entry from deleting a row that reused the rowid.
    fn remove(&self, tx: &Transaction<'_>, id: &str) -> rusqlite::Result<()> {
        let Index { fts, map, key, .. } = self;
        tx.prepare_cached(&format!(
            "DELETE FROM {fts} WHERE rowid = (SELECT fts_rowid FROM {map} WHERE {key} = ?1)
               AND {key} = ?1"
        ))?
        .execute(params![id])?;
        tx.prepare_cached(&format!("DELETE FROM {map} WHERE {key} = ?1"))?
            .execute(params![id])?;
        Ok(())
    }

    /// Indexes a live row that has no FTS row (nothing happens for trashed or unknown ids).
    fn add(&self, tx: &Transaction<'_>, id: &str) -> rusqlite::Result<()> {
        let Index {
            fts,
            map,
            key,
            base,
        } = self;
        let added = tx
            .prepare_cached(&format!(
                "INSERT INTO {fts} ({key}, title, content_text)
                 SELECT id, title, content_text FROM {base} WHERE id = ?1 AND deleted_at IS NULL"
            ))?
            .execute(params![id])?;
        if added > 0 {
            tx.prepare_cached(&format!(
                "INSERT OR REPLACE INTO {map} ({key}, fts_rowid) VALUES (?1, last_insert_rowid())"
            ))?
            .execute(params![id])?;
        }
        Ok(())
    }

    fn rebuild(&self, tx: &Transaction<'_>) -> rusqlite::Result<usize> {
        let Index {
            fts,
            map,
            key,
            base,
        } = self;
        tx.execute(&format!("DELETE FROM {fts}"), [])?;
        tx.execute(&format!("DELETE FROM {map}"), [])?;
        let rebuilt = tx.execute(
            &format!(
                "INSERT INTO {fts} ({key}, title, content_text)
                 SELECT id, title, content_text FROM {base} WHERE deleted_at IS NULL"
            ),
            [],
        )?;
        tx.execute(
            &format!("INSERT INTO {map} ({key}, fts_rowid) SELECT {key}, rowid FROM {fts}"),
            [],
        )?;
        Ok(rebuilt)
    }
}

/// Indexes a task that has no FTS row yet (just created).
pub fn insert(tx: &Transaction<'_>, task_id: &str) -> rusqlite::Result<()> {
    TASKS.add(tx, task_id)
}

/// Re-reads the task and makes its FTS row match: replaced if the task is live, removed if it
/// is soft-deleted or no longer exists.
pub fn sync(tx: &Transaction<'_>, task_id: &str) -> rusqlite::Result<()> {
    TASKS.remove(tx, task_id)?;
    TASKS.add(tx, task_id)
}

/// Rebuilds the whole index from `tasks`. Needed when the text extraction changes (D-03).
pub fn rebuild(tx: &Transaction<'_>) -> rusqlite::Result<usize> {
    TASKS.rebuild(tx)
}

/// Indexes a note that has no FTS row yet (`notes_fts`, same design as `tasks_fts`).
pub fn insert_note(tx: &Transaction<'_>, note_id: &str) -> rusqlite::Result<()> {
    NOTES.add(tx, note_id)
}

/// Makes the FTS row of a note match the note: replaced if it is live, removed if it is trashed
/// or gone.
pub fn sync_note(tx: &Transaction<'_>, note_id: &str) -> rusqlite::Result<()> {
    NOTES.remove(tx, note_id)?;
    NOTES.add(tx, note_id)
}

/// Rebuilds the whole notes index (needed when the text extraction changes, D-03).
pub fn rebuild_notes(tx: &Transaction<'_>) -> rusqlite::Result<usize> {
    NOTES.rebuild(tx)
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rusqlite::Connection;

    use super::*;
    use crate::db::{MIGRATIONS, open_in_memory};

    const NOW: &str = "2026-01-01T00:00:00Z";

    fn database() -> Connection {
        open_in_memory(&MIGRATIONS).expect("database")
    }

    fn raw_task(conn: &Connection, id: &str, title: &str, text: &str, deleted: bool) {
        conn.execute(
            "INSERT INTO tasks (id, title, content_text, created_at, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?4, CASE WHEN ?5 THEN ?4 END)",
            params![id, title, text, NOW, deleted],
        )
        .expect("insert task");
    }

    fn indexed(conn: &Connection, task_id: &str) -> Vec<(String, String)> {
        conn.prepare("SELECT title, content_text FROM tasks_fts WHERE task_id = ?1")
            .expect("prepare")
            .query_map([task_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("query")
            .collect::<Result<_, _>>()
            .expect("rows")
    }

    #[test]
    fn insert_indexes_a_live_task_once() {
        let mut conn = database();
        raw_task(&conn, "t1", "Comprar pan", "integral", false);
        let tx = conn.transaction().expect("tx");
        insert(&tx, "t1").expect("insert");
        tx.commit().expect("commit");

        assert_eq!(
            indexed(&conn, "t1"),
            [("Comprar pan".to_string(), "integral".to_string())]
        );
    }

    #[test]
    fn insert_skips_deleted_and_unknown_tasks() {
        let mut conn = database();
        raw_task(&conn, "gone", "Borrada", "", true);
        let tx = conn.transaction().expect("tx");
        insert(&tx, "gone").expect("deleted");
        insert(&tx, "unknown").expect("unknown");
        tx.commit().expect("commit");

        assert!(indexed(&conn, "gone").is_empty());
        assert!(indexed(&conn, "unknown").is_empty());
    }

    #[test]
    fn sync_replaces_the_row_without_duplicating_it() {
        let mut conn = database();
        raw_task(&conn, "t1", "Antes", "uno", false);
        let tx = conn.transaction().expect("tx");
        insert(&tx, "t1").expect("insert");
        tx.execute("UPDATE tasks SET title = 'Después' WHERE id = 't1'", [])
            .expect("update");
        sync(&tx, "t1").expect("sync");
        tx.commit().expect("commit");

        assert_eq!(
            indexed(&conn, "t1"),
            [("Después".to_string(), "uno".to_string())]
        );
    }

    #[test]
    fn sync_removes_soft_deleted_and_vanished_tasks() {
        let mut conn = database();
        raw_task(&conn, "t1", "Una", "", false);
        raw_task(&conn, "t2", "Otra", "", false);
        let tx = conn.transaction().expect("tx");
        insert(&tx, "t1").expect("insert");
        insert(&tx, "t2").expect("insert");
        tx.execute("UPDATE tasks SET deleted_at = ?1 WHERE id = 't1'", [NOW])
            .expect("soft delete");
        tx.execute("DELETE FROM tasks WHERE id = 't2'", [])
            .expect("hard delete");
        sync(&tx, "t1").expect("sync soft-deleted");
        sync(&tx, "t2").expect("sync vanished");
        tx.commit().expect("commit");

        assert!(indexed(&conn, "t1").is_empty());
        assert!(indexed(&conn, "t2").is_empty());
    }

    #[test]
    fn rolled_back_transaction_leaves_the_index_unchanged() {
        let mut conn = database();
        raw_task(&conn, "t1", "Una", "", false);
        {
            let tx = conn.transaction().expect("tx");
            insert(&tx, "t1").expect("insert");
        }
        assert!(indexed(&conn, "t1").is_empty());
    }

    #[test]
    fn rebuild_reindexes_only_live_tasks() {
        let mut conn = database();
        raw_task(&conn, "live", "Viva", "", false);
        raw_task(&conn, "dead", "Muerta", "", true);
        conn.execute(
            "INSERT INTO tasks_fts (task_id, title, content_text) VALUES ('stale', 'x', 'y')",
            [],
        )
        .expect("stale row");

        let tx = conn.transaction().expect("tx");
        let rebuilt = rebuild(&tx).expect("rebuild");
        tx.commit().expect("commit");

        assert_eq!(rebuilt, 1);
        assert_eq!(indexed(&conn, "live").len(), 1);
        assert!(indexed(&conn, "dead").is_empty());
        assert!(indexed(&conn, "stale").is_empty());
    }

    /// D-11 / D-62: syncing one task must not depend on the size of the index. Measured at
    /// 10 000 and 50 000 rows (before the rowid map: 2.7 ms and 21.9 ms in release).
    #[test]
    fn sync_stays_fast_with_ten_and_fifty_thousand_indexed_tasks() {
        for count in [10_000_u32, 50_000] {
            let mut conn = database();
            conn.execute(
                "WITH RECURSIVE n(i) AS (SELECT 0 UNION ALL SELECT i + 1 FROM n WHERE i < ?2 - 1)
                 INSERT INTO tasks (id, title, content_text, created_at, updated_at)
                 SELECT 't' || i, 'Tarea número ' || i, 'contenido de la tarea ' || i, ?1, ?1
                 FROM n",
                params![NOW, count],
            )
            .expect("bulk tasks");
            let tx = conn.transaction().expect("tx");
            rebuild(&tx).expect("bulk index");
            tx.commit().expect("commit");

            let last = format!("t{}", count - 1);
            let mut best = std::time::Duration::MAX;
            for _ in 0..5 {
                let tx = conn.transaction().expect("tx");
                let start = Instant::now();
                sync(&tx, &last).expect("sync");
                best = best.min(start.elapsed());
                tx.commit().expect("commit");
            }

            eprintln!("D-62 fts::sync with {count} rows: {best:?}");
            let limit = if cfg!(debug_assertions) { 20 } else { 3 };
            assert!(best.as_millis() < limit, "sync took {best:?}");
            assert_eq!(indexed(&conn, &last).len(), 1);
        }
    }

    fn map_entries(conn: &Connection, sql: &str) -> Vec<String> {
        conn.prepare(sql)
            .expect("prepare")
            .query_map([], |row| row.get(0))
            .expect("query")
            .collect::<Result<_, _>>()
            .expect("rows")
    }

    /// The map and the index never drift: one map entry per live row, pointing at that row.
    #[test]
    fn the_rowid_map_follows_every_change_of_the_index() {
        let mut conn = database();
        raw_task(&conn, "a", "Alfa", "", false);
        raw_task(&conn, "b", "Beta", "", false);
        raw_note(&conn, "n", "Nota", "", false);
        let tx = conn.transaction().expect("tx");
        insert(&tx, "a").expect("a");
        insert(&tx, "b").expect("b");
        insert_note(&tx, "n").expect("n");
        // Deleting the newest row frees its rowid for the next insert.
        tx.execute("UPDATE tasks SET deleted_at = ?1 WHERE id = 'b'", [NOW])
            .expect("trash");
        sync(&tx, "b").expect("sync b");
        raw_task(&tx, "c", "Gamma", "", false);
        insert(&tx, "c").expect("c");
        sync(&tx, "a").expect("sync a");
        sync(&tx, "a").expect("sync a again");
        tx.commit().expect("commit");

        let joined = "SELECT m.task_id FROM tasks_fts_map m
                      JOIN tasks_fts f ON f.rowid = m.fts_rowid AND f.task_id = m.task_id
                      ORDER BY 1";
        assert_eq!(map_entries(&conn, joined), ["a", "c"]);
        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM tasks_fts", [], |r| r.get(0))
            .expect("count");
        let mapped: i64 = conn
            .query_row("SELECT COUNT(*) FROM tasks_fts_map", [], |r| r.get(0))
            .expect("count");
        assert_eq!((rows, mapped), (2, 2));
        assert_eq!(
            map_entries(&conn, "SELECT note_id FROM notes_fts_map"),
            ["n"]
        );
    }

    /// Migration 012 reindexes from the base tables: duplicate and orphan rows left by the old
    /// code disappear and every live row stays searchable.
    #[test]
    fn migration_twelve_keeps_the_searchable_content() {
        use crate::db::migrate;

        let mut conn = Connection::open_in_memory().expect("open");
        migrate::migrate(&mut conn, &MIGRATIONS[..11]).expect("schema 11");
        raw_task(&conn, "t1", "Informe anual", "cifras", false);
        raw_task(&conn, "t2", "Borrada", "", true);
        raw_note(&conn, "n1", "Apuntes de álgebra", "matrices", false);
        conn.execute_batch(
            "INSERT INTO tasks_fts (task_id, title, content_text) VALUES ('t1', 'Informe anual', 'cifras');
             INSERT INTO tasks_fts (task_id, title, content_text) VALUES ('t1', 'Informe anual', 'cifras');
             INSERT INTO tasks_fts (task_id, title, content_text) VALUES ('ghost', 'Fantasma', '');
             INSERT INTO notes_fts (note_id, title, content_text) VALUES ('n1', 'Apuntes de álgebra', 'matrices');",
        )
        .expect("old index");

        migrate::migrate(&mut conn, &MIGRATIONS).expect("schema 12");

        assert_eq!(
            map_entries(
                &conn,
                "SELECT task_id FROM tasks_fts WHERE tasks_fts MATCH 'informe'"
            ),
            ["t1"]
        );
        assert!(
            map_entries(
                &conn,
                "SELECT task_id FROM tasks_fts WHERE tasks_fts MATCH 'fantasma'"
            )
            .is_empty()
        );
        assert_eq!(
            map_entries(
                &conn,
                "SELECT note_id FROM notes_fts WHERE notes_fts MATCH 'algebra'"
            ),
            ["n1"]
        );
        assert_eq!(
            map_entries(&conn, "SELECT task_id FROM tasks_fts_map"),
            ["t1"]
        );
        assert_eq!(
            map_entries(&conn, "SELECT note_id FROM notes_fts_map"),
            ["n1"]
        );

        let tx = conn.transaction().expect("tx");
        tx.execute("UPDATE tasks SET title = 'Memoria' WHERE id = 't1'", [])
            .expect("edit");
        sync(&tx, "t1").expect("sync after migration");
        tx.commit().expect("commit");
        assert_eq!(
            map_entries(
                &conn,
                "SELECT task_id FROM tasks_fts WHERE tasks_fts MATCH 'informe'"
            )
            .len(),
            0
        );
        assert_eq!(
            map_entries(
                &conn,
                "SELECT task_id FROM tasks_fts WHERE tasks_fts MATCH 'memoria'"
            ),
            ["t1"]
        );
    }

    fn raw_note(conn: &Connection, id: &str, title: &str, text: &str, deleted: bool) {
        conn.execute(
            "INSERT INTO notes (id, title, content_text, created_at, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?4, CASE WHEN ?5 THEN ?4 END)",
            params![id, title, text, NOW, deleted],
        )
        .expect("insert note");
    }

    fn indexed_note(conn: &Connection, note_id: &str) -> Vec<(String, String)> {
        conn.prepare("SELECT title, content_text FROM notes_fts WHERE note_id = ?1")
            .expect("prepare")
            .query_map([note_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("query")
            .collect::<Result<_, _>>()
            .expect("rows")
    }

    #[test]
    fn note_sync_follows_the_note_through_edit_trash_and_removal() {
        let mut conn = database();
        raw_note(&conn, "n1", "Antes", "uno", false);
        raw_note(&conn, "n2", "Otra", "", false);
        let tx = conn.transaction().expect("tx");
        insert_note(&tx, "n1").expect("insert");
        insert_note(&tx, "n2").expect("insert");
        tx.execute("UPDATE notes SET title = 'Después' WHERE id = 'n1'", [])
            .expect("update");
        sync_note(&tx, "n1").expect("sync edit");
        tx.execute("UPDATE notes SET deleted_at = ?1 WHERE id = 'n2'", [NOW])
            .expect("trash");
        sync_note(&tx, "n2").expect("sync trash");
        tx.commit().expect("commit");

        assert_eq!(
            indexed_note(&conn, "n1"),
            [("Después".to_string(), "uno".to_string())]
        );
        assert!(indexed_note(&conn, "n2").is_empty());
    }

    #[test]
    fn note_index_skips_trashed_notes_and_rebuild_replaces_everything() {
        let mut conn = database();
        raw_note(&conn, "live", "Viva", "", false);
        raw_note(&conn, "dead", "Muerta", "", true);
        conn.execute(
            "INSERT INTO notes_fts (note_id, title, content_text) VALUES ('stale', 'x', 'y')",
            [],
        )
        .expect("stale row");
        let tx = conn.transaction().expect("tx");
        insert_note(&tx, "dead").expect("insert trashed");
        let rebuilt = rebuild_notes(&tx).expect("rebuild");
        tx.commit().expect("commit");

        assert_eq!(rebuilt, 1);
        assert_eq!(indexed_note(&conn, "live").len(), 1);
        assert!(indexed_note(&conn, "dead").is_empty());
        assert!(indexed_note(&conn, "stale").is_empty());
    }
}

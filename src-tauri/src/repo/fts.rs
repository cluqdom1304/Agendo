//! Keeps `tasks_fts` in step with `tasks` (SPEC §5.3, D-11). Callers pass the transaction that
//! writes the task, so both changes commit or roll back together.

use rusqlite::{Transaction, params};

/// Indexes a task that has no FTS row yet (just created). Skips the `DELETE`, which on a
/// non-external-content table scans the whole index.
pub fn insert(tx: &Transaction<'_>, task_id: &str) -> rusqlite::Result<()> {
    tx.prepare_cached(
        "INSERT INTO tasks_fts (task_id, title, content_text)
         SELECT id, title, content_text FROM tasks WHERE id = ?1 AND deleted_at IS NULL",
    )?
    .execute(params![task_id])?;
    Ok(())
}

/// Re-reads the task and makes its FTS row match: replaced if the task is live, removed if it
/// is soft-deleted or no longer exists.
pub fn sync(tx: &Transaction<'_>, task_id: &str) -> rusqlite::Result<()> {
    tx.prepare_cached("DELETE FROM tasks_fts WHERE task_id = ?1")?
        .execute(params![task_id])?;
    insert(tx, task_id)
}

/// Rebuilds the whole index from `tasks`. Needed when the text extraction changes (D-03).
pub fn rebuild(tx: &Transaction<'_>) -> rusqlite::Result<usize> {
    tx.execute("DELETE FROM tasks_fts", [])?;
    tx.execute(
        "INSERT INTO tasks_fts (task_id, title, content_text)
         SELECT id, title, content_text FROM tasks WHERE deleted_at IS NULL",
        [],
    )
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

    /// D-11: deleting by `task_id` scans the index. Measures it at the target volume.
    #[test]
    fn sync_stays_fast_with_ten_thousand_indexed_tasks() {
        let mut conn = database();
        conn.execute(
            "WITH RECURSIVE n(i) AS (SELECT 0 UNION ALL SELECT i + 1 FROM n WHERE i < 9999)
             INSERT INTO tasks (id, title, content_text, created_at, updated_at)
             SELECT 't' || i, 'Tarea número ' || i, 'contenido de la tarea ' || i, ?1, ?1 FROM n",
            [NOW],
        )
        .expect("bulk tasks");
        conn.execute(
            "INSERT INTO tasks_fts (task_id, title, content_text)
             SELECT id, title, content_text FROM tasks",
            [],
        )
        .expect("bulk index");

        let mut best = std::time::Duration::MAX;
        for _ in 0..5 {
            let tx = conn.transaction().expect("tx");
            let start = Instant::now();
            sync(&tx, "t9999").expect("sync");
            best = best.min(start.elapsed());
            tx.commit().expect("commit");
        }

        eprintln!("D-11 fts::sync with 10 000 rows: {best:?}");
        assert!(best.as_millis() < 100, "sync took {best:?}");
        assert_eq!(indexed(&conn, "t9999").len(), 1);
    }
}

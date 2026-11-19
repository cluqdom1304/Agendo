//! A2-03 / A2-04: the hot queries use an index instead of scanning a table. Each test reads
//! `EXPLAIN QUERY PLAN` of the real SQL, so a later change that loses an index fails here.

use rusqlite::Connection;

use super::{MIGRATIONS, open_in_memory, pragmas};

fn plan(conn: &Connection, sql: &str) -> String {
    let mut statement = conn
        .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))
        .expect("explain");
    let rows = statement
        .query_map([], |row| row.get::<_, String>(3))
        .expect("plan rows");
    rows.collect::<Result<Vec<_>, _>>()
        .expect("plan")
        .join("\n")
}

fn database() -> Connection {
    open_in_memory(&MIGRATIONS).expect("database")
}

fn assert_plans(conn: &Connection, cases: &[(&str, &str)]) {
    for (sql, index) in cases {
        let plan = plan(conn, sql);
        assert!(plan.contains(index), "{sql}\n{plan}");
    }
}

#[test]
fn every_hot_lookup_uses_its_index() {
    assert_plans(
        &database(),
        &[
            // The reminders subquery of every task row (repo/tasks.rs) and the per-task lookups.
            (
                "SELECT json_group_array(r.id) FROM reminders r WHERE r.task_id = 'x'",
                "ix_reminders_task",
            ),
            (
                "SELECT n.id FROM notes n WHERE n.deleted_at IS NULL AND n.is_pinned = 1",
                "ix_notes_pinned",
            ),
            (
                "SELECT 1 FROM notes n WHERE n.folder_id = 'f' AND n.deleted_at IS NULL",
                "ix_notes_folder",
            ),
            (
                "SELECT 1 FROM note_attachments n WHERE n.attachment_id = 'a'",
                "ix_note_attachments_attachment",
            ),
            (
                "SELECT id FROM tasks WHERE deleted_at IS NULL AND due_date BETWEEN 'a' AND 'b'",
                "ix_tasks_due",
            ),
            (
                "SELECT id FROM tasks WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC",
                "ix_tasks_trashed",
            ),
            (
                "SELECT id FROM notes WHERE deleted_at IS NOT NULL AND deleted_at < 'x'",
                "ix_notes_trashed",
            ),
            (
                "SELECT id FROM reminders WHERE is_sent = 0 AND notify_at <= 'x'",
                "ix_reminders_pending",
            ),
            // `desde:` / `hasta:` over the note's last change compare the column itself.
            (
                "SELECT id FROM notes n WHERE n.deleted_at IS NULL
                   AND n.updated_at >= '2026-03-01' AND n.updated_at <= '2026-03-31Z'
                 ORDER BY n.is_pinned DESC, n.updated_at DESC, n.id",
                "ix_notes_updated",
            ),
        ],
    );
}

/// With realistic volumes and statistics the planner still picks the right index.
#[test]
fn plans_with_ten_thousand_rows_and_statistics() {
    let conn = database();
    conn.execute_batch(
        "WITH RECURSIVE n(i) AS (SELECT 0 UNION ALL SELECT i + 1 FROM n WHERE i < 9999)
         INSERT INTO tasks (id, title, due_date, position, created_at, updated_at)
         SELECT 't' || i, 'T' || i, CASE WHEN i % 2 = 0 THEN NULL ELSE '2026-02-01' END, i,
                '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z' FROM n;
         WITH RECURSIVE n(i) AS (SELECT 0 UNION ALL SELECT i + 1 FROM n WHERE i < 9999)
         INSERT INTO notes (id, title, is_pinned, created_at, updated_at)
         SELECT 'n' || i, 'N' || i, i % 100 = 0, '2026-01-01T00:00:00Z',
                strftime('%Y-%m-%dT%H:%M:%SZ', '2026-01-01', '+' || i || ' seconds') FROM n;
         WITH RECURSIVE n(i) AS (SELECT 0 UNION ALL SELECT i + 1 FROM n WHERE i < 4999)
         INSERT INTO reminders (id, task_id, notify_at, created_at)
         SELECT 'r' || i, 't' || (i * 2), '2026-03-01T00:00:00Z', '2026-01-01T00:00:00Z' FROM n;",
    )
    .expect("seed");
    pragmas::analyze(&conn).expect("analyze");

    assert_plans(
        &conn,
        &[
            (
                "SELECT n.id FROM notes n WHERE n.deleted_at IS NULL AND n.is_pinned = 1",
                "ix_notes_pinned",
            ),
            (
                "SELECT id FROM reminders r WHERE r.task_id = 't2'",
                "ix_reminders_task",
            ),
            (
                "SELECT id FROM tasks WHERE deleted_at IS NULL AND due_date BETWEEN 'a' AND 'b'",
                "ix_tasks_due",
            ),
        ],
    );
    // The list of undated tasks returns every one of them, so an index scan is all it can do.
    let todo = plan(
        &conn,
        "SELECT t.id FROM tasks t WHERE t.deleted_at IS NULL AND t.due_date IS NULL
         ORDER BY t.position, t.id",
    );
    assert!(todo.contains("USING INDEX"), "{todo}");
}

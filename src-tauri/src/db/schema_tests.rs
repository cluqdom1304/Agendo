//! Behaviour of the schema created by `001_init.sql` (SPEC §5.2).

use rusqlite::{Connection, params};

use super::{MIGRATIONS, migrate, open_in_memory};

const NOW: &str = "2026-01-01T00:00:00Z";

fn database() -> Connection {
    open_in_memory(&MIGRATIONS).expect("migrated database")
}

fn object_names(conn: &Connection, kind: &str) -> Vec<String> {
    let mut statement = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = ?1 ORDER BY name")
        .expect("prepare");
    statement
        .query_map([kind], |row| row.get(0))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows")
}

fn insert_task(conn: &Connection, id: &str) {
    conn.execute(
        "INSERT INTO tasks (id, created_at, updated_at) VALUES (?1, ?2, ?2)",
        params![id, NOW],
    )
    .expect("insert task");
}

fn insert_tag(conn: &Connection, id: &str, name: &str) {
    conn.execute(
        "INSERT INTO tags (id, name, color_hex, created_at) VALUES (?1, ?2, '#A1b2C3', ?3)",
        params![id, name, NOW],
    )
    .expect("insert tag");
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |row| row.get(0)).expect("count")
}

#[test]
fn migrating_an_empty_database_reaches_version_one() {
    let conn = database();
    assert_eq!(migrate::current_version(&conn).expect("version"), 1);
    assert_eq!(migrate::latest_version(&MIGRATIONS), 1);
}

#[test]
fn creates_every_table_and_index_of_the_spec() {
    let conn = database();

    let tables = object_names(&conn, "table");
    for table in [
        "reminders",
        "settings",
        "tags",
        "task_tags",
        "tasks",
        "tasks_fts",
    ] {
        assert!(tables.iter().any(|name| name == table), "missing {table}");
    }

    let indexes = object_names(&conn, "index");
    for index in [
        "ix_reminders_pending",
        "ix_task_tags_tag",
        "ix_tasks_deleted",
        "ix_tasks_due",
        "ix_tasks_todo",
        "ux_tags_name",
    ] {
        assert!(indexes.iter().any(|name| name == index), "missing {index}");
    }
}

#[test]
fn task_defaults_match_the_spec() {
    let conn = database();
    insert_task(&conn, "t1");

    let row: (
        String,
        String,
        String,
        Option<String>,
        i64,
        i64,
        Option<String>,
    ) = conn
        .query_row(
            "SELECT title, content_json, content_text, due_date, is_completed, position, rrule
             FROM tasks WHERE id = 't1'",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .expect("row");
    assert_eq!(row.0, "");
    assert_eq!(row.1, r#"{"type":"doc","content":[]}"#);
    assert_eq!(row.2, "");
    assert_eq!(row.3, None);
    assert_eq!((row.4, row.5), (0, 0));
    assert_eq!(row.6, None);
}

#[test]
fn task_check_constraints_reject_invalid_values() {
    let conn = database();
    let long_title = "x".repeat(501);

    let attempts: [(&str, &str, &str); 5] = [
        ("title", "title", long_title.as_str()),
        ("due_date", "due_date", "hoy"),
        ("due_date", "due_date", "2026-1-05"),
        ("due_time", "due_time", "7:5"),
        ("due_time", "due_time", "12:60"),
    ];
    for (label, column, value) in attempts {
        let sql = format!(
            "INSERT INTO tasks (id, {column}, created_at, updated_at) VALUES ('bad', ?1, ?2, ?2)"
        );
        assert!(
            conn.execute(&sql, params![value, NOW]).is_err(),
            "{label} = {value:?} should be rejected"
        );
    }
    assert!(
        conn.execute(
            "INSERT INTO tasks (id, is_completed, created_at, updated_at) VALUES ('bad', 2, ?1, ?1)",
            [NOW],
        )
        .is_err()
    );
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM tasks"), 0);
}

#[test]
fn task_accepts_boundary_values() {
    let conn = database();
    conn.execute(
        "INSERT INTO tasks (id, title, due_date, due_time, created_at, updated_at)
         VALUES ('ok', ?1, '2026-12-31', '23:59', ?2, ?2)",
        params!["x".repeat(500), NOW],
    )
    .expect("boundary task");
}

#[test]
fn tag_check_constraints_reject_invalid_values() {
    let conn = database();
    let long_name = "n".repeat(61);

    for (name, color) in [
        ("   ", "#112233"),
        ("", "#112233"),
        (long_name.as_str(), "#112233"),
        ("ok", "red"),
        ("ok", "#12345"),
        ("ok", "#GGGGGG"),
    ] {
        let result = conn.execute(
            "INSERT INTO tags (id, name, color_hex, created_at) VALUES ('bad', ?1, ?2, ?3)",
            params![name, color, NOW],
        );
        assert!(result.is_err(), "{name:?} / {color:?} should be rejected");
    }
}

#[test]
fn tag_names_are_unique_ignoring_case() {
    let conn = database();
    insert_tag(&conn, "a", "Grado");
    let duplicate = conn.execute(
        "INSERT INTO tags (id, name, color_hex, created_at) VALUES ('b', 'GRADO', '#000000', ?1)",
        [NOW],
    );
    assert!(duplicate.is_err());
}

#[test]
fn foreign_keys_reject_orphans() {
    let conn = database();
    insert_task(&conn, "t1");
    insert_tag(&conn, "g1", "Grado");

    for sql in [
        "INSERT INTO task_tags (task_id, tag_id) VALUES ('missing', 'g1')",
        "INSERT INTO task_tags (task_id, tag_id) VALUES ('t1', 'missing')",
        "INSERT INTO reminders (id, task_id, notify_at, created_at) VALUES ('r', 'missing', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
    ] {
        assert!(conn.execute(sql, []).is_err(), "{sql}");
    }
}

#[test]
fn deleting_a_task_cascades_to_tags_links_and_reminders() {
    let conn = database();
    insert_task(&conn, "t1");
    insert_tag(&conn, "g1", "Grado");
    conn.execute("INSERT INTO task_tags VALUES ('t1', 'g1')", [])
        .expect("link");
    conn.execute(
        "INSERT INTO reminders (id, task_id, notify_at, created_at) VALUES ('r1', 't1', ?1, ?1)",
        [NOW],
    )
    .expect("reminder");

    conn.execute("DELETE FROM tasks WHERE id = 't1'", [])
        .expect("delete");

    assert_eq!(count(&conn, "SELECT COUNT(*) FROM task_tags"), 0);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM reminders"), 0);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM tags"), 1);
}

#[test]
fn deleting_a_tag_removes_its_links_and_orphans_its_children() {
    let conn = database();
    insert_task(&conn, "t1");
    insert_tag(&conn, "parent", "Grado");
    insert_tag(&conn, "child", "Álgebra");
    conn.execute(
        "UPDATE tags SET parent_id = 'parent' WHERE id = 'child'",
        [],
    )
    .expect("set parent");
    conn.execute("INSERT INTO task_tags VALUES ('t1', 'parent')", [])
        .expect("link");

    conn.execute("DELETE FROM tags WHERE id = 'parent'", [])
        .expect("delete");

    assert_eq!(count(&conn, "SELECT COUNT(*) FROM task_tags"), 0);
    assert_eq!(
        count(
            &conn,
            "SELECT COUNT(*) FROM tags WHERE id = 'child' AND parent_id IS NULL"
        ),
        1
    );
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM tasks"), 1);
}

#[test]
fn fts_matches_with_and_without_accents_and_ignores_case() {
    let conn = database();
    conn.execute(
        "INSERT INTO tasks_fts (task_id, title, content_text) VALUES ('t1', 'Reunión de planificación', 'Preparar el CALENDARIO')",
        [],
    )
    .expect("index");

    for query in ["reunion", "reunión", "PLANIFICACION", "calendario", "plan*"] {
        let hits: Vec<String> = conn
            .prepare("SELECT task_id FROM tasks_fts WHERE tasks_fts MATCH ?1")
            .expect("prepare")
            .query_map([query], |row| row.get(0))
            .expect("query")
            .collect::<Result<_, _>>()
            .expect("rows");
        assert_eq!(hits, ["t1"], "query {query:?}");
    }
    assert_eq!(
        count(
            &conn,
            "SELECT COUNT(*) FROM tasks_fts WHERE tasks_fts MATCH 'inexistente'"
        ),
        0
    );
}

#[test]
fn fts_rows_can_be_deleted_by_task_id() {
    let conn = database();
    for id in ["t1", "t2"] {
        conn.execute(
            "INSERT INTO tasks_fts (task_id, title, content_text) VALUES (?1, 'comprar pan', '')",
            [id],
        )
        .expect("index");
    }

    conn.execute("DELETE FROM tasks_fts WHERE task_id = ?1", ["t1"])
        .expect("delete");

    let remaining: String = conn
        .query_row("SELECT task_id FROM tasks_fts", [], |row| row.get(0))
        .expect("single row");
    assert_eq!(remaining, "t2");
}

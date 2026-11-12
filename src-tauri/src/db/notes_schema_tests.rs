//! Schema of migrations 004 (notes and folders) and 005 (`tasks.source_note_id`), and the
//! upgrade of a database created by v1.0.x (F7-08).

use rusqlite::{Connection, params};

use super::{MIGRATIONS, migrate, open_in_memory};

const NOW: &str = "2026-01-01T00:00:00Z";

fn database() -> Connection {
    open_in_memory(&MIGRATIONS).expect("migrated database")
}

fn folder(
    conn: &Connection,
    id: &str,
    name: &str,
    parent: Option<&str>,
) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO folders (id, name, parent_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id, name, parent, NOW],
    )
}

fn note(conn: &Connection, id: &str, folder_id: Option<&str>) {
    conn.execute(
        "INSERT INTO notes (id, folder_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        params![id, folder_id, NOW],
    )
    .expect("insert note");
}

fn task(conn: &Connection, id: &str) {
    conn.execute(
        "INSERT INTO tasks (id, created_at, updated_at) VALUES (?1, ?2, ?2)",
        params![id, NOW],
    )
    .expect("insert task");
}

fn tag(conn: &Connection, id: &str, name: &str) {
    conn.execute(
        "INSERT INTO tags (id, name, color_hex, created_at) VALUES (?1, ?2, '#112233', ?3)",
        params![id, name, NOW],
    )
    .expect("insert tag");
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |row| row.get(0)).expect("count")
}

#[test]
fn notes_tables_and_indexes_exist() {
    let conn = database();
    let names: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master")
        .expect("prepare")
        .query_map([], |row| row.get(0))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows");
    for expected in [
        "folders",
        "notes",
        "note_tags",
        "note_tasks",
        "note_links",
        "notes_fts",
        "ux_folders_sibling_name",
        "ix_notes_folder",
        "ix_note_tags_tag",
        "ix_note_tasks_task",
        "ix_note_links_target",
        "ix_tasks_source_note",
    ] {
        assert!(
            names.iter().any(|name| name == expected),
            "missing {expected}"
        );
    }
}

#[test]
fn note_defaults_and_checks() {
    let conn = database();
    note(&conn, "n1", None);
    let (title, content, pinned, deleted): (String, String, i64, Option<String>) = conn
        .query_row(
            "SELECT title, content_json, is_pinned, deleted_at FROM notes WHERE id = 'n1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("row");
    assert_eq!(title, "");
    assert_eq!(content, r#"{"type":"doc","content":[]}"#);
    assert_eq!(pinned, 0);
    assert_eq!(deleted, None);

    let long = "a".repeat(501);
    assert!(
        conn.execute(
            "INSERT INTO notes (id, title, created_at, updated_at) VALUES ('x', ?1, ?2, ?2)",
            params![long, NOW],
        )
        .is_err()
    );
    assert!(
        conn.execute(
            "INSERT INTO notes (id, is_pinned, created_at, updated_at) VALUES ('y', 2, ?1, ?1)",
            [NOW],
        )
        .is_err()
    );
}

#[test]
fn folder_names_are_limited_and_unique_among_siblings() {
    let conn = database();
    assert!(folder(&conn, "f1", "Trabajo", None).is_ok());
    assert!(
        folder(&conn, "f2", "trabajo", None).is_err(),
        "same name at the root"
    );
    assert!(
        folder(&conn, "f3", "Trabajo", Some("f1")).is_ok(),
        "same name, other parent"
    );
    assert!(
        folder(&conn, "f4", "TRABAJO", Some("f1")).is_err(),
        "sibling in f1"
    );
    assert!(folder(&conn, "f5", "   ", None).is_err(), "blank");
    assert!(
        folder(&conn, "f6", &"a".repeat(101), None).is_err(),
        "too long"
    );
    assert!(
        folder(&conn, "f7", &"a".repeat(100), None).is_ok(),
        "exactly 100"
    );
}

#[test]
fn deleting_a_folder_keeps_its_notes_in_the_inbox_and_promotes_children() {
    let conn = database();
    folder(&conn, "root", "Raíz", None).expect("root");
    folder(&conn, "child", "Hija", Some("root")).expect("child");
    note(&conn, "n1", Some("root"));

    conn.execute("DELETE FROM folders WHERE id = 'root'", [])
        .expect("delete");

    let folder_id: Option<String> = conn
        .query_row("SELECT folder_id FROM notes WHERE id = 'n1'", [], |row| {
            row.get(0)
        })
        .expect("note");
    assert_eq!(folder_id, None);
    let parent: Option<String> = conn
        .query_row(
            "SELECT parent_id FROM folders WHERE id = 'child'",
            [],
            |row| row.get(0),
        )
        .expect("child");
    assert_eq!(parent, None);
}

#[test]
fn note_tags_share_the_tags_table_and_cascade() {
    let conn = database();
    tag(&conn, "tag", "Física");
    note(&conn, "n1", None);
    task(&conn, "t1");
    conn.execute(
        "INSERT INTO note_tags (note_id, tag_id) VALUES ('n1', 'tag')",
        [],
    )
    .expect("link note");
    conn.execute(
        "INSERT INTO task_tags (task_id, tag_id) VALUES ('t1', 'tag')",
        [],
    )
    .expect("link task");
    assert!(
        conn.execute(
            "INSERT INTO note_tags (note_id, tag_id) VALUES ('n1', 'nope')",
            []
        )
        .is_err(),
        "unknown tag"
    );

    conn.execute("DELETE FROM tags WHERE id = 'tag'", [])
        .expect("delete tag");
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM note_tags"), 0);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM task_tags"), 0);

    tag(&conn, "t2", "B");
    conn.execute(
        "INSERT INTO note_tags (note_id, tag_id) VALUES ('n1', 't2')",
        [],
    )
    .expect("link");
    conn.execute("DELETE FROM notes WHERE id = 'n1'", [])
        .expect("delete note");
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM note_tags"), 0);
}

#[test]
fn note_references_cascade_from_either_end() {
    let conn = database();
    note(&conn, "a", None);
    note(&conn, "b", None);
    task(&conn, "t1");
    conn.execute("INSERT INTO note_links VALUES ('a', 'b')", [])
        .expect("link");
    conn.execute("INSERT INTO note_tasks VALUES ('a', 't1')", [])
        .expect("ref");
    assert!(
        conn.execute("INSERT INTO note_links VALUES ('a', 'ghost')", [])
            .is_err(),
        "dangling link target"
    );

    conn.execute("DELETE FROM tasks WHERE id = 't1'", [])
        .expect("delete task");
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM note_tasks"), 0);
    conn.execute("DELETE FROM notes WHERE id = 'b'", [])
        .expect("delete target");
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM note_links"), 0);
}

#[test]
fn a_task_keeps_living_when_its_source_note_is_deleted() {
    let conn = database();
    note(&conn, "n1", None);
    task(&conn, "t1");
    conn.execute("UPDATE tasks SET source_note_id = 'n1' WHERE id = 't1'", [])
        .expect("set source");
    assert!(
        conn.execute(
            "UPDATE tasks SET source_note_id = 'ghost' WHERE id = 't1'",
            []
        )
        .is_err(),
        "unknown source note"
    );

    conn.execute("DELETE FROM notes WHERE id = 'n1'", [])
        .expect("delete note");
    let source: Option<String> = conn
        .query_row(
            "SELECT source_note_id FROM tasks WHERE id = 't1'",
            [],
            |row| row.get(0),
        )
        .expect("task");
    assert_eq!(source, None);
}

#[test]
fn notes_fts_matches_without_accents_and_by_prefix() {
    let conn = database();
    conn.execute(
        "INSERT INTO notes_fts (note_id, title, content_text)
         VALUES ('n1', 'Reunión', 'Preparar el CALENDARIO')",
        [],
    )
    .expect("index");
    let hits = |query: &str| -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM notes_fts WHERE notes_fts MATCH ?1",
            [query],
            |row| row.get(0),
        )
        .expect("count")
    };
    assert_eq!(hits("reunion"), 1);
    assert_eq!(hits("calend*"), 1);
    assert_eq!(hits("inexistente"), 0);
}

/// F7-08: a database written by v1.0.x (schema 3) upgrades without losing a single row, and the
/// existing tasks come out with no source note.
#[test]
fn upgrading_a_v1_database_keeps_every_row() {
    let mut conn = Connection::open_in_memory().expect("connection");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("pragma");
    assert_eq!(
        migrate::migrate(&mut conn, &MIGRATIONS[..3]).expect("v3"),
        3
    );

    tag(&conn, "tag", "Física");
    for id in ["t1", "t2", "t3"] {
        task(&conn, id);
    }
    conn.execute("INSERT INTO task_tags VALUES ('t1', 'tag')", [])
        .expect("link");
    conn.execute(
        "INSERT INTO reminders (id, task_id, notify_at, is_sent, created_at)
         VALUES ('r1', 't1', ?1, 0, ?1)",
        [NOW],
    )
    .expect("reminder");
    conn.execute("INSERT INTO settings VALUES ('theme', 'dark')", [])
        .expect("setting");

    assert_eq!(migrate::migrate(&mut conn, &MIGRATIONS).expect("v6"), 6);

    assert_eq!(count(&conn, "SELECT COUNT(*) FROM tasks"), 3);
    assert_eq!(
        count(
            &conn,
            "SELECT COUNT(*) FROM tasks WHERE source_note_id IS NULL"
        ),
        3
    );
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM task_tags"), 1);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM reminders"), 1);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM settings"), 1);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM notes"), 0);
    let violations = conn
        .prepare("PRAGMA foreign_key_check")
        .expect("prepare")
        .query_map([], |_| Ok(()))
        .expect("query")
        .count();
    assert_eq!(violations, 0);
}

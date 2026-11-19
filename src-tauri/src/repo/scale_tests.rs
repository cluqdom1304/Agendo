//! F15-01 and F15-02: the notes stay fast with 10 000 of them, and a database written by
//! version 1.0.7 (schema 3) opens in this version with its data intact.

use std::time::{Duration, Instant};

use rusqlite::{Connection, params};

use super::{
    find,
    notes::{self, Scope, Sort},
};
use crate::{
    db::{self, MIGRATIONS, migrate},
    test_support::TempDir,
};

const NOW: &str = "2026-01-01T00:00:00Z";

fn best_of(runs: u32, mut work: impl FnMut()) -> Duration {
    (0..runs)
        .map(|_| {
            let start = Instant::now();
            work();
            start.elapsed()
        })
        .min()
        .unwrap_or_default()
}

fn seed_notes(conn: &Connection, count: u32) {
    conn.execute(
        "INSERT INTO folders (id, name, position, created_at, updated_at)
         SELECT 'f' || i, 'Carpeta ' || i, i, ?1, ?1 FROM
           (WITH RECURSIVE n(i) AS (SELECT 0 UNION ALL SELECT i + 1 FROM n WHERE i < 49)
            SELECT i FROM n)",
        params![NOW],
    )
    .expect("folders");
    conn.execute(
        "WITH RECURSIVE n(i) AS (SELECT 0 UNION ALL SELECT i + 1 FROM n WHERE i < ?1 - 1)
         INSERT INTO notes (id, folder_id, title, content_json, content_text, created_at, updated_at)
         SELECT 'n' || i, 'f' || (i % 50), 'Nota número ' || i,
                '{\"type\":\"doc\",\"content\":[]}',
                'texto de la nota ' || i || ' álgebra reunión proyecto',
                strftime('%Y-%m-%dT%H:%M:%SZ', ?2, '+' || i || ' seconds'),
                strftime('%Y-%m-%dT%H:%M:%SZ', ?2, '+' || i || ' seconds')
         FROM n",
        params![count, NOW],
    )
    .expect("notes");
    conn.execute(
        "INSERT INTO notes_fts (note_id, title, content_text)
         SELECT id, title, content_text FROM notes",
        [],
    )
    .expect("index");
}

/// F15-01: list and search under 50 ms with 10 000 notes (the full list is allowed more: it
/// returns every row, and the interface virtualises it).
#[test]
fn ten_thousand_notes_list_and_search_fast() {
    let conn = db::open_in_memory(&MIGRATIONS).expect("database");
    seed_notes(&conn, 10_000);

    let mut rows = 0;
    let list_all = best_of(5, || {
        rows = notes::list(&conn, Scope::All, None, Sort::Updated)
            .expect("list")
            .len();
    });
    assert_eq!(rows, 10_000);
    let list_folder = best_of(5, || {
        let found = notes::list(&conn, Scope::Folder("f7"), None, Sort::Title).expect("folder");
        assert_eq!(found.len(), 200);
    });
    let mut worst_search = Duration::ZERO;
    for query in ["algebra", "nota 99", "reunion proyecto", "numero 5000"] {
        let time = best_of(5, || {
            let hits = find::find(&conn, query, 50).expect("find");
            assert!(!hits.is_empty(), "{query}");
        });
        worst_search = worst_search.max(time);
    }
    eprintln!(
        "F15-01 10 000 notes: list all {list_all:?}, list folder {list_folder:?}, search worst {worst_search:?}"
    );
    assert!(list_all.as_millis() < 250, "list took {list_all:?}");
    assert!(list_folder.as_millis() < 50, "folder took {list_folder:?}");
    assert!(
        worst_search.as_millis() < 50,
        "search took {worst_search:?}"
    );
}

fn seed_tasks(conn: &Connection, count: u32) {
    // Half without a date, half spread over 365 days; one in ten has a reminder-less time.
    conn.execute(
        "WITH RECURSIVE n(i) AS (SELECT 0 UNION ALL SELECT i + 1 FROM n WHERE i < ?1 - 1)
         INSERT INTO tasks (id, title, due_date, due_time, position, created_at, updated_at)
         SELECT 't' || i, 'Tarea número ' || i,
                CASE WHEN i % 2 = 0 THEN NULL
                     ELSE strftime('%Y-%m-%d', ?2, '+' || (i % 365) || ' days') END,
                CASE WHEN i % 10 = 1 THEN '09:30' ELSE NULL END,
                i, ?2, ?2
         FROM n",
        params![count, NOW],
    )
    .expect("tasks");
}

/// F16-05: the three task queries stay fast with 10 000 tasks.
#[test]
fn ten_thousand_tasks_list_fast() {
    use super::tasks::{self, Query};

    // Debug builds share the CPU with the other timing tests, so they only check that the
    // queries work on a smaller set; the 10 000-task limits are checked in release builds.
    let (count, runs) = if cfg!(debug_assertions) {
        (1_000, 1)
    } else {
        (10_000, 5)
    };
    let conn = db::open_in_memory(&MIGRATIONS).expect("database");
    seed_tasks(&conn, count);

    let mut todo = 0;
    let list_todo = best_of(runs, || {
        todo = tasks::list(&conn, Query::Todo, None).expect("todo").len();
    });
    let mut agenda = 0;
    let list_agenda = best_of(runs, || {
        agenda = tasks::list(&conn, Query::Agenda, None)
            .expect("agenda")
            .len();
    });
    let list_month = best_of(runs, || {
        let month = tasks::list(
            &conn,
            Query::Range {
                from: "2026-02-01",
                to: "2026-02-28",
            },
            None,
        )
        .expect("month");
        assert!(!month.is_empty());
    });
    assert_eq!((todo, agenda), (count as usize / 2, count as usize / 2));
    eprintln!(
        "F16-05 {count} tasks: todo {list_todo:?}, agenda {list_agenda:?}, month {list_month:?}"
    );
    if !cfg!(debug_assertions) {
        assert!(list_todo.as_millis() < 250, "todo took {list_todo:?}");
        assert!(list_agenda.as_millis() < 250, "agenda took {list_agenda:?}");
        assert!(list_month.as_millis() < 30, "month took {list_month:?}");
    }
}

/// Start-up with 10 000 notes: open the file, check the schema, no migration pending.
#[test]
fn opening_a_database_with_ten_thousand_notes_is_fast() {
    let dir = TempDir::new();
    let path = dir.database();
    {
        let conn = db::open(&path, &MIGRATIONS).expect("create");
        seed_notes(&conn, 10_000);
    }
    let open = best_of(5, || {
        db::open(&path, &MIGRATIONS).expect("open");
    });
    eprintln!("F15-01 open with 10 000 notes: {open:?}");
    assert!(open.as_millis() < 200, "open took {open:?}");
}

/// F15-02: the schema of 1.0.7 is version 3. Build it from the first three migrations, fill it
/// like a user would, and open it with every migration of this version.
#[test]
fn a_1_0_7_database_migrates_to_the_current_schema_keeping_its_data() {
    let dir = TempDir::new();
    let path = dir.database();
    std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
    {
        let mut conn = Connection::open(&path).expect("open v1");
        migrate::migrate(&mut conn, &MIGRATIONS[..3]).expect("schema 3");
        assert_eq!(migrate::current_version(&conn).expect("version"), 3);
        conn.execute_batch(
            "INSERT INTO tags (id, name, color_hex, position, created_at)
               VALUES ('g1', 'Trabajo', '#4C8DFF', 0, '2026-01-01T00:00:00Z');
             INSERT INTO tasks (id, title, content_json, content_text, due_date, due_time,
                                duration_minutes, position, created_at, updated_at)
               VALUES ('t1', 'Entrega', '{\"type\":\"doc\",\"content\":[]}',
                       'informe', '2026-02-03', '09:30', 45, 0,
                       '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
             INSERT INTO tasks (id, title, is_completed, completed_at, position, created_at, updated_at)
               VALUES ('t2', 'Hecha', 1, '2026-01-05T10:00:00Z', 1,
                       '2026-01-01T00:00:00Z', '2026-01-05T10:00:00Z');
             INSERT INTO tasks (id, title, position, created_at, updated_at, deleted_at)
               VALUES ('t3', 'Borrada', 2, '2026-01-01T00:00:00Z', '2026-01-02T00:00:00Z',
                       '2026-01-02T00:00:00Z');
             INSERT INTO task_tags (task_id, tag_id) VALUES ('t1', 'g1');
             INSERT INTO reminders (id, task_id, notify_at, offset_minutes, is_sent, created_at)
               VALUES ('r1', 't1', '2026-02-03T08:30:00Z', 60, 0, '2026-01-01T00:00:00Z');
             INSERT INTO reminders (id, task_id, notify_at, is_sent, sent_at, created_at,
                                    repeat_every, repeat_unit, repeat_count)
               VALUES ('r2', 't1', '2026-02-04T08:30:00Z', 1, '2026-02-04T08:30:01Z',
                       '2026-01-01T00:00:00Z', 1, 'day', 3);
             INSERT INTO settings (key, value) VALUES ('theme', 'dark'), ('notify_time', '08:15');
             INSERT INTO tasks_fts (task_id, title, content_text) VALUES ('t1', 'Entrega', 'informe');",
        )
        .expect("user data");
    }

    let conn = db::open(&path, &MIGRATIONS).expect("migrated");
    assert_eq!(migrate::current_version(&conn).expect("version"), 12);

    let count = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).expect(sql) };
    assert_eq!(count("SELECT COUNT(*) FROM tasks"), 3);
    let (title, due, time, duration, rrule): (String, String, String, i64, Option<String>) = conn
        .query_row(
            "SELECT title, due_date, due_time, duration_minutes, rrule FROM tasks WHERE id = 't1'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .expect("task");
    assert_eq!(
        (title.as_str(), due.as_str(), time.as_str(), duration, rrule),
        ("Entrega", "2026-02-03", "09:30", 45, None)
    );
    assert_eq!(
        count("SELECT COUNT(*) FROM tasks WHERE id = 't1' AND source_note_id IS NULL"),
        1
    );
    assert_eq!(
        count("SELECT COUNT(*) FROM tasks WHERE id = 't3' AND deleted_at IS NOT NULL"),
        1
    );
    assert_eq!(
        count(
            "SELECT COUNT(*) FROM tasks WHERE id = 't2' AND is_completed = 1 AND completed_at = '2026-01-05T10:00:00Z'"
        ),
        1
    );
    assert_eq!(
        count("SELECT COUNT(*) FROM task_tags WHERE task_id = 't1' AND tag_id = 'g1'"),
        1
    );
    assert_eq!(
        count(
            "SELECT COUNT(*) FROM reminders WHERE id = 'r2' AND repeat_every = 1 AND repeat_unit = 'day' AND repeat_count = 3 AND is_sent = 1"
        ),
        1
    );
    assert_eq!(count("SELECT COUNT(*) FROM reminders"), 2);
    assert_eq!(
        count("SELECT COUNT(*) FROM settings WHERE key = 'theme' AND value = 'dark'"),
        1
    );

    // The notes tables exist and are empty, and the old search index still finds the task.
    for table in [
        "notes",
        "folders",
        "attachments",
        "note_templates",
        "note_versions",
        "note_reminders",
    ] {
        assert_eq!(
            count(&format!("SELECT COUNT(*) FROM {table}")),
            0,
            "{table}"
        );
    }
    assert_eq!(
        super::search::search(&conn, "informe", 10)
            .expect("search")
            .len(),
        1
    );
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .expect("integrity");
    assert_eq!(integrity, "ok");
    assert_eq!(count("SELECT COUNT(*) FROM pragma_foreign_key_check"), 0);
}

//! F15-01 and F15-02: the notes stay fast with 10 000 of them, and a database written by
//! version 1.0.7 (schema 3) opens in this version with its data intact.

use std::time::{Duration, Instant};

use rusqlite::{Connection, params};

use super::{
    find, fts,
    notes::{self, Scope, Sort},
};
use crate::{
    db::{self, MIGRATIONS, migrate, pragmas},
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
        "UPDATE notes SET is_pinned = 1 WHERE CAST(substr(id, 2) AS INTEGER) % 500 = 0",
        [],
    )
    .expect("pinned");
    let tx = conn.unchecked_transaction().expect("tx");
    fts::rebuild_notes(&tx).expect("index");
    tx.commit().expect("commit");
    pragmas::analyze(conn).expect("analyze");
}

/// A2-05: the same checks at 50 000 notes. Debug builds only prove the queries work on a
/// smaller set; the time budgets are checked in release builds (`cargo test --release`).
#[test]
fn fifty_thousand_notes_list_search_and_filter_fast() {
    let (count, runs) = if cfg!(debug_assertions) {
        (2_000, 1)
    } else {
        (50_000, 5)
    };
    let conn = db::open_in_memory(&MIGRATIONS).expect("database");
    seed_notes(&conn, count);

    let list_all = best_of(runs, || {
        let all = notes::list(&conn, Scope::All, None, Sort::Updated).expect("list");
        assert_eq!(all.len(), count as usize);
    });
    // A3-01: one window of 200 notes (what a paginating list asks for) against the whole list.
    let list_window = best_of(runs, || {
        let page = super::model::Page {
            limit: Some(200),
            offset: 1000,
        };
        let found = notes::list_page(&conn, Scope::All, None, Sort::Updated, page).expect("window");
        assert_eq!(found.len(), 200);
    });
    let list_pinned = best_of(runs, || {
        let pinned = notes::list(&conn, Scope::Pinned, None, Sort::Updated).expect("pinned");
        assert_eq!(pinned.len(), count as usize / 500);
    });
    let list_folder = best_of(runs, || {
        let found = notes::list(&conn, Scope::Folder("f7"), None, Sort::Title).expect("folder");
        assert_eq!(found.len(), count as usize / 50);
    });
    let mut worst_search = Duration::ZERO;
    for query in ["algebra", "nota 99", "reunion proyecto", "numero 500"] {
        let time = best_of(runs, || {
            let hits = find::find(&conn, query, 50).expect("find");
            assert!(!hits.is_empty(), "{query}");
        });
        worst_search = worst_search.max(time);
    }
    // Notes were saved one second apart from 2026-01-01: a one-day filter is a narrow range.
    let by_date = best_of(runs, || {
        let hits = find::find(&conn, "desde:2026-01-01 hasta:2026-01-01 is:nota", 50)
            .expect("date filter");
        assert_eq!(hits.len(), 50);
    });
    let by_empty_date = best_of(runs, || {
        let hits = find::find(&conn, "desde:2026-01-02 is:nota", 50).expect("empty range");
        assert!(hits.is_empty());
    });
    eprintln!(
        "A2-05 {count} notes: list all {list_all:?}, window of 200 {list_window:?}, pinned {list_pinned:?}, folder \
         {list_folder:?}, search worst {worst_search:?}, date filter {by_date:?} / \
         empty range {by_empty_date:?}"
    );
    if !cfg!(debug_assertions) {
        assert!(list_all.as_millis() < 1000, "list took {list_all:?}");
        assert!(list_window.as_millis() < 10, "window took {list_window:?}");
        assert!(list_pinned.as_millis() < 100, "pinned took {list_pinned:?}");
        assert!(list_folder.as_millis() < 100, "folder took {list_folder:?}");
        // Every note matches these words, so the search ranks all 50 000 of them.
        assert!(worst_search.as_millis() < 300, "search {worst_search:?}");
        assert!(by_date.as_millis() < 150, "date filter took {by_date:?}");
        assert!(
            by_empty_date.as_millis() < 10,
            "empty range {by_empty_date:?}"
        );
    }
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
    // Debug builds run the same search about four times slower (release: 12-16 ms).
    let search_limit = if cfg!(debug_assertions) { 200 } else { 50 };
    assert!(
        worst_search.as_millis() < search_limit,
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

/// A2-05: the three task queries and the reminder lookups at 50 000 tasks, 20 000 of them with
/// a reminder (the agenda reads the reminders of every task it returns).
#[test]
fn fifty_thousand_tasks_list_fast() {
    use super::tasks::{self, Query};

    let (count, runs) = if cfg!(debug_assertions) {
        (2_000, 1)
    } else {
        (50_000, 5)
    };
    let conn = db::open_in_memory(&MIGRATIONS).expect("database");
    seed_tasks(&conn, count);
    conn.execute(
        "INSERT INTO reminders (id, task_id, notify_at, created_at)
         SELECT 'r' || id, id, '2026-03-01T08:00:00Z', ?1 FROM tasks
         WHERE due_date IS NOT NULL AND CAST(substr(id, 2) AS INTEGER) % 5 < 2",
        params![NOW],
    )
    .expect("reminders");
    pragmas::analyze(&conn).expect("analyze");

    let list_todo = best_of(runs, || {
        let todo = tasks::list(&conn, Query::Todo, None).expect("todo");
        assert_eq!(todo.len(), count as usize / 2);
    });
    let list_agenda = best_of(runs, || {
        let agenda = tasks::list(&conn, Query::Agenda, None).expect("agenda");
        assert_eq!(agenda.len(), count as usize / 2);
    });
    let list_month = best_of(runs, || {
        let range = Query::Range {
            from: "2026-02-01",
            to: "2026-02-28",
        };
        assert!(!tasks::list(&conn, range, None).expect("month").is_empty());
    });
    let search = best_of(runs, || {
        super::search::search(&conn, "tarea", 50).expect("search");
    });
    eprintln!(
        "A2-05 {count} tasks: todo {list_todo:?}, agenda {list_agenda:?}, month {list_month:?}, \
         search {search:?}"
    );
    if !cfg!(debug_assertions) {
        assert!(list_todo.as_millis() < 500, "todo took {list_todo:?}");
        assert!(
            list_agenda.as_millis() < 1000,
            "agenda took {list_agenda:?}"
        );
        assert!(list_month.as_millis() < 100, "month took {list_month:?}");
        assert!(search.as_millis() < 100, "search took {search:?}");
    }
}

/// A3-02: tag counters with 100 tags, two tags per task and per note, at 50 000 rows each.
#[test]
fn tag_counters_stay_fast_with_fifty_thousand_tasks_and_notes() {
    use super::tags;

    let count: u32 = if cfg!(debug_assertions) {
        2_000
    } else {
        50_000
    };
    let conn = db::open_in_memory(&MIGRATIONS).expect("database");
    seed_tasks(&conn, count);
    seed_notes(&conn, count);
    conn.execute_batch(
        "WITH RECURSIVE n(i) AS (SELECT 0 UNION ALL SELECT i + 1 FROM n WHERE i < 99)
         INSERT INTO tags (id, name, color_hex, position, created_at)
         SELECT 'g' || i, 'Etiqueta ' || i, '#4C8DFF', i, '2026-01-01T00:00:00Z' FROM n;
         INSERT INTO task_tags (task_id, tag_id)
         SELECT id, 'g' || (CAST(substr(id, 2) AS INTEGER) % 100) FROM tasks;
         INSERT INTO task_tags (task_id, tag_id)
         SELECT id, 'g' || ((CAST(substr(id, 2) AS INTEGER) + 7) % 100) FROM tasks;
         INSERT INTO note_tags (note_id, tag_id)
         SELECT id, 'g' || (CAST(substr(id, 2) AS INTEGER) % 100) FROM notes;
         INSERT INTO note_tags (note_id, tag_id)
         SELECT id, 'g' || ((CAST(substr(id, 2) AS INTEGER) + 7) % 100) FROM notes;",
    )
    .expect("tag links");
    pragmas::analyze(&conn).expect("analyze");

    let runs = if cfg!(debug_assertions) { 1 } else { 5 };
    let counters = best_of(runs, || {
        let listed = tags::list(&conn).expect("tags");
        assert_eq!(listed.len(), 100);
        assert_eq!(listed[0].pending_count, i64::from(count) * 2 / 100);
    });
    eprintln!("A3-02 tags::list with 100 tags over {count} tasks and notes: {counters:?}");
    if !cfg!(debug_assertions) {
        assert!(counters.as_millis() < 400, "tags took {counters:?}");
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
    assert_eq!(migrate::current_version(&conn).expect("version"), 14);

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

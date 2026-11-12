//! Phase 1 DoD: 10 000 pseudo-random operations leave the database consistent, with `tasks_fts`
//! matching `tasks` after every kind of mutation.

use rusqlite::Connection;

use super::{
    folders,
    model::{FolderInput, NewNote, NewTask, NoteContent, NotePatch, TagInput, TaskPatch},
    note_tags, notes, tags,
    tasks::{self, Query},
    with_tx,
};
use crate::{
    db::{MIGRATIONS, open_in_memory},
    domain::validation::{FolderParent, validate_folder_placement},
};

const OPERATIONS: usize = 10_000;
const CHECK_EVERY: usize = 2_500;
const WORDS: [&str; 8] = [
    "álgebra",
    "física",
    "reunión",
    "examen",
    "compra",
    "informe",
    "lectura",
    "práctica",
];

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next() % bound as u64).unwrap_or(0)
    }

    fn pick<'a>(&mut self, items: &'a [String]) -> Option<&'a String> {
        if items.is_empty() {
            None
        } else {
            items.get(self.below(items.len()))
        }
    }

    fn phrase(&mut self) -> String {
        let first = WORDS[self.below(WORDS.len())];
        let second = WORDS[self.below(WORDS.len())];
        format!("{first} {second}")
    }

    fn date(&mut self) -> Option<String> {
        match self.below(4) {
            0 => None,
            _ => Some(format!("2026-03-{:02}", 1 + self.below(28))),
        }
    }

    fn time(&mut self) -> Option<String> {
        match self.below(3) {
            0 => Some(format!("{:02}:{:02}", self.below(24), self.below(60))),
            _ => None,
        }
    }
}

/// Simulated clock: one operation ≈ 36 minutes, so the run spans about 250 days.
fn instant(step: usize) -> String {
    let day = step / 40;
    format!(
        "2026-{:02}-{:02}T{:02}:{:02}:00Z",
        1 + day / 28,
        1 + day % 28,
        (step % 40) * 36 / 60,
        (step % 40) * 36 % 60
    )
}

fn scalar_text(conn: &Connection, sql: &str) -> String {
    conn.query_row(sql, [], |row| row.get(0)).expect("scalar")
}

fn scalar_count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |row| row.get(0)).expect("count")
}

fn column(conn: &Connection, sql: &str) -> Vec<String> {
    conn.prepare(sql)
        .expect("prepare")
        .query_map([], |row| row.get(0))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows")
}

fn assert_consistent(conn: &Connection, step: usize) {
    assert_eq!(
        scalar_text(conn, "PRAGMA integrity_check"),
        "ok",
        "step {step}"
    );
    assert!(
        column(conn, "SELECT \"table\" FROM pragma_foreign_key_check").is_empty(),
        "foreign key violations at step {step}"
    );
    assert_eq!(
        column(conn, "SELECT task_id FROM tasks_fts ORDER BY task_id"),
        column(
            conn,
            "SELECT id FROM tasks WHERE deleted_at IS NULL ORDER BY id"
        ),
        "FTS ids differ from live tasks at step {step}"
    );
    let stale = scalar_count(
        conn,
        "SELECT COUNT(*) FROM tasks t JOIN tasks_fts f ON f.task_id = t.id
          WHERE f.title <> t.title OR f.content_text <> t.content_text",
    );
    assert_eq!(stale, 0, "stale FTS rows at step {step}");
    let broken = scalar_count(
        conn,
        "SELECT COUNT(*) FROM tasks WHERE (is_completed = 1) <> (completed_at IS NOT NULL)",
    );
    assert_eq!(
        broken, 0,
        "completed_at out of step with is_completed at step {step}"
    );
}

#[test]
fn ten_thousand_random_operations_keep_the_database_consistent() {
    let mut conn = open_in_memory(&MIGRATIONS).expect("database");
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut task_ids: Vec<String> = Vec::new();
    let mut tag_ids: Vec<String> = Vec::new();
    let mut next_id = 0_usize;

    for step in 0..OPERATIONS {
        let now = instant(step);
        match rng.below(12) {
            0 | 1 => {
                next_id += 1;
                let id = format!("t{next_id}");
                let mut chosen = Vec::new();
                for _ in 0..rng.below(3) {
                    chosen.extend(rng.pick(&tag_ids).cloned());
                }
                let new = NewTask {
                    id: id.clone(),
                    title: rng.phrase(),
                    content_text: rng.phrase(),
                    due_date: rng.date(),
                    due_time: rng.time(),
                    tag_ids: chosen,
                    now,
                    ..NewTask::default()
                };
                with_tx(&mut conn, |tx| tasks::create(tx, &new)).expect("create");
                task_ids.push(id);
            }
            2 => {
                let Some(id) = rng.pick(&task_ids).cloned() else {
                    continue;
                };
                let patch = TaskPatch {
                    title: (rng.below(2) == 0).then(|| rng.phrase()),
                    content_text: (rng.below(2) == 0).then(|| rng.phrase()),
                    ..TaskPatch::default()
                };
                with_tx(&mut conn, |tx| tasks::update(tx, &id, &patch, &now)).expect("update text");
            }
            3 => {
                let Some(id) = rng.pick(&task_ids).cloned() else {
                    continue;
                };
                let patch = TaskPatch {
                    due_date: Some(rng.date()),
                    due_time: Some(rng.time()),
                    ..TaskPatch::default()
                };
                with_tx(&mut conn, |tx| tasks::update(tx, &id, &patch, &now)).expect("update date");
            }
            4 => {
                let Some(id) = rng.pick(&task_ids).cloned() else {
                    continue;
                };
                let completed = rng.below(3) != 0;
                with_tx(&mut conn, |tx| {
                    tasks::set_completed(tx, &id, completed, &now)
                })
                .expect("complete");
            }
            5 => {
                let Some(id) = rng.pick(&task_ids).cloned() else {
                    continue;
                };
                with_tx(&mut conn, |tx| tasks::soft_delete(tx, &id, &now)).expect("delete");
            }
            6 => {
                let Some(id) = rng.pick(&task_ids).cloned() else {
                    continue;
                };
                with_tx(&mut conn, |tx| tasks::restore(tx, &id, &now)).expect("restore");
            }
            7 => {
                let Some(id) = rng.pick(&task_ids).cloned() else {
                    continue;
                };
                let mut chosen = Vec::new();
                for _ in 0..rng.below(4) {
                    chosen.extend(rng.pick(&tag_ids).cloned());
                }
                with_tx(&mut conn, |tx| tags::set_task_tags(tx, &id, &chosen, &now))
                    .expect("set tags");
            }
            8 => {
                next_id += 1;
                let existing = rng.pick(&tag_ids).cloned().filter(|_| rng.below(2) == 0);
                let id = existing.unwrap_or_else(|| format!("g{next_id}"));
                let input = TagInput {
                    id: id.clone(),
                    name: format!("etiqueta {next_id}"),
                    color_hex: format!("#{:06X}", rng.below(0x0100_0000)),
                    parent_id: None,
                    now,
                };
                with_tx(&mut conn, |tx| tags::upsert(tx, &input)).expect("upsert tag");
                if !tag_ids.contains(&id) {
                    tag_ids.push(id);
                }
            }
            9 => {
                if let Some(id) = rng.pick(&tag_ids).cloned() {
                    with_tx(&mut conn, |tx| tags::delete(tx, &id)).expect("delete tag");
                    tag_ids.retain(|existing| *existing != id);
                }
            }
            10 => {
                with_tx(&mut conn, |tx| tasks::purge_deleted(tx, &now)).expect("purge");
            }
            _ => {
                tasks::list(&conn, Query::Agenda, rng.pick(&tag_ids).map(String::as_str))
                    .expect("list");
            }
        }
        if (step + 1) % CHECK_EVERY == 0 {
            assert_consistent(&conn, step + 1);
        }
    }

    assert_consistent(&conn, OPERATIONS);
    let live: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE deleted_at IS NULL",
            [],
            |r| r.get(0),
        )
        .expect("count");
    assert!(
        live > 100,
        "the run should leave a populated database, got {live}"
    );
}

/// F7-07: the same kind of run with notes, folders, shared tags and references between notes
/// and tasks. After every kind of mutation `notes_fts` matches the live notes, no reference
/// dangles, and the folder tree has no cycles and at most five levels.
fn assert_notes_consistent(conn: &Connection, step: usize) {
    assert_eq!(
        scalar_text(conn, "PRAGMA integrity_check"),
        "ok",
        "step {step}"
    );
    assert!(
        column(conn, "SELECT \"table\" FROM pragma_foreign_key_check").is_empty(),
        "foreign key violations at step {step}"
    );
    assert_eq!(
        column(conn, "SELECT note_id FROM notes_fts ORDER BY note_id"),
        column(
            conn,
            "SELECT id FROM notes WHERE deleted_at IS NULL ORDER BY id"
        ),
        "notes FTS ids differ from live notes at step {step}"
    );
    let stale = scalar_count(
        conn,
        "SELECT COUNT(*) FROM notes n JOIN notes_fts f ON f.note_id = n.id
          WHERE f.title <> n.title OR f.content_text <> n.content_text",
    );
    assert_eq!(stale, 0, "stale notes FTS rows at step {step}");
    assert_eq!(
        column(conn, "SELECT task_id FROM tasks_fts ORDER BY task_id"),
        column(
            conn,
            "SELECT id FROM tasks WHERE deleted_at IS NULL ORDER BY id"
        ),
        "tasks FTS ids differ from live tasks at step {step}"
    );
    let self_links = scalar_count(
        conn,
        "SELECT COUNT(*) FROM note_links WHERE source_note_id = target_note_id",
    );
    assert_eq!(self_links, 0, "self link at step {step}");

    let reachable = scalar_count(
        conn,
        "WITH RECURSIVE tree(id, level) AS (
             SELECT id, 1 FROM folders WHERE parent_id IS NULL
             UNION ALL
             SELECT f.id, tree.level + 1 FROM folders f JOIN tree ON f.parent_id = tree.id
         ) SELECT COUNT(*) FROM tree",
    );
    let total = scalar_count(conn, "SELECT COUNT(*) FROM folders");
    assert_eq!(reachable, total, "folder cycle or orphan at step {step}");
    let deepest = scalar_count(
        conn,
        "WITH RECURSIVE tree(id, level) AS (
             SELECT id, 1 FROM folders WHERE parent_id IS NULL
             UNION ALL
             SELECT f.id, tree.level + 1 FROM folders f JOIN tree ON f.parent_id = tree.id
         ) SELECT COALESCE(MAX(level), 0) FROM tree",
    );
    assert!(
        deepest <= 5,
        "folder tree too deep ({deepest}) at step {step}"
    );
}

fn random_content(rng: &mut Rng, task_ids: &[String], note_ids: &[String]) -> NoteContent {
    let mut chosen_tasks = Vec::new();
    for _ in 0..rng.below(3) {
        chosen_tasks.extend(rng.pick(task_ids).cloned());
    }
    let mut chosen_notes = Vec::new();
    for _ in 0..rng.below(3) {
        chosen_notes.extend(rng.pick(note_ids).cloned());
    }
    NoteContent {
        json: r#"{"type":"doc","content":[]}"#.to_string(),
        text: rng.phrase(),
        task_ids: chosen_tasks,
        note_ids: chosen_notes,
    }
}

#[test]
fn ten_thousand_random_operations_with_notes_keep_the_database_consistent() {
    let mut conn = open_in_memory(&MIGRATIONS).expect("database");
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    let mut task_ids: Vec<String> = Vec::new();
    let mut note_ids: Vec<String> = Vec::new();
    let mut folder_ids: Vec<String> = Vec::new();
    let mut tag_ids: Vec<String> = Vec::new();
    let mut next_id = 0_usize;

    for step in 0..OPERATIONS {
        let now = instant(step);
        match rng.below(16) {
            0 => {
                next_id += 1;
                let id = format!("t{next_id}");
                let new = NewTask {
                    id: id.clone(),
                    title: rng.phrase(),
                    content_text: rng.phrase(),
                    source_note_id: rng.pick(&note_ids).cloned(),
                    now,
                    ..NewTask::default()
                };
                with_tx(&mut conn, |tx| tasks::create(tx, &new)).expect("create task");
                task_ids.push(id);
            }
            1..=3 => {
                next_id += 1;
                let id = format!("n{next_id}");
                let content = random_content(&mut rng, &task_ids, &note_ids);
                let mut chosen = Vec::new();
                for _ in 0..rng.below(3) {
                    chosen.extend(rng.pick(&tag_ids).cloned());
                }
                let new = NewNote {
                    id: id.clone(),
                    folder_id: rng.pick(&folder_ids).cloned().filter(|_| rng.below(3) != 0),
                    title: rng.phrase(),
                    content_json: Some(content.json),
                    content_text: content.text,
                    task_ids: content.task_ids,
                    note_ids: content.note_ids,
                    tag_ids: chosen,
                    now,
                };
                with_tx(&mut conn, |tx| notes::create(tx, &new)).expect("create note");
                note_ids.push(id);
            }
            4 | 5 => {
                let Some(id) = rng.pick(&note_ids).cloned() else {
                    continue;
                };
                let patch = NotePatch {
                    title: (rng.below(2) == 0).then(|| rng.phrase()),
                    content: (rng.below(3) != 0)
                        .then(|| random_content(&mut rng, &task_ids, &note_ids)),
                };
                with_tx(&mut conn, |tx| notes::update(tx, &id, &patch, &now)).expect("update note");
            }
            6 => {
                let Some(id) = rng.pick(&note_ids).cloned() else {
                    continue;
                };
                let folder = rng.pick(&folder_ids).cloned().filter(|_| rng.below(3) != 0);
                with_tx(&mut conn, |tx| {
                    notes::move_note(tx, &id, folder.as_deref(), &now)
                })
                .expect("move note");
            }
            7 => {
                let Some(id) = rng.pick(&note_ids).cloned() else {
                    continue;
                };
                with_tx(&mut conn, |tx| notes::soft_delete(tx, &id, &now)).expect("trash note");
            }
            8 => {
                let Some(id) = rng.pick(&note_ids).cloned() else {
                    continue;
                };
                with_tx(&mut conn, |tx| notes::restore(tx, &id, &now)).expect("restore note");
            }
            9 => {
                let Some(id) = rng.pick(&note_ids).cloned() else {
                    continue;
                };
                let mut chosen = Vec::new();
                for _ in 0..rng.below(4) {
                    chosen.extend(rng.pick(&tag_ids).cloned());
                }
                with_tx(&mut conn, |tx| {
                    note_tags::set_note_tags(tx, &id, &chosen, &now)
                })
                .expect("set note tags");
            }
            10 => {
                next_id += 1;
                let parent = rng.pick(&folder_ids).cloned().filter(|_| rng.below(3) != 0);
                let depth = match &parent {
                    Some(parent) => folders::depth(&conn, parent).expect("depth").unwrap_or(0),
                    None => 0,
                };
                if depth >= 5 {
                    continue;
                }
                let id = format!("f{next_id}");
                let input = FolderInput {
                    id: id.clone(),
                    name: format!("carpeta {next_id}"),
                    parent_id: parent,
                    now,
                };
                with_tx(&mut conn, |tx| folders::upsert(tx, &input)).expect("create folder");
                folder_ids.push(id);
            }
            11 => {
                // Move or rename a folder, applying the same rule as the command.
                let Some(id) = rng.pick(&folder_ids).cloned() else {
                    continue;
                };
                next_id += 1;
                let parent = rng.pick(&folder_ids).cloned().filter(|_| rng.below(2) == 0);
                let placement = parent.as_ref().map(|parent_id| FolderParent {
                    depth: folders::depth(&conn, parent_id)
                        .expect("depth")
                        .unwrap_or(0),
                    inside_moved_subtree: folders::is_in_subtree(&conn, &id, parent_id)
                        .expect("subtree"),
                });
                let height = folders::height(&conn, &id).expect("height");
                if validate_folder_placement(placement, height).is_err() {
                    continue;
                }
                let input = FolderInput {
                    id,
                    name: format!("carpeta {next_id}"),
                    parent_id: parent,
                    now,
                };
                with_tx(&mut conn, |tx| folders::upsert(tx, &input)).expect("move folder");
            }
            12 => {
                let Some(id) = rng.pick(&folder_ids).cloned() else {
                    continue;
                };
                with_tx(&mut conn, |tx| folders::delete_tree(tx, &id, &now))
                    .expect("delete folder");
                let remaining = column(&conn, "SELECT id FROM folders");
                folder_ids.retain(|known| remaining.contains(known));
            }
            13 => {
                next_id += 1;
                let id = format!("g{next_id}");
                let input = TagInput {
                    id: id.clone(),
                    name: format!("etiqueta {next_id}"),
                    color_hex: format!("#{:06X}", rng.below(0x0100_0000)),
                    parent_id: None,
                    now,
                };
                with_tx(&mut conn, |tx| tags::upsert(tx, &input)).expect("create tag");
                tag_ids.push(id);
            }
            14 => {
                if let Some(id) = rng.pick(&tag_ids).cloned() {
                    with_tx(&mut conn, |tx| tags::delete(tx, &id)).expect("delete tag");
                    tag_ids.retain(|existing| *existing != id);
                }
            }
            _ => {
                if rng.below(2) == 0 {
                    with_tx(&mut conn, |tx| notes::purge_deleted(tx, &now)).expect("purge notes");
                    with_tx(&mut conn, |tx| tasks::purge_deleted(tx, &now)).expect("purge tasks");
                    let (kept_notes, kept_tasks) = (
                        column(&conn, "SELECT id FROM notes"),
                        column(&conn, "SELECT id FROM tasks"),
                    );
                    note_ids.retain(|known| kept_notes.contains(known));
                    task_ids.retain(|known| kept_tasks.contains(known));
                } else if let Some(id) = rng.pick(&task_ids).cloned() {
                    with_tx(&mut conn, |tx| tasks::soft_delete(tx, &id, &now)).expect("trash task");
                }
            }
        }
        if (step + 1) % CHECK_EVERY == 0 {
            assert_notes_consistent(&conn, step + 1);
        }
    }

    assert_notes_consistent(&conn, OPERATIONS);
    let live = scalar_count(&conn, "SELECT COUNT(*) FROM notes WHERE deleted_at IS NULL");
    assert!(
        live > 100,
        "the run should leave a populated database, got {live}"
    );
}

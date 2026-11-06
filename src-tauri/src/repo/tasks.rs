use rusqlite::{Connection, OptionalExtension, Row, Transaction, params, types::Type};
use serde::de::DeserializeOwned;

use super::{
    fts,
    model::{EMPTY_DOC, NewTask, Task, TaskPatch},
    tags,
};

/// Soft-deleted tasks older than this are removed for good (SPEC §5.3).
pub const PURGE_AFTER_DAYS: i64 = 30;

/// Builds a task query: the shared column list (tags and reminders come back as JSON arrays
/// from correlated subqueries, so no N+1 reads) followed by `$suffix`.
macro_rules! task_sql {
    ($suffix:literal) => {
        concat!(
            "SELECT t.id, t.title, t.content_json, t.due_date, t.due_time, t.is_completed,
                    t.completed_at, t.position, t.created_at, t.updated_at,
                    (SELECT json_group_array(tt.tag_id ORDER BY tt.tag_id)
                       FROM task_tags tt WHERE tt.task_id = t.id),
                    (SELECT json_group_array(
                                json_object('id', r.id,
                                            'notifyAt', r.notify_at,
                                            'offsetMinutes', r.offset_minutes,
                                            'isSent', json(CASE r.is_sent WHEN 1 THEN 'true' ELSE 'false' END))
                                ORDER BY r.notify_at, r.id)
                       FROM reminders r WHERE r.task_id = t.id)
             FROM tasks t ",
            $suffix
        )
    };
}

/// Which tasks `list` returns. Always excludes the trash.
#[derive(Debug, Clone, Copy)]
pub enum Query<'a> {
    /// Tasks without date, by `position`.
    Todo,
    /// Every dated task, latest day first.
    Agenda,
    /// Dated tasks from `from` to `to` (both `YYYY-MM-DD`, inclusive), earliest day first.
    Range { from: &'a str, to: &'a str },
}

fn json_column<T: DeserializeOwned>(row: &Row<'_>, index: usize) -> rusqlite::Result<T> {
    let raw: String = row.get(index)?;
    serde_json::from_str(&raw).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(index, Type::Text, Box::new(error))
    })
}

fn map_task(row: &Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        content_json: row.get(2)?,
        due_date: row.get(3)?,
        due_time: row.get(4)?,
        is_completed: row.get(5)?,
        completed_at: row.get(6)?,
        position: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        tag_ids: json_column(row, 10)?,
        reminders: json_column(row, 11)?,
    })
}

/// A live (not trashed) task.
pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Task>> {
    conn.prepare_cached(task_sql!("WHERE t.id = ?1 AND t.deleted_at IS NULL"))?
        .query_row([id], map_task)
        .optional()
}

pub fn list(
    conn: &Connection,
    query: Query<'_>,
    tag_id: Option<&str>,
) -> rusqlite::Result<Vec<Task>> {
    match query {
        Query::Todo => conn
            .prepare_cached(task_sql!(
                "WHERE t.deleted_at IS NULL AND t.due_date IS NULL
                   AND (?1 IS NULL OR EXISTS (
                        SELECT 1 FROM task_tags x WHERE x.task_id = t.id AND x.tag_id = ?1))
                 ORDER BY t.position, t.id"
            ))?
            .query_map(params![tag_id], map_task)?
            .collect(),
        Query::Agenda => conn
            .prepare_cached(task_sql!(
                "WHERE t.deleted_at IS NULL AND t.due_date IS NOT NULL
                   AND (?1 IS NULL OR EXISTS (
                        SELECT 1 FROM task_tags x WHERE x.task_id = t.id AND x.tag_id = ?1))
                 ORDER BY t.due_date DESC, t.due_time IS NULL, t.due_time, t.position, t.id"
            ))?
            .query_map(params![tag_id], map_task)?
            .collect(),
        Query::Range { from, to } => conn
            .prepare_cached(task_sql!(
                "WHERE t.deleted_at IS NULL AND t.due_date BETWEEN ?2 AND ?3
                   AND (?1 IS NULL OR EXISTS (
                        SELECT 1 FROM task_tags x WHERE x.task_id = t.id AND x.tag_id = ?1))
                 ORDER BY t.due_date, t.due_time IS NULL, t.due_time, t.position, t.id"
            ))?
            .query_map(params![tag_id, from, to], map_task)?
            .collect(),
    }
}

/// Position that places a task last in its section: the "to do" list (`due_date` NULL) or its day.
fn next_position(conn: &Connection, due_date: Option<&str>) -> rusqlite::Result<i64> {
    conn.prepare_cached(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM tasks
         WHERE deleted_at IS NULL AND due_date IS ?1",
    )?
    .query_row([due_date], |row| row.get(0))
}

pub fn create(tx: &Transaction<'_>, new: &NewTask) -> rusqlite::Result<Task> {
    let position = next_position(tx, new.due_date.as_deref())?;
    tx.prepare_cached(
        "INSERT INTO tasks (id, title, content_json, content_text, due_date, due_time,
                            position, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
    )?
    .execute(params![
        new.id,
        new.title,
        new.content_json.as_deref().unwrap_or(EMPTY_DOC),
        new.content_text,
        new.due_date,
        new.due_time,
        position,
        new.now
    ])?;
    tags::replace_links(tx, &new.id, &new.tag_ids)?;
    fts::insert(tx, &new.id)?;
    get(tx, &new.id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

/// Applies `patch` to a live task. Changing `due_date` moves the task to the end of its new
/// section. Returns `None` if the task does not exist or is in the trash.
pub fn update(
    tx: &Transaction<'_>,
    id: &str,
    patch: &TaskPatch,
    now: &str,
) -> rusqlite::Result<Option<Task>> {
    let Some(current) = get(tx, id)? else {
        return Ok(None);
    };

    let new_position = match &patch.due_date {
        Some(date) if *date != current.due_date => Some(next_position(tx, date.as_deref())?),
        _ => None,
    };

    tx.prepare_cached(
        "UPDATE tasks SET
             title = COALESCE(?2, title),
             content_json = COALESCE(?3, content_json),
             content_text = COALESCE(?4, content_text),
             due_date = CASE WHEN ?5 THEN ?6 ELSE due_date END,
             due_time = CASE WHEN ?7 THEN ?8 ELSE due_time END,
             position = COALESCE(?9, position),
             updated_at = ?10
         WHERE id = ?1",
    )?
    .execute(params![
        id,
        patch.title,
        patch.content_json,
        patch.content_text,
        patch.due_date.is_some(),
        patch.due_date.clone().flatten(),
        patch.due_time.is_some(),
        patch.due_time.clone().flatten(),
        new_position,
        now
    ])?;

    if patch.title.is_some() || patch.content_text.is_some() {
        fts::sync(tx, id)?;
    }
    get(tx, id)
}

/// Completes or reopens a live task. Completing an already completed task keeps its original
/// `completed_at`.
pub fn set_completed(
    tx: &Transaction<'_>,
    id: &str,
    completed: bool,
    now: &str,
) -> rusqlite::Result<Option<Task>> {
    let changed = tx
        .prepare_cached(
            "UPDATE tasks SET
                 completed_at = CASE
                     WHEN ?2 = 0 THEN NULL
                     WHEN is_completed = 1 THEN completed_at
                     ELSE ?3 END,
                 is_completed = ?2,
                 updated_at = ?3
             WHERE id = ?1 AND deleted_at IS NULL",
        )?
        .execute(params![id, completed, now])?;
    if changed == 0 {
        return Ok(None);
    }
    get(tx, id)
}

/// Moves a live task into the "to do" list (`due_date` `None`) or onto a day, placing it before
/// `before_id` or, if that is `None` or not in the destination section, last. The positions of
/// the whole destination section are renumbered 0..n. Moving to the to-do list clears the time.
/// Returns `None` if the task does not exist or is in the trash.
pub fn move_task(
    tx: &Transaction<'_>,
    id: &str,
    due_date: Option<&str>,
    before_id: Option<&str>,
    now: &str,
) -> rusqlite::Result<Option<Task>> {
    if get(tx, id)?.is_none() {
        return Ok(None);
    }

    let mut ordered: Vec<String> = tx
        .prepare_cached(
            "SELECT id FROM tasks
             WHERE deleted_at IS NULL AND due_date IS ?1 AND id <> ?2
             ORDER BY position, id",
        )?
        .query_map(params![due_date, id], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    let index = before_id
        .and_then(|anchor| ordered.iter().position(|other| other == anchor))
        .unwrap_or(ordered.len());
    ordered.insert(index, id.to_string());

    tx.prepare_cached(
        "UPDATE tasks SET
             due_date = ?2,
             due_time = CASE WHEN ?2 IS NULL THEN NULL ELSE due_time END,
             updated_at = ?3
         WHERE id = ?1",
    )?
    .execute(params![id, due_date, now])?;
    let mut renumber =
        tx.prepare_cached("UPDATE tasks SET position = ?2 WHERE id = ?1 AND position <> ?2")?;
    for (position, task_id) in (0_i64..).zip(&ordered) {
        renumber.execute(params![task_id, position])?;
    }
    get(tx, id)
}

/// Moves a live task to the trash. Returns `false` if there was nothing to delete.
pub fn soft_delete(tx: &Transaction<'_>, id: &str, now: &str) -> rusqlite::Result<bool> {
    let changed = tx
        .prepare_cached(
            "UPDATE tasks SET deleted_at = ?2, updated_at = ?2
             WHERE id = ?1 AND deleted_at IS NULL",
        )?
        .execute(params![id, now])?;
    if changed == 0 {
        return Ok(false);
    }
    fts::sync(tx, id)?;
    Ok(true)
}

/// Takes a task out of the trash. Returns `None` if it is not in the trash.
pub fn restore(tx: &Transaction<'_>, id: &str, now: &str) -> rusqlite::Result<Option<Task>> {
    let changed = tx
        .prepare_cached(
            "UPDATE tasks SET deleted_at = NULL, updated_at = ?2
             WHERE id = ?1 AND deleted_at IS NOT NULL",
        )?
        .execute(params![id, now])?;
    if changed == 0 {
        return Ok(None);
    }
    fts::sync(tx, id)?;
    get(tx, id)
}

/// Trashed tasks, most recently deleted first.
pub fn list_trash(conn: &Connection) -> rusqlite::Result<Vec<Task>> {
    conn.prepare_cached(task_sql!(
        "WHERE t.deleted_at IS NOT NULL ORDER BY t.deleted_at DESC, t.id"
    ))?
    .query_map([], map_task)?
    .collect()
}

/// Permanently removes tasks trashed more than [`PURGE_AFTER_DAYS`] days before `now`
/// (UTC `YYYY-MM-DDTHH:MM:SSZ`). Tag links and reminders go with them; the FTS row was
/// already removed when the task was trashed. Returns how many tasks were purged.
pub fn purge_deleted(tx: &Transaction<'_>, now: &str) -> rusqlite::Result<usize> {
    tx.prepare_cached(
        "DELETE FROM tasks
         WHERE deleted_at IS NOT NULL
           AND deleted_at < strftime('%Y-%m-%dT%H:%M:%SZ', ?1, '-' || ?2 || ' days')",
    )?
    .execute(params![now, PURGE_AFTER_DAYS])
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rusqlite::Connection;

    use super::*;
    use crate::{
        db::{MIGRATIONS, open_in_memory},
        repo::{model::TagInput, with_tx},
    };

    const T0: &str = "2026-03-01T10:00:00Z";
    const T1: &str = "2026-03-02T10:00:00Z";

    fn database() -> Connection {
        open_in_memory(&MIGRATIONS).expect("database")
    }

    fn new_task(id: &str, title: &str, due: Option<&str>) -> NewTask {
        NewTask {
            id: id.to_string(),
            title: title.to_string(),
            due_date: due.map(str::to_string),
            now: T0.to_string(),
            ..NewTask::default()
        }
    }

    fn add(conn: &mut Connection, task: &NewTask) -> Task {
        with_tx(conn, |tx| create(tx, task)).expect("create")
    }

    fn add_tag(conn: &mut Connection, id: &str, name: &str) {
        let tag = TagInput {
            id: id.to_string(),
            name: name.to_string(),
            color_hex: "#112233".to_string(),
            parent_id: None,
            now: T0.to_string(),
        };
        with_tx(conn, |tx| tags::upsert(tx, &tag)).expect("tag");
    }

    fn fts_hits(conn: &Connection, term: &str) -> Vec<String> {
        conn.prepare("SELECT task_id FROM tasks_fts WHERE tasks_fts MATCH ?1 ORDER BY task_id")
            .expect("prepare")
            .query_map([term], |row| row.get(0))
            .expect("query")
            .collect::<Result<_, _>>()
            .expect("rows")
    }

    fn titles(tasks: &[Task]) -> Vec<&str> {
        tasks.iter().map(|task| task.title.as_str()).collect()
    }

    #[test]
    fn create_stores_defaults_and_indexes_the_task() {
        let mut conn = database();
        let task = add(&mut conn, &new_task("t1", "Comprar pan", None));

        assert_eq!(task.id, "t1");
        assert_eq!(task.title, "Comprar pan");
        assert_eq!(task.content_json, EMPTY_DOC);
        assert_eq!((task.due_date, task.due_time), (None, None));
        assert!(!task.is_completed);
        assert_eq!(task.completed_at, None);
        assert_eq!(task.position, 0);
        assert!(task.tag_ids.is_empty() && task.reminders.is_empty());
        assert_eq!(
            (task.created_at.as_str(), task.updated_at.as_str()),
            (T0, T0)
        );
        assert_eq!(fts_hits(&conn, "pan"), ["t1"]);
    }

    #[test]
    fn create_keeps_content_tags_and_time() {
        let mut conn = database();
        add_tag(&mut conn, "g1", "Física");
        add_tag(&mut conn, "g2", "Química");
        let mut input = new_task("t1", "Examen", Some("2026-03-10"));
        input.due_time = Some("09:30".to_string());
        input.content_json = Some(r#"{"type":"doc","content":[{"type":"paragraph"}]}"#.to_string());
        input.content_text = "temario completo".to_string();
        input.tag_ids = vec!["g2".to_string(), "g1".to_string(), "g2".to_string()];

        let task = add(&mut conn, &input);

        assert_eq!(task.due_date.as_deref(), Some("2026-03-10"));
        assert_eq!(task.due_time.as_deref(), Some("09:30"));
        assert!(task.content_json.contains("paragraph"));
        assert_eq!(task.tag_ids, ["g1", "g2"]);
        assert_eq!(fts_hits(&conn, "temario"), ["t1"]);
    }

    #[test]
    fn create_appends_at_the_end_of_its_section() {
        let mut conn = database();
        let todo_a = add(&mut conn, &new_task("a", "A", None));
        let todo_b = add(&mut conn, &new_task("b", "B", None));
        let day_a = add(&mut conn, &new_task("c", "C", Some("2026-03-10")));
        let day_b = add(&mut conn, &new_task("d", "D", Some("2026-03-10")));
        let other_day = add(&mut conn, &new_task("e", "E", Some("2026-03-11")));

        assert_eq!((todo_a.position, todo_b.position), (0, 1));
        assert_eq!((day_a.position, day_b.position), (0, 1));
        assert_eq!(other_day.position, 0);
    }

    #[test]
    fn create_with_invalid_data_or_unknown_tag_leaves_nothing_behind() {
        let mut conn = database();
        let bad_date = new_task("t1", "Mal", Some("mañana"));
        let mut bad_tag = new_task("t2", "Mal", None);
        bad_tag.tag_ids = vec!["nope".to_string()];

        assert!(with_tx(&mut conn, |tx| create(tx, &bad_date)).is_err());
        assert!(with_tx(&mut conn, |tx| create(tx, &bad_tag)).is_err());

        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            .expect("count");
        assert_eq!(remaining, 0);
        assert!(fts_hits(&conn, "mal").is_empty());
    }

    #[test]
    fn get_ignores_trashed_and_unknown_tasks() {
        let mut conn = database();
        add(&mut conn, &new_task("t1", "Una", None));
        with_tx(&mut conn, |tx| soft_delete(tx, "t1", T1)).expect("delete");

        assert_eq!(get(&conn, "t1").expect("get"), None);
        assert_eq!(get(&conn, "missing").expect("get"), None);
    }

    #[test]
    fn update_changes_only_the_patched_fields() {
        let mut conn = database();
        let mut input = new_task("t1", "Título", Some("2026-03-10"));
        input.due_time = Some("08:00".to_string());
        input.content_text = "notas".to_string();
        add(&mut conn, &input);

        let patch = TaskPatch {
            title: Some("Nuevo título".to_string()),
            ..TaskPatch::default()
        };
        let updated = with_tx(&mut conn, |tx| update(tx, "t1", &patch, T1))
            .expect("update")
            .expect("found");

        assert_eq!(updated.title, "Nuevo título");
        assert_eq!(updated.due_date.as_deref(), Some("2026-03-10"));
        assert_eq!(updated.due_time.as_deref(), Some("08:00"));
        assert_eq!(
            (updated.created_at.as_str(), updated.updated_at.as_str()),
            (T0, T1)
        );
        assert_eq!(fts_hits(&conn, "nuevo"), ["t1"]);
        assert!(fts_hits(&conn, "titulo").contains(&"t1".to_string()));
        assert_eq!(fts_hits(&conn, "notas"), ["t1"], "content_text kept");
    }

    #[test]
    fn update_can_clear_date_and_time_and_moves_to_the_todo_section() {
        let mut conn = database();
        add(&mut conn, &new_task("a", "A", None));
        let mut dated = new_task("b", "B", Some("2026-03-10"));
        dated.due_time = Some("08:00".to_string());
        add(&mut conn, &dated);

        let patch = TaskPatch {
            due_date: Some(None),
            due_time: Some(None),
            ..TaskPatch::default()
        };
        let moved = with_tx(&mut conn, |tx| update(tx, "b", &patch, T1))
            .expect("update")
            .expect("found");

        assert_eq!((moved.due_date, moved.due_time), (None, None));
        assert_eq!(moved.position, 1, "after the existing todo task");
    }

    #[test]
    fn update_keeps_position_when_the_date_does_not_change() {
        let mut conn = database();
        add(&mut conn, &new_task("a", "A", Some("2026-03-10")));
        add(&mut conn, &new_task("b", "B", Some("2026-03-10")));

        let patch = TaskPatch {
            due_date: Some(Some("2026-03-10".to_string())),
            due_time: Some(Some("10:15".to_string())),
            ..TaskPatch::default()
        };
        let same_day = with_tx(&mut conn, |tx| update(tx, "a", &patch, T1))
            .expect("update")
            .expect("found");

        assert_eq!(same_day.position, 0);
        assert_eq!(same_day.due_time.as_deref(), Some("10:15"));
    }

    #[test]
    fn update_moves_to_the_end_of_a_new_day() {
        let mut conn = database();
        add(&mut conn, &new_task("a", "A", Some("2026-03-11")));
        add(&mut conn, &new_task("b", "B", Some("2026-03-10")));

        let patch = TaskPatch {
            due_date: Some(Some("2026-03-11".to_string())),
            ..TaskPatch::default()
        };
        let moved = with_tx(&mut conn, |tx| update(tx, "b", &patch, T1))
            .expect("update")
            .expect("found");

        assert_eq!(moved.position, 1);
    }

    #[test]
    fn update_content_reindexes_and_an_invalid_patch_changes_nothing() {
        let mut conn = database();
        add(&mut conn, &new_task("t1", "Tarea", None));

        let notes = TaskPatch {
            content_json: Some(r#"{"type":"doc","content":[]}"#.to_string()),
            content_text: Some("apuntes de álgebra".to_string()),
            ..TaskPatch::default()
        };
        with_tx(&mut conn, |tx| update(tx, "t1", &notes, T1)).expect("update");
        assert_eq!(fts_hits(&conn, "algebra"), ["t1"]);

        let invalid = TaskPatch {
            title: Some("x".repeat(501)),
            ..TaskPatch::default()
        };
        assert!(with_tx(&mut conn, |tx| update(tx, "t1", &invalid, T1)).is_err());
        let task = get(&conn, "t1").expect("get").expect("found");
        assert_eq!(
            (task.title.as_str(), task.updated_at.as_str()),
            ("Tarea", T1),
            "title and the updated_at of the last valid update are kept"
        );
    }

    #[test]
    fn update_returns_none_for_missing_or_trashed_tasks() {
        let mut conn = database();
        add(&mut conn, &new_task("t1", "Una", None));
        with_tx(&mut conn, |tx| soft_delete(tx, "t1", T1)).expect("delete");
        let patch = TaskPatch {
            title: Some("X".to_string()),
            ..TaskPatch::default()
        };

        for id in ["t1", "missing"] {
            assert_eq!(
                with_tx(&mut conn, |tx| update(tx, id, &patch, T1)).expect("update"),
                None
            );
        }
    }

    #[test]
    fn complete_and_reopen_manage_completed_at() {
        let mut conn = database();
        add(&mut conn, &new_task("t1", "Una", None));

        let done = with_tx(&mut conn, |tx| set_completed(tx, "t1", true, T0))
            .expect("complete")
            .expect("found");
        assert!(done.is_completed);
        assert_eq!(done.completed_at.as_deref(), Some(T0));

        let again = with_tx(&mut conn, |tx| set_completed(tx, "t1", true, T1))
            .expect("complete again")
            .expect("found");
        assert_eq!(
            again.completed_at.as_deref(),
            Some(T0),
            "keeps the first time"
        );
        assert_eq!(again.updated_at, T1);

        let reopened = with_tx(&mut conn, |tx| set_completed(tx, "t1", false, T1))
            .expect("reopen")
            .expect("found");
        assert!(!reopened.is_completed);
        assert_eq!(reopened.completed_at, None);

        let missing =
            with_tx(&mut conn, |tx| set_completed(tx, "missing", true, T1)).expect("none");
        assert_eq!(missing, None);
    }

    fn positions_of(conn: &Connection, due_date: Option<&str>) -> Vec<(String, i64)> {
        conn.prepare(
            "SELECT id, position FROM tasks WHERE deleted_at IS NULL AND due_date IS ?1
             ORDER BY position, id",
        )
        .expect("prepare")
        .query_map([due_date], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows")
    }

    fn at(id: &str, position: i64) -> (String, i64) {
        (id.to_string(), position)
    }

    fn relocate(
        conn: &mut Connection,
        id: &str,
        date: Option<&str>,
        before: Option<&str>,
    ) -> Option<Task> {
        with_tx(conn, |tx| move_task(tx, id, date, before, T1)).expect("move")
    }

    #[test]
    fn move_within_a_section_reorders_and_renumbers() {
        let mut conn = database();
        for id in ["a", "b", "c", "d"] {
            add(&mut conn, &new_task(id, id, None));
        }

        let moved = relocate(&mut conn, "d", None, Some("b")).expect("found");
        assert_eq!(moved.position, 1);
        assert_eq!(
            positions_of(&conn, None),
            [at("a", 0), at("d", 1), at("b", 2), at("c", 3)]
        );

        relocate(&mut conn, "a", None, None);
        assert_eq!(
            positions_of(&conn, None),
            [at("d", 0), at("b", 1), at("c", 2), at("a", 3)]
        );
    }

    #[test]
    fn move_to_a_day_sets_the_date_and_keeps_the_time() {
        let mut conn = database();
        add(&mut conn, &new_task("x", "x", Some("2026-03-11")));
        let mut timed = new_task("t", "t", Some("2026-03-10"));
        timed.due_time = Some("09:00".to_string());
        add(&mut conn, &timed);
        add(&mut conn, &new_task("u", "u", None));

        let to_day = relocate(&mut conn, "t", Some("2026-03-11"), Some("x")).expect("found");
        assert_eq!(to_day.due_date.as_deref(), Some("2026-03-11"));
        assert_eq!(to_day.due_time.as_deref(), Some("09:00"));
        assert_eq!(to_day.updated_at, T1);
        assert_eq!(
            positions_of(&conn, Some("2026-03-11")),
            [at("t", 0), at("x", 1)]
        );

        let from_todo = relocate(&mut conn, "u", Some("2026-03-11"), None).expect("found");
        assert_eq!(from_todo.position, 2);
        assert_eq!(from_todo.due_time, None);
    }

    #[test]
    fn move_to_the_todo_list_clears_date_and_time() {
        let mut conn = database();
        add(&mut conn, &new_task("a", "a", None));
        let mut timed = new_task("b", "b", Some("2026-03-10"));
        timed.due_time = Some("09:00".to_string());
        add(&mut conn, &timed);

        let moved = relocate(&mut conn, "b", None, Some("a")).expect("found");

        assert_eq!((moved.due_date, moved.due_time), (None, None));
        assert_eq!(positions_of(&conn, None), [at("b", 0), at("a", 1)]);
        assert!(positions_of(&conn, Some("2026-03-10")).is_empty());
    }

    #[test]
    fn move_ignores_an_anchor_outside_the_destination_and_skips_missing_tasks() {
        let mut conn = database();
        add(&mut conn, &new_task("a", "a", None));
        add(&mut conn, &new_task("b", "b", Some("2026-03-10")));
        add(&mut conn, &new_task("c", "c", None));

        relocate(&mut conn, "a", None, Some("b")).expect("found");
        assert_eq!(positions_of(&conn, None), [at("c", 0), at("a", 1)]);

        assert_eq!(relocate(&mut conn, "missing", None, None), None);
        with_tx(&mut conn, |tx| soft_delete(tx, "c", T1)).expect("delete");
        assert_eq!(relocate(&mut conn, "c", None, None), None);
    }

    #[test]
    fn soft_delete_hides_the_task_and_removes_it_from_search() {
        let mut conn = database();
        add(&mut conn, &new_task("t1", "Comprar pan", None));
        add(&mut conn, &new_task("t2", "Comprar leche", None));

        assert!(with_tx(&mut conn, |tx| soft_delete(tx, "t1", T1)).expect("delete"));

        assert_eq!(
            titles(&list(&conn, Query::Todo, None).expect("list")),
            ["Comprar leche"]
        );
        assert!(fts_hits(&conn, "pan").is_empty());
        assert_eq!(fts_hits(&conn, "comprar"), ["t2"]);
        let trash = list_trash(&conn).expect("trash");
        assert_eq!(titles(&trash), ["Comprar pan"]);
        assert!(!with_tx(&mut conn, |tx| soft_delete(tx, "t1", T1)).expect("again"));
        assert!(!with_tx(&mut conn, |tx| soft_delete(tx, "missing", T1)).expect("missing"));
    }

    #[test]
    fn restore_brings_the_task_back_into_list_and_search() {
        let mut conn = database();
        add(&mut conn, &new_task("t1", "Comprar pan", None));
        with_tx(&mut conn, |tx| soft_delete(tx, "t1", T0)).expect("delete");

        let restored = with_tx(&mut conn, |tx| restore(tx, "t1", T1))
            .expect("restore")
            .expect("found");

        assert_eq!(restored.updated_at, T1);
        assert_eq!(fts_hits(&conn, "pan"), ["t1"]);
        assert!(list_trash(&conn).expect("trash").is_empty());
        assert_eq!(list(&conn, Query::Todo, None).expect("list").len(), 1);
        assert_eq!(
            with_tx(&mut conn, |tx| restore(tx, "t1", T1)).expect("again"),
            None
        );
        assert_eq!(
            with_tx(&mut conn, |tx| restore(tx, "missing", T1)).expect("missing"),
            None
        );
    }

    #[test]
    fn trash_is_ordered_by_deletion_time_descending() {
        let mut conn = database();
        for id in ["a", "b", "c"] {
            add(&mut conn, &new_task(id, id, None));
        }
        with_tx(&mut conn, |tx| soft_delete(tx, "b", "2026-03-05T00:00:00Z")).expect("b");
        with_tx(&mut conn, |tx| soft_delete(tx, "a", "2026-03-06T00:00:00Z")).expect("a");
        with_tx(&mut conn, |tx| soft_delete(tx, "c", "2026-03-04T00:00:00Z")).expect("c");

        assert_eq!(titles(&list_trash(&conn).expect("trash")), ["a", "b", "c"]);
    }

    #[test]
    fn purge_removes_only_tasks_trashed_more_than_thirty_days_ago() {
        let mut conn = database();
        for id in ["old", "edge", "recent", "live"] {
            add(&mut conn, &new_task(id, id, None));
        }
        add_tag(&mut conn, "g1", "Etiqueta");
        with_tx(&mut conn, |tx| {
            tags::set_task_tags(tx, "old", &["g1".to_string()], T0)
        })
        .expect("tag");
        conn.execute(
            "INSERT INTO reminders (id, task_id, notify_at, created_at) VALUES ('r1', 'old', ?1, ?1)",
            [T0],
        )
        .expect("reminder");
        with_tx(&mut conn, |tx| {
            soft_delete(tx, "old", "2026-01-01T09:59:59Z")
        })
        .expect("old");
        with_tx(&mut conn, |tx| {
            soft_delete(tx, "edge", "2026-01-01T10:00:00Z")
        })
        .expect("edge");
        with_tx(&mut conn, |tx| {
            soft_delete(tx, "recent", "2026-01-20T00:00:00Z")
        })
        .expect("recent");

        // 2026-01-31T10:00:00Z is exactly 30 days after `edge` and 30 days + 1 s after `old`.
        let purged =
            with_tx(&mut conn, |tx| purge_deleted(tx, "2026-01-31T10:00:00Z")).expect("purge");

        assert_eq!(purged, 1);
        assert_eq!(
            titles(&list_trash(&conn).expect("trash")),
            ["recent", "edge"]
        );
        assert_eq!(list(&conn, Query::Todo, None).expect("list").len(), 1);
        for table in ["task_tags", "reminders"] {
            let rows: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .expect("count");
            assert_eq!(rows, 0, "{table} cascades");
        }
    }

    #[test]
    fn purge_with_nothing_to_remove_returns_zero() {
        let mut conn = database();
        assert_eq!(
            with_tx(&mut conn, |tx| purge_deleted(tx, T1)).expect("purge"),
            0
        );
    }

    #[test]
    fn list_todo_and_agenda_are_split_by_date_and_ordered() {
        let mut conn = database();
        add(&mut conn, &new_task("t1", "todo 1", None));
        add(&mut conn, &new_task("t2", "todo 2", None));
        add(&mut conn, &new_task("a1", "early", Some("2026-03-01")));
        add(&mut conn, &new_task("a2", "late", Some("2026-03-20")));
        let mut timed = new_task("a3", "late timed", Some("2026-03-20"));
        timed.due_time = Some("07:00".to_string());
        add(&mut conn, &timed);

        let todo = list(&conn, Query::Todo, None).expect("todo");
        let agenda = list(&conn, Query::Agenda, None).expect("agenda");

        assert_eq!(titles(&todo), ["todo 1", "todo 2"]);
        assert_eq!(titles(&agenda), ["late timed", "late", "early"]);
    }

    #[test]
    fn list_range_is_inclusive_and_ascending() {
        let mut conn = database();
        for (id, date) in [
            ("a", "2026-02-28"),
            ("b", "2026-03-01"),
            ("c", "2026-03-31"),
            ("d", "2026-04-01"),
        ] {
            add(&mut conn, &new_task(id, id, Some(date)));
        }
        add(&mut conn, &new_task("undated", "undated", None));

        let march = list(
            &conn,
            Query::Range {
                from: "2026-03-01",
                to: "2026-03-31",
            },
            None,
        )
        .expect("range");

        assert_eq!(titles(&march), ["b", "c"]);
    }

    #[test]
    fn list_filters_by_tag_in_every_view() {
        let mut conn = database();
        add_tag(&mut conn, "g1", "Física");
        add_tag(&mut conn, "g2", "Química");
        let mut tagged = new_task("t1", "con etiqueta", None);
        tagged.tag_ids = vec!["g1".to_string()];
        add(&mut conn, &tagged);
        add(&mut conn, &new_task("t2", "sin etiqueta", None));
        let mut dated = new_task("t3", "fechada", Some("2026-03-10"));
        dated.tag_ids = vec!["g1".to_string(), "g2".to_string()];
        add(&mut conn, &dated);

        let range = Query::Range {
            from: "2026-03-01",
            to: "2026-03-31",
        };
        assert_eq!(
            titles(&list(&conn, Query::Todo, Some("g1")).expect("todo")),
            ["con etiqueta"]
        );
        assert_eq!(
            titles(&list(&conn, Query::Agenda, Some("g2")).expect("agenda")),
            ["fechada"]
        );
        assert_eq!(
            titles(&list(&conn, range, Some("g1")).expect("range")),
            ["fechada"]
        );
        assert!(
            list(&conn, Query::Todo, Some("g2"))
                .expect("todo")
                .is_empty()
        );
        assert_eq!(list(&conn, Query::Todo, None).expect("todo").len(), 2);
    }

    #[test]
    fn tasks_carry_their_tags_and_reminders() {
        let mut conn = database();
        add_tag(&mut conn, "g1", "Física");
        let mut input = new_task("t1", "Con avisos", Some("2026-03-10"));
        input.tag_ids = vec!["g1".to_string()];
        add(&mut conn, &input);
        conn.execute_batch(
            "INSERT INTO reminders (id, task_id, notify_at, offset_minutes, is_sent, created_at)
             VALUES ('r2', 't1', '2026-03-10T09:00:00Z', NULL, 1, '2026-03-01T00:00:00Z'),
                    ('r1', 't1', '2026-03-10T08:00:00Z', 60, 0, '2026-03-01T00:00:00Z');",
        )
        .expect("reminders");

        let task = get(&conn, "t1").expect("get").expect("found");

        assert_eq!(task.tag_ids, ["g1"]);
        let reminders: Vec<_> = task
            .reminders
            .iter()
            .map(|r| {
                (
                    r.id.as_str(),
                    r.notify_at.as_str(),
                    r.offset_minutes,
                    r.is_sent,
                )
            })
            .collect();
        assert_eq!(
            reminders,
            [
                ("r1", "2026-03-10T08:00:00Z", Some(60), false),
                ("r2", "2026-03-10T09:00:00Z", None, true)
            ]
        );
        let listed = list(&conn, Query::Agenda, None).expect("agenda");
        assert_eq!(listed[0], task);
    }

    #[test]
    fn stored_tasks_follow_the_display_order_of_the_domain() {
        use crate::domain::ordering::{group_by_day, sort_todo};

        let mut conn = database();
        add(
            &mut conn,
            &new_task("untimed", "untimed", Some("2026-03-10")),
        );
        let mut timed = new_task("timed", "timed", Some("2026-03-10"));
        timed.due_time = Some("09:00".to_string());
        add(&mut conn, &timed);
        add(&mut conn, &new_task("done", "done", Some("2026-03-10")));
        add(
            &mut conn,
            &new_task("other-day", "other", Some("2026-03-12")),
        );
        add(&mut conn, &new_task("t1", "todo-1", None));
        add(&mut conn, &new_task("t2", "todo-2", None));
        for id in ["done", "t1"] {
            with_tx(&mut conn, |tx| set_completed(tx, id, true, T1)).expect("complete");
        }

        let days = group_by_day(list(&conn, Query::Agenda, None).expect("agenda"));
        let mut todo = list(&conn, Query::Todo, None).expect("todo");
        sort_todo(&mut todo);

        let summary: Vec<(&str, Vec<&str>)> = days
            .iter()
            .map(|day| (day.date.as_str(), titles(&day.tasks)))
            .collect();
        assert_eq!(
            summary,
            [
                ("2026-03-12", vec!["other"]),
                ("2026-03-10", vec!["timed", "untimed", "done"]),
            ]
        );
        assert_eq!(titles(&todo), ["todo-2", "todo-1"]);
    }

    #[test]
    fn serializes_with_the_camel_case_contract() {
        let mut conn = database();
        let task = add(&mut conn, &new_task("t1", "Una", Some("2026-03-10")));

        let value = serde_json::to_value(&task).expect("json");

        let mut keys: Vec<&str> = value
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "completedAt",
                "contentJson",
                "createdAt",
                "dueDate",
                "dueTime",
                "id",
                "isCompleted",
                "position",
                "reminders",
                "tagIds",
                "title",
                "updatedAt"
            ]
        );
    }

    #[test]
    fn month_query_with_ten_thousand_tasks_is_fast() {
        let conn = database();
        conn.execute(
            "WITH RECURSIVE n(i) AS (SELECT 0 UNION ALL SELECT i + 1 FROM n WHERE i < 9999)
             INSERT INTO tasks (id, title, due_date, due_time, position, created_at, updated_at)
             SELECT 't' || i, 'Tarea ' || i,
                    date('2025-01-01', '+' || (i % 730) || ' days'),
                    CASE WHEN i % 3 = 0 THEN printf('%02d:00', i % 24) END,
                    i, ?1, ?1
               FROM n",
            [T0],
        )
        .expect("bulk");
        conn.execute(
            "INSERT INTO tags (id, name, color_hex, created_at) VALUES ('g1', 'Física', '#112233', ?1)",
            [T0],
        )
        .expect("tag");
        conn.execute(
            "INSERT INTO task_tags (task_id, tag_id)
             SELECT t.id, 'g1' FROM tasks t WHERE CAST(substr(t.id, 2) AS INTEGER) % 5 = 0",
            [],
        )
        .expect("links");
        let range = Query::Range {
            from: "2026-03-01",
            to: "2026-03-31",
        };

        let mut best = std::time::Duration::MAX;
        let mut found = 0;
        for _ in 0..5 {
            let start = Instant::now();
            found = list(&conn, range, None).expect("range").len();
            best = best.min(start.elapsed());
        }

        eprintln!("N-05 month query, 10 000 tasks, {found} rows: {best:?}");
        assert!(found > 300, "expected a populated month, got {found}");
        assert!(best.as_millis() < 30, "month query took {best:?}");
    }
}

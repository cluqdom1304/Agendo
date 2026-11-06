use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use super::model::{Tag, TagInput, TagWithCount};

fn map_tag(row: &Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        color_hex: row.get(2)?,
        parent_id: row.get(3)?,
        position: row.get(4)?,
    })
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Tag>> {
    conn.prepare_cached("SELECT id, name, color_hex, parent_id, position FROM tags WHERE id = ?1")?
        .query_row([id], map_tag)
        .optional()
}

/// Whether other tags have `id` as their parent.
pub fn has_children(conn: &Connection, id: &str) -> rusqlite::Result<bool> {
    conn.prepare_cached("SELECT EXISTS (SELECT 1 FROM tags WHERE parent_id = ?1)")?
        .query_row([id], |row| row.get(0))
}

/// All tags with their pending-task counts, in display order.
pub fn list(conn: &Connection) -> rusqlite::Result<Vec<TagWithCount>> {
    let mut statement = conn.prepare_cached(
        "SELECT g.id, g.name, g.color_hex, g.parent_id, g.position,
                (SELECT COUNT(*)
                   FROM task_tags tt JOIN tasks t ON t.id = tt.task_id
                  WHERE tt.tag_id = g.id AND t.deleted_at IS NULL AND t.is_completed = 0)
         FROM tags g
         ORDER BY g.position, g.id",
    )?;
    statement
        .query_map([], |row| {
            Ok(TagWithCount {
                tag: map_tag(row)?,
                pending_count: row.get(5)?,
            })
        })?
        .collect()
}

/// Creates the tag at the end of the list, or updates name, colour and parent of an existing
/// one (keeping its position and creation time).
pub fn upsert(tx: &Transaction<'_>, input: &TagInput) -> rusqlite::Result<Tag> {
    tx.prepare_cached(
        "INSERT INTO tags (id, name, color_hex, parent_id, position, created_at)
         VALUES (?1, ?2, ?3, ?4,
                 (SELECT COALESCE(MAX(position), -1) + 1 FROM tags), ?5)
         ON CONFLICT(id) DO UPDATE SET
             name = excluded.name,
             color_hex = excluded.color_hex,
             parent_id = excluded.parent_id",
    )?
    .execute(params![
        input.id,
        input.name,
        input.color_hex,
        input.parent_id,
        input.now
    ])?;
    get(tx, &input.id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

/// Deletes the tag. Links to tasks go with it; child tags become top-level (schema cascades).
pub fn delete(tx: &Transaction<'_>, id: &str) -> rusqlite::Result<bool> {
    let removed = tx
        .prepare_cached("DELETE FROM tags WHERE id = ?1")?
        .execute([id])?;
    Ok(removed > 0)
}

/// Replaces the tag set of a live task and touches `updated_at`. Returns `false` if the task
/// does not exist or is in the trash. An unknown tag id fails with a foreign-key error.
pub fn set_task_tags(
    tx: &Transaction<'_>,
    task_id: &str,
    tag_ids: &[String],
    now: &str,
) -> rusqlite::Result<bool> {
    let touched = tx
        .prepare_cached("UPDATE tasks SET updated_at = ?2 WHERE id = ?1 AND deleted_at IS NULL")?
        .execute(params![task_id, now])?;
    if touched == 0 {
        return Ok(false);
    }
    replace_links(tx, task_id, tag_ids)?;
    Ok(true)
}

/// Replaces the links of `task_id` without checking the task. Duplicated ids are ignored.
pub(super) fn replace_links(
    tx: &Transaction<'_>,
    task_id: &str,
    tag_ids: &[String],
) -> rusqlite::Result<()> {
    tx.prepare_cached("DELETE FROM task_tags WHERE task_id = ?1")?
        .execute([task_id])?;
    let mut insert =
        tx.prepare_cached("INSERT OR IGNORE INTO task_tags (task_id, tag_id) VALUES (?1, ?2)")?;
    for tag_id in tag_ids {
        insert.execute(params![task_id, tag_id])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;
    use crate::db::{MIGRATIONS, open_in_memory};
    use crate::repo::with_tx;

    const NOW: &str = "2026-01-01T00:00:00Z";

    fn database() -> Connection {
        open_in_memory(&MIGRATIONS).expect("database")
    }

    fn input(id: &str, name: &str, parent: Option<&str>) -> TagInput {
        TagInput {
            id: id.to_string(),
            name: name.to_string(),
            color_hex: "#0A84FF".to_string(),
            parent_id: parent.map(str::to_string),
            now: NOW.to_string(),
        }
    }

    fn save(conn: &mut Connection, tag: &TagInput) -> Tag {
        with_tx(conn, |tx| upsert(tx, tag)).expect("upsert")
    }

    fn raw_task(conn: &Connection, id: &str, completed: bool, deleted: bool) {
        conn.execute(
            "INSERT INTO tasks (id, is_completed, created_at, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?3, CASE WHEN ?4 THEN ?3 END)",
            params![id, completed, NOW, deleted],
        )
        .expect("insert task");
    }

    fn link(conn: &Connection, task: &str, tag: &str) {
        conn.execute("INSERT INTO task_tags VALUES (?1, ?2)", [task, tag])
            .expect("link");
    }

    fn tag_ids_of(conn: &Connection, task: &str) -> Vec<String> {
        conn.prepare("SELECT tag_id FROM task_tags WHERE task_id = ?1 ORDER BY tag_id")
            .expect("prepare")
            .query_map([task], |row| row.get(0))
            .expect("query")
            .collect::<Result<_, _>>()
            .expect("rows")
    }

    #[test]
    fn upsert_creates_tags_at_the_end_of_the_list() {
        let mut conn = database();
        let first = save(&mut conn, &input("a", "Matemáticas", None));
        let second = save(&mut conn, &input("b", "Física", None));

        assert_eq!(first.position, 0);
        assert_eq!(second.position, 1);
        assert_eq!(second.name, "Física");
        assert_eq!(second.color_hex, "#0A84FF");
        assert_eq!(get(&conn, "b").expect("get"), Some(second));
    }

    #[test]
    fn upsert_updates_without_moving_the_tag() {
        let mut conn = database();
        save(&mut conn, &input("a", "Uno", None));
        save(&mut conn, &input("b", "Dos", None));
        save(&mut conn, &input("c", "Tres", None));

        let mut change = input("a", "Uno renombrada", Some("c"));
        change.color_hex = "#112233".to_string();
        change.now = "2030-01-01T00:00:00Z".to_string();
        let updated = save(&mut conn, &change);

        assert_eq!(updated.position, 0);
        assert_eq!(updated.name, "Uno renombrada");
        assert_eq!(updated.color_hex, "#112233");
        assert_eq!(updated.parent_id.as_deref(), Some("c"));
        let created_at: String = conn
            .query_row("SELECT created_at FROM tags WHERE id = 'a'", [], |r| {
                r.get(0)
            })
            .expect("created_at");
        assert_eq!(created_at, NOW);
    }

    #[test]
    fn upsert_rejects_duplicate_names_ignoring_case_and_bad_data() {
        let mut conn = database();
        save(&mut conn, &input("a", "Grado", None));

        assert!(with_tx(&mut conn, |tx| upsert(tx, &input("b", "GRADO", None))).is_err());
        let mut bad_color = input("c", "Otra", None);
        bad_color.color_hex = "azul".to_string();
        assert!(with_tx(&mut conn, |tx| upsert(tx, &bad_color)).is_err());
        assert!(with_tx(&mut conn, |tx| upsert(tx, &input("d", "X", Some("nope")))).is_err());
        assert_eq!(list(&conn).expect("list").len(), 1);
    }

    #[test]
    fn has_children_reports_parents_only() {
        let mut conn = database();
        save(&mut conn, &input("parent", "Grado", None));
        save(&mut conn, &input("child", "Álgebra", Some("parent")));

        assert!(has_children(&conn, "parent").expect("parent"));
        assert!(!has_children(&conn, "child").expect("child"));
        assert!(!has_children(&conn, "missing").expect("missing"));
    }

    #[test]
    fn get_returns_none_for_unknown_ids() {
        assert_eq!(get(&database(), "missing").expect("get"), None);
    }

    #[test]
    fn list_counts_only_pending_live_tasks() {
        let mut conn = database();
        save(&mut conn, &input("a", "Con tareas", None));
        save(&mut conn, &input("b", "Vacía", None));
        raw_task(&conn, "open", false, false);
        raw_task(&conn, "done", true, false);
        raw_task(&conn, "trashed", false, true);
        for task in ["open", "done", "trashed"] {
            link(&conn, task, "a");
        }

        let tags = list(&conn).expect("list");

        assert_eq!(tags.len(), 2);
        assert_eq!((tags[0].tag.id.as_str(), tags[0].pending_count), ("a", 1));
        assert_eq!((tags[1].tag.id.as_str(), tags[1].pending_count), ("b", 0));
    }

    #[test]
    fn delete_unlinks_tasks_and_orphans_children() {
        let mut conn = database();
        save(&mut conn, &input("parent", "Grado", None));
        save(&mut conn, &input("child", "Álgebra", Some("parent")));
        raw_task(&conn, "t1", false, false);
        link(&conn, "t1", "parent");
        link(&conn, "t1", "child");

        let removed = with_tx(&mut conn, |tx| delete(tx, "parent")).expect("delete");

        assert!(removed);
        assert_eq!(tag_ids_of(&conn, "t1"), ["child"]);
        assert_eq!(
            get(&conn, "child").expect("get").map(|t| t.parent_id),
            Some(None)
        );
        assert!(!with_tx(&mut conn, |tx| delete(tx, "parent")).expect("again"));
    }

    #[test]
    fn set_task_tags_replaces_the_set_and_touches_the_task() {
        let mut conn = database();
        for (id, name) in [("a", "A"), ("b", "B"), ("c", "C")] {
            save(&mut conn, &input(id, name, None));
        }
        raw_task(&conn, "t1", false, false);
        link(&conn, "t1", "a");

        let ids = ["b".to_string(), "c".to_string(), "b".to_string()];
        let found = with_tx(&mut conn, |tx| {
            set_task_tags(tx, "t1", &ids, "2026-02-02T00:00:00Z")
        })
        .expect("set");

        assert!(found);
        assert_eq!(tag_ids_of(&conn, "t1"), ["b", "c"]);
        let updated_at: String = conn
            .query_row("SELECT updated_at FROM tasks WHERE id = 't1'", [], |r| {
                r.get(0)
            })
            .expect("updated_at");
        assert_eq!(updated_at, "2026-02-02T00:00:00Z");

        with_tx(&mut conn, |tx| set_task_tags(tx, "t1", &[], NOW)).expect("clear");
        assert!(tag_ids_of(&conn, "t1").is_empty());
    }

    #[test]
    fn set_task_tags_reports_missing_and_trashed_tasks() {
        let mut conn = database();
        save(&mut conn, &input("a", "A", None));
        raw_task(&conn, "trashed", false, true);
        let ids = ["a".to_string()];

        for task in ["missing", "trashed"] {
            let found = with_tx(&mut conn, |tx| set_task_tags(tx, task, &ids, NOW)).expect("set");
            assert!(!found, "{task}");
        }
        assert!(tag_ids_of(&conn, "trashed").is_empty());
    }

    #[test]
    fn set_task_tags_with_unknown_tag_fails_and_keeps_previous_links() {
        let mut conn = database();
        save(&mut conn, &input("a", "A", None));
        raw_task(&conn, "t1", false, false);
        link(&conn, "t1", "a");

        let ids = ["nope".to_string()];
        let result = with_tx(&mut conn, |tx| set_task_tags(tx, "t1", &ids, NOW));

        assert!(result.is_err());
        assert_eq!(tag_ids_of(&conn, "t1"), ["a"]);
    }
}

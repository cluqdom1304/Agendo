//! Notes (D-43 to D-46). Same conventions as `tasks.rs`: writes take a `&Transaction`, so the FTS
//! row, the tag links and the references of the document commit together with the note; the
//! caller supplies ids and the UTC instant `now`.

use rusqlite::{Connection, OptionalExtension, Row, Transaction, params, types::Type};
use serde::de::DeserializeOwned;

use super::{
    fts,
    model::{EMPTY_DOC, NewNote, Note, NotePatch, NoteSummary},
    note_tags,
};

/// Trashed notes older than this are removed for good, like tasks (SPEC §5.3).
pub const PURGE_AFTER_DAYS: i64 = 30;

/// Characters of plain text kept as the excerpt of a listed note.
pub const EXCERPT_CHARS: usize = 200;

/// Which notes `list` returns. Always excludes the trash.
#[derive(Debug, Clone, Copy)]
pub enum Scope<'a> {
    All,
    /// Notes without folder.
    Inbox,
    Pinned,
    /// The folder and all its subfolders.
    Folder(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    /// Most recently modified first.
    #[default]
    Updated,
    /// Most recently created first.
    Created,
    /// A to Z, ignoring case.
    Title,
}

fn json_column<T: DeserializeOwned>(row: &Row<'_>, index: usize) -> rusqlite::Result<T> {
    let raw: String = row.get(index)?;
    serde_json::from_str(&raw).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(index, Type::Text, Box::new(error))
    })
}

macro_rules! note_sql {
    ($suffix:literal) => {
        concat!(
            "SELECT n.id, n.folder_id, n.title, n.content_json, n.is_pinned,
                    n.created_at, n.updated_at,
                    (SELECT json_group_array(nt.tag_id ORDER BY nt.tag_id)
                       FROM note_tags nt WHERE nt.note_id = n.id)
             FROM notes n ",
            $suffix
        )
    };
}

/// Columns of a listed note: the excerpt is the first [`EXCERPT_CHARS`] characters of the text
/// with line breaks turned into spaces.
macro_rules! summary_columns {
    () => {
        "SELECT n.id, n.folder_id, n.title,
                replace(substr(n.content_text, 1, 200), char(10), ' '),
                n.is_pinned, n.created_at, n.updated_at,
                (SELECT json_group_array(nt.tag_id ORDER BY nt.tag_id)
                   FROM note_tags nt WHERE nt.note_id = n.id)
         FROM notes n "
    };
}

fn map_note(row: &Row<'_>) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        folder_id: row.get(1)?,
        title: row.get(2)?,
        content_json: row.get(3)?,
        is_pinned: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        tag_ids: json_column(row, 7)?,
    })
}

fn map_summary(row: &Row<'_>) -> rusqlite::Result<NoteSummary> {
    Ok(NoteSummary {
        id: row.get(0)?,
        folder_id: row.get(1)?,
        title: row.get(2)?,
        excerpt: row.get(3)?,
        is_pinned: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        tag_ids: json_column(row, 7)?,
    })
}

/// A live (not trashed) note.
pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Note>> {
    conn.prepare_cached(note_sql!("WHERE n.id = ?1 AND n.deleted_at IS NULL"))?
        .query_row([id], map_note)
        .optional()
}

/// A note that sits in the trash, to show it before it is restored.
pub fn get_trashed(conn: &Connection, id: &str) -> rusqlite::Result<Option<Note>> {
    conn.prepare_cached(note_sql!("WHERE n.id = ?1 AND n.deleted_at IS NOT NULL"))?
        .query_row([id], map_note)
        .optional()
}

/// Live notes of `scope`, optionally with one tag, pinned ones first, then by `sort`.
pub fn list(
    conn: &Connection,
    scope: Scope<'_>,
    tag_id: Option<&str>,
    sort: Sort,
) -> rusqlite::Result<Vec<NoteSummary>> {
    // Only fixed fragments are concatenated; every value is a bound parameter (?1 folder, ?2 tag).
    let (prefix, filter, folder) = match scope {
        Scope::All => ("", "AND ?1 IS NULL", None),
        Scope::Inbox => ("", "AND ?1 IS NULL AND n.folder_id IS NULL", None),
        Scope::Pinned => ("", "AND ?1 IS NULL AND n.is_pinned = 1", None),
        Scope::Folder(id) => (
            "WITH RECURSIVE down(id) AS (
                 SELECT id FROM folders WHERE id = ?1
                 UNION ALL
                 SELECT f.id FROM folders f JOIN down ON f.parent_id = down.id) ",
            "AND n.folder_id IN (SELECT id FROM down)",
            Some(id),
        ),
    };
    let order = match sort {
        Sort::Updated => "n.is_pinned DESC, n.updated_at DESC, n.id",
        Sort::Created => "n.is_pinned DESC, n.created_at DESC, n.id",
        Sort::Title => "n.is_pinned DESC, n.title COLLATE NOCASE, n.id",
    };
    let sql = [
        prefix,
        summary_columns!(),
        "WHERE n.deleted_at IS NULL ",
        filter,
        " AND (?2 IS NULL OR EXISTS (SELECT 1 FROM note_tags f
                                      WHERE f.note_id = n.id AND f.tag_id = ?2))
          ORDER BY ",
        order,
    ]
    .concat();
    conn.prepare_cached(&sql)?
        .query_map(params![folder, tag_id], map_summary)?
        .collect()
}

/// Live notes that link to the note, most recently edited first (D-46 backlinks).
pub fn backlinks(conn: &Connection, id: &str) -> rusqlite::Result<Vec<NoteSummary>> {
    conn.prepare_cached(concat!(
        summary_columns!(),
        "WHERE n.deleted_at IS NULL
           AND n.id IN (SELECT source_note_id FROM note_links WHERE target_note_id = ?1)
         ORDER BY n.updated_at DESC, n.id"
    ))?
    .query_map([id], map_summary)?
    .collect()
}

/// Live notes with no link to or from another live note, most recently edited first.
pub fn orphans(conn: &Connection) -> rusqlite::Result<Vec<NoteSummary>> {
    conn.prepare_cached(concat!(
        summary_columns!(),
        "WHERE n.deleted_at IS NULL
           AND NOT EXISTS (SELECT 1 FROM note_links l JOIN notes o ON o.id = l.target_note_id
                            WHERE l.source_note_id = n.id AND o.deleted_at IS NULL)
           AND NOT EXISTS (SELECT 1 FROM note_links l JOIN notes o ON o.id = l.source_note_id
                            WHERE l.target_note_id = n.id AND o.deleted_at IS NULL)
         ORDER BY n.updated_at DESC, n.id"
    ))?
    .query_map([], map_summary)?
    .collect()
}

/// Replaces the tasks and notes a note points to. Targets that do not exist (a purged task, a
/// note that is gone) and a link to the note itself are skipped instead of failing the save: the
/// document may legitimately hold a reference whose target disappeared.
fn replace_references(
    tx: &Transaction<'_>,
    note_id: &str,
    task_ids: &[String],
    note_ids: &[String],
) -> rusqlite::Result<()> {
    tx.prepare_cached("DELETE FROM note_tasks WHERE note_id = ?1")?
        .execute([note_id])?;
    tx.prepare_cached("DELETE FROM note_links WHERE source_note_id = ?1")?
        .execute([note_id])?;
    let mut link_task = tx.prepare_cached(
        "INSERT OR IGNORE INTO note_tasks (note_id, task_id)
         SELECT ?1, id FROM tasks WHERE id = ?2",
    )?;
    for task_id in task_ids {
        link_task.execute(params![note_id, task_id])?;
    }
    let mut link_note = tx.prepare_cached(
        "INSERT OR IGNORE INTO note_links (source_note_id, target_note_id)
         SELECT ?1, id FROM notes WHERE id = ?2 AND id <> ?1",
    )?;
    for target in note_ids {
        link_note.execute(params![note_id, target])?;
    }
    Ok(())
}

/// Id of the note that belongs to `date`, trashed or not (F13-01): the unique index spans both.
pub fn daily_id(conn: &Connection, date: &str) -> rusqlite::Result<Option<(String, bool)>> {
    conn.prepare_cached("SELECT id, deleted_at IS NOT NULL FROM notes WHERE daily_date = ?1")?
        .query_row([date], |row| Ok((row.get(0)?, row.get(1)?)))
        .optional()
}

/// Marks a note as the one of `date`.
pub fn set_daily_date(tx: &Transaction<'_>, id: &str, date: &str) -> rusqlite::Result<()> {
    tx.prepare_cached("UPDATE notes SET daily_date = ?2 WHERE id = ?1")?
        .execute(params![id, date])?;
    Ok(())
}

pub fn create(tx: &Transaction<'_>, new: &NewNote) -> rusqlite::Result<Note> {
    tx.prepare_cached(
        "INSERT INTO notes (id, folder_id, title, content_json, content_text, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
    )?
    .execute(params![
        new.id,
        new.folder_id,
        new.title,
        new.content_json.as_deref().unwrap_or(EMPTY_DOC),
        new.content_text,
        new.now
    ])?;
    note_tags::replace_links(tx, &new.id, &new.tag_ids)?;
    replace_references(tx, &new.id, &new.task_ids, &new.note_ids)?;
    fts::insert_note(tx, &new.id)?;
    get(tx, &new.id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

/// Applies `patch` to a live note. A new document replaces the references of the note too.
/// Returns `None` if the note does not exist or is in the trash.
pub fn update(
    tx: &Transaction<'_>,
    id: &str,
    patch: &NotePatch,
    now: &str,
) -> rusqlite::Result<Option<Note>> {
    if get(tx, id)?.is_none() {
        return Ok(None);
    }
    let (json, text) = match &patch.content {
        Some(content) => (Some(content.json.as_str()), Some(content.text.as_str())),
        None => (None, None),
    };
    tx.prepare_cached(
        "UPDATE notes SET
             title = COALESCE(?2, title),
             content_json = COALESCE(?3, content_json),
             content_text = COALESCE(?4, content_text),
             updated_at = ?5
         WHERE id = ?1",
    )?
    .execute(params![id, patch.title, json, text, now])?;
    if let Some(content) = &patch.content {
        replace_references(tx, id, &content.task_ids, &content.note_ids)?;
    }
    if patch.title.is_some() || patch.content.is_some() {
        fts::sync_note(tx, id)?;
    }
    get(tx, id)
}

/// Pins or unpins a live note. It does not change `updated_at`: pinning is not an edit and
/// must not reshuffle the list. Returns `None` if the note does not exist or is in the trash.
pub fn set_pinned(tx: &Transaction<'_>, id: &str, pinned: bool) -> rusqlite::Result<Option<Note>> {
    let changed = tx
        .prepare_cached("UPDATE notes SET is_pinned = ?2 WHERE id = ?1 AND deleted_at IS NULL")?
        .execute(params![id, pinned])?;
    if changed == 0 {
        return Ok(None);
    }
    get(tx, id)
}

/// Moves a live note to a folder (`None` = inbox) and touches `updated_at`. An unknown folder
/// fails with a foreign-key error. Returns `None` if the note does not exist or is in the trash.
pub fn move_note(
    tx: &Transaction<'_>,
    id: &str,
    folder_id: Option<&str>,
    now: &str,
) -> rusqlite::Result<Option<Note>> {
    let changed = tx
        .prepare_cached(
            "UPDATE notes SET folder_id = ?2, updated_at = ?3
             WHERE id = ?1 AND deleted_at IS NULL",
        )?
        .execute(params![id, folder_id, now])?;
    if changed == 0 {
        return Ok(None);
    }
    get(tx, id)
}

/// Moves a live note to the trash. Returns `false` if there was nothing to delete.
pub fn soft_delete(tx: &Transaction<'_>, id: &str, now: &str) -> rusqlite::Result<bool> {
    let changed = tx
        .prepare_cached(
            "UPDATE notes SET deleted_at = ?2, updated_at = ?2
             WHERE id = ?1 AND deleted_at IS NULL",
        )?
        .execute(params![id, now])?;
    if changed == 0 {
        return Ok(false);
    }
    fts::sync_note(tx, id)?;
    Ok(true)
}

/// Takes a note out of the trash. Returns `None` if it is not in the trash.
pub fn restore(tx: &Transaction<'_>, id: &str, now: &str) -> rusqlite::Result<Option<Note>> {
    let changed = tx
        .prepare_cached(
            "UPDATE notes SET deleted_at = NULL, updated_at = ?2
             WHERE id = ?1 AND deleted_at IS NOT NULL",
        )?
        .execute(params![id, now])?;
    if changed == 0 {
        return Ok(None);
    }
    fts::sync_note(tx, id)?;
    get(tx, id)
}

/// Trashed notes, most recently deleted first.
pub fn list_trash(conn: &Connection) -> rusqlite::Result<Vec<NoteSummary>> {
    conn.prepare_cached(concat!(
        summary_columns!(),
        "WHERE n.deleted_at IS NOT NULL ORDER BY n.deleted_at DESC, n.id"
    ))?
    .query_map([], map_summary)?
    .collect()
}

/// Permanently removes notes trashed more than [`PURGE_AFTER_DAYS`] days before `now`. Tag
/// links, references and links go with them (the tasks they pointed to stay). Returns how many
/// notes were purged.
pub fn purge_deleted(tx: &Transaction<'_>, now: &str) -> rusqlite::Result<usize> {
    tx.prepare_cached(
        "DELETE FROM notes
         WHERE deleted_at IS NOT NULL
           AND deleted_at < strftime('%Y-%m-%dT%H:%M:%SZ', ?1, '-' || ?2 || ' days')",
    )?
    .execute(params![now, PURGE_AFTER_DAYS])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{MIGRATIONS, open_in_memory},
        repo::{
            folders, fts,
            model::{FolderInput, NewTask, NoteContent, TagInput},
            tags, tasks, with_tx,
        },
    };

    const T0: &str = "2026-03-01T10:00:00Z";
    const T1: &str = "2026-03-02T10:00:00Z";
    const T2: &str = "2026-03-03T10:00:00Z";

    fn database() -> Connection {
        open_in_memory(&MIGRATIONS).expect("database")
    }

    fn new_note(id: &str, title: &str) -> NewNote {
        NewNote {
            id: id.into(),
            title: title.into(),
            now: T0.into(),
            ..NewNote::default()
        }
    }

    fn add(conn: &mut Connection, note: &NewNote) -> Note {
        with_tx(conn, |tx| create(tx, note)).expect("create")
    }

    fn add_in(conn: &mut Connection, id: &str, title: &str, folder: Option<&str>) -> Note {
        add(
            conn,
            &NewNote {
                folder_id: folder.map(str::to_string),
                ..new_note(id, title)
            },
        )
    }

    fn add_folder(conn: &mut Connection, id: &str, parent: Option<&str>) {
        with_tx(conn, |tx| {
            folders::upsert(
                tx,
                &FolderInput {
                    id: id.into(),
                    name: id.into(),
                    parent_id: parent.map(str::to_string),
                    now: T0.into(),
                },
            )
        })
        .expect("folder");
    }

    fn add_tag(conn: &mut Connection, id: &str) {
        with_tx(conn, |tx| {
            tags::upsert(
                tx,
                &TagInput {
                    id: id.into(),
                    name: id.into(),
                    color_hex: "#112233".into(),
                    parent_id: None,
                    now: T0.into(),
                },
            )
        })
        .expect("tag");
    }

    fn ids(notes: Vec<NoteSummary>) -> Vec<String> {
        notes.into_iter().map(|note| note.id).collect()
    }

    fn indexed(conn: &Connection, id: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM notes_fts WHERE note_id = ?1",
            [id],
            |row| row.get(0),
        )
        .expect("count")
    }

    #[test]
    fn create_stores_defaults_tags_and_indexes_the_note() {
        let mut conn = database();
        add_tag(&mut conn, "tag");
        let note = add(
            &mut conn,
            &NewNote {
                tag_ids: vec!["tag".into(), "tag".into()],
                content_text: "texto".into(),
                ..new_note("n1", "Apuntes")
            },
        );

        assert_eq!(note.title, "Apuntes");
        assert_eq!(note.content_json, EMPTY_DOC);
        assert_eq!(note.folder_id, None);
        assert!(!note.is_pinned);
        assert_eq!(note.tag_ids, ["tag"]);
        assert_eq!(
            (note.created_at.as_str(), note.updated_at.as_str()),
            (T0, T0)
        );
        assert_eq!(indexed(&conn, "n1"), 1);
    }

    #[test]
    fn get_ignores_trashed_and_unknown_notes() {
        let mut conn = database();
        add_in(&mut conn, "n1", "Una", None);
        with_tx(&mut conn, |tx| soft_delete(tx, "n1", T1)).expect("trash");
        assert_eq!(get(&conn, "n1").expect("get"), None);
        assert_eq!(get(&conn, "ghost").expect("get"), None);
    }

    #[test]
    fn get_trashed_only_finds_notes_in_the_trash() {
        let mut conn = database();
        add_in(&mut conn, "n1", "Una", None);
        add_in(&mut conn, "n2", "Otra", None);
        with_tx(&mut conn, |tx| soft_delete(tx, "n1", T1)).expect("trash");
        let found = get_trashed(&conn, "n1").expect("get").expect("note");
        assert_eq!(found.title, "Una");
        assert_eq!(get_trashed(&conn, "n2").expect("get"), None);
        assert_eq!(get_trashed(&conn, "ghost").expect("get"), None);
    }

    #[test]
    fn update_changes_title_and_content_and_keeps_the_index_in_step() {
        let mut conn = database();
        add_in(&mut conn, "n1", "Antes", None);
        let patch = NotePatch {
            title: Some("Después".into()),
            content: Some(NoteContent {
                json: r#"{"type":"doc","content":[{"type":"paragraph"}]}"#.into(),
                text: "nuevo cuerpo".into(),
                task_ids: vec![],
                note_ids: vec![],
            }),
        };
        let note = with_tx(&mut conn, |tx| update(tx, "n1", &patch, T1))
            .expect("update")
            .expect("note");

        assert_eq!(note.title, "Después");
        assert!(note.content_json.contains("paragraph"));
        assert_eq!(
            (note.created_at.as_str(), note.updated_at.as_str()),
            (T0, T1)
        );
        let hit: String = conn
            .query_row(
                "SELECT note_id FROM notes_fts WHERE notes_fts MATCH 'cuerpo'",
                [],
                |row| row.get(0),
            )
            .expect("indexed");
        assert_eq!(hit, "n1");
    }

    #[test]
    fn update_with_an_empty_patch_keeps_the_content_and_unknown_notes_are_none() {
        let mut conn = database();
        add_in(&mut conn, "n1", "Igual", None);
        let note = with_tx(&mut conn, |tx| update(tx, "n1", &NotePatch::default(), T1))
            .expect("update")
            .expect("note");
        assert_eq!(note.title, "Igual");
        assert_eq!(note.content_json, EMPTY_DOC);
        assert_eq!(
            with_tx(&mut conn, |tx| update(
                tx,
                "ghost",
                &NotePatch::default(),
                T1
            ))
            .expect("none"),
            None
        );
    }

    #[test]
    fn references_are_replaced_on_every_save_and_dangling_targets_are_skipped() {
        let mut conn = database();
        add_in(&mut conn, "a", "A", None);
        add_in(&mut conn, "b", "B", None);
        add_in(&mut conn, "c", "C", None);
        with_tx(&mut conn, |tx| {
            tasks::create(
                tx,
                &NewTask {
                    id: "t1".into(),
                    now: T0.into(),
                    ..NewTask::default()
                },
            )
        })
        .expect("task");
        let save = |conn: &mut Connection, task_ids: &[&str], note_ids: &[&str]| {
            let patch = NotePatch {
                title: None,
                content: Some(NoteContent {
                    json: r#"{"type":"doc","content":[]}"#.into(),
                    text: String::new(),
                    task_ids: task_ids.iter().map(|id| (*id).to_string()).collect(),
                    note_ids: note_ids.iter().map(|id| (*id).to_string()).collect(),
                }),
            };
            with_tx(conn, |tx| update(tx, "a", &patch, T1)).expect("save");
        };
        let links = |conn: &Connection| -> (Vec<String>, Vec<String>) {
            let collect = |sql: &str| -> Vec<String> {
                conn.prepare(sql)
                    .expect("prepare")
                    .query_map([], |row| row.get(0))
                    .expect("query")
                    .collect::<Result<_, _>>()
                    .expect("rows")
            };
            (
                collect("SELECT task_id FROM note_tasks WHERE note_id = 'a' ORDER BY 1"),
                collect(
                    "SELECT target_note_id FROM note_links WHERE source_note_id = 'a' ORDER BY 1",
                ),
            )
        };

        save(&mut conn, &["t1", "gone"], &["b", "c", "a", "ghost"]);
        assert_eq!(
            links(&conn),
            (vec!["t1".into()], vec!["b".into(), "c".into()])
        );

        save(&mut conn, &[], &["c"]);
        assert_eq!(links(&conn), (vec![], vec!["c".into()]));
    }

    #[test]
    fn backlinks_and_orphans_follow_the_links_between_live_notes() {
        let mut conn = database();
        for id in ["a", "b", "c", "d"] {
            add_in(&mut conn, id, id, None);
        }
        let link = |conn: &mut Connection, from: &str, to: &[&str]| {
            let patch = NotePatch {
                title: None,
                content: Some(NoteContent {
                    json: r#"{"type":"doc","content":[]}"#.into(),
                    text: String::new(),
                    task_ids: vec![],
                    note_ids: to.iter().map(|id| (*id).to_string()).collect(),
                }),
            };
            with_tx(conn, |tx| update(tx, from, &patch, T1)).expect("save");
        };
        link(&mut conn, "a", &["b"]);
        link(&mut conn, "c", &["b"]);

        assert_eq!(ids(backlinks(&conn, "b").expect("backlinks")), ["a", "c"]);
        assert!(backlinks(&conn, "a").expect("backlinks").is_empty());
        assert_eq!(ids(orphans(&conn).expect("orphans")), ["d"]);

        with_tx(&mut conn, |tx| soft_delete(tx, "a", T1)).expect("trash");
        assert_eq!(ids(backlinks(&conn, "b").expect("backlinks")), ["c"]);
        link(&mut conn, "c", &[]);
        // `c` was saved last, so it comes first.
        assert_eq!(ids(orphans(&conn).expect("orphans")), ["c", "b", "d"]);
    }

    #[test]
    fn list_orders_pinned_first_then_by_the_chosen_sort() {
        let mut conn = database();
        for (id, title, at) in [("a", "beta", T0), ("b", "Alfa", T2), ("c", "gamma", T1)] {
            add(
                &mut conn,
                &NewNote {
                    now: at.into(),
                    ..new_note(id, title)
                },
            );
        }
        let query =
            |conn: &Connection, sort| ids(list(conn, Scope::All, None, sort).expect("list"));
        assert_eq!(query(&conn, Sort::Updated), ["b", "c", "a"]);
        assert_eq!(query(&conn, Sort::Created), ["b", "c", "a"]);
        assert_eq!(
            query(&conn, Sort::Title),
            ["b", "a", "c"],
            "case-insensitive"
        );

        with_tx(&mut conn, |tx| set_pinned(tx, "a", true)).expect("pin");
        assert_eq!(query(&conn, Sort::Updated), ["a", "b", "c"]);
        assert_eq!(query(&conn, Sort::Title), ["a", "b", "c"]);
    }

    #[test]
    fn list_scopes_select_inbox_pinned_and_folder_with_descendants() {
        let mut conn = database();
        add_folder(&mut conn, "r", None);
        add_folder(&mut conn, "child", Some("r"));
        add_folder(&mut conn, "other", None);
        add_in(&mut conn, "inbox", "i", None);
        add_in(&mut conn, "in-r", "r", Some("r"));
        add_in(&mut conn, "in-child", "c", Some("child"));
        add_in(&mut conn, "in-other", "o", Some("other"));
        add_in(&mut conn, "trashed", "t", Some("r"));
        with_tx(&mut conn, |tx| soft_delete(tx, "trashed", T1)).expect("trash");
        with_tx(&mut conn, |tx| set_pinned(tx, "in-other", true)).expect("pin");

        let scoped = |scope| {
            let mut found = ids(list(&conn, scope, None, Sort::Title).expect("list"));
            found.sort();
            found
        };
        assert_eq!(scoped(Scope::All).len(), 4);
        assert_eq!(scoped(Scope::Inbox), ["inbox"]);
        assert_eq!(scoped(Scope::Pinned), ["in-other"]);
        assert_eq!(scoped(Scope::Folder("r")), ["in-child", "in-r"]);
        assert_eq!(scoped(Scope::Folder("child")), ["in-child"]);
        assert!(scoped(Scope::Folder("ghost")).is_empty());
    }

    #[test]
    fn list_filters_by_tag_and_returns_excerpt_and_tags_without_the_document() {
        let mut conn = database();
        add_tag(&mut conn, "física");
        add_tag(&mut conn, "otra");
        add(
            &mut conn,
            &NewNote {
                tag_ids: vec!["física".into(), "otra".into()],
                content_text: format!("línea uno\nlínea dos {}", "x".repeat(300)),
                ..new_note("tagged", "Con etiqueta")
            },
        );
        add_in(&mut conn, "plain", "Sin etiqueta", None);

        let found = list(&conn, Scope::All, Some("física"), Sort::Updated).expect("list");
        assert_eq!(ids(found.clone()), ["tagged"]);
        assert_eq!(found[0].tag_ids, ["física", "otra"]);
        assert!(found[0].excerpt.starts_with("línea uno línea dos x"));
        assert_eq!(found[0].excerpt.chars().count(), EXCERPT_CHARS);
        assert!(
            list(&conn, Scope::All, Some("nope"), Sort::Updated)
                .expect("list")
                .is_empty()
        );
    }

    #[test]
    fn pinning_does_not_touch_updated_at_but_moving_does() {
        let mut conn = database();
        add_folder(&mut conn, "f", None);
        add_in(&mut conn, "n1", "Una", None);

        let pinned = with_tx(&mut conn, |tx| set_pinned(tx, "n1", true))
            .expect("pin")
            .expect("note");
        assert!(pinned.is_pinned);
        assert_eq!(pinned.updated_at, T0);

        let moved = with_tx(&mut conn, |tx| move_note(tx, "n1", Some("f"), T1))
            .expect("move")
            .expect("note");
        assert_eq!(
            (moved.folder_id.as_deref(), moved.updated_at.as_str()),
            (Some("f"), T1)
        );
        let inbox = with_tx(&mut conn, |tx| move_note(tx, "n1", None, T2))
            .expect("move")
            .expect("note");
        assert_eq!(inbox.folder_id, None);
        assert!(with_tx(&mut conn, |tx| move_note(tx, "n1", Some("ghost"), T2)).is_err());
        assert_eq!(
            with_tx(&mut conn, |tx| set_pinned(tx, "ghost", true)).expect("none"),
            None
        );
    }

    #[test]
    fn trash_hides_the_note_from_search_and_restore_brings_it_back() {
        let mut conn = database();
        add_in(&mut conn, "n1", "Una", None);
        assert!(with_tx(&mut conn, |tx| soft_delete(tx, "n1", T1)).expect("trash"));
        assert!(!with_tx(&mut conn, |tx| soft_delete(tx, "n1", T1)).expect("twice"));
        assert_eq!(indexed(&conn, "n1"), 0);
        assert_eq!(ids(list_trash(&conn).expect("trash")), ["n1"]);
        assert!(
            list(&conn, Scope::All, None, Sort::Updated)
                .expect("list")
                .is_empty()
        );

        let restored = with_tx(&mut conn, |tx| restore(tx, "n1", T2))
            .expect("restore")
            .expect("note");
        assert_eq!(restored.updated_at, T2);
        assert_eq!(indexed(&conn, "n1"), 1);
        assert!(list_trash(&conn).expect("trash").is_empty());
        assert_eq!(
            with_tx(&mut conn, |tx| restore(tx, "n1", T2)).expect("none"),
            None
        );
    }

    #[test]
    fn purge_removes_only_notes_trashed_more_than_thirty_days_ago() {
        let mut conn = database();
        add_in(&mut conn, "old", "Vieja", None);
        add_in(&mut conn, "edge", "Límite", None);
        add_in(&mut conn, "recent", "Reciente", None);
        add_in(&mut conn, "live", "Viva", None);
        with_tx(&mut conn, |tx| {
            soft_delete(tx, "old", "2026-01-01T00:00:00Z")
        })
        .expect("trash");
        with_tx(&mut conn, |tx| {
            soft_delete(tx, "edge", "2026-02-01T00:00:00Z")
        })
        .expect("trash");
        with_tx(&mut conn, |tx| {
            soft_delete(tx, "recent", "2026-02-20T00:00:00Z")
        })
        .expect("trash");

        let purged =
            with_tx(&mut conn, |tx| purge_deleted(tx, "2026-03-03T00:00:00Z")).expect("purge");
        assert_eq!(purged, 1, "only `old`: `edge` is exactly 30 days old");
        assert_eq!(list_trash(&conn).expect("trash").len(), 2);
        assert!(get(&conn, "live").expect("get").is_some());
    }

    #[test]
    fn a_failed_operation_rolls_back_the_note_and_its_index() {
        let mut conn = database();
        let result: rusqlite::Result<()> = with_tx(&mut conn, |tx| {
            create(
                tx,
                &NewNote {
                    tag_ids: vec!["no-such-tag".into()],
                    ..new_note("n1", "Una")
                },
            )?;
            Ok(())
        });
        assert!(result.is_err());
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .expect("count");
        assert_eq!(count, 0);
        assert_eq!(indexed(&conn, "n1"), 0);
    }

    #[test]
    fn a_hard_deleted_note_leaves_no_index_row_after_sync() {
        let mut conn = database();
        add_in(&mut conn, "a", "A", None);
        let tx = conn.transaction().expect("tx");
        tx.execute("DELETE FROM notes WHERE id = 'a'", [])
            .expect("delete");
        fts::sync_note(&tx, "a").expect("sync");
        tx.commit().expect("commit");
        assert_eq!(indexed(&conn, "a"), 0);
    }
}

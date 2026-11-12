//! Folders of notes (D-43): a tree with at most five levels. The repository only reads and
//! writes; the depth and cycle rules live in `domain::validation` and are applied by the
//! command, which loads [`depth`], [`height`] and [`is_in_subtree`] to decide.

use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use super::{
    model::{Folder, FolderInput, FolderWithCount},
    notes,
};

fn map_folder(row: &Row<'_>) -> rusqlite::Result<Folder> {
    Ok(Folder {
        id: row.get(0)?,
        name: row.get(1)?,
        parent_id: row.get(2)?,
        position: row.get(3)?,
    })
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Folder>> {
    conn.prepare_cached("SELECT id, name, parent_id, position FROM folders WHERE id = ?1")?
        .query_row([id], map_folder)
        .optional()
}

/// Every folder, ordered by `position` and then name (the frontend builds the tree).
/// `note_count` counts the live notes directly inside each one.
pub fn list(conn: &Connection) -> rusqlite::Result<Vec<FolderWithCount>> {
    conn.prepare_cached(
        "SELECT f.id, f.name, f.parent_id, f.position,
                (SELECT COUNT(*) FROM notes n
                  WHERE n.folder_id = f.id AND n.deleted_at IS NULL)
           FROM folders f
          ORDER BY f.position, f.name COLLATE NOCASE, f.id",
    )?
    .query_map([], |row| {
        Ok(FolderWithCount {
            folder: map_folder(row)?,
            note_count: row.get(4)?,
        })
    })?
    .collect()
}

/// Levels from the top down to `id`, itself included: 1 for a top-level folder. `None` if the
/// folder does not exist.
pub fn depth(conn: &Connection, id: &str) -> rusqlite::Result<Option<usize>> {
    let levels: i64 = conn
        .prepare_cached(
            "WITH RECURSIVE up(id, parent_id) AS (
                 SELECT id, parent_id FROM folders WHERE id = ?1
                 UNION ALL
                 SELECT f.id, f.parent_id FROM folders f JOIN up ON f.id = up.parent_id
             ) SELECT COUNT(*) FROM up",
        )?
        .query_row([id], |row| row.get(0))?;
    Ok(usize::try_from(levels).ok().filter(|levels| *levels > 0))
}

/// Levels of the subtree that starts at `id`, itself included: 1 for a folder without
/// subfolders.
pub fn height(conn: &Connection, id: &str) -> rusqlite::Result<usize> {
    let levels: i64 = conn
        .prepare_cached(
            "WITH RECURSIVE down(id, level) AS (
                 SELECT id, 1 FROM folders WHERE id = ?1
                 UNION ALL
                 SELECT f.id, down.level + 1 FROM folders f JOIN down ON f.parent_id = down.id
             ) SELECT COALESCE(MAX(level), 0) FROM down",
        )?
        .query_row([id], |row| row.get(0))?;
    Ok(usize::try_from(levels).unwrap_or(0))
}

/// Whether `candidate` is `root` or lies below it.
pub fn is_in_subtree(conn: &Connection, root: &str, candidate: &str) -> rusqlite::Result<bool> {
    conn.prepare_cached(
        "WITH RECURSIVE down(id) AS (
             SELECT id FROM folders WHERE id = ?1
             UNION ALL
             SELECT f.id FROM folders f JOIN down ON f.parent_id = down.id
         ) SELECT EXISTS(SELECT 1 FROM down WHERE id = ?2)",
    )?
    .query_row(params![root, candidate], |row| row.get(0))
}

/// Last position among the children of `parent_id` (`None` = top level).
fn next_position(conn: &Connection, parent_id: Option<&str>) -> rusqlite::Result<i64> {
    conn.prepare_cached(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM folders WHERE parent_id IS ?1",
    )?
    .query_row([parent_id], |row| row.get(0))
}

/// Creates the folder, or renames/moves it if the id exists. A moved folder goes last among its
/// new siblings; a renamed one keeps its place. A sibling with the same name is a `CONFLICT`.
pub fn upsert(tx: &Transaction<'_>, input: &FolderInput) -> rusqlite::Result<Folder> {
    let current = get(tx, &input.id)?;
    match current {
        None => {
            let position = next_position(tx, input.parent_id.as_deref())?;
            tx.prepare_cached(
                "INSERT INTO folders (id, name, parent_id, position, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            )?
            .execute(params![
                input.id,
                input.name,
                input.parent_id,
                position,
                input.now
            ])?;
        }
        Some(existing) => {
            let moved = existing.parent_id != input.parent_id;
            let position = if moved {
                next_position(tx, input.parent_id.as_deref())?
            } else {
                existing.position
            };
            tx.prepare_cached(
                "UPDATE folders SET name = ?2, parent_id = ?3, position = ?4, updated_at = ?5
                 WHERE id = ?1",
            )?
            .execute(params![
                input.id,
                input.name,
                input.parent_id,
                position,
                input.now
            ])?;
        }
    }
    get(tx, &input.id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

/// Deletes the folder and every subfolder. Their live notes go to the trash (and lose their
/// folder, so a restored note lands in the inbox). Returns how many notes were trashed, or
/// `None` if the folder does not exist.
pub fn delete_tree(tx: &Transaction<'_>, id: &str, now: &str) -> rusqlite::Result<Option<usize>> {
    if get(tx, id)?.is_none() {
        return Ok(None);
    }
    let note_ids: Vec<String> = tx
        .prepare_cached(
            "WITH RECURSIVE down(id) AS (
                 SELECT id FROM folders WHERE id = ?1
                 UNION ALL
                 SELECT f.id FROM folders f JOIN down ON f.parent_id = down.id
             ) SELECT n.id FROM notes n JOIN down ON n.folder_id = down.id
                WHERE n.deleted_at IS NULL",
        )?
        .query_map([id], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    for note_id in &note_ids {
        notes::soft_delete(tx, note_id, now)?;
    }
    // Children first would also work; the foreign key promotes orphans, so delete the whole
    // subtree in one statement.
    tx.prepare_cached(
        "WITH RECURSIVE down(id) AS (
             SELECT id FROM folders WHERE id = ?1
             UNION ALL
             SELECT f.id FROM folders f JOIN down ON f.parent_id = down.id
         ) DELETE FROM folders WHERE id IN (SELECT id FROM down)",
    )?
    .execute([id])?;
    Ok(Some(note_ids.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{MIGRATIONS, open_in_memory},
        repo::{model::NewNote, notes as note_repo, with_tx},
    };

    const NOW: &str = "2026-03-10T10:00:00Z";

    fn input(id: &str, name: &str, parent: Option<&str>) -> FolderInput {
        FolderInput {
            id: id.into(),
            name: name.into(),
            parent_id: parent.map(str::to_string),
            now: NOW.into(),
        }
    }

    fn add(conn: &mut Connection, id: &str, name: &str, parent: Option<&str>) -> Folder {
        with_tx(conn, |tx| upsert(tx, &input(id, name, parent))).expect("upsert")
    }

    fn add_note(conn: &mut Connection, id: &str, folder: Option<&str>) {
        with_tx(conn, |tx| {
            note_repo::create(
                tx,
                &NewNote {
                    id: id.into(),
                    folder_id: folder.map(str::to_string),
                    now: NOW.into(),
                    ..NewNote::default()
                },
            )
        })
        .expect("note");
    }

    fn database() -> Connection {
        open_in_memory(&MIGRATIONS).expect("database")
    }

    #[test]
    fn new_folders_go_last_among_their_siblings() {
        let mut conn = database();
        assert_eq!(add(&mut conn, "a", "A", None).position, 0);
        assert_eq!(add(&mut conn, "b", "B", None).position, 1);
        assert_eq!(add(&mut conn, "c", "C", Some("a")).position, 0);
        assert_eq!(add(&mut conn, "d", "D", Some("a")).position, 1);
    }

    #[test]
    fn renaming_keeps_the_position_and_moving_goes_last() {
        let mut conn = database();
        add(&mut conn, "a", "A", None);
        add(&mut conn, "b", "B", None);
        add(&mut conn, "c", "C", Some("a"));

        let renamed = add(&mut conn, "a", "Renombrada", None);
        assert_eq!((renamed.name.as_str(), renamed.position), ("Renombrada", 0));

        let moved = add(&mut conn, "b", "B", Some("a"));
        assert_eq!(moved.parent_id.as_deref(), Some("a"));
        assert_eq!(moved.position, 1, "after C");
    }

    #[test]
    fn a_sibling_with_the_same_name_is_a_constraint_error() {
        let mut conn = database();
        add(&mut conn, "a", "Trabajo", None);
        let result = with_tx(&mut conn, |tx| upsert(tx, &input("b", "trabajo", None)));
        assert!(result.is_err());
        assert!(
            with_tx(&mut conn, |tx| upsert(
                tx,
                &input("c", "Trabajo", Some("a"))
            ))
            .is_ok(),
            "same name under another parent"
        );
    }

    #[test]
    fn depth_height_and_subtree_follow_the_tree() {
        let mut conn = database();
        add(&mut conn, "r", "R", None);
        add(&mut conn, "a", "A", Some("r"));
        add(&mut conn, "b", "B", Some("a"));
        add(&mut conn, "other", "Otra", None);

        assert_eq!(depth(&conn, "r").expect("depth"), Some(1));
        assert_eq!(depth(&conn, "b").expect("depth"), Some(3));
        assert_eq!(depth(&conn, "ghost").expect("depth"), None);
        assert_eq!(height(&conn, "r").expect("height"), 3);
        assert_eq!(height(&conn, "b").expect("height"), 1);
        assert_eq!(height(&conn, "ghost").expect("height"), 0);
        assert!(is_in_subtree(&conn, "r", "b").expect("subtree"));
        assert!(is_in_subtree(&conn, "r", "r").expect("itself"));
        assert!(!is_in_subtree(&conn, "a", "r").expect("ancestor"));
        assert!(!is_in_subtree(&conn, "r", "other").expect("unrelated"));
    }

    #[test]
    fn list_counts_only_live_notes_directly_inside() {
        let mut conn = database();
        add(&mut conn, "r", "R", None);
        add(&mut conn, "a", "A", Some("r"));
        add_note(&mut conn, "n1", Some("r"));
        add_note(&mut conn, "n2", Some("r"));
        add_note(&mut conn, "n3", Some("a"));
        add_note(&mut conn, "inbox", None);
        with_tx(&mut conn, |tx| note_repo::soft_delete(tx, "n2", NOW)).expect("trash");

        let counts: Vec<(String, i64)> = list(&conn)
            .expect("list")
            .into_iter()
            .map(|item| (item.folder.id, item.note_count))
            .collect();
        assert_eq!(counts, [("a".to_string(), 1), ("r".to_string(), 1)]);
    }

    #[test]
    fn delete_tree_trashes_the_notes_and_removes_every_subfolder() {
        let mut conn = database();
        add(&mut conn, "r", "R", None);
        add(&mut conn, "a", "A", Some("r"));
        add(&mut conn, "keep", "Otra", None);
        add_note(&mut conn, "n1", Some("r"));
        add_note(&mut conn, "n2", Some("a"));
        add_note(&mut conn, "safe", Some("keep"));

        let trashed = with_tx(&mut conn, |tx| delete_tree(tx, "r", NOW)).expect("delete");

        assert_eq!(trashed, Some(2));
        let remaining: Vec<String> = list(&conn)
            .expect("list")
            .into_iter()
            .map(|item| item.folder.id)
            .collect();
        assert_eq!(remaining, ["keep"]);
        let trash: Vec<String> = note_repo::list_trash(&conn)
            .expect("trash")
            .into_iter()
            .map(|note| note.id)
            .collect();
        assert_eq!(trash.len(), 2);
        assert!(
            note_repo::get(&conn, "safe").expect("get").is_some(),
            "notes of other folders are untouched"
        );
        let restored = with_tx(&mut conn, |tx| note_repo::restore(tx, "n1", NOW))
            .expect("restore")
            .expect("note");
        assert_eq!(restored.folder_id, None, "restored into the inbox");
    }

    #[test]
    fn delete_tree_of_an_unknown_folder_is_none() {
        let mut conn = database();
        assert_eq!(
            with_tx(&mut conn, |tx| delete_tree(tx, "ghost", NOW)).expect("delete"),
            None
        );
    }
}

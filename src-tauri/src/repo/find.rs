//! Unified search over tasks, notes and tags (F11-03). The text part uses the same FTS5
//! expression as `search`; the filters narrow the results:
//!
//! - `tag:nombre`: carries the tag or one of its subtags (repeat it to require several).
//! - `in:carpeta`: notes in the folder or its subfolders.
//! - `desde:AAAA-MM-DD` / `hasta:AAAA-MM-DD`: tasks whose day, or notes last changed (UTC day),
//!   fall inside the range, both ends included (F17-08).
//! - `is:tarea`, `is:nota`, `is:etiqueta`: only that kind. `is:hecha` / `is:pendiente` (tasks)
//!   and `is:fijada` (notes) narrow by state.
//!
//! A value with spaces goes in quotes: `tag:"mi etiqueta"`.

use rusqlite::{Connection, params_from_iter, types::Value};
use serde::Serialize;

use super::{model::Tag, search::match_expression};
use crate::domain::dates;

/// One result, tagged by what it is so the UI can group and open it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FoundItem {
    #[serde(rename_all = "camelCase")]
    Task {
        task_id: String,
        title: String,
        snippet: String,
        due_date: Option<String>,
        is_completed: bool,
    },
    #[serde(rename_all = "camelCase")]
    Note {
        note_id: String,
        title: String,
        snippet: String,
        folder_id: Option<String>,
        is_pinned: bool,
    },
    #[serde(rename_all = "camelCase")]
    Tag {
        tag_id: String,
        name: String,
        color_hex: String,
        note_count: i64,
        pending_count: i64,
    },
}

/// The filters and free text of a search box.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedQuery {
    pub text: String,
    pub tags: Vec<String>,
    pub folder: Option<String>,
    pub tasks: bool,
    pub notes: bool,
    pub tag_items: bool,
    pub completed: Option<bool>,
    pub pinned: bool,
    /// `desde:` and `hasta:` as `YYYY-MM-DD`.
    pub from: Option<String>,
    pub to: Option<String>,
    /// An `is:` value was not understood or the filters exclude every kind: nothing can match.
    pub impossible: bool,
}

/// Splits the box into words, keeping a quoted value (`tag:"a b"`) in one piece.
fn words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for character in input.chars() {
        match character {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// Lower case without the accents of Spanish, so "Álgebra" and "algebra" are the same name.
pub fn fold(text: &str) -> String {
    text.chars()
        .flat_map(|c| {
            let base = match c {
                'á' | 'à' | 'ä' | 'â' | 'Á' | 'À' | 'Ä' | 'Â' => 'a',
                'é' | 'è' | 'ë' | 'ê' | 'É' | 'È' | 'Ë' | 'Ê' => 'e',
                'í' | 'ì' | 'ï' | 'î' | 'Í' | 'Ì' | 'Ï' | 'Î' => 'i',
                'ó' | 'ò' | 'ö' | 'ô' | 'Ó' | 'Ò' | 'Ö' | 'Ô' => 'o',
                'ú' | 'ù' | 'ü' | 'û' | 'Ú' | 'Ù' | 'Ü' | 'Û' => 'u',
                other => return other.to_lowercase().collect::<Vec<_>>(),
            };
            vec![base]
        })
        .collect()
}

pub fn parse(input: &str) -> ParsedQuery {
    let mut query = ParsedQuery {
        tasks: true,
        notes: true,
        tag_items: true,
        ..ParsedQuery::default()
    };
    let mut text = Vec::new();
    for word in words(input) {
        let lower = word.to_lowercase();
        if let Some(value) = lower.strip_prefix("tag:") {
            // `tag:` alone is a filter still being typed: it does not narrow anything yet.
            if !value.is_empty() {
                query.tags.push(value.to_string());
            }
        } else if let Some(value) = lower
            .strip_prefix("desde:")
            .map(|value| (true, value))
            .or_else(|| lower.strip_prefix("hasta:").map(|value| (false, value)))
        {
            let (is_from, value) = value;
            // Still being typed: nothing to narrow yet.
            if !value.is_empty() {
                if dates::parse_date(value).is_err() {
                    query.impossible = true;
                } else if is_from {
                    query.from = Some(value.to_string());
                } else {
                    query.to = Some(value.to_string());
                }
            }
        } else if let Some(value) = lower.strip_prefix("in:") {
            if !value.is_empty() {
                query.folder = Some(value.to_string());
            }
        } else if let Some(value) = lower.strip_prefix("is:") {
            match fold(value).as_str() {
                "" => {}
                "tarea" | "tareas" => {
                    query.notes = false;
                    query.tag_items = false;
                }
                "nota" | "notas" => {
                    query.tasks = false;
                    query.tag_items = false;
                }
                "etiqueta" | "etiquetas" => {
                    query.tasks = false;
                    query.notes = false;
                }
                "hecha" | "hechas" | "completada" | "done" => {
                    query.completed = Some(true);
                    query.notes = false;
                    query.tag_items = false;
                }
                "pendiente" | "pendientes" | "pending" => {
                    query.completed = Some(false);
                    query.notes = false;
                    query.tag_items = false;
                }
                "fijada" | "fijadas" => {
                    query.pinned = true;
                    query.tasks = false;
                    query.tag_items = false;
                }
                _ => query.impossible = true,
            }
        } else {
            text.push(word);
        }
    }
    // Tags and folders are not tagged or foldered themselves, and tasks have no folder.
    if !query.tags.is_empty() {
        query.tag_items = false;
    }
    if query.folder.is_some() {
        query.tasks = false;
        query.tag_items = false;
    }
    if query.from.is_some() || query.to.is_some() {
        query.tag_items = false;
    }
    query.text = text.join(" ");
    query
}

impl ParsedQuery {
    fn has_filters(&self) -> bool {
        !self.tags.is_empty()
            || self.folder.is_some()
            || self.completed.is_some()
            || self.pinned
            || self.from.is_some()
            || self.to.is_some()
            || !(self.tasks && self.notes && self.tag_items)
    }
}

fn names_to_ids(names: &[String], pairs: &[(String, String)]) -> Vec<String> {
    names
        .iter()
        .flat_map(|name| {
            let wanted = fold(name);
            pairs
                .iter()
                .filter(move |(_, candidate)| fold(candidate) == wanted)
                .map(|(id, _)| id.clone())
        })
        .collect()
}

/// A tag filter accepts the tag and every tag below it.
fn with_descendants(roots: &[String], tags: &[(String, Option<String>)]) -> Vec<String> {
    let mut found: Vec<String> = roots.to_vec();
    let mut grew = true;
    while grew {
        grew = false;
        for (id, parent) in tags {
            if !found.contains(id) && parent.as_ref().is_some_and(|p| found.contains(p)) {
                found.push(id.clone());
                grew = true;
            }
        }
    }
    found
}

fn json_array(ids: &[String]) -> String {
    serde_json::to_string(ids).unwrap_or_else(|_| "[]".to_string())
}

/// A search with its text and filters; at most `limit` of each kind.
pub fn find(conn: &Connection, input: &str, limit: u32) -> rusqlite::Result<Vec<FoundItem>> {
    let query = parse(input);
    let expression = match_expression(&query.text);
    if query.impossible || (expression.is_none() && !query.has_filters()) {
        return Ok(Vec::new());
    }
    if !query.text.trim().is_empty() && expression.is_none() {
        return Ok(Vec::new());
    }

    let tag_rows: Vec<(String, Option<String>, Tag)> = {
        let mut statement = conn.prepare_cached(
            "SELECT id, name, color_hex, parent_id, position FROM tags ORDER BY position, id",
        )?;
        statement
            .query_map([], |row| {
                let tag = Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color_hex: row.get(2)?,
                    parent_id: row.get(3)?,
                    position: row.get(4)?,
                };
                Ok((tag.id.clone(), tag.parent_id.clone(), tag))
            })?
            .collect::<Result<_, _>>()?
    };
    let tag_names: Vec<(String, String)> = tag_rows
        .iter()
        .map(|(id, _, tag)| (id.clone(), tag.name.clone()))
        .collect();
    let tag_tree: Vec<(String, Option<String>)> = tag_rows
        .iter()
        .map(|(id, parent, _)| (id.clone(), parent.clone()))
        .collect();
    // One group of acceptable tag ids per `tag:` filter: all groups must be satisfied.
    let tag_groups: Vec<String> = query
        .tags
        .iter()
        .map(|name| {
            let roots = names_to_ids(std::slice::from_ref(name), &tag_names);
            json_array(&with_descendants(&roots, &tag_tree))
        })
        .collect();

    let folder_roots: Option<String> = match &query.folder {
        None => None,
        Some(name) => {
            let mut statement = conn.prepare_cached("SELECT id, name FROM folders")?;
            let folders: Vec<(String, String)> = statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<Result<_, _>>()?;
            Some(json_array(&names_to_ids(
                std::slice::from_ref(name),
                &folders,
            )))
        }
    };

    let mut items = Vec::new();

    if query.tag_items {
        let wanted = fold(&query.text);
        let mut statement = conn.prepare_cached(
            "SELECT g.id,
                    (SELECT COUNT(*) FROM note_tags nt JOIN notes n ON n.id = nt.note_id
                      WHERE nt.tag_id = g.id AND n.deleted_at IS NULL),
                    (SELECT COUNT(*) FROM task_tags tt JOIN tasks t ON t.id = tt.task_id
                      WHERE tt.tag_id = g.id AND t.deleted_at IS NULL AND t.is_completed = 0)
             FROM tags g",
        )?;
        let counts: Vec<(String, i64, i64)> = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<Result<_, _>>()?;
        for (id, _, tag) in &tag_rows {
            if !wanted.is_empty() && !fold(&tag.name).contains(&wanted) {
                continue;
            }
            let (note_count, pending_count) = counts
                .iter()
                .find(|(candidate, _, _)| candidate == id)
                .map_or((0, 0), |(_, notes, pending)| (*notes, *pending));
            items.push(FoundItem::Tag {
                tag_id: id.clone(),
                name: tag.name.clone(),
                color_hex: tag.color_hex.clone(),
                note_count,
                pending_count,
            });
            if items.len() >= limit as usize {
                break;
            }
        }
    }

    if query.notes {
        notes(
            conn,
            expression.as_deref(),
            &tag_groups,
            folder_roots.as_deref(),
            &query,
            limit,
            &mut items,
        )?;
    }
    if query.tasks {
        tasks(
            conn,
            expression.as_deref(),
            &tag_groups,
            query.completed,
            (&query.from, &query.to),
            limit,
            &mut items,
        )?;
    }
    Ok(items)
}

/// ` AND EXISTS (...)` once per tag filter. The `?N` placeholders start at `first`; only fixed
/// text and counters are written into the SQL, the ids travel as bound JSON arrays.
fn tag_clauses(link_table: &str, key: &str, owner: &str, groups: usize, first: usize) -> String {
    (0..groups)
        .map(|offset| {
            format!(
                " AND EXISTS (SELECT 1 FROM {link_table} x WHERE x.{key} = {owner}
                              AND x.tag_id IN (SELECT value FROM json_each(?{})))",
                first + offset
            )
        })
        .collect()
}

/// ` AND column >= ?N AND column <= ?M` for `desde:`/`hasta:`, written only for the ends that
/// were given (a `?N IS NULL OR ...` guard would keep SQLite from using the column's index),
/// with their values. Placeholders start at `first`.
///
/// `upper_suffix` is appended to the upper bound: notes compare their UTC timestamp
/// (`2026-03-31T23:00:00Z`) with a day, and `2026-03-31Z` sorts after every instant of that day
/// and before the next, so the filter stays a plain range over the indexed column.
fn range_clause(
    column: &str,
    first: usize,
    range: (&Option<String>, &Option<String>),
    upper_suffix: &str,
) -> (String, Vec<Value>) {
    let mut sql = String::new();
    let mut values = Vec::new();
    if let Some(from) = range.0 {
        sql.push_str(&format!(" AND {column} >= ?{}", first + values.len()));
        values.push(Value::Text(from.clone()));
    }
    if let Some(to) = range.1 {
        sql.push_str(&format!(" AND {column} <= ?{}", first + values.len()));
        values.push(Value::Text(format!("{to}{upper_suffix}")));
    }
    (sql, values)
}

fn tasks(
    conn: &Connection,
    expression: Option<&str>,
    groups: &[String],
    completed: Option<bool>,
    range: (&Option<String>, &Option<String>),
    limit: u32,
    items: &mut Vec<FoundItem>,
) -> rusqlite::Result<()> {
    let mut values: Vec<Value> = Vec::new();
    let sql = if let Some(expression) = expression {
        values.push(Value::Text(expression.to_string()));
        values.push(Value::Integer(i64::from(limit)));
        values.push(completed.map_or(Value::Null, |done| Value::Integer(i64::from(done))));
        values.extend(groups.iter().cloned().map(Value::Text));
        let (range_sql, range_params) = range_clause("t.due_date", 4 + groups.len(), range, "");
        values.extend(range_params);
        format!(
            "SELECT t.id, t.title, snippet(tasks_fts, 2, '', '', '…', 12), t.due_date, t.is_completed
             FROM tasks_fts JOIN tasks t ON t.id = tasks_fts.task_id
             WHERE tasks_fts MATCH ?1 AND t.deleted_at IS NULL
               AND (?3 IS NULL OR t.is_completed = ?3){}{}
             ORDER BY bm25(tasks_fts, 0.0, 4.0, 1.0), t.is_completed, t.id
             LIMIT ?2",
            tag_clauses("task_tags", "task_id", "t.id", groups.len(), 4),
            range_sql
        )
    } else {
        values.push(Value::Integer(i64::from(limit)));
        values.push(completed.map_or(Value::Null, |done| Value::Integer(i64::from(done))));
        values.extend(groups.iter().cloned().map(Value::Text));
        let (range_sql, range_params) = range_clause("t.due_date", 3 + groups.len(), range, "");
        values.extend(range_params);
        format!(
            "SELECT t.id, t.title, '', t.due_date, t.is_completed
             FROM tasks t
             WHERE t.deleted_at IS NULL AND (?2 IS NULL OR t.is_completed = ?2){}{}
             ORDER BY t.is_completed, t.updated_at DESC, t.id
             LIMIT ?1",
            tag_clauses("task_tags", "task_id", "t.id", groups.len(), 3),
            range_sql
        )
    };
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(values), |row| {
        Ok(FoundItem::Task {
            task_id: row.get(0)?,
            title: row.get(1)?,
            snippet: row.get(2)?,
            due_date: row.get(3)?,
            is_completed: row.get(4)?,
        })
    })?;
    for row in rows {
        items.push(row?);
    }
    Ok(())
}

fn notes(
    conn: &Connection,
    expression: Option<&str>,
    groups: &[String],
    folder_roots: Option<&str>,
    query: &ParsedQuery,
    limit: u32,
    items: &mut Vec<FoundItem>,
) -> rusqlite::Result<()> {
    let (pinned, range) = (query.pinned, (&query.from, &query.to));
    const IN_FOLDER: &str = "(?4 IS NULL OR n.folder_id IN (
            WITH RECURSIVE down(id) AS (
                SELECT value FROM json_each(?4)
                UNION ALL
                SELECT f.id FROM folders f JOIN down ON f.parent_id = down.id)
            SELECT id FROM down))";
    let mut values: Vec<Value> = Vec::new();
    let folder = folder_roots.map_or(Value::Null, |roots| Value::Text(roots.to_string()));
    let sql = if let Some(expression) = expression {
        values.push(Value::Text(expression.to_string()));
        values.push(Value::Integer(i64::from(limit)));
        values.push(Value::Integer(i64::from(pinned)));
        values.push(folder);
        values.extend(groups.iter().cloned().map(Value::Text));
        let (range_sql, range_params) = range_clause("n.updated_at", 5 + groups.len(), range, "Z");
        values.extend(range_params);
        format!(
            "SELECT n.id, n.title, snippet(notes_fts, 2, '', '', '…', 12), n.folder_id, n.is_pinned
             FROM notes_fts JOIN notes n ON n.id = notes_fts.note_id
             WHERE notes_fts MATCH ?1 AND n.deleted_at IS NULL
               AND (?3 = 0 OR n.is_pinned = 1) AND {IN_FOLDER}{}{}
             ORDER BY bm25(notes_fts, 0.0, 4.0, 1.0), n.is_pinned DESC, n.id
             LIMIT ?2",
            tag_clauses("note_tags", "note_id", "n.id", groups.len(), 5),
            range_sql
        )
    } else {
        // Without text there is no index to ask: the first words of the note stand in.
        values.push(Value::Null);
        values.push(Value::Integer(i64::from(limit)));
        values.push(Value::Integer(i64::from(pinned)));
        values.push(folder);
        values.extend(groups.iter().cloned().map(Value::Text));
        let (range_sql, range_params) = range_clause("n.updated_at", 5 + groups.len(), range, "Z");
        values.extend(range_params);
        format!(
            "SELECT n.id, n.title, replace(substr(n.content_text, 1, 120), char(10), ' '),
                    n.folder_id, n.is_pinned
             FROM notes n
             WHERE ?1 IS NULL AND n.deleted_at IS NULL
               AND (?3 = 0 OR n.is_pinned = 1) AND {IN_FOLDER}{}{}
             ORDER BY n.is_pinned DESC, n.updated_at DESC, n.id
             LIMIT ?2",
            tag_clauses("note_tags", "note_id", "n.id", groups.len(), 5),
            range_sql
        )
    };
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(values), |row| {
        Ok(FoundItem::Note {
            note_id: row.get(0)?,
            title: row.get(1)?,
            snippet: row.get(2)?,
            folder_id: row.get(3)?,
            is_pinned: row.get(4)?,
        })
    })?;
    for row in rows {
        items.push(row?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::params;

    use super::*;
    use crate::db::{MIGRATIONS, open_in_memory};

    const NOW: &str = "2026-01-01T00:00:00Z";

    fn database() -> Connection {
        let conn = open_in_memory(&MIGRATIONS).expect("database");
        for (id, name, parent) in [
            ("g1", "Trabajo", None),
            ("g2", "Informes", Some("g1")),
            ("g3", "Casa", None),
        ] {
            conn.execute(
                "INSERT INTO tags (id, name, color_hex, parent_id, position, created_at)
                 VALUES (?1, ?2, '#4C8DFF', ?3, 0, ?4)",
                params![id, name, parent, NOW],
            )
            .expect("tag");
        }
        for (id, name, parent) in [("f1", "Clase", None), ("f2", "Álgebra", Some("f1"))] {
            conn.execute(
                "INSERT INTO folders (id, name, parent_id, position, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 0, ?4, ?4)",
                params![id, name, parent, NOW],
            )
            .expect("folder");
        }
        conn
    }

    fn task(conn: &Connection, id: &str, title: &str, done: bool, tags: &[&str]) {
        conn.execute(
            "INSERT INTO tasks (id, title, is_completed, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![id, title, done, NOW],
        )
        .expect("task");
        conn.execute(
            "INSERT INTO tasks_fts (task_id, title, content_text) VALUES (?1, ?2, '')",
            params![id, title],
        )
        .expect("index");
        for tag in tags {
            conn.execute(
                "INSERT INTO task_tags (task_id, tag_id) VALUES (?1, ?2)",
                params![id, tag],
            )
            .expect("task tag");
        }
    }

    fn note(
        conn: &Connection,
        id: &str,
        title: &str,
        folder: Option<&str>,
        pinned: bool,
        tags: &[&str],
    ) {
        conn.execute(
            "INSERT INTO notes (id, folder_id, title, content_text, is_pinned, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?3, ?4, ?5, ?5)",
            params![id, folder, title, pinned, NOW],
        )
        .expect("note");
        conn.execute(
            "INSERT INTO notes_fts (note_id, title, content_text) VALUES (?1, ?2, ?2)",
            params![id, title],
        )
        .expect("index");
        for tag in tags {
            conn.execute(
                "INSERT INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
                params![id, tag],
            )
            .expect("note tag");
        }
    }

    /// Short names of the results: `t:id`, `n:id`, `g:id`.
    fn found(conn: &Connection, input: &str) -> Vec<String> {
        find(conn, input, 50)
            .expect("find")
            .into_iter()
            .map(|item| match item {
                FoundItem::Task { task_id, .. } => format!("t:{task_id}"),
                FoundItem::Note { note_id, .. } => format!("n:{note_id}"),
                FoundItem::Tag { tag_id, .. } => format!("g:{tag_id}"),
            })
            .collect()
    }

    fn sorted(mut values: Vec<String>) -> Vec<String> {
        values.sort();
        values
    }

    #[test]
    fn parses_filters_quotes_and_keeps_the_rest_as_text() {
        let query = parse("informe tag:Trabajo is:tarea tag:\"mi etiqueta\" final");
        assert_eq!(query.text, "informe final");
        assert_eq!(query.tags, ["trabajo", "mi etiqueta"]);
        assert!(query.tasks && !query.notes && !query.tag_items);

        let query = parse("tag: in: is:");
        assert_eq!(query.text, "");
        assert!(query.tags.is_empty() && query.folder.is_none() && !query.has_filters());

        assert!(parse("is:marciano").impossible);
        let query = parse("is:nota is:tarea");
        assert!(!query.tasks && !query.notes);
        assert!(parse("in:clase").folder.is_some() && !parse("in:clase").tasks);
    }

    #[test]
    fn date_range_filters_tasks_by_day_and_notes_by_last_change() {
        let conn = database();
        task(&conn, "t1", "Informe uno", false, &["g1"]);
        task(&conn, "t2", "Informe dos", false, &[]);
        task(&conn, "t3", "Informe sin fecha", false, &[]);
        for (id, day) in [("t1", "2026-03-05"), ("t2", "2026-04-20")] {
            conn.execute(
                "UPDATE tasks SET due_date = ?2 WHERE id = ?1",
                params![id, day],
            )
            .expect("date");
        }
        note(&conn, "n1", "Informe nota", None, false, &[]);
        conn.execute(
            "UPDATE notes SET updated_at = '2026-03-31T23:00:00Z' WHERE id = 'n1'",
            [],
        )
        .expect("note date");

        assert_eq!(
            sorted(found(&conn, "informe")),
            ["g:g2", "n:n1", "t:t1", "t:t2", "t:t3"]
        );
        assert_eq!(
            sorted(found(&conn, "informe desde:2026-03-01 hasta:2026-03-31")),
            ["n:n1", "t:t1"]
        );
        assert_eq!(found(&conn, "informe desde:2026-04-01"), ["t:t2"]);
        assert_eq!(found(&conn, "informe hasta:2026-03-05 is:tarea"), ["t:t1"]);
        // Both ends are included, and it combines with the other filters.
        assert_eq!(
            found(
                &conn,
                "informe desde:2026-03-05 hasta:2026-03-05 tag:trabajo"
            ),
            ["t:t1"]
        );
        // Without text the range alone is a search; tags are not dated, so they drop out.
        assert_eq!(
            sorted(found(&conn, "desde:2026-03-01 hasta:2026-03-31")),
            ["n:n1", "t:t1"]
        );
        // A half-typed or invalid date does not match anything surprising.
        assert!(found(&conn, "informe desde:2026-13-40").is_empty());
        assert_eq!(found(&conn, "informe hasta:").len(), 5);
    }

    #[test]
    fn searches_tasks_notes_and_tags_at_once_without_regard_to_accents() {
        let conn = database();
        task(&conn, "t1", "Estudiar álgebra", false, &[]);
        note(&conn, "n1", "Apuntes de algebra", None, false, &[]);
        note(&conn, "n2", "Recetas", None, false, &[]);
        assert_eq!(sorted(found(&conn, "algebra")), ["n:n1", "t:t1"]);
        assert_eq!(found(&conn, "trab"), ["g:g1"]);
        assert!(found(&conn, "").is_empty());
        assert!(found(&conn, "zzz").is_empty());
    }

    #[test]
    fn is_filters_pick_the_kind_and_the_state() {
        let conn = database();
        task(&conn, "t1", "Informe uno", false, &[]);
        task(&conn, "t2", "Informe dos", true, &[]);
        note(&conn, "n1", "Informe notas", None, true, &[]);
        note(&conn, "n2", "Informe otras", None, false, &[]);
        assert_eq!(sorted(found(&conn, "informe is:tarea")), ["t:t1", "t:t2"]);
        assert_eq!(found(&conn, "informe is:hecha"), ["t:t2"]);
        assert_eq!(found(&conn, "informe is:pendiente"), ["t:t1"]);
        assert_eq!(found(&conn, "informe is:fijada"), ["n:n1"]);
        assert_eq!(sorted(found(&conn, "informe is:nota")), ["n:n1", "n:n2"]);
        assert!(found(&conn, "informe is:nota is:tarea").is_empty());
        assert!(found(&conn, "informe is:marciano").is_empty());
    }

    #[test]
    fn tag_filter_accepts_subtags_and_every_tag_must_match() {
        let conn = database();
        task(&conn, "t1", "Uno", false, &["g2"]);
        task(&conn, "t2", "Dos", false, &["g3"]);
        note(&conn, "n1", "Tres", None, false, &["g1", "g3"]);
        // `tag:` with no text lists what carries the tag, subtags included.
        assert_eq!(sorted(found(&conn, "tag:trabajo")), ["n:n1", "t:t1"]);
        assert_eq!(sorted(found(&conn, "tag:CASA")), ["n:n1", "t:t2"]);
        assert_eq!(found(&conn, "tag:trabajo tag:casa"), ["n:n1"]);
        assert_eq!(found(&conn, "dos tag:casa"), ["t:t2"]);
        assert!(found(&conn, "tag:inexistente").is_empty());
    }

    #[test]
    fn folder_filter_includes_subfolders_and_leaves_tasks_out() {
        let conn = database();
        task(&conn, "t1", "Apuntes", false, &[]);
        note(&conn, "n1", "Apuntes de clase", Some("f1"), false, &[]);
        note(&conn, "n2", "Apuntes de álgebra", Some("f2"), false, &[]);
        note(&conn, "n3", "Apuntes sueltos", None, false, &[]);
        assert_eq!(sorted(found(&conn, "apuntes in:clase")), ["n:n1", "n:n2"]);
        assert_eq!(found(&conn, "apuntes in:algebra"), ["n:n2"]);
        assert!(found(&conn, "apuntes in:nada").is_empty());
        assert_eq!(sorted(found(&conn, "in:clase")), ["n:n1", "n:n2"]);
    }

    #[test]
    fn trashed_items_are_not_found() {
        let conn = database();
        note(&conn, "n1", "Borrada", None, false, &[]);
        conn.execute("UPDATE notes SET deleted_at = ?1", [NOW])
            .expect("trash");
        assert!(found(&conn, "borrada").is_empty());
        assert!(found(&conn, "is:nota").is_empty());
    }

    #[test]
    fn hostile_input_is_plain_text() {
        let conn = database();
        note(&conn, "n1", "Algo", None, false, &[]);
        for query in ["\"", "tag:\"", "in:'; DROP TABLE notes;--", "NEAR(", "a OR"] {
            assert!(find(&conn, query, 50).is_ok(), "{query}");
        }
    }
}

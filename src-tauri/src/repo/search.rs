//! Full-text search over title and notes (SPEC F-09). The index folds accents
//! (`unicode61 remove_diacritics 2`), so "algebra" finds "álgebra" and the other way round.

use rusqlite::{Connection, params};
use serde::Serialize;

/// One search result, with only what the Spotlight list draws.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub task_id: String,
    pub title: String,
    /// Fragment of the notes around the match; empty when the task has no notes.
    pub snippet: String,
    pub due_date: Option<String>,
    pub is_completed: bool,
}

pub const DEFAULT_LIMIT: u32 = 50;
pub const MAX_LIMIT: u32 = 200;

/// Turns free text into an FTS5 expression: every word is quoted (so operators typed by the
/// user are plain text) and is a prefix, which makes search-as-you-type work.
/// `None` when the text has no searchable word.
pub fn match_expression(query: &str) -> Option<String> {
    let words: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| format!("\"{word}\"*"))
        .collect();
    if words.is_empty() {
        None
    } else {
        Some(words.join(" "))
    }
}

pub fn search(conn: &Connection, query: &str, limit: u32) -> rusqlite::Result<Vec<SearchHit>> {
    let Some(expression) = match_expression(query) else {
        return Ok(Vec::new());
    };
    let mut statement = conn.prepare_cached(
        "SELECT t.id, t.title, snippet(tasks_fts, 2, '', '', '…', 12), t.due_date, t.is_completed
         FROM tasks_fts
         JOIN tasks t ON t.id = tasks_fts.task_id
         WHERE tasks_fts MATCH ?1 AND t.deleted_at IS NULL
         ORDER BY bm25(tasks_fts, 0.0, 4.0, 1.0), t.is_completed, t.id
         LIMIT ?2",
    )?;
    let rows = statement.query_map(params![expression, limit], |row| {
        Ok(SearchHit {
            task_id: row.get(0)?,
            title: row.get(1)?,
            snippet: row.get(2)?,
            due_date: row.get(3)?,
            is_completed: row.get(4)?,
        })
    })?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use crate::db::{MIGRATIONS, open_in_memory};

    const NOW: &str = "2026-01-01T00:00:00Z";

    fn database() -> Connection {
        open_in_memory(&MIGRATIONS).expect("database")
    }

    fn add(conn: &Connection, id: &str, title: &str, text: &str, deleted: bool) {
        conn.execute(
            "INSERT INTO tasks (id, title, content_text, created_at, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?4, CASE WHEN ?5 THEN ?4 END)",
            params![id, title, text, NOW, deleted],
        )
        .expect("task");
        if !deleted {
            conn.execute(
                "INSERT INTO tasks_fts (task_id, title, content_text) VALUES (?1, ?2, ?3)",
                params![id, title, text],
            )
            .expect("index");
        }
    }

    fn ids(conn: &Connection, query: &str) -> Vec<String> {
        search(conn, query, DEFAULT_LIMIT)
            .expect("search")
            .into_iter()
            .map(|hit| hit.task_id)
            .collect()
    }

    #[test]
    fn expression_quotes_words_and_prefixes_the_last() {
        assert_eq!(
            match_expression("pan leche").as_deref(),
            Some("\"pan\"* \"leche\"*")
        );
        assert_eq!(
            match_expression("  álgebra ").as_deref(),
            Some("\"álgebra\"*")
        );
        assert_eq!(
            match_expression("a\" OR b*").as_deref(),
            Some("\"a\"* \"OR\"* \"b\"*")
        );
        assert_eq!(match_expression(""), None);
        assert_eq!(match_expression(" ** \"\" - "), None);
    }

    #[test]
    fn finds_with_and_without_accents_in_both_directions() {
        let conn = database();
        add(&conn, "t1", "Estudiar álgebra", "", false);
        add(
            &conn,
            "t2",
            "Llamar",
            "revisar la canción y el corazón",
            false,
        );
        add(&conn, "t3", "Comprar pan", "", false);

        assert_eq!(ids(&conn, "algebra"), ["t1"]);
        assert_eq!(ids(&conn, "álgebra"), ["t1"]);
        assert_eq!(ids(&conn, "ALGEBRA"), ["t1"]);
        assert_eq!(ids(&conn, "cancion"), ["t2"]);
        assert_eq!(ids(&conn, "corazón"), ["t2"]);
        assert_eq!(ids(&conn, "pan"), ["t3"]);
    }

    #[test]
    fn matches_word_prefixes_and_requires_every_word() {
        let conn = database();
        add(&conn, "t1", "Comprar pan integral", "", false);
        add(&conn, "t2", "Comprar leche", "", false);

        assert_eq!(ids(&conn, "comp pan"), ["t1"]);
        assert_eq!(ids(&conn, "compr").len(), 2);
        assert!(ids(&conn, "pan leche").is_empty());
    }

    #[test]
    fn hostile_input_is_plain_text_not_syntax() {
        let conn = database();
        add(&conn, "t1", "Comprar pan", "", false);
        for query in [
            "\"",
            "pan OR",
            "NEAR(",
            "col:pan",
            "pan*)",
            "-pan",
            "'; DROP TABLE tasks;--",
        ] {
            assert!(search(&conn, query, DEFAULT_LIMIT).is_ok(), "{query}");
        }
        assert!(ids(&conn, "").is_empty());
    }

    #[test]
    fn deleted_tasks_are_not_found_and_hits_carry_state() {
        let conn = database();
        add(&conn, "t1", "Informe final", "", true);
        add(
            &conn,
            "t2",
            "Informe parcial",
            "cálculo de costes del informe",
            false,
        );
        conn.execute(
            "UPDATE tasks SET is_completed = 1, due_date = '2026-03-12' WHERE id = 't2'",
            [],
        )
        .expect("update");

        let hits = search(&conn, "informe", DEFAULT_LIMIT).expect("search");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].task_id, "t2");
        assert_eq!(hits[0].due_date.as_deref(), Some("2026-03-12"));
        assert!(hits[0].is_completed);
        assert!(hits[0].snippet.contains("informe"));
    }

    #[test]
    fn title_matches_rank_above_note_matches_and_limit_applies() {
        let conn = database();
        add(&conn, "t1", "Reunión", "preparar el informe", false);
        add(&conn, "t2", "Informe", "", false);
        assert_eq!(ids(&conn, "informe"), ["t2", "t1"]);
        assert_eq!(search(&conn, "informe", 1).expect("search").len(), 1);
    }

    /// F-09: results in under 100 ms with 10 000 tasks.
    #[test]
    fn search_is_fast_with_ten_thousand_tasks() {
        let conn = database();
        conn.execute(
            "WITH RECURSIVE n(i) AS (SELECT 0 UNION ALL SELECT i + 1 FROM n WHERE i < 9999)
             INSERT INTO tasks (id, title, content_text, created_at, updated_at)
             SELECT 't' || i, 'Tarea número ' || i, 'contenido de la tarea ' || i || ' álgebra', ?1, ?1
             FROM n",
            [NOW],
        )
        .expect("bulk tasks");
        conn.execute(
            "INSERT INTO tasks_fts (task_id, title, content_text)
             SELECT id, title, content_text FROM tasks",
            [],
        )
        .expect("bulk index");

        let mut worst = std::time::Duration::ZERO;
        for query in ["algebra", "tarea 99", "conten", "numero 5000"] {
            let start = Instant::now();
            let hits = search(&conn, query, DEFAULT_LIMIT).expect("search");
            worst = worst.max(start.elapsed());
            assert!(!hits.is_empty(), "{query}");
        }
        eprintln!("F-09 search with 10 000 tasks, worst case: {worst:?}");
        assert!(worst.as_millis() < 100, "search took {worst:?}");
    }
}

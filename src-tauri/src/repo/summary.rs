//! Reads behind the weekly summary (D-100).

use rusqlite::{Connection, params};

/// A task that matters to a week: open with a due date in it, or completed during it.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskFact {
    pub due_date: Option<String>,
    pub completed: bool,
    pub tag_ids: Vec<String>,
}

/// Open tasks due between `from` and `to` (local dates, inclusive) plus the ones completed in
/// `[done_from, done_to)` (UTC instants).
pub fn task_facts(
    conn: &Connection,
    from: &str,
    to: &str,
    done_from: &str,
    done_to: &str,
) -> rusqlite::Result<Vec<TaskFact>> {
    conn.prepare_cached(
        "SELECT t.due_date, t.is_completed,
                (SELECT json_group_array(tt.tag_id ORDER BY tt.tag_id)
                   FROM task_tags tt WHERE tt.task_id = t.id)
         FROM tasks t
         WHERE t.deleted_at IS NULL AND (
               (t.is_completed = 0 AND t.due_date BETWEEN ?1 AND ?2)
            OR (t.is_completed = 1 AND t.completed_at >= ?3 AND t.completed_at < ?4))",
    )?
    .query_map(params![from, to, done_from, done_to], |row| {
        let tags: String = row.get(2)?;
        Ok(TaskFact {
            due_date: row.get(0)?,
            completed: row.get(1)?,
            tag_ids: serde_json::from_str(&tags).unwrap_or_default(),
        })
    })?
    .collect()
}

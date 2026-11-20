//! Whole-database reads for the JSON export (F14-02). Rows are read generically and named like
//! the IPC contract (`camelCase`), so adding a column to a table only needs it added to its
//! `SELECT` here. Documents (`content_json`) come out as JSON, not as a string inside JSON.

use std::collections::HashMap;

use rusqlite::{Connection, types::ValueRef};
use serde_json::{Map, Value, json};

fn camel(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut upper = false;
    for c in name.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

fn cell(name: &str, value: ValueRef<'_>) -> Value {
    match value {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(number) if name.starts_with("is_") => json!(number != 0),
        ValueRef::Integer(number) => json!(number),
        ValueRef::Real(number) => json!(number),
        ValueRef::Text(bytes) => {
            let text = String::from_utf8_lossy(bytes);
            if name == "content_json" || name == "reminder_offsets" {
                serde_json::from_str(&text).unwrap_or_else(|_| json!(text))
            } else {
                json!(text)
            }
        }
        ValueRef::Blob(_) => Value::Null,
    }
}

/// Every row of `sql` as an object keyed by the camel-cased column names.
fn rows(conn: &Connection, sql: &str) -> rusqlite::Result<Vec<Map<String, Value>>> {
    let mut statement = conn.prepare(sql)?;
    let names: Vec<String> = statement.column_names().map(str::to_string).collect();
    statement
        .query_map([], |row| {
            let mut object = Map::new();
            for (index, name) in names.iter().enumerate() {
                object.insert(camel(name), cell(name, row.get_ref(index)?));
            }
            Ok(object)
        })?
        .collect()
}

/// Attaches to each object of `owners` the list `key` of the `value` column of `links`
/// (`owner_id, value`), in the order the query gives them.
fn attach(owners: &mut [Map<String, Value>], key: &str, links: &[(String, Value)]) {
    let mut by_owner: HashMap<&str, Vec<Value>> = HashMap::new();
    for (owner, value) in links {
        by_owner.entry(owner).or_default().push(value.clone());
    }
    for object in owners {
        let id = object.get("id").and_then(Value::as_str).unwrap_or("");
        let list = by_owner.remove(id).unwrap_or_default();
        object.insert(key.into(), Value::Array(list));
    }
}

fn pairs(conn: &Connection, sql: &str) -> rusqlite::Result<Vec<(String, Value)>> {
    conn.prepare(sql)?
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, json!(row.get::<_, String>(1)?)))
        })?
        .collect()
}

/// Titles of every task, for the references a note holds.
pub fn task_titles(conn: &Connection) -> rusqlite::Result<HashMap<String, String>> {
    conn.prepare("SELECT id, title FROM tasks")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect()
}

/// The data of the whole application, trash included, as one JSON object.
pub fn everything(conn: &Connection, exported_at: &str) -> rusqlite::Result<Value> {
    let folders = rows(
        conn,
        "SELECT id, name, parent_id, position, created_at, updated_at
           FROM folders ORDER BY position, id",
    )?;
    let tags = rows(
        conn,
        "SELECT id, name, color_hex, parent_id, position, created_at
           FROM tags ORDER BY position, id",
    )?;
    let mut notes = rows(
        conn,
        "SELECT id, folder_id, title, content_json, is_pinned, daily_date,
                created_at, updated_at, deleted_at
           FROM notes ORDER BY created_at, id",
    )?;
    attach(
        &mut notes,
        "tagIds",
        &pairs(
            conn,
            "SELECT note_id, tag_id FROM note_tags ORDER BY tag_id",
        )?,
    );
    attach(&mut notes, "reminders", &note_reminders(conn)?);
    let mut tasks = rows(
        conn,
        "SELECT id, title, content_json, due_date, due_time, duration_minutes,
                is_completed, completed_at, rrule, priority, checklist_done, checklist_total,
                source_note_id, position,
                created_at, updated_at, deleted_at
           FROM tasks ORDER BY created_at, id",
    )?;
    attach(
        &mut tasks,
        "tagIds",
        &pairs(
            conn,
            "SELECT task_id, tag_id FROM task_tags ORDER BY tag_id",
        )?,
    );
    let reminders: Vec<(String, Value)> = rows(
        conn,
        "SELECT task_id, notify_at, offset_minutes, is_sent, repeat_every, repeat_unit,
                repeat_count, repeat_until
           FROM reminders ORDER BY notify_at, id",
    )?
    .into_iter()
    .map(|mut reminder| {
        let task = reminder
            .remove("taskId")
            .and_then(|id| id.as_str().map(str::to_string))
            .unwrap_or_default();
        (task, Value::Object(reminder))
    })
    .collect();
    attach(&mut tasks, "reminders", &reminders);
    let mut events = rows(
        conn,
        "SELECT id, title, notes, start_date, start_time, end_date, end_time, rrule,
                reminder_offsets, created_at, updated_at, deleted_at
           FROM events ORDER BY start_date, id",
    )?;
    attach(
        &mut events,
        "tagIds",
        &pairs(
            conn,
            "SELECT event_id, tag_id FROM event_tags ORDER BY tag_id",
        )?,
    );
    let mut habits = rows(
        conn,
        "SELECT id, title, days, time, tag_id, position, created_at, updated_at,
                archived_at, deleted_at
           FROM habits ORDER BY position, id",
    )?;
    attach(
        &mut habits,
        "doneDates",
        &pairs(conn, "SELECT habit_id, date FROM habit_logs ORDER BY date")?,
    );
    let time_entries = rows(
        conn,
        "SELECT id, task_id, started_at, ended_at, planned_minutes, paused_seconds, paused_at
           FROM time_entries ORDER BY started_at, id",
    )?;
    let templates = rows(
        conn,
        "SELECT id, name, content_json, created_at, updated_at
           FROM note_templates ORDER BY name COLLATE NOCASE",
    )?;
    let attachments = rows(
        conn,
        "SELECT id, mime, size_bytes FROM attachments ORDER BY id",
    )?;
    Ok(json!({
        "format": "agendo-export",
        "version": 1,
        "exportedAt": exported_at,
        "folders": folders,
        "tags": tags,
        "notes": notes,
        "tasks": tasks,
        "events": events,
        "habits": habits,
        "timeEntries": time_entries,
        "noteTemplates": templates,
        "attachments": attachments,
    }))
}

fn note_reminders(conn: &Connection) -> rusqlite::Result<Vec<(String, Value)>> {
    let items = rows(
        conn,
        "SELECT note_id, notify_at, is_sent FROM note_reminders ORDER BY notify_at, id",
    )?;
    Ok(items
        .into_iter()
        .map(|mut reminder| {
            let note = reminder
                .remove("noteId")
                .and_then(|id| id.as_str().map(str::to_string))
                .unwrap_or_default();
            (note, Value::Object(reminder))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{MIGRATIONS, open_in_memory},
        repo::{
            model::{NewNote, NewTask, TagInput},
            notes, tags, tasks, with_tx,
        },
    };

    #[test]
    fn column_names_become_camel_case() {
        assert_eq!(camel("source_note_id"), "sourceNoteId");
        assert_eq!(camel("id"), "id");
    }

    #[test]
    fn the_export_holds_documents_flags_links_and_reminders() {
        let mut conn = open_in_memory(&MIGRATIONS).expect("db");
        with_tx::<_, rusqlite::Error>(&mut conn, |tx| {
            tags::upsert(
                tx,
                &TagInput {
                    id: "tag".into(),
                    name: "Etiqueta".into(),
                    color_hex: "#4C8DFF".into(),
                    parent_id: None,
                    now: "2026-01-01T00:00:00Z".into(),
                },
            )?;
            notes::create(
                tx,
                &NewNote {
                    id: "note".into(),
                    title: "Nota".into(),
                    content_json: Some(r#"{"type":"doc","content":[]}"#.into()),
                    tag_ids: vec!["tag".into()],
                    now: "2026-01-01T00:00:00Z".into(),
                    ..NewNote::default()
                },
            )?;
            tasks::create(
                tx,
                &NewTask {
                    id: "task".into(),
                    title: "Tarea".into(),
                    tag_ids: vec!["tag".into()],
                    now: "2026-01-01T00:00:00Z".into(),
                    ..NewTask::default()
                },
            )?;
            tx.execute(
                "INSERT INTO reminders (id, task_id, notify_at, created_at)
                 VALUES ('r', 'task', '2026-02-01T08:00:00Z', '2026-01-01T00:00:00Z')",
                [],
            )?;
            Ok(())
        })
        .expect("seed");

        let data = everything(&conn, "2026-03-01T00:00:00Z").expect("export");
        assert_eq!(data["format"], "agendo-export");
        assert_eq!(data["notes"][0]["contentJson"]["type"], "doc");
        assert_eq!(data["notes"][0]["isPinned"], false);
        assert_eq!(data["notes"][0]["tagIds"], json!(["tag"]));
        assert_eq!(data["tasks"][0]["tagIds"], json!(["tag"]));
        assert_eq!(
            data["tasks"][0]["reminders"][0]["notifyAt"],
            "2026-02-01T08:00:00Z"
        );
        assert_eq!(data["tasks"][0]["reminders"][0]["isSent"], false);
        assert_eq!(data["tags"][0]["colorHex"], "#4C8DFF");
        assert!(data["notes"][0].get("contentText").is_none());
        assert_eq!(task_titles(&conn).expect("titles")["task"], "Tarea");
    }

    #[test]
    fn the_export_holds_priority_events_habits_and_focus_sessions() {
        let conn = open_in_memory(&MIGRATIONS).expect("db");
        conn.execute_batch(
            "INSERT INTO tags (id, name, color_hex, position, created_at)
               VALUES ('tag', 'Etiqueta', '#4C8DFF', 0, 'x');
             INSERT INTO tasks (id, title, priority, checklist_done, checklist_total, created_at, updated_at)
               VALUES ('task', 'Tarea', 2, 1, 3, 'x', 'x');
             INSERT INTO events (id, title, start_date, start_time, end_date, end_time, rrule,
                                 reminder_offsets, created_at, updated_at)
               VALUES ('ev', 'Seminario', '2026-03-12', '10:00', '2026-03-12', '11:00',
                       'FREQ=WEEKLY;INTERVAL=1', '[15,60]', 'x', 'x');
             INSERT INTO event_tags VALUES ('ev', 'tag');
             INSERT INTO habits (id, title, days, tag_id, created_at, updated_at)
               VALUES ('hab', 'Leer', 31, 'tag', 'x', 'x');
             INSERT INTO habit_logs VALUES ('hab', '2026-03-09'), ('hab', '2026-03-10');
             INSERT INTO time_entries (id, task_id, started_at, ended_at, planned_minutes)
               VALUES ('te', 'task', '2026-03-10T10:00:00Z', '2026-03-10T10:25:00Z', 25);",
        )
        .expect("seed");

        let data = everything(&conn, "2026-03-11T00:00:00Z").expect("export");

        assert_eq!(data["tasks"][0]["priority"], 2);
        assert_eq!(data["tasks"][0]["checklistTotal"], 3);
        assert_eq!(data["events"][0]["title"], "Seminario");
        assert_eq!(data["events"][0]["reminderOffsets"], json!([15, 60]));
        assert_eq!(data["events"][0]["tagIds"], json!(["tag"]));
        assert_eq!(data["events"][0]["rrule"], "FREQ=WEEKLY;INTERVAL=1");
        assert_eq!(data["habits"][0]["days"], 31);
        assert_eq!(
            data["habits"][0]["doneDates"],
            json!(["2026-03-09", "2026-03-10"])
        );
        assert_eq!(data["timeEntries"][0]["plannedMinutes"], 25);
        assert_eq!(data["timeEntries"][0]["endedAt"], "2026-03-10T10:25:00Z");
    }
}

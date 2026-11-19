//! Task commands (SPEC §7.3). Each command is a thin async wrapper over a plain function that
//! validates the input, calls the repository inside one transaction and maps errors to
//! `CODE: message` strings; the plain functions are what the tests exercise.

use std::sync::Arc;

use chrono::Local;
use serde::{Deserialize, Deserializer, Serialize};
use tauri::State;

use crate::{
    domain::{
        content, dates,
        ordering::{self, DayGroup},
        recurrence::Rule,
        validation,
    },
    error::{AppError, AppResult},
    repo::{
        model::{NewTask, Task, TaskPatch},
        reminders::{self, NewReminder},
        tasks::{self, Query},
    },
    state::{AppState, run_blocking},
};

fn task_not_found(id: &str) -> AppError {
    AppError::NotFound(format!("tarea {id} no encontrada"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListView {
    Todo,
    Agenda,
    Range,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Section {
    Todo,
    Agenda,
}

/// Result of `tasks_list`: both sections for the `todo` and `agenda` views (the one that was not
/// asked for is empty), or a flat list for `range`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum TasksListOutput {
    Sections(Sections),
    Range(Vec<Task>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sections {
    pub todo: Vec<Task>,
    pub agenda: Vec<DayGroup<Task>>,
}

#[derive(Debug, Clone)]
pub struct TasksListInput {
    pub view: ListView,
    pub from: Option<String>,
    pub to: Option<String>,
    pub tag_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TaskCreateInput {
    pub title: Option<String>,
    pub due_date: Option<String>,
    pub due_time: Option<String>,
    pub duration_minutes: Option<i64>,
    pub tag_ids: Option<Vec<String>>,
    pub content_json: Option<String>,
    /// Note the task is created from (D-45).
    pub source_note_id: Option<String>,
    /// Repeat rule (F13-05); needs a date.
    pub rrule: Option<String>,
}

/// Distinguishes an absent field (leave as is) from `null` (clear it).
fn present_or_null<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(deserializer).map(Some)
}

fn present_or_null_number<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<i64>>, D::Error> {
    Option::<i64>::deserialize(deserializer).map(Some)
}

/// Partial update of a task, as sent by the frontend.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPatchInput {
    pub title: Option<String>,
    pub content_json: Option<String>,
    #[serde(default, deserialize_with = "present_or_null")]
    pub due_date: Option<Option<String>>,
    #[serde(default, deserialize_with = "present_or_null")]
    pub due_time: Option<Option<String>>,
    #[serde(default, deserialize_with = "present_or_null_number")]
    pub duration_minutes: Option<Option<i64>>,
    #[serde(default, deserialize_with = "present_or_null")]
    pub rrule: Option<Option<String>>,
}

/// Validates a repeat rule and anchors it to the task's date (F13-05).
fn normalize_rule(rule: &str, due_date: Option<&str>) -> AppResult<String> {
    let Some(date) = due_date else {
        return Err(AppError::Validation(
            "una tarea que se repite necesita fecha".into(),
        ));
    };
    let start = dates::parse_date(date)?;
    Ok(Rule::parse(rule)?.anchored(start)?.format())
}

pub fn list(state: &AppState, input: TasksListInput) -> AppResult<TasksListOutput> {
    let tag = input.tag_id.as_deref();
    state.db.read(|conn| match input.view {
        ListView::Todo => {
            let mut todo = tasks::list(conn, Query::Todo, tag)?;
            ordering::sort_todo(&mut todo);
            Ok(TasksListOutput::Sections(Sections {
                todo,
                agenda: Vec::new(),
            }))
        }
        ListView::Agenda => {
            let agenda = ordering::group_by_day(tasks::list(conn, Query::Agenda, tag)?);
            Ok(TasksListOutput::Sections(Sections {
                todo: Vec::new(),
                agenda,
            }))
        }
        ListView::Range => {
            let (from, to) = match (input.from.as_deref(), input.to.as_deref()) {
                (Some(from), Some(to)) => (from, to),
                _ => {
                    return Err(AppError::Validation(
                        "la vista de rango necesita las fechas «from» y «to»".into(),
                    ));
                }
            };
            if dates::parse_date(from)? > dates::parse_date(to)? {
                return Err(AppError::Validation(
                    "la fecha inicial del rango es posterior a la final".into(),
                ));
            }
            let found = tasks::list(conn, Query::Range { from, to }, tag)?;
            let days = ordering::group_by_day(found);
            Ok(TasksListOutput::Range(
                days.into_iter().rev().flat_map(|day| day.tasks).collect(),
            ))
        }
    })
}

/// Both sections (undated tasks and the agenda) from one snapshot, for a single refresh.
pub fn list_all(state: &AppState, tag_id: Option<&str>) -> AppResult<Sections> {
    state.db.read(|conn| {
        let mut todo = tasks::list(conn, Query::Todo, tag_id)?;
        ordering::sort_todo(&mut todo);
        let agenda = ordering::group_by_day(tasks::list(conn, Query::Agenda, tag_id)?);
        Ok(Sections { todo, agenda })
    })
}

pub fn create(state: &AppState, input: TaskCreateInput) -> AppResult<Task> {
    let title = input.title.unwrap_or_default();
    validation::validate_title(&title)?;
    validation::validate_schedule(input.due_date.as_deref(), input.due_time.as_deref())?;
    validation::validate_duration(input.due_time.as_deref(), input.duration_minutes)?;
    let rrule = input
        .rrule
        .as_deref()
        .map(|rule| normalize_rule(rule, input.due_date.as_deref()))
        .transpose()?;
    let content_text = match &input.content_json {
        Some(json) => content::text_of(json)?,
        None => String::new(),
    };
    let new = NewTask {
        id: state.ids.new_id(),
        title,
        content_json: input.content_json,
        content_text,
        due_date: input.due_date,
        due_time: input.due_time,
        duration_minutes: input.duration_minutes,
        source_note_id: input.source_note_id,
        rrule,
        tag_ids: input.tag_ids.unwrap_or_default(),
        now: state.clock.now_string(),
    };
    state.db.write(|tx| Ok(tasks::create(tx, &new)?))
}

/// Removing a task's date also removes its time (a time needs a date), unless the patch sets a
/// time explicitly, which is then rejected.
pub fn update(state: &AppState, id: &str, patch: TaskPatchInput) -> AppResult<Task> {
    if let Some(title) = &patch.title {
        validation::validate_title(title)?;
    }
    let content_text = patch
        .content_json
        .as_deref()
        .map(content::text_of)
        .transpose()?;
    let now = state.clock.now_string();

    let task = state.db.write(|tx| {
        let current = tasks::get(tx, id)?.ok_or_else(|| task_not_found(id))?;

        let due_time_patch = match (&patch.due_date, &patch.due_time) {
            (Some(None), None) => Some(None),
            (_, time) => time.clone(),
        };
        let due_date = patch.due_date.clone().unwrap_or(current.due_date.clone());
        let due_time = due_time_patch.clone().unwrap_or(current.due_time.clone());
        validation::validate_schedule(due_date.as_deref(), due_time.as_deref())?;

        // A task without a time has no duration; changing the time keeps the duration unless
        // the patch sets one, and the pair must still fit in the day.
        let duration_patch = match (&patch.duration_minutes, &due_time) {
            (_, None) if current.duration_minutes.is_some() => Some(None),
            (patch_duration, _) => *patch_duration,
        };
        let duration = duration_patch.unwrap_or(current.duration_minutes);
        validation::validate_duration(due_time.as_deref(), duration)?;

        let repo_patch = TaskPatch {
            title: patch.title.clone(),
            content_json: patch.content_json.clone(),
            content_text,
            due_date: patch.due_date.clone(),
            due_time: due_time_patch,
            duration_minutes: duration_patch,
        };
        // The rule follows the date: without one the task stops repeating, and a new date
        // re-anchors it (the 31st of a month, the day of a year).
        let rrule_patch: Option<Option<String>> = match (&patch.rrule, due_date.as_deref()) {
            (Some(Some(_)), None) => {
                return Err(AppError::Validation(
                    "una tarea que se repite necesita fecha".into(),
                ));
            }
            (_, None) => current.rrule.is_some().then_some(None),
            (Some(Some(rule)), date) => Some(Some(normalize_rule(rule, date)?)),
            (Some(None), _) => Some(None),
            (None, date) => match &current.rrule {
                Some(rule) if date != current.due_date.as_deref() => {
                    Some(Some(normalize_rule(rule, date)?))
                }
                _ => None,
            },
        };
        let untouched = rrule_patch.is_none()
            && repo_patch.title.is_none()
            && repo_patch.content_json.is_none()
            && repo_patch.due_date.is_none()
            && repo_patch.due_time.is_none()
            && repo_patch.duration_minutes.is_none();
        if untouched {
            return Ok(current);
        }
        tasks::update(tx, id, &repo_patch, &now)?.ok_or_else(|| task_not_found(id))?;
        if let Some(rule) = &rrule_patch {
            tasks::set_rrule(tx, id, rule.as_deref())?;
        }
        if repo_patch.due_date.is_some() || repo_patch.due_time.is_some() {
            reminders::recalculate(tx, id, &Local)?;
        }
        tasks::get(tx, id)?.ok_or_else(|| task_not_found(id))
    })?;
    state.wake.notify_one();
    Ok(task)
}

#[cfg(test)]
pub fn set_completed(state: &AppState, id: &str, completed: bool) -> AppResult<Task> {
    complete(state, id, completed, None)
}

/// `local_date` is today in the user's time zone, which only the frontend knows; a task that is
/// completed moves to that day, whatever day it had (B-03, D-23).
pub fn complete(
    state: &AppState,
    id: &str,
    completed: bool,
    local_date: Option<&str>,
) -> AppResult<Task> {
    if let Some(date) = local_date {
        dates::parse_date(date)?;
    }
    let now = state.clock.now_string();
    let task = state.db.write(|tx| {
        let before = tasks::get(tx, id)?.ok_or_else(|| task_not_found(id))?;
        tasks::set_completed_on(tx, id, completed, &now, local_date)?
            .ok_or_else(|| task_not_found(id))?;
        reminders::recalculate(tx, id, &Local)?;
        if completed && !before.is_completed {
            spawn_next_occurrence(state, tx, &before, local_date, &now)?;
        }
        tasks::get(tx, id)?.ok_or_else(|| task_not_found(id))
    })?;
    state.wake.notify_one();
    Ok(task)
}

/// Completing a repeating task hands the series over to a new task on the next date (B-05):
/// same title, notes, tags, time, duration and relative reminders. The completed one stops
/// repeating, so reopening and completing it again cannot create a second copy.
fn spawn_next_occurrence(
    state: &AppState,
    tx: &rusqlite::Transaction<'_>,
    before: &Task,
    local_date: Option<&str>,
    now: &str,
) -> AppResult<()> {
    let (Some(text), Some(date)) = (&before.rrule, &before.due_date) else {
        return Ok(());
    };
    let rule = Rule::parse(text)?;
    let due = dates::parse_date(date)?;
    let floor = match local_date {
        Some(day) => dates::parse_date(day)?,
        None => dates::utc_to_local(&Local, state.clock.now()).date(),
    };
    tasks::set_rrule(tx, &before.id, None)?;
    let Some(next) = rule.next_after(due, floor) else {
        return Ok(());
    };
    let created = tasks::create(
        tx,
        &NewTask {
            id: state.ids.new_id(),
            title: before.title.clone(),
            content_text: content::text_of(&before.content_json)?,
            content_json: Some(before.content_json.clone()),
            due_date: Some(dates::format_date(next)),
            due_time: before.due_time.clone(),
            duration_minutes: before.duration_minutes,
            source_note_id: before.source_note_id.clone(),
            rrule: Some(rule.successor().format()),
            tag_ids: before.tag_ids.clone(),
            now: now.to_string(),
        },
    )?;
    let relative: Vec<NewReminder> = before
        .reminders
        .iter()
        .filter(|reminder| reminder.offset_minutes.is_some())
        .map(|reminder| NewReminder {
            id: state.ids.new_id(),
            notify_at: now.to_string(),
            offset_minutes: reminder.offset_minutes,
            repeat: None,
        })
        .collect();
    if !relative.is_empty() {
        reminders::replace_unsent(tx, &created.id, &relative, now)?;
        // The placeholders get their real time from the new date; those that do not fit go.
        reminders::recalculate(tx, &created.id, &Local)?;
    }
    Ok(())
}

/// Moves a task into POR HACER or onto a day, before `before_id` (a task of the destination
/// section) or last.
pub fn move_task(
    state: &AppState,
    id: &str,
    to_section: Section,
    date: Option<String>,
    before_id: Option<String>,
) -> AppResult<Task> {
    let target = match (to_section, date.as_deref()) {
        (Section::Todo, None) => None,
        (Section::Todo, Some(_)) => {
            return Err(AppError::Validation(
                "la sección «por hacer» no admite fecha".into(),
            ));
        }
        (Section::Agenda, Some(date)) => {
            dates::parse_date(date)?;
            Some(date)
        }
        (Section::Agenda, None) => {
            return Err(AppError::Validation(
                "mover a la agenda necesita una fecha".into(),
            ));
        }
    };
    let now = state.clock.now_string();

    let task = state.db.write(|tx| {
        if let Some(anchor_id) = before_id.as_deref() {
            let anchor = tasks::get(tx, anchor_id)?.ok_or_else(|| task_not_found(anchor_id))?;
            if anchor.due_date.as_deref() != target {
                return Err(AppError::Validation(
                    "la tarea de referencia no está en la sección de destino".into(),
                ));
            }
        }
        let moved = tasks::move_task(tx, id, target, before_id.as_deref(), &now)?
            .ok_or_else(|| task_not_found(id))?;
        if let Some(rule) = &moved.rrule {
            let anchored = target
                .map(|date| normalize_rule(rule, Some(date)))
                .transpose()?;
            tasks::set_rrule(tx, id, anchored.as_deref())?;
        }
        reminders::recalculate(tx, id, &Local)?;
        tasks::get(tx, id)?.ok_or_else(|| task_not_found(id))
    })?;
    state.wake.notify_one();
    Ok(task)
}

/// Soft delete: the task goes to the trash and can be restored for 30 days.
pub fn delete(state: &AppState, id: &str) -> AppResult<()> {
    let now = state.clock.now_string();
    state.db.write(|tx| {
        if tasks::soft_delete(tx, id, &now)? {
            Ok(())
        } else {
            Err(task_not_found(id))
        }
    })?;
    state.wake.notify_one();
    Ok(())
}

pub fn restore(state: &AppState, id: &str) -> AppResult<Task> {
    let now = state.clock.now_string();
    let task = state
        .db
        .write(|tx| tasks::restore(tx, id, &now)?.ok_or_else(|| task_not_found(id)))?;
    state.wake.notify_one();
    Ok(task)
}

pub fn trash(state: &AppState) -> AppResult<Vec<Task>> {
    state.db.read(|conn| Ok(tasks::list_trash(conn)?))
}

#[tauri::command]
pub async fn tasks_list(
    state: State<'_, Arc<AppState>>,
    view: ListView,
    from: Option<String>,
    to: Option<String>,
    tag_id: Option<String>,
) -> Result<TasksListOutput, String> {
    let input = TasksListInput {
        view,
        from,
        to,
        tag_id,
    };
    run_blocking(state.inner().clone(), move |state| list(state, input)).await
}

#[tauri::command]
pub async fn tasks_list_all(
    state: State<'_, Arc<AppState>>,
    tag_id: Option<String>,
) -> Result<Sections, String> {
    run_blocking(state.inner().clone(), move |state| {
        list_all(state, tag_id.as_deref())
    })
    .await
}

// The IPC contract passes each field as its own argument (SPEC §7.3).
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn task_create(
    state: State<'_, Arc<AppState>>,
    title: Option<String>,
    due_date: Option<String>,
    due_time: Option<String>,
    duration_minutes: Option<i64>,
    tag_ids: Option<Vec<String>>,
    content_json: Option<String>,
    source_note_id: Option<String>,
    rrule: Option<String>,
) -> Result<Task, String> {
    let input = TaskCreateInput {
        title,
        due_date,
        due_time,
        duration_minutes,
        tag_ids,
        content_json,
        source_note_id,
        rrule,
    };
    run_blocking(state.inner().clone(), move |state| create(state, input)).await
}

#[tauri::command]
pub async fn task_update(
    state: State<'_, Arc<AppState>>,
    id: String,
    patch: TaskPatchInput,
) -> Result<Task, String> {
    run_blocking(state.inner().clone(), move |state| {
        update(state, &id, patch)
    })
    .await
}

#[tauri::command]
pub async fn task_set_completed(
    state: State<'_, Arc<AppState>>,
    id: String,
    completed: bool,
    completed_date: Option<String>,
) -> Result<Task, String> {
    run_blocking(state.inner().clone(), move |state| {
        complete(state, &id, completed, completed_date.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn task_move(
    state: State<'_, Arc<AppState>>,
    id: String,
    to_section: Section,
    date: Option<String>,
    before_id: Option<String>,
) -> Result<Task, String> {
    run_blocking(state.inner().clone(), move |state| {
        move_task(state, &id, to_section, date, before_id)
    })
    .await
}

#[tauri::command]
pub async fn task_delete(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    run_blocking(state.inner().clone(), move |state| delete(state, &id)).await
}

#[tauri::command]
pub async fn task_restore(state: State<'_, Arc<AppState>>, id: String) -> Result<Task, String> {
    run_blocking(state.inner().clone(), move |state| restore(state, &id)).await
}

#[tauri::command]
pub async fn trash_list(state: State<'_, Arc<AppState>>) -> Result<Vec<Task>, String> {
    run_blocking(state.inner().clone(), trash).await
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::{repo::model::EMPTY_DOC, state::test_state};

    fn state() -> AppState {
        test_state().0
    }

    fn create_titled(state: &AppState, title: &str, date: Option<&str>) -> Task {
        create(
            state,
            TaskCreateInput {
                title: Some(title.to_string()),
                due_date: date.map(str::to_string),
                ..TaskCreateInput::default()
            },
        )
        .expect("create")
    }

    fn view(state: &AppState, view: ListView) -> TasksListOutput {
        list(
            state,
            TasksListInput {
                view,
                from: None,
                to: None,
                tag_id: None,
            },
        )
        .expect("list")
    }

    fn sections(output: TasksListOutput) -> Sections {
        match output {
            TasksListOutput::Sections(sections) => sections,
            TasksListOutput::Range(_) => panic!("expected sections"),
        }
    }

    fn titles(tasks: &[Task]) -> Vec<&str> {
        tasks.iter().map(|task| task.title.as_str()).collect()
    }

    #[test]
    fn list_all_returns_both_sections_like_the_two_separate_views() {
        let state = state();
        create_titled(&state, "Sin fecha", None);
        create_titled(&state, "Mañana", Some("2026-03-12"));
        create_titled(&state, "Antes", Some("2026-03-10"));
        let trashed = create_titled(&state, "Borrada", None);
        delete(&state, &trashed.id).expect("trash");

        let all = list_all(&state, None).expect("list all");
        let todo_view = sections(view(&state, ListView::Todo));
        let agenda_view = sections(view(&state, ListView::Agenda));
        assert_eq!(all.todo, todo_view.todo);
        assert_eq!(all.agenda, agenda_view.agenda);
        assert_eq!(titles(&all.todo), ["Sin fecha"]);
        assert_eq!(all.agenda.len(), 2);
    }

    #[test]
    fn list_all_filters_by_tag() {
        let state = state();
        create_titled(&state, "Una", None);
        let none = list_all(&state, Some("no-such-tag")).expect("list all");
        assert!(none.todo.is_empty() && none.agenda.is_empty());
    }

    fn patch(json: &str) -> TaskPatchInput {
        serde_json::from_str(json).expect("patch json")
    }

    fn code_of<T: std::fmt::Debug>(result: AppResult<T>) -> String {
        result.expect_err("must fail").to_string()
    }

    fn local_utc(date: &str, time: &str, offset: i64) -> String {
        dates::format_instant(
            crate::domain::reminders::offset_notify_at(&Local, date, time, offset).expect("at"),
        )
    }

    fn reminders_of(state: &AppState, id: &str) -> Vec<crate::repo::model::Reminder> {
        state
            .db
            .read(|conn| Ok(reminders::for_task(conn, id)?))
            .expect("reminders")
    }

    fn timed_task_with_reminder(state: &AppState) -> Task {
        use crate::commands::reminders::{ReminderInput, set};
        let task = create(
            state,
            TaskCreateInput {
                due_date: Some("2030-05-01".into()),
                due_time: Some("09:00".into()),
                ..TaskCreateInput::default()
            },
        )
        .expect("task");
        let relative = ReminderInput {
            offset_minutes: Some(15),
            notify_at: None,
            repeat: None,
        };
        set(state, &task.id, &[relative], &Local).expect("reminder");
        task
    }

    #[test]
    fn changing_the_time_of_a_task_reschedules_its_relative_reminders() {
        let state = state();
        let task = timed_task_with_reminder(&state);
        assert_eq!(
            reminders_of(&state, &task.id)[0].notify_at,
            local_utc("2030-05-01", "09:00", 15)
        );

        let updated =
            update(&state, &task.id, patch(r#"{"dueTime":"18:30"}"#)).expect("change time");
        assert_eq!(
            updated.reminders[0].notify_at,
            local_utc("2030-05-01", "18:30", 15)
        );

        let moved = update(&state, &task.id, patch(r#"{"dueDate":"2030-05-03"}"#)).expect("date");
        assert_eq!(
            moved.reminders[0].notify_at,
            local_utc("2030-05-03", "18:30", 15)
        );
    }

    #[test]
    fn dragging_a_task_to_another_day_reschedules_its_reminders() {
        let state = state();
        let task = timed_task_with_reminder(&state);
        let moved = move_task(
            &state,
            &task.id,
            Section::Agenda,
            Some("2030-06-10".into()),
            None,
        )
        .expect("move");
        assert_eq!(
            moved.reminders[0].notify_at,
            local_utc("2030-06-10", "09:00", 15)
        );
    }

    #[test]
    fn removing_the_time_removes_unsent_relative_reminders_but_not_custom_ones() {
        use crate::commands::reminders::{ReminderInput, set};
        let state = state();
        let task = timed_task_with_reminder(&state);
        let custom = ReminderInput {
            offset_minutes: None,
            notify_at: Some("2030-04-30T08:00:00Z".into()),
            repeat: None,
        };
        let relative = ReminderInput {
            offset_minutes: Some(15),
            notify_at: None,
            repeat: None,
        };
        set(&state, &task.id, &[relative, custom], &Local).expect("both");

        let updated = update(&state, &task.id, patch(r#"{"dueTime":null}"#)).expect("clear");

        assert_eq!(updated.reminders.len(), 1);
        assert_eq!(updated.reminders[0].offset_minutes, None);
    }

    #[test]
    fn sent_reminders_are_not_touched_when_the_time_changes() {
        let state = state();
        let task = timed_task_with_reminder(&state);
        let before = reminders_of(&state, &task.id)[0].clone();
        let now = state.clock.now_string();
        state
            .db
            .write(|tx| Ok(reminders::mark_sent(tx, &before.id, &now)?))
            .expect("sent");

        update(&state, &task.id, patch(r#"{"dueTime":"18:30"}"#)).expect("change time");

        let after = reminders_of(&state, &task.id);
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].notify_at, before.notify_at);
        assert!(after[0].is_sent);
    }

    #[test]
    fn create_uses_generated_id_and_clock_and_defaults() {
        let state = state();
        let task = create(&state, TaskCreateInput::default()).expect("create");

        assert_eq!(task.id, "id-1");
        assert_eq!(task.title, "");
        assert_eq!(task.content_json, EMPTY_DOC);
        assert_eq!(task.created_at, "2026-03-10T10:00:00Z");
        assert_eq!(task.position, 0);
    }

    #[test]
    fn create_validates_every_field() {
        let state = state();
        let cases = [
            (
                TaskCreateInput {
                    title: Some("x".repeat(501)),
                    ..TaskCreateInput::default()
                },
                "VALIDATION: el título no puede superar 500 caracteres (tiene 501)",
            ),
            (
                TaskCreateInput {
                    due_date: Some("2026-02-30".into()),
                    ..TaskCreateInput::default()
                },
                "VALIDATION: fecha no válida «2026-02-30»: se espera AAAA-MM-DD",
            ),
            (
                TaskCreateInput {
                    due_time: Some("09:00".into()),
                    ..TaskCreateInput::default()
                },
                "VALIDATION: una hora requiere una fecha",
            ),
            (
                TaskCreateInput {
                    content_json: Some("not json".into()),
                    ..TaskCreateInput::default()
                },
                "VALIDATION: las notas no tienen un formato válido",
            ),
            (
                TaskCreateInput {
                    tag_ids: Some(vec!["missing".into()]),
                    ..TaskCreateInput::default()
                },
                "NOT_FOUND: el elemento referenciado no existe",
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(code_of(create(&state, input)), expected);
        }
        assert!(sections(view(&state, ListView::Todo)).todo.is_empty());
    }

    #[test]
    fn create_derives_content_text_for_search() {
        let state = state();
        let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"apuntes de álgebra"}]}]}"#;
        let task = create(
            &state,
            TaskCreateInput {
                content_json: Some(json.to_string()),
                ..TaskCreateInput::default()
            },
        )
        .expect("create");

        let indexed: i64 = state
            .db
            .read(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM tasks_fts WHERE tasks_fts MATCH 'algebra' AND task_id = ?1",
                    [&task.id],
                    |row| row.get(0),
                )?)
            })
            .expect("fts");
        assert_eq!(indexed, 1);
    }

    #[test]
    fn todo_and_agenda_views_are_ordered_by_the_domain_rules() {
        let state = state();
        create_titled(&state, "todo 1", None);
        let done = create_titled(&state, "todo 2", None);
        create_titled(&state, "todo 3", None);
        set_completed(&state, &done.id, true).expect("complete");
        create_titled(&state, "old day", Some("2026-03-01"));
        create_titled(&state, "new day", Some("2026-03-20"));

        let todo = sections(view(&state, ListView::Todo));
        assert_eq!(titles(&todo.todo), ["todo 1", "todo 3", "todo 2"]);
        assert!(todo.agenda.is_empty());

        let agenda = sections(view(&state, ListView::Agenda));
        let days: Vec<(&str, Vec<&str>)> = agenda
            .agenda
            .iter()
            .map(|day| (day.date.as_str(), titles(&day.tasks)))
            .collect();
        assert_eq!(
            days,
            [
                ("2026-03-20", vec!["new day"]),
                ("2026-03-01", vec!["old day"])
            ]
        );
        assert!(agenda.todo.is_empty());
    }

    #[test]
    fn range_view_is_flat_ascending_and_validated() {
        let state = state();
        create_titled(&state, "outside", Some("2026-04-02"));
        create_titled(&state, "late", Some("2026-03-20"));
        create_titled(&state, "early", Some("2026-03-02"));

        let request = |from: Option<&str>, to: Option<&str>| TasksListInput {
            view: ListView::Range,
            from: from.map(str::to_string),
            to: to.map(str::to_string),
            tag_id: None,
        };

        let TasksListOutput::Range(found) =
            list(&state, request(Some("2026-03-01"), Some("2026-03-31"))).expect("range")
        else {
            panic!("expected a flat list");
        };
        assert_eq!(titles(&found), ["early", "late"]);

        for (input, expected) in [
            (
                request(None, Some("2026-03-31")),
                "VALIDATION: la vista de rango necesita las fechas «from» y «to»",
            ),
            (
                request(Some("2026-03-31"), Some("2026-03-01")),
                "VALIDATION: la fecha inicial del rango es posterior a la final",
            ),
            (
                request(Some("ayer"), Some("2026-03-01")),
                "VALIDATION: fecha no válida «ayer»: se espera AAAA-MM-DD",
            ),
        ] {
            assert_eq!(code_of(list(&state, input)), expected);
        }
    }

    #[test]
    fn list_filters_by_tag() {
        let state = state();
        let tag = crate::commands::tags::upsert(
            &state,
            crate::commands::tags::TagUpsertInput {
                id: None,
                name: "Física".into(),
                color_hex: "#112233".into(),
                parent_id: None,
            },
        )
        .expect("tag");
        create(
            &state,
            TaskCreateInput {
                title: Some("tagged".into()),
                tag_ids: Some(vec![tag.id.clone()]),
                ..TaskCreateInput::default()
            },
        )
        .expect("tagged");
        create_titled(&state, "plain", None);

        let found = list(
            &state,
            TasksListInput {
                view: ListView::Todo,
                from: None,
                to: None,
                tag_id: Some(tag.id),
            },
        )
        .expect("list");

        assert_eq!(titles(&sections(found).todo), ["tagged"]);
    }

    #[test]
    fn update_changes_only_the_given_fields_and_stamps_the_clock() {
        let (state, clock) = test_state();
        let task = create_titled(&state, "antes", Some("2026-03-12"));
        clock.advance(Duration::hours(2));

        let updated = update(&state, &task.id, patch(r#"{"title":"después"}"#)).expect("update");

        assert_eq!(updated.title, "después");
        assert_eq!(updated.due_date.as_deref(), Some("2026-03-12"));
        assert_eq!(updated.updated_at, "2026-03-10T12:00:00Z");
        assert_eq!(updated.created_at, "2026-03-10T10:00:00Z");
    }

    #[test]
    fn update_distinguishes_absent_fields_from_null() {
        let state = state();
        let task = create(
            &state,
            TaskCreateInput {
                title: Some("t".into()),
                due_date: Some("2026-03-12".into()),
                due_time: Some("08:30".into()),
                ..TaskCreateInput::default()
            },
        )
        .expect("create");

        let only_time = update(&state, &task.id, patch(r#"{"dueTime":null}"#)).expect("clear time");
        assert_eq!(only_time.due_date.as_deref(), Some("2026-03-12"));
        assert_eq!(only_time.due_time, None);

        let retimed = update(&state, &task.id, patch(r#"{"dueTime":"17:45"}"#)).expect("set time");
        assert_eq!(retimed.due_time.as_deref(), Some("17:45"));
        assert_eq!(retimed.due_date.as_deref(), Some("2026-03-12"));
    }

    #[test]
    fn a_duration_is_stored_kept_cleared_and_validated() {
        let state = state();
        let task = create(
            &state,
            TaskCreateInput {
                due_date: Some("2026-03-12".into()),
                due_time: Some("09:00".into()),
                duration_minutes: Some(45),
                ..TaskCreateInput::default()
            },
        )
        .expect("create");
        assert_eq!(task.duration_minutes, Some(45));

        let moved = update(&state, &task.id, patch(r#"{"dueTime":"10:00"}"#)).expect("retime");
        assert_eq!(moved.duration_minutes, Some(45));

        let longer = update(&state, &task.id, patch(r#"{"durationMinutes":90}"#)).expect("longer");
        assert_eq!(longer.duration_minutes, Some(90));

        assert_eq!(
            code_of(update(&state, &task.id, patch(r#"{"dueTime":"23:00"}"#))),
            "VALIDATION: la tarea tiene que terminar antes de medianoche"
        );
        assert_eq!(
            update(&state, &task.id, patch(r#"{}"#))
                .expect("noop")
                .duration_minutes,
            Some(90)
        );

        let cleared =
            update(&state, &task.id, patch(r#"{"durationMinutes":null}"#)).expect("clear");
        assert_eq!(cleared.duration_minutes, None);

        update(&state, &task.id, patch(r#"{"durationMinutes":30}"#)).expect("again");
        let untimed = update(&state, &task.id, patch(r#"{"dueTime":null}"#)).expect("no time");
        assert_eq!((untimed.due_time, untimed.duration_minutes), (None, None));
    }

    #[test]
    fn a_duration_without_a_time_is_rejected_on_create() {
        let state = state();
        let result = create(
            &state,
            TaskCreateInput {
                due_date: Some("2026-03-12".into()),
                duration_minutes: Some(30),
                ..TaskCreateInput::default()
            },
        );
        assert_eq!(
            code_of(result),
            "VALIDATION: una duración requiere una hora"
        );
    }

    #[test]
    fn clearing_the_date_clears_the_time_too() {
        let state = state();
        let task = create(
            &state,
            TaskCreateInput {
                due_date: Some("2026-03-12".into()),
                due_time: Some("08:30".into()),
                ..TaskCreateInput::default()
            },
        )
        .expect("create");

        let cleared = update(&state, &task.id, patch(r#"{"dueDate":null}"#)).expect("clear date");

        assert_eq!((cleared.due_date, cleared.due_time), (None, None));
    }

    #[test]
    fn update_rejects_invalid_results_without_changing_the_task() {
        let state = state();
        let undated = create_titled(&state, "sin fecha", None);
        let dated = create_titled(&state, "con fecha", Some("2026-03-12"));

        assert_eq!(
            code_of(update(&state, &undated.id, patch(r#"{"dueTime":"09:00"}"#))),
            "VALIDATION: una hora requiere una fecha"
        );
        assert_eq!(
            code_of(update(
                &state,
                &dated.id,
                patch(r#"{"dueDate":null,"dueTime":"09:00"}"#)
            )),
            "VALIDATION: una hora requiere una fecha"
        );
        assert_eq!(
            code_of(update(
                &state,
                &dated.id,
                patch(r#"{"dueDate":"2026-13-01"}"#)
            )),
            "VALIDATION: fecha no válida «2026-13-01»: se espera AAAA-MM-DD"
        );
        let long = format!(r#"{{"title":"{}"}}"#, "x".repeat(501));
        assert!(code_of(update(&state, &dated.id, patch(&long))).starts_with("VALIDATION: "));
        assert!(
            code_of(update(&state, &dated.id, patch(r#"{"contentJson":"[]"}"#)))
                .starts_with("VALIDATION: ")
        );

        let unchanged = sections(view(&state, ListView::Agenda));
        assert_eq!(
            unchanged.agenda[0].tasks[0].due_date.as_deref(),
            Some("2026-03-12")
        );
    }

    #[test]
    fn update_with_an_empty_patch_changes_nothing() {
        let (state, clock) = test_state();
        let task = create_titled(&state, "igual", None);
        clock.advance(Duration::hours(1));

        let same = update(&state, &task.id, patch("{}")).expect("update");

        assert_eq!(same, task);
    }

    #[test]
    fn update_reindexes_title_and_notes() {
        let state = state();
        let task = create_titled(&state, "borrador", None);
        let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"cálculo diferencial"}]}]}"#;
        let body = serde_json::json!({ "title": "informe", "contentJson": json });

        update(&state, &task.id, patch(&body.to_string())).expect("update");

        let hits = |term: &str| -> i64 {
            state
                .db
                .read(|conn| {
                    Ok(conn.query_row(
                        "SELECT COUNT(*) FROM tasks_fts WHERE tasks_fts MATCH ?1",
                        [term],
                        |row| row.get(0),
                    )?)
                })
                .expect("fts")
        };
        assert_eq!(
            (hits("informe"), hits("calculo"), hits("borrador")),
            (1, 1, 0)
        );
    }

    #[test]
    fn commands_report_missing_tasks() {
        let state = state();
        let missing = "NOT_FOUND: tarea nope no encontrada";

        assert_eq!(code_of(update(&state, "nope", patch("{}"))), missing);
        assert_eq!(code_of(set_completed(&state, "nope", true)), missing);
        assert_eq!(code_of(delete(&state, "nope")), missing);
        assert_eq!(code_of(restore(&state, "nope")), missing);
        assert_eq!(
            code_of(move_task(&state, "nope", Section::Todo, None, None)),
            missing
        );
    }

    #[test]
    fn completing_a_task_moves_it_to_the_completion_day() {
        let state = state();
        let undated = create_titled(&state, "sin fecha", None);
        let dated = create_titled(&state, "con fecha", Some("2026-03-12"));

        let done = complete(&state, &undated.id, true, Some("2026-03-20")).expect("complete");
        assert_eq!(done.due_date.as_deref(), Some("2026-03-20"));
        assert!(done.is_completed);

        let moved = complete(&state, &dated.id, true, Some("2026-03-20")).expect("complete");
        assert_eq!(moved.due_date.as_deref(), Some("2026-03-20"));

        let reopened = complete(&state, &undated.id, false, Some("2026-03-25")).expect("reopen");
        assert_eq!(reopened.due_date.as_deref(), Some("2026-03-20"));
        assert!(!reopened.is_completed);
    }

    #[test]
    fn completing_without_a_local_date_keeps_the_task_undated() {
        let state = state();
        let task = create_titled(&state, "t", None);
        let done = set_completed(&state, &task.id, true).expect("complete");
        assert_eq!(done.due_date, None);
    }

    #[test]
    fn completing_rejects_a_malformed_local_date() {
        let state = state();
        let task = create_titled(&state, "t", None);
        assert!(
            code_of(complete(&state, &task.id, true, Some("20-03-2026")))
                .starts_with("VALIDATION:")
        );
        assert!(
            !state
                .db
                .read(|c| Ok(tasks::get(c, &task.id)?))
                .expect("get")
                .expect("task")
                .is_completed
        );
    }

    #[test]
    fn complete_and_reopen_use_the_clock() {
        let (state, clock) = test_state();
        let task = create_titled(&state, "t", None);
        clock.advance(Duration::minutes(5));

        let done = set_completed(&state, &task.id, true).expect("complete");
        assert!(done.is_completed);
        assert_eq!(done.completed_at.as_deref(), Some("2026-03-10T10:05:00Z"));

        let reopened = set_completed(&state, &task.id, false).expect("reopen");
        assert!(!reopened.is_completed);
        assert_eq!(reopened.completed_at, None);
    }

    #[test]
    fn delete_trash_and_restore_round_trip() {
        let state = state();
        create_titled(&state, "keep", None);
        let gone = create_titled(&state, "gone", None);

        delete(&state, &gone.id).expect("delete");

        assert_eq!(
            titles(&sections(view(&state, ListView::Todo)).todo),
            ["keep"]
        );
        assert_eq!(titles(&trash(&state).expect("trash")), ["gone"]);
        assert_eq!(
            code_of(delete(&state, &gone.id)),
            format!("NOT_FOUND: tarea {} no encontrada", gone.id)
        );

        let restored = restore(&state, &gone.id).expect("restore");
        assert_eq!(restored.id, gone.id);
        assert!(trash(&state).expect("trash").is_empty());
        assert_eq!(
            titles(&sections(view(&state, ListView::Todo)).todo),
            ["keep", "gone"]
        );
    }

    #[test]
    fn move_reorders_within_todo_and_between_sections() {
        let state = state();
        let a = create_titled(&state, "a", None);
        let b = create_titled(&state, "b", None);
        let c = create_titled(&state, "c", None);

        move_task(&state, &c.id, Section::Todo, None, Some(a.id.clone())).expect("reorder");
        assert_eq!(
            titles(&sections(view(&state, ListView::Todo)).todo),
            ["c", "a", "b"]
        );

        let dated = move_task(
            &state,
            &a.id,
            Section::Agenda,
            Some("2026-03-15".into()),
            None,
        )
        .expect("to agenda");
        assert_eq!(dated.due_date.as_deref(), Some("2026-03-15"));
        assert_eq!(
            titles(&sections(view(&state, ListView::Todo)).todo),
            ["c", "b"]
        );

        let back = move_task(&state, &a.id, Section::Todo, None, Some(b.id)).expect("to todo");
        assert_eq!(back.due_date, None);
        assert_eq!(
            titles(&sections(view(&state, ListView::Todo)).todo),
            ["c", "a", "b"]
        );
    }

    #[test]
    fn move_validates_destination_and_anchor() {
        let state = state();
        let todo = create_titled(&state, "todo", None);
        let dated = create_titled(&state, "dated", Some("2026-03-15"));

        let cases = [
            (
                move_task(&state, &todo.id, Section::Agenda, None, None),
                "VALIDATION: mover a la agenda necesita una fecha".to_string(),
            ),
            (
                move_task(
                    &state,
                    &todo.id,
                    Section::Todo,
                    Some("2026-03-15".into()),
                    None,
                ),
                "VALIDATION: la sección «por hacer» no admite fecha".to_string(),
            ),
            (
                move_task(&state, &todo.id, Section::Agenda, Some("hoy".into()), None),
                "VALIDATION: fecha no válida «hoy»: se espera AAAA-MM-DD".to_string(),
            ),
            (
                move_task(
                    &state,
                    &todo.id,
                    Section::Todo,
                    None,
                    Some(dated.id.clone()),
                ),
                "VALIDATION: la tarea de referencia no está en la sección de destino".to_string(),
            ),
            (
                move_task(&state, &todo.id, Section::Todo, None, Some("nope".into())),
                "NOT_FOUND: tarea nope no encontrada".to_string(),
            ),
        ];
        for (result, expected) in cases {
            assert_eq!(code_of(result), expected);
        }
        assert_eq!(
            titles(&sections(view(&state, ListView::Todo)).todo),
            ["todo"]
        );
    }

    #[test]
    fn inputs_deserialize_from_the_frontend_shape() {
        let request: ListView = serde_json::from_str("\"agenda\"").expect("view");
        assert_eq!(request, ListView::Agenda);
        let section: Section = serde_json::from_str("\"todo\"").expect("section");
        assert_eq!(section, Section::Todo);
        assert!(serde_json::from_str::<ListView>("\"week\"").is_err());

        let full =
            patch(r#"{"title":"t","contentJson":"{}","dueDate":"2026-03-12","dueTime":null}"#);
        assert_eq!(full.title.as_deref(), Some("t"));
        assert_eq!(full.due_date, Some(Some("2026-03-12".to_string())));
        assert_eq!(full.due_time, Some(None));
        let empty = patch("{}");
        assert_eq!((empty.due_date, empty.due_time), (None, None));
    }

    #[test]
    fn list_output_serializes_as_documented() {
        let state = state();
        create_titled(&state, "t", Some("2026-03-12"));

        let agenda = serde_json::to_value(view(&state, ListView::Agenda)).expect("json");
        assert_eq!(agenda["todo"], serde_json::json!([]));
        assert_eq!(agenda["agenda"][0]["date"], "2026-03-12");
        assert_eq!(agenda["agenda"][0]["tasks"][0]["title"], "t");

        let range = serde_json::to_value(TasksListOutput::Range(vec![])).expect("json");
        assert_eq!(range, serde_json::json!([]));
    }
    fn repeating(state: &AppState, date: &str, rule: &str) -> Task {
        create(
            state,
            TaskCreateInput {
                title: Some("Regar".into()),
                due_date: Some(date.into()),
                due_time: Some("09:00".into()),
                rrule: Some(rule.into()),
                ..TaskCreateInput::default()
            },
        )
        .expect("repeating")
    }

    fn open_tasks(state: &AppState) -> Vec<Task> {
        let mut found: Vec<Task> = state
            .db
            .read(|conn| Ok(tasks::list(conn, tasks::Query::Agenda, None)?))
            .expect("list")
            .into_iter()
            .filter(|task| !task.is_completed)
            .collect();
        found.sort_by(|a, b| a.due_date.cmp(&b.due_date));
        found
    }

    fn open_dates(state: &AppState) -> Vec<String> {
        open_tasks(state)
            .into_iter()
            .filter_map(|task| task.due_date)
            .collect()
    }

    #[test]
    fn a_rule_is_validated_and_anchored_to_the_date() {
        let (state, _) = test_state();
        let made = repeating(&state, "2026-01-31", "FREQ=MONTHLY");
        assert_eq!(
            made.rrule.as_deref(),
            Some("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=31")
        );

        let code = |r: AppResult<Task>| r.expect_err("fails").to_string();
        let undated = TaskCreateInput {
            rrule: Some("FREQ=DAILY".into()),
            ..TaskCreateInput::default()
        };
        assert!(code(create(&state, undated)).starts_with("VALIDATION"));
        let bad = TaskCreateInput {
            due_date: Some("2026-03-10".into()),
            rrule: Some("FREQ=SOMETIMES".into()),
            ..TaskCreateInput::default()
        };
        assert!(code(create(&state, bad)).starts_with("VALIDATION"));
    }

    #[test]
    fn completing_a_repeating_task_creates_the_next_one_and_hands_over_the_series() {
        let (state, _) = test_state();
        let tag = crate::commands::tags::upsert(
            &state,
            crate::commands::tags::TagUpsertInput {
                id: None,
                name: "Casa".into(),
                color_hex: "#112233".into(),
                parent_id: None,
            },
        )
        .expect("tag");
        let first = create(
            &state,
            TaskCreateInput {
                title: Some("Regar".into()),
                due_date: Some("2026-03-10".into()),
                due_time: Some("09:00".into()),
                duration_minutes: Some(30),
                tag_ids: Some(vec![tag.id.clone()]),
                rrule: Some("FREQ=WEEKLY".into()),
                ..TaskCreateInput::default()
            },
        )
        .expect("first");

        let done = complete(&state, &first.id, true, Some("2026-03-10")).expect("complete");
        assert!(done.is_completed);
        assert_eq!(done.rrule, None, "the series moves on");

        let open = open_tasks(&state);
        assert_eq!(open.len(), 1);
        let next = &open[0];
        assert_eq!(next.due_date.as_deref(), Some("2026-03-17"));
        assert_eq!(next.title, "Regar");
        assert_eq!(next.due_time.as_deref(), Some("09:00"));
        assert_eq!(next.duration_minutes, Some(30));
        assert_eq!(next.tag_ids, [tag.id]);
        assert_eq!(next.rrule.as_deref(), first.rrule.as_deref());
        assert_ne!(next.id, first.id);
    }

    #[test]
    fn a_counted_weekday_series_stops_after_its_last_occurrence() {
        let (state, _) = test_state();
        // 2026-03-09 is a Monday.
        let first = repeating(&state, "2026-03-09", "FREQ=WEEKLY;BYDAY=MO,WE;COUNT=3");
        complete(&state, &first.id, true, Some("2026-03-09")).expect("first");
        let second = open_tasks(&state).remove(0);
        assert_eq!(second.due_date.as_deref(), Some("2026-03-11"));
        assert_eq!(
            second.rrule.as_deref(),
            Some("FREQ=WEEKLY;INTERVAL=1;BYDAY=MO,WE;COUNT=2")
        );
        complete(&state, &second.id, true, Some("2026-03-11")).expect("second");
        let third = open_tasks(&state).remove(0);
        assert_eq!(third.due_date.as_deref(), Some("2026-03-16"));
        complete(&state, &third.id, true, Some("2026-03-16")).expect("third");
        assert!(open_tasks(&state).is_empty(), "the series is over");
    }

    #[test]
    fn reopening_and_completing_again_does_not_duplicate_and_late_completion_skips_ahead() {
        let (state, _) = test_state();
        let first = repeating(&state, "2026-03-10", "FREQ=DAILY");
        complete(&state, &first.id, true, Some("2026-03-10")).expect("done");
        complete(&state, &first.id, false, None).expect("reopen");
        complete(&state, &first.id, true, Some("2026-03-10")).expect("done again");
        let after_reopen = open_dates(&state);
        assert_eq!(after_reopen, ["2026-03-11"], "one next task, no copy");

        let (state, _) = test_state();
        let late = repeating(&state, "2026-03-01", "FREQ=WEEKLY");
        complete(&state, &late.id, true, Some("2026-03-10")).expect("late");
        assert_eq!(
            open_dates(&state),
            ["2026-03-15"],
            "still on its weekly cycle"
        );
    }

    #[test]
    fn the_series_ends_at_until_and_a_plain_task_creates_nothing() {
        let (state, _) = test_state();
        let last = repeating(&state, "2026-03-10", "FREQ=DAILY;UNTIL=20260310");
        complete(&state, &last.id, true, Some("2026-03-10")).expect("done");
        assert!(open_dates(&state).is_empty());

        let plain = create(
            &state,
            TaskCreateInput {
                title: Some("Suelta".into()),
                due_date: Some("2026-03-10".into()),
                ..TaskCreateInput::default()
            },
        )
        .expect("plain");
        complete(&state, &plain.id, true, Some("2026-03-10")).expect("done");
        assert!(open_dates(&state).is_empty());
    }

    #[test]
    fn relative_reminders_follow_the_series() {
        let (state, _) = test_state();
        let first = repeating(&state, "2026-03-20", "FREQ=WEEKLY");
        crate::commands::reminders::set(
            &state,
            &first.id,
            &[crate::commands::reminders::ReminderInput {
                offset_minutes: Some(60),
                notify_at: None,
                repeat: None,
            }],
            &chrono::Local,
        )
        .expect("reminder");
        complete(&state, &first.id, true, Some("2026-03-20")).expect("done");
        let open = open_tasks(&state);
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].due_date.as_deref(), Some("2026-03-27"));
        assert_eq!(open[0].reminders.len(), 1);
        assert_eq!(open[0].reminders[0].offset_minutes, Some(60));
        assert!(!open[0].reminders[0].is_sent);
    }

    #[test]
    fn changing_or_clearing_the_date_keeps_the_rule_coherent() {
        let (state, _) = test_state();
        let task = repeating(&state, "2026-01-31", "FREQ=MONTHLY");
        let moved = update(
            &state,
            &task.id,
            TaskPatchInput {
                due_date: Some(Some("2026-02-05".into())),
                ..TaskPatchInput::default()
            },
        )
        .expect("moved");
        assert_eq!(
            moved.rrule.as_deref(),
            Some("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=5")
        );

        let off = update(
            &state,
            &task.id,
            TaskPatchInput {
                rrule: Some(None),
                ..TaskPatchInput::default()
            },
        )
        .expect("off");
        assert_eq!(off.rrule, None);

        let again = update(
            &state,
            &task.id,
            TaskPatchInput {
                rrule: Some(Some("FREQ=DAILY".into())),
                ..TaskPatchInput::default()
            },
        )
        .expect("again");
        assert!(again.rrule.is_some());
        let undated = update(
            &state,
            &task.id,
            TaskPatchInput {
                due_date: Some(None),
                ..TaskPatchInput::default()
            },
        )
        .expect("undated");
        assert_eq!(undated.rrule, None, "no date, no repetition");
    }

    #[test]
    fn dragging_a_repeating_task_re_anchors_it_and_the_to_do_list_ends_it() {
        let (state, _) = test_state();
        let task = repeating(&state, "2026-01-31", "FREQ=MONTHLY");
        let moved = move_task(
            &state,
            &task.id,
            Section::Agenda,
            Some("2026-02-09".into()),
            None,
        )
        .expect("moved");
        assert_eq!(
            moved.rrule.as_deref(),
            Some("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=9")
        );
        let parked = move_task(&state, &task.id, Section::Todo, None, None).expect("parked");
        assert_eq!(parked.rrule, None);
    }
}

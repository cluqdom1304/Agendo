//! Task commands (SPEC §7.3). Each command is a thin async wrapper over a plain function that
//! validates the input, calls the repository inside one transaction and maps errors to
//! `CODE: message` strings; the plain functions are what the tests exercise.

use std::sync::Arc;

use chrono::Local;
use serde::{Deserialize, Deserializer, Serialize};
use tauri::State;

use crate::{
    commands::page_of,
    domain::{
        content, dates,
        ordering::{self, DayGroup},
        recurrence::Rule,
        validation,
    },
    error::{AppError, AppResult},
    repo::{
        model::{NewTask, Page, Task, TaskPatch},
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

#[cfg(test)]
pub fn trash(state: &AppState) -> AppResult<Vec<Task>> {
    trash_page(state, Page::ALL)
}

pub fn trash_page(state: &AppState, page: Page) -> AppResult<Vec<Task>> {
    state
        .db
        .read(|conn| Ok(tasks::list_trash_page(conn, page)?))
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
pub async fn trash_list(
    state: State<'_, Arc<AppState>>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<Task>, String> {
    run_blocking(state.inner().clone(), move |state| {
        trash_page(state, page_of(limit, offset)?)
    })
    .await
}

#[cfg(test)]
mod tests;

//! Tag use cases and `set_task_tags` (SPEC §7.3). The Tauri commands in `commands/tags.rs` only wrap these functions.

use crate::{
    domain::validation::{self, ParentTag},
    error::{AppError, AppResult},
    repo::{
        model::{Tag, TagInput, TagWithCount, Task},
        tags, tasks,
    },
    state::AppState,
};

#[derive(Debug, Clone)]
pub struct TagUpsertInput {
    /// `None` creates a tag; `Some` renames/recolours/re-parents an existing one.
    pub id: Option<String>,
    pub name: String,
    pub color_hex: String,
    pub parent_id: Option<String>,
}

fn tag_not_found(id: &str) -> AppError {
    AppError::NotFound(format!("etiqueta {id} no encontrada"))
}

pub fn list(state: &AppState) -> AppResult<Vec<TagWithCount>> {
    state.db.read(|conn| Ok(tags::list(conn)?))
}

pub fn upsert(state: &AppState, input: TagUpsertInput) -> AppResult<Tag> {
    let name = validation::validate_tag_name(&input.name)?.to_string();
    validation::validate_color_hex(&input.color_hex)?;
    let now = state.clock.now_string();

    state.db.write(|tx| {
        let (id, is_update) = match &input.id {
            Some(id) => {
                tags::get(tx, id)?.ok_or_else(|| tag_not_found(id))?;
                (id.clone(), true)
            }
            None => (state.ids.new_id(), false),
        };
        let parent = match &input.parent_id {
            Some(parent_id) => Some(tags::get(tx, parent_id)?.ok_or_else(|| {
                AppError::NotFound(format!("etiqueta padre {parent_id} no encontrada"))
            })?),
            None => None,
        };
        let has_children = is_update && tags::has_children(tx, &id)?;
        validation::validate_tag_parent(
            &id,
            parent.as_ref().map(|parent| ParentTag {
                id: &parent.id,
                parent_id: parent.parent_id.as_deref(),
            }),
            has_children,
        )?;

        let tag = TagInput {
            id,
            name,
            color_hex: input.color_hex.clone(),
            parent_id: input.parent_id.clone(),
            now,
        };
        tags::upsert(tx, &tag).map_err(|error| match AppError::from(error) {
            AppError::Conflict(_) => {
                AppError::Conflict("ya existe una etiqueta con ese nombre".into())
            }
            other => other,
        })
    })
}

/// Deletes the tag: its tasks lose the link and its child tags become top-level.
pub fn delete(state: &AppState, id: &str) -> AppResult<()> {
    state.db.write(|tx| {
        if tags::delete(tx, id)? {
            Ok(())
        } else {
            Err(tag_not_found(id))
        }
    })
}

/// Folds `source` into `target`: everything tagged with the first gets the second.
pub fn merge(state: &AppState, source: &str, target: &str) -> AppResult<()> {
    if source == target {
        return Err(AppError::Validation(
            "no se puede fusionar una etiqueta consigo misma".into(),
        ));
    }
    state.db.write(|tx| {
        if tags::merge(tx, source, target)? {
            Ok(())
        } else {
            Err(AppError::NotFound("etiqueta no encontrada".into()))
        }
    })
}

/// Replaces the tags of a task with exactly `tag_ids`.
pub fn set_task_tags(state: &AppState, task_id: &str, tag_ids: &[String]) -> AppResult<Task> {
    let now = state.clock.now_string();
    state.db.write(|tx| {
        let task_missing = || AppError::NotFound(format!("tarea {task_id} no encontrada"));
        if !tags::set_task_tags(tx, task_id, tag_ids, &now)? {
            return Err(task_missing());
        }
        tasks::get(tx, task_id)?.ok_or_else(task_missing)
    })
}

#[cfg(test)]
mod tests;

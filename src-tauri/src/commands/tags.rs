//! Tag commands and `task_tags_set` (SPEC §7.3). Same shape as `tasks.rs`: thin async wrappers
//! over plain, tested functions.

use std::sync::Arc;

use tauri::State;

use crate::{
    domain::validation::{self, ParentTag},
    error::{AppError, AppResult},
    repo::{
        model::{Tag, TagInput, TagWithCount, Task},
        tags, tasks,
    },
    state::{AppState, run_blocking},
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

#[tauri::command]
pub async fn tag_list(state: State<'_, Arc<AppState>>) -> Result<Vec<TagWithCount>, String> {
    run_blocking(state.inner().clone(), list).await
}

#[tauri::command]
pub async fn tag_upsert(
    state: State<'_, Arc<AppState>>,
    id: Option<String>,
    name: String,
    color_hex: String,
    parent_id: Option<String>,
) -> Result<Tag, String> {
    let input = TagUpsertInput {
        id,
        name,
        color_hex,
        parent_id,
    };
    run_blocking(state.inner().clone(), move |state| upsert(state, input)).await
}

#[tauri::command]
pub async fn tag_delete(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    run_blocking(state.inner().clone(), move |state| delete(state, &id)).await
}

#[tauri::command]
pub async fn task_tags_set(
    state: State<'_, Arc<AppState>>,
    task_id: String,
    tag_ids: Vec<String>,
) -> Result<Task, String> {
    run_blocking(state.inner().clone(), move |state| {
        set_task_tags(state, &task_id, &tag_ids)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        commands::tasks::{TaskCreateInput, create},
        state::test_state,
    };

    fn input(name: &str, parent: Option<&str>) -> TagUpsertInput {
        TagUpsertInput {
            id: None,
            name: name.to_string(),
            color_hex: "#0A84FF".to_string(),
            parent_id: parent.map(str::to_string),
        }
    }

    fn make(state: &AppState, name: &str, parent: Option<&str>) -> Tag {
        upsert(state, input(name, parent)).expect("upsert")
    }

    fn code_of<T: std::fmt::Debug>(result: AppResult<T>) -> String {
        result.expect_err("must fail").to_string()
    }

    fn new_task(state: &AppState, tag_ids: Vec<String>) -> Task {
        create(
            state,
            TaskCreateInput {
                tag_ids: Some(tag_ids),
                ..TaskCreateInput::default()
            },
        )
        .expect("task")
    }

    #[test]
    fn upsert_creates_with_generated_id_and_trimmed_name() {
        let (state, _) = test_state();
        let tag = make(&state, "  Matemáticas  ", None);

        assert_eq!(tag.id, "id-1");
        assert_eq!(tag.name, "Matemáticas");
        assert_eq!(tag.color_hex, "#0A84FF");
        assert_eq!(tag.position, 0);
        assert_eq!(make(&state, "Física", None).position, 1);
    }

    #[test]
    fn upsert_updates_an_existing_tag_in_place() {
        let (state, _) = test_state();
        let tag = make(&state, "Física", None);

        let renamed = upsert(
            &state,
            TagUpsertInput {
                id: Some(tag.id.clone()),
                name: "Física II".into(),
                color_hex: "#ff0000".into(),
                parent_id: None,
            },
        )
        .expect("rename");

        assert_eq!(renamed.id, tag.id);
        assert_eq!(renamed.name, "Física II");
        assert_eq!(renamed.color_hex, "#ff0000");
        assert_eq!(list(&state).expect("list").len(), 1);
    }

    #[test]
    fn upsert_validates_name_color_and_ids() {
        let (state, _) = test_state();
        let existing = make(&state, "Grado", None);
        let with = |change: &dyn Fn(&mut TagUpsertInput)| {
            let mut value = input("Nueva", None);
            change(&mut value);
            upsert(&state, value)
        };

        assert_eq!(
            code_of(with(&|i| i.name = "   ".into())),
            "VALIDATION: el nombre de la etiqueta no puede estar vacío"
        );
        assert_eq!(
            code_of(with(&|i| i.color_hex = "azul".into())),
            "VALIDATION: el color debe tener el formato #RRGGBB"
        );
        assert_eq!(
            code_of(with(&|i| i.name = "GRADO".into())),
            "CONFLICT: ya existe una etiqueta con ese nombre"
        );
        assert_eq!(
            code_of(with(&|i| i.id = Some("nope".into()))),
            "NOT_FOUND: etiqueta nope no encontrada"
        );
        assert_eq!(
            code_of(with(&|i| i.parent_id = Some("nope".into()))),
            "NOT_FOUND: etiqueta padre nope no encontrada"
        );
        assert_eq!(list(&state).expect("list").len(), 1);
        assert_eq!(existing.name, "Grado");
    }

    #[test]
    fn hierarchy_is_limited_to_one_level() {
        let (state, _) = test_state();
        let grade = make(&state, "Grado", None);
        let subject = make(&state, "Álgebra", Some(&grade.id));
        assert_eq!(subject.parent_id.as_deref(), Some(grade.id.as_str()));

        let grandchild = upsert(&state, input("Tema 1", Some(&subject.id)));
        assert!(code_of(grandchild).starts_with("VALIDATION: solo se permite un nivel"));

        let other = make(&state, "Otro", None);
        let adopt_parent = upsert(
            &state,
            TagUpsertInput {
                id: Some(grade.id.clone()),
                name: "Grado".into(),
                color_hex: "#112233".into(),
                parent_id: Some(other.id.clone()),
            },
        );
        assert_eq!(
            code_of(adopt_parent),
            "VALIDATION: una etiqueta con subetiquetas no puede pasar a ser subetiqueta"
        );

        let own_parent = upsert(
            &state,
            TagUpsertInput {
                id: Some(other.id.clone()),
                name: "Otro".into(),
                color_hex: "#112233".into(),
                parent_id: Some(other.id),
            },
        );
        assert_eq!(
            code_of(own_parent),
            "VALIDATION: una etiqueta no puede ser su propia etiqueta padre"
        );
    }

    #[test]
    fn a_child_can_be_detached_by_clearing_its_parent() {
        let (state, _) = test_state();
        let grade = make(&state, "Grado", None);
        let subject = make(&state, "Álgebra", Some(&grade.id));

        let detached = upsert(
            &state,
            TagUpsertInput {
                id: Some(subject.id),
                name: "Álgebra".into(),
                color_hex: "#0A84FF".into(),
                parent_id: None,
            },
        )
        .expect("detach");

        assert_eq!(detached.parent_id, None);
    }

    #[test]
    fn list_counts_pending_tasks_per_tag() {
        let (state, _) = test_state();
        let tag = make(&state, "Física", None);
        let empty = make(&state, "Vacía", None);
        new_task(&state, vec![tag.id.clone()]);
        let done = new_task(&state, vec![tag.id.clone()]);
        crate::commands::tasks::set_completed(&state, &done.id, true).expect("complete");

        let tags = list(&state).expect("list");

        let counts: Vec<(&str, i64)> = tags
            .iter()
            .map(|t| (t.tag.id.as_str(), t.pending_count))
            .collect();
        assert_eq!(counts, [(tag.id.as_str(), 1), (empty.id.as_str(), 0)]);
    }

    #[test]
    fn delete_unlinks_tasks_and_reports_missing_tags() {
        let (state, _) = test_state();
        let tag = make(&state, "Física", None);
        let task = new_task(&state, vec![tag.id.clone()]);

        delete(&state, &tag.id).expect("delete");

        let after = set_task_tags(&state, &task.id, &[]).expect("task");
        assert!(after.tag_ids.is_empty());
        assert_eq!(
            code_of(delete(&state, &tag.id)),
            format!("NOT_FOUND: etiqueta {} no encontrada", tag.id)
        );
    }

    #[test]
    fn set_task_tags_replaces_the_set_and_returns_the_task() {
        let (state, clock) = test_state();
        let a = make(&state, "A", None);
        let b = make(&state, "B", None);
        let c = make(&state, "C", None);
        let task = new_task(&state, vec![a.id.clone()]);
        clock.advance(chrono::Duration::minutes(1));

        let updated =
            set_task_tags(&state, &task.id, &[c.id.clone(), b.id.clone()]).expect("replace");

        let mut expected = vec![b.id, c.id];
        expected.sort();
        assert_eq!(updated.tag_ids, expected);
        assert_eq!(updated.updated_at, "2026-03-10T10:01:00Z");
    }

    #[test]
    fn set_task_tags_rejects_unknown_tasks_and_tags() {
        let (state, _) = test_state();
        let tag = make(&state, "A", None);
        let task = new_task(&state, vec![tag.id.clone()]);

        assert_eq!(
            code_of(set_task_tags(&state, "nope", std::slice::from_ref(&tag.id))),
            "NOT_FOUND: tarea nope no encontrada"
        );
        assert_eq!(
            code_of(set_task_tags(&state, &task.id, &["nope".to_string()])),
            "NOT_FOUND: el elemento referenciado no existe"
        );
        let unchanged =
            set_task_tags(&state, &task.id, std::slice::from_ref(&tag.id)).expect("same");
        assert_eq!(unchanged.tag_ids, [tag.id]);
    }
}

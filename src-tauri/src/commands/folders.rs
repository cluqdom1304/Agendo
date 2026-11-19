//! Folder commands (D-43). Same shape as `tags.rs`: thin async wrappers over plain, tested
//! functions that validate, run one transaction and map errors to `CODE: message`.

use std::sync::Arc;

use tauri::State;

use crate::{
    domain::validation::{self, FolderParent},
    error::{AppError, AppResult},
    repo::{
        folders,
        model::{Folder, FolderInput, FolderWithCount},
    },
    state::{AppState, run_blocking},
};

#[derive(Debug, Clone)]
pub struct FolderUpsertInput {
    /// `None` creates a folder; `Some` renames or moves an existing one.
    pub id: Option<String>,
    pub name: String,
    /// `None` is the top level.
    pub parent_id: Option<String>,
}

fn folder_not_found(id: &str) -> AppError {
    AppError::not_found("carpeta", id)
}

pub fn list(state: &AppState) -> AppResult<Vec<FolderWithCount>> {
    state.db.read(|conn| Ok(folders::list(conn)?))
}

pub fn upsert(state: &AppState, input: FolderUpsertInput) -> AppResult<Folder> {
    let name = validation::validate_folder_name(&input.name)?.to_string();
    let now = state.clock.now_string();

    state.db.write(|tx| {
        let (id, subtree_height) = match &input.id {
            Some(id) => {
                folders::get(tx, id)?.ok_or_else(|| folder_not_found(id))?;
                (id.clone(), folders::height(tx, id)?)
            }
            None => (state.ids.new_id(), 1),
        };
        let parent = match &input.parent_id {
            Some(parent_id) => {
                let depth = folders::depth(tx, parent_id)?.ok_or_else(|| {
                    AppError::NotFound(format!("carpeta padre {parent_id} no encontrada"))
                })?;
                let inside_moved_subtree =
                    input.id.is_some() && folders::is_in_subtree(tx, &id, parent_id)?;
                Some(FolderParent {
                    depth,
                    inside_moved_subtree,
                })
            }
            None => None,
        };
        validation::validate_folder_placement(parent, subtree_height)?;

        let folder = FolderInput {
            id,
            name,
            parent_id: input.parent_id.clone(),
            now,
        };
        folders::upsert(tx, &folder).map_err(|error| match AppError::from(error) {
            AppError::Conflict(_) => {
                AppError::Conflict("ya existe una carpeta con ese nombre en este lugar".into())
            }
            other => other,
        })
    })
}

/// Deletes the folder and its subfolders; their notes go to the trash. Returns how many notes
/// were trashed.
pub fn delete(state: &AppState, id: &str) -> AppResult<usize> {
    let now = state.clock.now_string();
    state
        .db
        .write(|tx| folders::delete_tree(tx, id, &now)?.ok_or_else(|| folder_not_found(id)))
}

#[tauri::command]
pub async fn folder_list(state: State<'_, Arc<AppState>>) -> Result<Vec<FolderWithCount>, String> {
    run_blocking(state.inner().clone(), list).await
}

#[tauri::command]
pub async fn folder_upsert(
    state: State<'_, Arc<AppState>>,
    id: Option<String>,
    name: String,
    parent_id: Option<String>,
) -> Result<Folder, String> {
    let input = FolderUpsertInput {
        id,
        name,
        parent_id,
    };
    run_blocking(state.inner().clone(), move |state| upsert(state, input)).await
}

#[tauri::command]
pub async fn folder_delete(state: State<'_, Arc<AppState>>, id: String) -> Result<usize, String> {
    run_blocking(state.inner().clone(), move |state| delete(state, &id)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        services::notes::{self, NoteCreateInput},
        state::test_state,
    };

    fn input(name: &str, parent: Option<&str>) -> FolderUpsertInput {
        FolderUpsertInput {
            id: None,
            name: name.to_string(),
            parent_id: parent.map(str::to_string),
        }
    }

    fn make(state: &AppState, name: &str, parent: Option<&str>) -> Folder {
        upsert(state, input(name, parent)).expect("upsert")
    }

    fn code_of<T: std::fmt::Debug>(result: AppResult<T>) -> String {
        result.expect_err("must fail").to_string()
    }

    fn move_to(state: &AppState, id: &str, name: &str, parent: Option<&str>) -> AppResult<Folder> {
        upsert(
            state,
            FolderUpsertInput {
                id: Some(id.to_string()),
                name: name.to_string(),
                parent_id: parent.map(str::to_string),
            },
        )
    }

    #[test]
    fn upsert_creates_with_a_generated_id_and_a_trimmed_name() {
        let (state, _) = test_state();
        let folder = make(&state, "  Trabajo  ", None);
        assert_eq!(folder.id, "id-1");
        assert_eq!(folder.name, "Trabajo");
        assert_eq!(folder.parent_id, None);
        assert_eq!(make(&state, "Casa", None).position, 1);
    }

    #[test]
    fn upsert_rejects_bad_names_and_unknown_parents_and_folders() {
        let (state, _) = test_state();
        assert_eq!(
            code_of(upsert(&state, input("   ", None))),
            "VALIDATION: el nombre de la carpeta no puede estar vacío"
        );
        assert_eq!(
            code_of(upsert(&state, input("A", Some("ghost")))),
            "NOT_FOUND: carpeta padre ghost no encontrada"
        );
        assert_eq!(
            code_of(move_to(&state, "ghost", "A", None)),
            "NOT_FOUND: carpeta ghost no encontrada"
        );
    }

    #[test]
    fn a_sibling_with_the_same_name_is_a_conflict_but_another_parent_is_fine() {
        let (state, _) = test_state();
        let a = make(&state, "Trabajo", None);
        assert_eq!(
            code_of(upsert(&state, input("trabajo", None))),
            "CONFLICT: ya existe una carpeta con ese nombre en este lugar"
        );
        make(&state, "Trabajo", Some(&a.id));
    }

    #[test]
    fn the_tree_is_limited_to_five_levels() {
        let (state, _) = test_state();
        let mut parent: Option<String> = None;
        for level in 1..=5 {
            let folder = make(&state, &format!("Nivel {level}"), parent.as_deref());
            parent = Some(folder.id);
        }
        assert_eq!(
            code_of(upsert(&state, input("Nivel 6", parent.as_deref()))),
            "VALIDATION: las carpetas admiten como máximo 5 niveles"
        );
    }

    #[test]
    fn moving_a_subtree_checks_its_height_and_forbids_cycles() {
        let (state, _) = test_state();
        let a = make(&state, "A", None);
        let b = make(&state, "B", Some(&a.id));
        let c = make(&state, "C", Some(&b.id));
        let deep = (0..3).fold(None::<String>, |parent, level| {
            Some(make(&state, &format!("D{level}"), parent.as_deref()).id)
        });
        let deep_leaf = deep.expect("chain");

        assert_eq!(
            code_of(move_to(&state, &a.id, "A", Some(&c.id))),
            "VALIDATION: una carpeta no puede moverse dentro de sí misma ni de sus subcarpetas"
        );
        assert_eq!(
            code_of(move_to(&state, &a.id, "A", Some(&a.id))),
            "VALIDATION: una carpeta no puede moverse dentro de sí misma ni de sus subcarpetas"
        );
        // A has height 3 and the leaf of the D chain is at depth 3: 3 + 3 > 5.
        assert_eq!(
            code_of(move_to(&state, &a.id, "A", Some(&deep_leaf))),
            "VALIDATION: las carpetas admiten como máximo 5 niveles"
        );
        // C is a leaf (height 1): it fits under the leaf of the D chain (3 + 1 <= 5).
        let moved = move_to(&state, &c.id, "C", Some(&deep_leaf)).expect("move");
        assert_eq!(moved.parent_id.as_deref(), Some(deep_leaf.as_str()));
    }

    #[test]
    fn list_reports_the_live_note_count_of_each_folder() {
        let (state, _) = test_state();
        let folder = make(&state, "Trabajo", None);
        for _ in 0..2 {
            notes::create(
                &state,
                NoteCreateInput {
                    folder_id: Some(folder.id.clone()),
                    ..NoteCreateInput::default()
                },
            )
            .expect("note");
        }
        let listed = list(&state).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].note_count, 2);
    }

    #[test]
    fn delete_removes_the_subtree_and_trashes_its_notes() {
        let (state, _) = test_state();
        let a = make(&state, "A", None);
        let b = make(&state, "B", Some(&a.id));
        for folder in [&a, &b] {
            notes::create(
                &state,
                NoteCreateInput {
                    folder_id: Some(folder.id.clone()),
                    ..NoteCreateInput::default()
                },
            )
            .expect("note");
        }

        assert_eq!(delete(&state, &a.id).expect("delete"), 2);
        assert!(list(&state).expect("list").is_empty());
        assert_eq!(notes::trash(&state).expect("trash").len(), 2);
        assert_eq!(
            code_of(delete(&state, &a.id)),
            format!("NOT_FOUND: carpeta {} no encontrada", a.id)
        );
    }
}

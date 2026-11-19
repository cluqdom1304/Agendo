//! Note templates (F13-02). A new note from a template is the ordinary `note_create` with the
//! template's document, so no command is needed for it.

use std::sync::Arc;

use tauri::State;

use crate::{
    domain::{content, validation},
    error::{AppError, AppResult},
    repo::{model::NoteTemplate, templates},
    state::{AppState, run_blocking},
};

fn not_found(id: &str) -> AppError {
    AppError::not_found("plantilla", id)
}

fn name_taken(error: rusqlite::Error) -> AppError {
    match AppError::from(error) {
        AppError::Conflict(_) => {
            AppError::Conflict("ya existe una plantilla con ese nombre".into())
        }
        other => other,
    }
}

pub fn list(state: &AppState) -> AppResult<Vec<NoteTemplate>> {
    state.db.read(|conn| Ok(templates::list(conn)?))
}

/// Creates a template (`id` absent, document required) or renames / rewrites one.
pub fn save(
    state: &AppState,
    id: Option<&str>,
    name: &str,
    content_json: Option<&str>,
) -> AppResult<NoteTemplate> {
    let name = validation::validate_template_name(name)?;
    if let Some(json) = content_json {
        content::text_of(json)?;
    }
    let now = state.clock.now_string();
    match id {
        Some(id) => state.db.write(|tx| {
            templates::update(tx, id, name, content_json, &now)
                .map_err(name_taken)?
                .ok_or_else(|| not_found(id))
        }),
        None => {
            let json = content_json.ok_or_else(|| {
                AppError::Validation("una plantilla nueva necesita un documento".into())
            })?;
            let new_id = state.ids.new_id();
            state
                .db
                .write(|tx| templates::insert(tx, &new_id, name, json, &now).map_err(name_taken))
        }
    }
}

pub fn delete(state: &AppState, id: &str) -> AppResult<()> {
    state.db.write(|tx| {
        if templates::delete(tx, id)? {
            Ok(())
        } else {
            Err(not_found(id))
        }
    })
}

#[tauri::command]
pub async fn template_list(state: State<'_, Arc<AppState>>) -> Result<Vec<NoteTemplate>, String> {
    run_blocking(state.inner().clone(), list).await
}

#[tauri::command]
pub async fn template_save(
    state: State<'_, Arc<AppState>>,
    id: Option<String>,
    name: String,
    content_json: Option<String>,
) -> Result<NoteTemplate, String> {
    run_blocking(state.inner().clone(), move |state| {
        save(state, id.as_deref(), &name, content_json.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn template_delete(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    run_blocking(state.inner().clone(), move |state| delete(state, &id)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{repo::model::EMPTY_DOC, state::test_state};

    const DOC: &str = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Orden del día"}]}]}"#;

    fn code_of<T: std::fmt::Debug>(result: AppResult<T>) -> String {
        result.expect_err("must fail").to_string()
    }

    #[test]
    fn templates_are_created_listed_in_name_order_and_deleted() {
        let (state, _) = test_state();
        save(&state, None, "Reunión", Some(EMPTY_DOC)).expect("a");
        let b = save(&state, None, "  Acta ", Some(EMPTY_DOC)).expect("b");
        assert_eq!(b.name, "Acta");
        let names: Vec<_> = list(&state)
            .expect("list")
            .into_iter()
            .map(|t| t.name)
            .collect();
        assert_eq!(names, ["Acta", "Reunión"]);
        delete(&state, &b.id).expect("delete");
        assert_eq!(list(&state).expect("list").len(), 1);
        assert!(code_of(delete(&state, &b.id)).starts_with("NOT_FOUND"));
    }

    #[test]
    fn rename_keeps_the_document_and_a_new_document_replaces_it() {
        let (state, clock) = test_state();
        let made = save(&state, None, "Reunión", Some(DOC)).expect("made");
        clock.advance(chrono::Duration::minutes(5));
        let renamed = save(&state, Some(&made.id), "Reunión semanal", None).expect("rename");
        assert_eq!(renamed.content_json, DOC);
        assert_ne!(renamed.updated_at, made.updated_at);
        let rewritten =
            save(&state, Some(&made.id), "Reunión semanal", Some(EMPTY_DOC)).expect("doc");
        assert_eq!(rewritten.content_json, EMPTY_DOC);
    }

    #[test]
    fn invalid_input_is_rejected_and_names_are_unique_ignoring_case() {
        let (state, _) = test_state();
        save(&state, None, "Reunión", Some(EMPTY_DOC)).expect("first");
        assert!(code_of(save(&state, None, "reunión", Some(EMPTY_DOC))).starts_with("CONFLICT"));
        assert!(code_of(save(&state, None, "  ", Some(EMPTY_DOC))).starts_with("VALIDATION"));
        assert!(code_of(save(&state, None, "Nueva", None)).starts_with("VALIDATION"));
        assert!(code_of(save(&state, None, "Rota", Some("no es json"))).starts_with("VALIDATION"));
        assert!(code_of(save(&state, Some("no-existe"), "X", None)).starts_with("NOT_FOUND"));
    }
}

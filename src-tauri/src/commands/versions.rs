//! Version history of a note (F13-03, D-48). Snapshots are taken by `notes::update`; these
//! commands read them and restore one.

use std::sync::Arc;

use tauri::State;

use crate::{
    error::{AppError, AppResult},
    repo::{
        model::{Note, NoteVersion, NoteVersionSummary},
        versions,
    },
    services::notes::{self, NotePatchInput},
    state::{AppState, run_blocking},
};

pub fn list(state: &AppState, note_id: &str) -> AppResult<Vec<NoteVersionSummary>> {
    state.db.read(|conn| Ok(versions::list(conn, note_id)?))
}

pub fn get(state: &AppState, id: &str) -> AppResult<NoteVersion> {
    state.db.read(|conn| {
        versions::get(conn, id)?
            .ok_or_else(|| AppError::NotFound(format!("versión {id} no encontrada")))
    })
}

/// Puts the document and title of a version back in its note. The document being replaced is
/// kept as a new version, so restoring is never destructive.
pub fn restore(state: &AppState, id: &str) -> AppResult<Note> {
    let version = get(state, id)?;
    notes::update_with(
        state,
        &version.note_id,
        NotePatchInput {
            title: Some(version.title),
            content_json: Some(version.content_json),
        },
        true,
    )
}

#[tauri::command]
pub async fn note_versions_list(
    state: State<'_, Arc<AppState>>,
    note_id: String,
) -> Result<Vec<NoteVersionSummary>, String> {
    run_blocking(state.inner().clone(), move |state| list(state, &note_id)).await
}

#[tauri::command]
pub async fn note_version_get(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<NoteVersion, String> {
    run_blocking(state.inner().clone(), move |state| get(state, &id)).await
}

#[tauri::command]
pub async fn note_version_restore(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Note, String> {
    run_blocking(state.inner().clone(), move |state| restore(state, &id)).await
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::{
        repo::versions::{MAX_VERSIONS, SNAPSHOT_EVERY_MINUTES},
        services::notes::{self as notes_cmd, NoteCreateInput},
        state::test_state,
    };

    fn doc(text: &str) -> String {
        format!(
            r#"{{"type":"doc","content":[{{"type":"paragraph","content":[{{"type":"text","text":"{text}"}}]}}]}}"#
        )
    }

    fn save(state: &AppState, id: &str, text: &str) {
        notes_cmd::update(
            state,
            id,
            NotePatchInput {
                title: None,
                content_json: Some(doc(text)),
            },
        )
        .expect("save");
    }

    fn make(state: &AppState) -> Note {
        notes_cmd::create(
            state,
            NoteCreateInput {
                title: Some("Apuntes".into()),
                ..NoteCreateInput::default()
            },
        )
        .expect("note")
    }

    #[test]
    fn the_first_edit_of_an_empty_note_leaves_no_version() {
        let (state, _) = test_state();
        let note = make(&state);
        save(&state, &note.id, "uno");
        assert!(list(&state, &note.id).expect("list").is_empty());
    }

    #[test]
    fn saves_close_together_make_one_version_and_later_ones_make_another() {
        let (state, clock) = test_state();
        let note = make(&state);
        save(&state, &note.id, "uno");
        clock.advance(Duration::minutes(1));
        save(&state, &note.id, "dos"); // snapshots "uno"
        clock.advance(Duration::minutes(1));
        save(&state, &note.id, "tres"); // too soon
        assert_eq!(list(&state, &note.id).expect("list").len(), 1);

        clock.advance(Duration::minutes(SNAPSHOT_EVERY_MINUTES + 1));
        save(&state, &note.id, "cuatro"); // snapshots "tres"
        let found = list(&state, &note.id).expect("list");
        assert_eq!(found.len(), 2);
        assert!(found[0].created_at > found[1].created_at);
        assert!(
            get(&state, &found[0].id)
                .expect("v")
                .content_json
                .contains("tres")
        );
        assert!(
            get(&state, &found[1].id)
                .expect("v")
                .content_json
                .contains("uno")
        );
    }

    #[test]
    fn saving_the_same_document_or_only_the_title_takes_no_snapshot() {
        let (state, clock) = test_state();
        let note = make(&state);
        save(&state, &note.id, "uno");
        clock.advance(Duration::minutes(20));
        save(&state, &note.id, "uno");
        notes_cmd::update(
            &state,
            &note.id,
            NotePatchInput {
                title: Some("Otro".into()),
                content_json: None,
            },
        )
        .expect("title");
        assert!(list(&state, &note.id).expect("list").is_empty());
    }

    #[test]
    fn only_the_newest_versions_are_kept() {
        let (state, clock) = test_state();
        let note = make(&state);
        for n in 0..(MAX_VERSIONS + 5) {
            clock.advance(Duration::minutes(SNAPSHOT_EVERY_MINUTES + 1));
            save(&state, &note.id, &format!("texto {n}"));
        }
        let found = list(&state, &note.id).expect("list");
        assert_eq!(found.len() as i64, MAX_VERSIONS);
        let oldest = get(&state, &found[found.len() - 1].id).expect("oldest");
        assert!(!oldest.content_json.contains("texto 0\""));
    }

    #[test]
    fn restoring_brings_back_the_document_and_keeps_the_replaced_one() {
        let (state, clock) = test_state();
        let note = make(&state);
        save(&state, &note.id, "uno");
        clock.advance(Duration::minutes(1));
        save(&state, &note.id, "dos");
        let first = list(&state, &note.id).expect("list")[0].id.clone();

        // Right after a save: `force` must snapshot "dos" even though one is recent.
        let restored = restore(&state, &first).expect("restore");
        assert!(restored.content_json.contains("uno"));
        let found = list(&state, &note.id).expect("list");
        assert_eq!(found.len(), 2);
        assert!(
            get(&state, &found[0].id)
                .expect("v")
                .content_json
                .contains("dos")
        );
        assert_eq!(
            notes_cmd::get(&state, &note.id).expect("get").content_json,
            restored.content_json
        );
    }

    #[test]
    fn versions_go_with_the_note_and_unknown_ones_are_not_found() {
        let (state, clock) = test_state();
        let note = make(&state);
        save(&state, &note.id, "uno");
        clock.advance(Duration::minutes(1));
        save(&state, &note.id, "dos");
        let id = list(&state, &note.id).expect("list")[0].id.clone();
        notes_cmd::delete(&state, &note.id).expect("trash");
        clock.advance(Duration::days(31));
        state.purge_trash().expect("purge");
        assert!(
            get(&state, &id)
                .expect_err("gone")
                .to_string()
                .starts_with("NOT_FOUND")
        );
        assert!(
            restore(&state, "nope")
                .expect_err("nope")
                .to_string()
                .starts_with("NOT_FOUND")
        );
    }
}

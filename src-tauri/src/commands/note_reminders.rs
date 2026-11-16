//! Reminders of notes (F13-04, D-52): one time to be told about a note. They fire through the
//! same scheduler as the reminders of tasks.

use std::sync::Arc;

use tauri::State;

use crate::{
    domain::dates,
    error::{AppError, AppResult},
    repo::{model::NoteReminder, note_reminders, notes},
    state::{AppState, run_blocking},
};

pub fn get(state: &AppState, note_id: &str) -> AppResult<Option<NoteReminder>> {
    state
        .db
        .read(|conn| Ok(note_reminders::pending(conn, note_id)?))
}

/// Sets the reminder of a live note at `notify_at` (UTC), or removes it with `None`.
pub fn set(
    state: &AppState,
    note_id: &str,
    notify_at: Option<&str>,
) -> AppResult<Option<NoteReminder>> {
    let instant = notify_at.map(dates::parse_instant).transpose()?;
    if instant.is_some_and(|at| at <= state.clock.now()) {
        return Err(AppError::Validation(
            "ese recordatorio ya habría pasado".into(),
        ));
    }
    let now = state.clock.now_string();
    let id = state.ids.new_id();
    let reminder = state.db.write(|tx| {
        if notes::get(tx, note_id)?.is_none() {
            return Err(AppError::NotFound(format!("nota {note_id} no encontrada")));
        }
        match instant {
            Some(at) => {
                note_reminders::replace_unsent(tx, &id, note_id, &dates::format_instant(at), &now)?
            }
            None => note_reminders::clear_unsent(tx, note_id)?,
        }
        Ok(note_reminders::pending(tx, note_id)?)
    })?;
    state.wake.notify_one();
    Ok(reminder)
}

#[tauri::command]
pub async fn note_reminder_get(
    state: State<'_, Arc<AppState>>,
    note_id: String,
) -> Result<Option<NoteReminder>, String> {
    run_blocking(state.inner().clone(), move |state| get(state, &note_id)).await
}

#[tauri::command]
pub async fn note_reminder_set(
    state: State<'_, Arc<AppState>>,
    note_id: String,
    notify_at: Option<String>,
) -> Result<Option<NoteReminder>, String> {
    run_blocking(state.inner().clone(), move |state| {
        set(state, &note_id, notify_at.as_deref())
    })
    .await
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::Duration;

    use super::*;
    use crate::{
        commands::notes::{self as notes_cmd, NoteCreateInput, NotePatchInput},
        error::AppError,
        notify::{Fired, FiredSink, Notification, Notifier},
        repo::reminders,
        scheduler::Engine,
        state::test_state,
    };

    #[derive(Default)]
    struct Shown(Mutex<Vec<Notification>>);

    impl Notifier for Shown {
        fn show(&self, notification: &Notification) -> Result<(), AppError> {
            self.0.lock().expect("lock").push(notification.clone());
            Ok(())
        }
    }

    #[derive(Default)]
    struct Events(Mutex<Vec<Fired>>);

    impl FiredSink for Events {
        fn fired(&self, fired: &Fired) {
            self.0.lock().expect("lock").push(fired.clone());
        }
    }

    fn make(state: &AppState, title: &str, text: &str) -> String {
        let note = notes_cmd::create(
            state,
            NoteCreateInput {
                title: Some(title.into()),
                ..NoteCreateInput::default()
            },
        )
        .expect("note");
        let doc = format!(
            r#"{{"type":"doc","content":[{{"type":"paragraph","content":[{{"type":"text","text":"{text}"}}]}}]}}"#
        );
        notes_cmd::update(
            state,
            &note.id,
            NotePatchInput {
                title: None,
                content_json: Some(doc),
            },
        )
        .expect("text");
        note.id
    }

    #[test]
    fn a_note_has_one_pending_reminder_that_can_be_replaced_and_cleared() {
        let (state, _) = test_state();
        let id = make(&state, "Reunión", "agenda");
        assert!(get(&state, &id).expect("none").is_none());

        let first = set(&state, &id, Some("2026-03-10T12:00:00Z"))
            .expect("first")
            .expect("some");
        assert_eq!(first.notify_at, "2026-03-10T12:00:00Z");
        let second = set(&state, &id, Some("2026-03-11T08:00:00Z"))
            .expect("second")
            .expect("some");
        assert_eq!(second.notify_at, "2026-03-11T08:00:00Z");
        assert_eq!(get(&state, &id).expect("get").expect("some").id, second.id);
        assert!(set(&state, &id, None).expect("clear").is_none());
    }

    #[test]
    fn past_times_bad_instants_and_unknown_notes_are_rejected() {
        let (state, _) = test_state();
        let id = make(&state, "Reunión", "agenda");
        let code = |r: AppResult<Option<NoteReminder>>| r.expect_err("fails").to_string();
        assert!(code(set(&state, &id, Some("2026-03-10T10:00:00Z"))).starts_with("VALIDATION"));
        assert!(code(set(&state, &id, Some("mañana"))).starts_with("VALIDATION"));
        assert!(code(set(&state, "nope", Some("2026-03-11T08:00:00Z"))).starts_with("NOT_FOUND"));
    }

    #[test]
    fn the_scheduler_notifies_a_note_once_with_its_title_and_text() {
        let (state, clock) = test_state();
        let (shown, events) = (Shown::default(), Events::default());
        let id = make(&state, "Reunión", "preparar la agenda");
        let reminder = set(&state, &id, Some("2026-03-10T10:30:00Z"))
            .expect("set")
            .expect("some");
        let mut engine = Engine::default();

        clock.advance(Duration::minutes(29));
        assert!(
            engine
                .tick(&state, &shown, &events)
                .expect("tick")
                .fired
                .is_empty()
        );
        clock.advance(Duration::minutes(1));
        let report = engine.tick(&state, &shown, &events).expect("tick");
        assert_eq!(report.fired, std::slice::from_ref(&reminder.id));
        {
            let shown = shown.0.lock().expect("lock");
            assert_eq!(shown[0].title, "Reunión");
            assert!(shown[0].body.contains("preparar la agenda"));
        }
        assert_eq!(
            *events.0.lock().expect("lock"),
            [Fired {
                task_id: None,
                note_id: Some(id.clone()),
                reminder_id: reminder.id
            }]
        );

        clock.advance(Duration::hours(1));
        assert!(
            engine
                .tick(&state, &shown, &events)
                .expect("tick")
                .fired
                .is_empty()
        );
        assert!(get(&state, &id).expect("get").is_none());
    }

    #[test]
    fn a_trashed_note_does_not_notify_and_a_stale_reminder_expires_silently() {
        let (state, clock) = test_state();
        let (shown, events) = (Shown::default(), Events::default());
        let trashed = make(&state, "Borrada", "x");
        set(&state, &trashed, Some("2026-03-10T10:30:00Z")).expect("set");
        notes_cmd::delete(&state, &trashed).expect("trash");
        let stale = make(&state, "Vieja", "y");
        set(&state, &stale, Some("2026-03-10T10:05:00Z")).expect("set");
        let mut engine = Engine::default();

        clock.advance(Duration::hours(30));
        let report = engine.tick(&state, &shown, &events).expect("tick");
        assert!(report.fired.is_empty());
        assert_eq!(report.expired.len(), 2);
        assert!(shown.0.lock().expect("lock").is_empty());
        assert!(get(&state, &stale).expect("get").is_none());
    }

    #[test]
    fn the_next_wake_up_counts_the_reminders_of_notes() {
        let (state, _) = test_state();
        let id = make(&state, "Reunión", "agenda");
        set(&state, &id, Some("2026-03-10T10:20:00Z")).expect("set");
        let next = state
            .db
            .read(|conn| Ok(reminders::next_after(conn, "2026-03-10T10:00:00Z")?))
            .expect("next");
        assert_eq!(next.as_deref(), Some("2026-03-10T10:20:00Z"));
    }

    #[test]
    fn deleting_the_note_for_good_removes_its_reminders() {
        let (state, clock) = test_state();
        let id = make(&state, "Reunión", "agenda");
        set(&state, &id, Some("2026-03-10T10:20:00Z")).expect("set");
        notes_cmd::delete(&state, &id).expect("trash");
        clock.advance(Duration::days(31));
        state.purge_trash().expect("purge");
        let left: i64 = state
            .db
            .read(|conn| {
                Ok(conn.query_row("SELECT COUNT(*) FROM note_reminders", [], |r| r.get(0))?)
            })
            .expect("count");
        assert_eq!(left, 0);
    }
}

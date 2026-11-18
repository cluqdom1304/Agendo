//! `settings_get` / `settings_set` (SPEC §7.3). Only whitelisted keys, with validated values.

use std::sync::Arc;

use chrono::Local;
use tauri::State;

use crate::{
    autobackup,
    domain::dates,
    error::{AppError, AppResult},
    repo::{reminders, settings},
    state::{AppState, run_blocking},
};

/// Which view the main area shows: the list or one of the calendar views (F-10).
pub const VIEW_MODE: &str = "view_mode";
/// Colour theme: follow the system or force light or dark (F6-05).
pub const THEME: &str = "theme";
/// Which section is open: the agenda, the notes or «Hoy» (V2 F8-07, F13-07).
pub const APP_SECTION: &str = "app_section";
/// What the notes list shows: `all`, `inbox`, `pinned` or `folder:<id>` (F8-07).
pub const NOTES_SCOPE: &str = "notes_scope";
/// Id of the note open in the editor, or empty (F8-07).
pub const NOTES_OPEN: &str = "notes_open";

/// Ids of the notes opened last, most recent first, separated by commas (F11-05).
pub const NOTES_RECENT: &str = "notes_recent";

/// Saved searches of the palette: a JSON list of query texts (F17-08).
pub const SAVED_SEARCHES: &str = "saved_searches";

/// Most saved searches, and the longest one.
const MAX_SAVED_SEARCHES: usize = 20;
const MAX_SEARCH_CHARS: usize = 200;

/// Longest free-form value (an id plus a prefix).
const MAX_FREE_VALUE: usize = 64;
/// Longest list of recent notes: a dozen ids and their separators.
const MAX_RECENT_VALUE: usize = 480;

/// Native window material requested at startup (F6-05); the OS may still fall back.
pub const WINDOW_EFFECT: &str = "window_effect";

/// Hour (`HH:MM`) at which reminders of tasks without a time sound (D-35).
pub const DEFAULT_NOTIFY_TIME: &str = settings::NOTIFY_TIME_KEY;

/// Whitelist: every setting with the values it accepts.
const ALLOWED: [(&str, &[&str]); 5] = [
    (APP_SECTION, &["agenda", "notes", "today"]),
    (VIEW_MODE, &["list", "day", "week", "month"]),
    (THEME, &["system", "light", "dark"]),
    (WINDOW_EFFECT, &["mica", "acrylic", "none"]),
    (autobackup::MODE_KEY, &autobackup::MODES),
];

fn allowed_values(key: &str) -> AppResult<&'static [&'static str]> {
    if key == DEFAULT_NOTIFY_TIME
        || key == SAVED_SEARCHES
        || key == autobackup::KEEP_KEY
        || key == autobackup::LAST_KEY
        || is_free_form(key)
    {
        return Ok(&[]);
    }
    ALLOWED
        .iter()
        .find(|(name, _)| *name == key)
        .map(|(_, values)| *values)
        .ok_or_else(|| AppError::Validation(format!("el ajuste «{key}» no existe")))
}

/// Settings that hold an id-like string the backend does not enumerate.
fn is_free_form(key: &str) -> bool {
    key == NOTES_SCOPE || key == NOTES_OPEN || key == NOTES_RECENT
}

fn saved_searches_are_valid(value: &str) -> bool {
    serde_json::from_str::<Vec<String>>(value).is_ok_and(|list| {
        list.len() <= MAX_SAVED_SEARCHES
            && list.iter().all(|text| {
                !text.trim().is_empty()
                    && text.chars().count() <= MAX_SEARCH_CHARS
                    && !text.chars().any(char::is_control)
            })
    })
}

fn validate(key: &str, value: &str) -> AppResult<()> {
    let valid = if key == SAVED_SEARCHES {
        saved_searches_are_valid(value)
    } else if is_free_form(key) {
        let longest = if key == NOTES_RECENT {
            MAX_RECENT_VALUE
        } else {
            MAX_FREE_VALUE
        };
        value.chars().count() <= longest && !value.chars().any(char::is_control)
    } else if key == autobackup::KEEP_KEY {
        value
            .parse::<usize>()
            .is_ok_and(|count| (1..=autobackup::MAX_KEEP).contains(&count))
    } else if key == autobackup::LAST_KEY {
        dates::parse_instant(value).is_ok()
    } else if key == DEFAULT_NOTIFY_TIME {
        dates::parse_time(value).is_ok()
    } else {
        allowed_values(key)?.contains(&value)
    };
    if valid {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "«{value}» no es un valor válido para «{key}»"
        )))
    }
}

pub fn get(state: &AppState, key: &str) -> AppResult<Option<String>> {
    allowed_values(key)?;
    state.db.read(|conn| Ok(settings::get(conn, key)?))
}

pub fn set(state: &AppState, key: &str, value: &str) -> AppResult<()> {
    validate(key, value)?;
    state.db.write(|tx| {
        settings::set(tx, key, value)?;
        if key == DEFAULT_NOTIFY_TIME {
            reminders::recalculate_untimed(tx, &Local)?;
        }
        Ok(())
    })?;
    if key == DEFAULT_NOTIFY_TIME {
        state.wake.notify_one();
    }
    Ok(())
}

#[tauri::command]
pub async fn settings_get(
    state: State<'_, Arc<AppState>>,
    key: String,
) -> Result<Option<String>, String> {
    run_blocking(state.inner().clone(), move |state| get(state, &key)).await
}

#[tauri::command]
pub async fn settings_set(
    state: State<'_, Arc<AppState>>,
    key: String,
    value: String,
) -> Result<(), String> {
    run_blocking(state.inner().clone(), move |state| set(state, &key, &value)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::test_state;

    fn code<T>(result: AppResult<T>) -> &'static str {
        result.err().map(|error| error.code()).unwrap_or("OK")
    }

    #[test]
    fn saved_searches_are_a_short_list_of_plain_texts() {
        let (state, _) = test_state();
        assert_eq!(get(&state, SAVED_SEARCHES).expect("get"), None);
        let ok = r#"["is:pendiente tag:SMA","desde:2026-01-01 informe"]"#;
        set(&state, SAVED_SEARCHES, ok).expect("set");
        assert_eq!(
            get(&state, SAVED_SEARCHES).expect("get").as_deref(),
            Some(ok)
        );
        set(&state, SAVED_SEARCHES, "[]").expect("empty list");

        let too_many = serde_json::to_string(&vec!["a"; 21]).expect("json");
        let too_long = serde_json::to_string(&vec!["a".repeat(201)]).expect("json");
        for bad in [
            "no es json",
            r#"{"a":1}"#,
            r#"[1,2]"#,
            r#"["  "]"#,
            r#"["ab"]"#,
            too_many.as_str(),
            too_long.as_str(),
        ] {
            assert_eq!(
                code(set(&state, SAVED_SEARCHES, bad)),
                "VALIDATION",
                "{bad}"
            );
        }
    }

    #[test]
    fn the_automatic_backup_settings_are_validated() {
        let (state, _) = test_state();
        set(&state, autobackup::MODE_KEY, "weekly").expect("mode");
        set(&state, autobackup::KEEP_KEY, "30").expect("keep");
        set(&state, autobackup::LAST_KEY, "2026-03-10T10:00:00Z").expect("last");
        assert_eq!(
            code(set(&state, autobackup::MODE_KEY, "hourly")),
            "VALIDATION"
        );
        for bad in ["0", "31", "-1", "varias", ""] {
            assert_eq!(
                code(set(&state, autobackup::KEEP_KEY, bad)),
                "VALIDATION",
                "{bad}"
            );
        }
        assert_eq!(
            code(set(&state, autobackup::LAST_KEY, "ayer")),
            "VALIDATION"
        );
        assert_eq!(
            get(&state, autobackup::KEEP_KEY).expect("get").as_deref(),
            Some("30")
        );
    }

    #[test]
    fn stores_and_returns_the_view_mode() {
        let (state, _) = test_state();
        assert_eq!(get(&state, VIEW_MODE).expect("get"), None);
        for &mode in allowed_values(VIEW_MODE).expect("view modes") {
            set(&state, VIEW_MODE, mode).expect("set");
            assert_eq!(get(&state, VIEW_MODE).expect("get").as_deref(), Some(mode));
        }
    }

    #[test]
    fn remembers_the_section_and_the_open_note() {
        let (state, _) = test_state();
        set(&state, APP_SECTION, "today").expect("today");
        set(&state, APP_SECTION, "notes").expect("section");
        set(&state, NOTES_SCOPE, "folder:abc-123").expect("scope");
        set(&state, NOTES_OPEN, "").expect("clear");
        assert_eq!(
            get(&state, NOTES_SCOPE).expect("get").as_deref(),
            Some("folder:abc-123")
        );
        assert_eq!(code(set(&state, APP_SECTION, "tasks")), "VALIDATION");
        assert_eq!(code(set(&state, NOTES_OPEN, &"x".repeat(65))), "VALIDATION");
        assert_eq!(
            code(set(
                &state, NOTES_OPEN, "a
b"
            )),
            "VALIDATION"
        );
    }

    #[test]
    fn remembers_the_recent_notes_with_a_longer_limit() {
        let (state, _) = test_state();
        let ids = ["0190b3a4-7c1e-7d2f-8a55-3f2f6c1d9e10"; 8].join(",");
        set(&state, NOTES_RECENT, &ids).expect("recent");
        assert_eq!(get(&state, NOTES_RECENT).expect("get"), Some(ids));
        assert_eq!(
            code(set(&state, NOTES_RECENT, &"x".repeat(481))),
            "VALIDATION"
        );
    }

    #[test]
    fn theme_and_window_effect_accept_only_their_own_values() {
        let (state, _) = test_state();
        set(&state, THEME, "dark").expect("theme");
        set(&state, WINDOW_EFFECT, "acrylic").expect("effect");
        assert_eq!(get(&state, THEME).expect("get").as_deref(), Some("dark"));

        assert_eq!(code(set(&state, THEME, "week")), "VALIDATION");
        assert_eq!(code(set(&state, WINDOW_EFFECT, "dark")), "VALIDATION");
        assert_eq!(code(set(&state, VIEW_MODE, "mica")), "VALIDATION");
        assert_eq!(get(&state, THEME).expect("get").as_deref(), Some("dark"));
    }

    #[test]
    fn rejects_unknown_keys_and_values_without_storing_anything() {
        let (state, _) = test_state();
        set(&state, VIEW_MODE, "week").expect("set");

        assert_eq!(code(set(&state, VIEW_MODE, "year")), "VALIDATION");
        assert_eq!(code(set(&state, VIEW_MODE, "")), "VALIDATION");
        assert_eq!(
            code(set(&state, "theme'; DROP TABLE tasks;--", "x")),
            "VALIDATION"
        );
        assert_eq!(code(get(&state, "other")), "VALIDATION");

        assert_eq!(
            get(&state, VIEW_MODE).expect("get").as_deref(),
            Some("week")
        );
    }
}

//! `settings_get` / `settings_set` (SPEC §7.3). Only whitelisted keys, with validated values.

use std::sync::Arc;

use chrono::Local;
use tauri::State;

use crate::{
    domain::dates,
    error::{AppError, AppResult},
    repo::{reminders, settings},
    state::{AppState, run_blocking},
};

/// Which view the main area shows: the list or one of the calendar views (F-10).
pub const VIEW_MODE: &str = "view_mode";
/// Colour theme: follow the system or force light or dark (F6-05).
pub const THEME: &str = "theme";
/// Native window material requested at startup (F6-05); the OS may still fall back.
pub const WINDOW_EFFECT: &str = "window_effect";

/// Hour (`HH:MM`) at which reminders of tasks without a time sound (D-35).
pub const DEFAULT_NOTIFY_TIME: &str = settings::NOTIFY_TIME_KEY;

/// Whitelist: every setting with the values it accepts.
const ALLOWED: [(&str, &[&str]); 3] = [
    (VIEW_MODE, &["list", "day", "week", "month"]),
    (THEME, &["system", "light", "dark"]),
    (WINDOW_EFFECT, &["mica", "acrylic", "none"]),
];

fn allowed_values(key: &str) -> AppResult<&'static [&'static str]> {
    if key == DEFAULT_NOTIFY_TIME {
        return Ok(&[]);
    }
    ALLOWED
        .iter()
        .find(|(name, _)| *name == key)
        .map(|(_, values)| *values)
        .ok_or_else(|| AppError::Validation(format!("el ajuste «{key}» no existe")))
}

fn validate(key: &str, value: &str) -> AppResult<()> {
    let valid = if key == DEFAULT_NOTIFY_TIME {
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
    fn stores_and_returns_the_view_mode() {
        let (state, _) = test_state();
        assert_eq!(get(&state, VIEW_MODE).expect("get"), None);
        for &mode in allowed_values(VIEW_MODE).expect("view modes") {
            set(&state, VIEW_MODE, mode).expect("set");
            assert_eq!(get(&state, VIEW_MODE).expect("get").as_deref(), Some(mode));
        }
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

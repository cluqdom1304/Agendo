//! `settings_get` / `settings_set` (SPEC §7.3). Only whitelisted keys, with validated values.

use std::sync::Arc;

use tauri::State;

use crate::{
    error::{AppError, AppResult},
    repo::settings,
    state::{AppState, run_blocking},
};

/// Which view the main area shows: the list or one of the calendar views (F-10).
pub const VIEW_MODE: &str = "view_mode";
const VIEW_MODES: [&str; 4] = ["list", "day", "week", "month"];

fn unknown_key(key: &str) -> AppError {
    AppError::Validation(format!("el ajuste «{key}» no existe"))
}

fn validate_key(key: &str) -> AppResult<()> {
    match key {
        VIEW_MODE => Ok(()),
        _ => Err(unknown_key(key)),
    }
}

fn validate(key: &str, value: &str) -> AppResult<()> {
    validate_key(key)?;
    if VIEW_MODES.contains(&value) {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "«{value}» no es una vista válida"
        )))
    }
}

pub fn get(state: &AppState, key: &str) -> AppResult<Option<String>> {
    validate_key(key)?;
    state.db.read(|conn| Ok(settings::get(conn, key)?))
}

pub fn set(state: &AppState, key: &str, value: &str) -> AppResult<()> {
    validate(key, value)?;
    state.db.write(|tx| Ok(settings::set(tx, key, value)?))
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
        for mode in VIEW_MODES {
            set(&state, VIEW_MODE, mode).expect("set");
            assert_eq!(get(&state, VIEW_MODE).expect("get").as_deref(), Some(mode));
        }
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

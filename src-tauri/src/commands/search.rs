//! Search command (SPEC §7.3): a thin wrapper over `repo::search`.

use std::sync::Arc;

use tauri::State;

use crate::{
    error::AppResult,
    repo::search::{self, DEFAULT_LIMIT, MAX_LIMIT, SearchHit},
    state::{AppState, run_blocking},
};

pub fn run(state: &AppState, query: &str, limit: Option<u32>) -> AppResult<Vec<SearchHit>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    state
        .db
        .read(|conn| Ok(search::search(conn, query, limit)?))
}

#[tauri::command]
pub async fn search(
    state: State<'_, Arc<AppState>>,
    query: String,
    limit: Option<u32>,
) -> Result<Vec<SearchHit>, String> {
    run_blocking(state.inner().clone(), move |state| {
        run(state, &query, limit)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        commands::tasks::{self, TaskCreateInput},
        state::test_state,
    };

    #[test]
    fn finds_tasks_created_through_the_commands_and_clamps_the_limit() {
        let (state, _) = test_state();
        for title in ["Estudiar álgebra", "Álgebra lineal", "Otra cosa"] {
            tasks::create(
                &state,
                TaskCreateInput {
                    title: Some(title.to_string()),
                    ..TaskCreateInput::default()
                },
            )
            .expect("create");
        }
        assert_eq!(run(&state, "algebra", None).expect("search").len(), 2);
        assert_eq!(run(&state, "algebra", Some(0)).expect("search").len(), 1);
        assert!(run(&state, "   ", Some(10_000)).expect("search").is_empty());
    }
}

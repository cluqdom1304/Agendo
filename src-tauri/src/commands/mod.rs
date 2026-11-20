pub mod attachments;
pub mod autostart;
pub mod backup;
pub mod events;
pub mod exchange;
pub mod folders;
pub mod info;
pub mod note_reminders;
pub mod notes;
pub mod reminders;
pub mod search;
pub mod settings;
pub mod startup;
pub mod tags;
pub mod tasks;
pub mod templates;
pub mod versions;
pub mod window;

use crate::{
    error::{AppError, AppResult},
    repo::model::Page,
};

/// Largest window a list command returns in one call.
pub const MAX_PAGE_SIZE: u32 = 10_000;

/// Builds the window of a list command from its optional `limit` / `offset` arguments. Without
/// `limit` the whole list is returned (what callers that do not paginate expect).
pub fn page_of(limit: Option<u32>, offset: Option<u32>) -> AppResult<Page> {
    if matches!(limit, Some(0)) || limit.is_some_and(|rows| rows > MAX_PAGE_SIZE) {
        return Err(AppError::Validation(format!(
            "«limit» debe estar entre 1 y {MAX_PAGE_SIZE}"
        )));
    }
    Ok(Page {
        limit,
        offset: offset.unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_of_defaults_to_the_whole_list_and_rejects_bad_limits() {
        assert_eq!(page_of(None, None).expect("all"), Page::ALL);
        assert_eq!(
            page_of(Some(50), Some(100)).expect("window"),
            Page {
                limit: Some(50),
                offset: 100
            }
        );
        assert!(page_of(Some(0), None).is_err());
        assert!(page_of(Some(MAX_PAGE_SIZE + 1), None).is_err());
    }
}

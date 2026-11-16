//! Pure business rules (SPEC §4.1). Nothing here depends on `tauri` or on SQLite.

pub mod codec;
pub mod content;
pub mod dates;
pub mod enex;
pub mod html;
pub mod markdown;
pub mod ordering;
pub mod recurrence;
pub mod reminders;
pub mod validation;

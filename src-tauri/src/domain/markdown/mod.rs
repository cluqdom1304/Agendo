//! Markdown in both directions for the document of a note (F14-01, F14-03). The document is
//! TipTap's JSON; this module covers exactly what the editor can produce: paragraphs, headings,
//! bullet/ordered/task lists, quotes, code blocks, rules, tables, images and the marks bold,
//! italic, code, highlight and link. Nothing here touches the disk: images and the titles of
//! references are resolved through closures the caller supplies.

use serde_json::{Map, Value, json};

use super::validation::is_allowed_link;

/// Deepest nesting of quotes and lists followed when reading; beyond it the text stays literal,
/// so a hostile file cannot exhaust the stack.
const MAX_DEPTH: usize = 20;
/// Deepest nesting of emphasis followed when reading inline text.
const MAX_INLINE_DEPTH: usize = 8;

/// What the writer needs to know about the world outside the document.
pub struct Labels<'a> {
    /// Relative path of the stored file of an attachment, `None` if it is not available.
    pub image_path: &'a dyn Fn(&str) -> Option<String>,
    /// Text that stands for a link to a note.
    pub note_label: &'a dyn Fn(&str) -> String,
    /// Text that stands for a reference to a task.
    pub task_label: &'a dyn Fn(&str) -> String,
}

fn kind(node: &Value) -> &str {
    node.get("type").and_then(Value::as_str).unwrap_or("")
}

fn children(node: &Value) -> &[Value] {
    node.get("content")
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn attr<'a>(node: &'a Value, name: &str) -> Option<&'a Value> {
    node.get("attrs").and_then(|attrs| attrs.get(name))
}

mod inline;
mod read;
mod table;
#[cfg(test)]
mod tests;
mod write;

pub use read::{FrontMatter, ImageFile, from_markdown, from_plain_text, split_front_matter};
pub use write::to_markdown;

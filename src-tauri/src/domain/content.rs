//! Plain text of a notes document, for the search index (D-03). `content_json` is the source of
//! truth; `content_text` is derived from it every time the notes are saved.

use serde_json::Value;

use super::validation::{ValidationError, validate_content_json};

/// Validates `content_json` and returns its text: one line per block, line breaks kept, marks
/// (bold, links…) dropped.
pub fn text_of(content_json: &str) -> Result<String, ValidationError> {
    validate_content_json(content_json)?;
    let document: Value = serde_json::from_str(content_json)
        .map_err(|_| ValidationError::new("las notas no tienen un formato válido"))?;
    let mut text = String::new();
    collect(&document, &mut text);
    text.truncate(text.trim_end_matches('\n').len());
    Ok(text)
}

/// Tasks and notes a document points to (D-45, D-46), in order of first appearance and without
/// repeats. They come from `taskRef` (`attrs.taskId`) and `noteLink` (`attrs.noteId`) nodes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct References {
    pub task_ids: Vec<String>,
    pub note_ids: Vec<String>,
}

/// Validates `content_json` and returns the references it contains. A reference node without a
/// non-empty id is rejected: it could never be resolved.
pub fn references_of(content_json: &str) -> Result<References, ValidationError> {
    validate_content_json(content_json)?;
    let document: Value = serde_json::from_str(content_json)
        .map_err(|_| ValidationError::new("las notas no tienen un formato válido"))?;
    let mut references = References::default();
    collect_references(&document, &mut references)?;
    Ok(references)
}

fn collect_references(node: &Value, references: &mut References) -> Result<(), ValidationError> {
    let target = match node.get("type").and_then(Value::as_str) {
        Some("taskRef") => Some(("taskId", &mut references.task_ids, "tarea")),
        Some("noteLink") => Some(("noteId", &mut references.note_ids, "nota")),
        _ => None,
    };
    if let Some((attribute, ids, what)) = target {
        let id = node
            .get("attrs")
            .and_then(|attrs| attrs.get(attribute))
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| {
                ValidationError::new(format!("una referencia a {what} no tiene identificador"))
            })?;
        if !ids.iter().any(|known| known == id) {
            ids.push(id.to_string());
        }
    }
    if let Some(children) = node.get("content").and_then(Value::as_array) {
        for child in children {
            collect_references(child, references)?;
        }
    }
    Ok(())
}

fn collect(node: &Value, text: &mut String) {
    let kind = node.get("type").and_then(Value::as_str);
    match kind {
        Some("text") => {
            if let Some(value) = node.get("text").and_then(Value::as_str) {
                text.push_str(value);
            }
        }
        Some("hardBreak") => text.push('\n'),
        _ => {}
    }
    if let Some(children) = node.get("content").and_then(Value::as_array) {
        for child in children {
            collect(child, text);
        }
    }
    let is_block = !matches!(kind, Some("doc" | "text" | "hardBreak") | None);
    if is_block && !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(json: &str) -> String {
        text_of(json).expect("valid document")
    }

    #[test]
    fn references_are_collected_once_in_order_of_appearance() {
        let json = r#"{"type":"doc","content":[
            {"type":"paragraph","content":[
                {"type":"noteLink","attrs":{"noteId":"n2"}},
                {"type":"text","text":" y "},
                {"type":"noteLink","attrs":{"noteId":"n1"}}]},
            {"type":"taskRef","attrs":{"taskId":"t1"}},
            {"type":"bulletList","content":[{"type":"listItem","content":[
                {"type":"taskRef","attrs":{"taskId":"t2"}},
                {"type":"taskRef","attrs":{"taskId":"t1"}},
                {"type":"noteLink","attrs":{"noteId":"n2"}}]}]}
        ]}"#;
        let references = references_of(json).expect("valid");
        assert_eq!(references.task_ids, ["t1", "t2"]);
        assert_eq!(references.note_ids, ["n2", "n1"]);
    }

    #[test]
    fn a_document_without_references_has_none() {
        let references =
            references_of(r#"{"type":"doc","content":[{"type":"paragraph"}]}"#).expect("valid");
        assert_eq!(references, References::default());
    }

    #[test]
    fn a_reference_without_an_id_is_rejected() {
        for json in [
            r#"{"type":"doc","content":[{"type":"taskRef"}]}"#,
            r#"{"type":"doc","content":[{"type":"taskRef","attrs":{"taskId":""}}]}"#,
            r#"{"type":"doc","content":[{"type":"noteLink","attrs":{"noteId":5}}]}"#,
        ] {
            assert!(references_of(json).is_err(), "{json}");
        }
        assert_eq!(
            references_of(r#"{"type":"doc","content":[{"type":"taskRef"}]}"#)
                .expect_err("rejected")
                .to_string(),
            "una referencia a tarea no tiene identificador"
        );
    }

    #[test]
    fn references_of_rejects_what_text_of_rejects() {
        assert!(references_of("not json").is_err());
        assert!(references_of(r#"{"type":"paragraph"}"#).is_err());
    }

    #[test]
    fn empty_document_has_no_text() {
        assert_eq!(text(r#"{"type":"doc","content":[]}"#), "");
        assert_eq!(text(r#"{"type":"doc"}"#), "");
    }

    #[test]
    fn paragraphs_become_lines() {
        let json = r#"{"type":"doc","content":[
            {"type":"paragraph","content":[{"type":"text","text":"Primera"}]},
            {"type":"paragraph","content":[{"type":"text","text":"Segunda"}]}
        ]}"#;
        assert_eq!(text(json), "Primera\nSegunda");
    }

    #[test]
    fn marks_and_inline_nodes_are_flattened_into_the_line() {
        let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[
            {"type":"text","text":"Hola "},
            {"type":"text","text":"mundo","marks":[{"type":"bold"}]},
            {"type":"text","text":" y ","marks":[{"type":"italic"}]},
            {"type":"text","text":"enlace","marks":[{"type":"link","attrs":{"href":"https://example.com"}}]}
        ]}]}"#;
        assert_eq!(text(json), "Hola mundo y enlace");
    }

    #[test]
    fn checklist_items_become_lines() {
        let json = r#"{"type":"doc","content":[{"type":"taskList","content":[
            {"type":"taskItem","attrs":{"checked":true},"content":[
                {"type":"paragraph","content":[{"type":"text","text":"uno"}]}]},
            {"type":"taskItem","attrs":{"checked":false},"content":[
                {"type":"paragraph","content":[{"type":"text","text":"dos"}]}]}
        ]}]}"#;
        assert_eq!(text(json), "uno\ndos");
    }

    #[test]
    fn hard_breaks_start_a_new_line() {
        let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[
            {"type":"text","text":"uno"},{"type":"hardBreak"},{"type":"text","text":"dos"}
        ]}]}"#;
        assert_eq!(text(json), "uno\ndos");
    }

    #[test]
    fn nested_lists_give_one_line_per_item() {
        let item = |label: &str, nested: &str| {
            format!(
                r#"{{"type":"listItem","content":[
                    {{"type":"paragraph","content":[{{"type":"text","text":"{label}"}}]}}{nested}
                ]}}"#
            )
        };
        let inner = item("hijo", "");
        let nested = format!(r#",{{"type":"bulletList","content":[{inner}]}}"#);
        let outer = item("padre", &nested);
        let json = format!(
            r#"{{"type":"doc","content":[{{"type":"orderedList","content":[{outer},{}]}}]}}"#,
            item("otro", "")
        );

        assert_eq!(text(&json), "padre\nhijo\notro");
    }

    #[test]
    fn code_blocks_and_unicode_are_kept() {
        let json = r#"{"type":"doc","content":[
            {"type":"codeBlock","content":[{"type":"text","text":"let ñ = 1;"}]},
            {"type":"paragraph","content":[{"type":"text","text":"Reunión — álgebra"}]}
        ]}"#;
        assert_eq!(text(json), "let ñ = 1;\nReunión — álgebra");
    }

    #[test]
    fn empty_paragraphs_do_not_add_blank_lines() {
        let json = r#"{"type":"doc","content":[
            {"type":"paragraph"},
            {"type":"paragraph","content":[{"type":"text","text":"solo"}]},
            {"type":"paragraph"}
        ]}"#;
        assert_eq!(text(json), "solo");
    }

    #[test]
    fn malformed_input_is_a_validation_error() {
        for invalid in ["", "{", "[]", r#"{"type":"paragraph"}"#] {
            let error = text_of(invalid).expect_err("must be rejected");
            assert_eq!(error.message(), "las notas no tienen un formato válido");
        }
    }
}

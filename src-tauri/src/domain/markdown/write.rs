//! Document to Markdown: blocks and lists.

use super::*;
use super::{inline::*, table::*};

/// The Markdown of a whole document, without a trailing newline.
pub fn to_markdown(doc: &Value, labels: &Labels<'_>) -> String {
    blocks_md(children(doc), labels)
}

pub(super) fn blocks_md(blocks: &[Value], labels: &Labels<'_>) -> String {
    blocks
        .iter()
        .map(|block| block_md(block, labels))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub(super) fn block_md(node: &Value, labels: &Labels<'_>) -> String {
    match kind(node) {
        "paragraph" => escape_block_start(&inline_md(children(node), labels, "  \n")),
        "heading" => {
            let level = attr(node, "level")
                .and_then(Value::as_u64)
                .unwrap_or(1)
                .clamp(1, 6) as usize;
            format!(
                "{} {}",
                "#".repeat(level),
                inline_md(children(node), labels, " ")
            )
        }
        "bulletList" | "orderedList" | "taskList" => list_md(node, labels),
        "blockquote" => blocks_md(children(node), labels)
            .lines()
            .map(|line| {
                if line.is_empty() {
                    ">".to_string()
                } else {
                    format!("> {line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        "codeBlock" => code_block_md(node),
        "horizontalRule" => "---".to_string(),
        "table" => table_md(node, labels),
        "image" => {
            let path = attr(node, "attachmentId")
                .and_then(Value::as_str)
                .and_then(|id| (labels.image_path)(id));
            match path {
                Some(path) => format!("![imagen]({})", encode_url(&path)),
                None => String::new(),
            }
        }
        "taskRef" => {
            let id = attr(node, "taskId").and_then(Value::as_str).unwrap_or("");
            escape_text(&(labels.task_label)(id))
        }
        _ => inline_md(children(node), labels, " "),
    }
}

pub(super) fn code_block_md(node: &Value) -> String {
    let code: String = children(node)
        .iter()
        .filter_map(|child| child.get("text").and_then(Value::as_str))
        .collect();
    let longest = longest_run(&code, '`');
    let fence = "`".repeat(longest.max(2) + 1);
    let language = attr(node, "language")
        .and_then(Value::as_str)
        .filter(|language| {
            language
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "+-#._".contains(c))
        })
        .unwrap_or("");
    format!("{fence}{language}\n{code}\n{fence}")
}

pub(super) fn longest_run(text: &str, symbol: char) -> usize {
    let mut longest = 0;
    let mut current = 0;
    for c in text.chars() {
        if c == symbol {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    longest
}

pub(super) fn list_md(list: &Value, labels: &Labels<'_>) -> String {
    let ordered = kind(list) == "orderedList";
    let start = attr(list, "start").and_then(Value::as_i64).unwrap_or(1);
    let mut items = Vec::new();
    for (index, item) in children(list).iter().enumerate() {
        let marker = match kind(list) {
            "taskList" => {
                if attr(item, "checked").and_then(Value::as_bool) == Some(true) {
                    "- [x] ".to_string()
                } else {
                    "- [ ] ".to_string()
                }
            }
            _ if ordered => format!("{}. ", start + i64::try_from(index).unwrap_or(0)),
            _ => "- ".to_string(),
        };
        let indent = " ".repeat(marker.len());
        let mut body = String::new();
        for (position, block) in children(item).iter().enumerate() {
            let text = block_md(block, labels);
            if position > 0 {
                let nested = matches!(kind(block), "bulletList" | "orderedList" | "taskList");
                body.push_str(if nested { "\n" } else { "\n\n" });
            }
            body.push_str(&text);
        }
        let mut lines = body.lines();
        let mut text = format!("{marker}{}", lines.next().unwrap_or(""));
        for line in lines {
            text.push('\n');
            if !line.is_empty() {
                text.push_str(&indent);
                text.push_str(line);
            }
        }
        items.push(text);
    }
    items.join("\n")
}

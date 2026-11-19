//! Markdown to document: front matter and block structure.

use super::*;
use super::{inline::*, table::*};

/// An image the reader found, to be stored by the caller.
pub struct ImageFile {
    pub attachment_id: String,
    pub mime: String,
}

/// Parses Markdown into a document. `image` receives the path an image points to and returns
/// the stored file, or `None` if it cannot be used (the image is then left out).
pub fn from_markdown(text: &str, image: &mut dyn FnMut(&str) -> Option<ImageFile>) -> Value {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<String> = normalized.lines().map(expand_tabs).collect();
    let blocks = parse_blocks(&lines, image, 0);
    json!({ "type": "doc", "content": blocks })
}

/// Plain text as a document: blank lines separate paragraphs, single line breaks are kept.
pub fn from_plain_text(text: &str) -> Value {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut blocks = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let flush = |current: &mut Vec<&str>, blocks: &mut Vec<Value>| {
        if current.is_empty() {
            return;
        }
        let mut content = Vec::new();
        for (index, line) in current.iter().enumerate() {
            if index > 0 {
                content.push(json!({ "type": "hardBreak" }));
            }
            content.push(json!({ "type": "text", "text": line.trim_end() }));
        }
        blocks.push(json!({ "type": "paragraph", "content": content }));
        current.clear();
    };
    for line in normalized.lines() {
        if line.trim().is_empty() {
            flush(&mut current, &mut blocks);
        } else {
            current.push(line);
        }
    }
    flush(&mut current, &mut blocks);
    json!({ "type": "doc", "content": blocks })
}

/// What a Markdown file declares before its body (`---` block at the top).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

/// Splits the front matter off `text`. Without a well-formed block the text is returned as is.
pub fn split_front_matter(text: &str) -> (FrontMatter, &str) {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let Some(rest) = text
        .strip_prefix("---\r\n")
        .or_else(|| text.strip_prefix("---\n"))
    else {
        return (FrontMatter::default(), text);
    };
    let mut offset = 0;
    let mut block = Vec::new();
    for line in rest.split_inclusive('\n') {
        offset += line.len();
        if line.trim_end() == "---" {
            return (parse_front_matter(&block), &rest[offset..]);
        }
        block.push(line.trim_end());
    }
    (FrontMatter::default(), text)
}

pub(super) fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.starts_with('"')
        && let Ok(text) = serde_json::from_str::<String>(value)
    {
        return text;
    }
    for quote in ['"', '\''] {
        if let Some(inner) = value
            .strip_prefix(quote)
            .and_then(|rest| rest.strip_suffix(quote))
        {
            return inner.to_string();
        }
    }
    value.to_string()
}

pub(super) fn parse_front_matter(lines: &[&str]) -> FrontMatter {
    let mut data = FrontMatter::default();
    let mut in_tags = false;
    for line in lines {
        if in_tags && let Some(item) = line.trim_start().strip_prefix("- ") {
            let tag = unquote(item);
            let tag = tag.trim_start_matches('#');
            if !tag.is_empty() {
                data.tags.push(tag.to_string());
            }
            continue;
        }
        in_tags = false;
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match key.trim().to_ascii_lowercase().as_str() {
            "title" if !value.is_empty() => data.title = Some(unquote(value).to_string()),
            "created" | "date" if !value.is_empty() => {
                data.created = Some(unquote(value).to_string());
            }
            "updated" | "modified" if !value.is_empty() => {
                data.updated = Some(unquote(value).to_string());
            }
            "tags" | "tag" => {
                if value.is_empty() {
                    in_tags = true;
                } else {
                    let list = value.trim_start_matches('[').trim_end_matches(']');
                    data.tags.extend(
                        list.split(',')
                            .map(|tag| unquote(tag).trim_start_matches('#').to_string())
                            .filter(|tag| !tag.is_empty()),
                    );
                }
            }
            _ => {}
        }
    }
    data
}

pub(super) fn expand_tabs(line: &str) -> String {
    let leading = line.len() - line.trim_start_matches(['\t', ' ']).len();
    let mut out = String::with_capacity(line.len());
    for c in line[..leading].chars() {
        out.push_str(if c == '\t' { "    " } else { " " });
    }
    out.push_str(&line[leading..]);
    out
}

pub(super) fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

pub(super) fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

pub(super) fn fence_of(line: &str) -> Option<(char, usize, &str)> {
    let trimmed = line.trim_start();
    if indent_of(line) > 3 {
        return None;
    }
    let symbol = trimmed.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let run = trimmed.chars().take_while(|c| *c == symbol).count();
    (run >= 3).then(|| (symbol, run, trimmed[run..].trim()))
}

pub(super) fn heading_of(line: &str) -> Option<(usize, &str)> {
    if indent_of(line) > 3 {
        return None;
    }
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let rest = &trimmed[level..];
    if !rest.is_empty() && !rest.starts_with(' ') {
        return None;
    }
    let text = rest.trim();
    let closed = text.trim_end_matches('#');
    let text = if closed.is_empty() || closed.ends_with(' ') {
        closed.trim()
    } else {
        text
    };
    Some((level, text))
}

pub(super) fn is_rule(line: &str) -> bool {
    let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    indent_of(line) <= 3
        && compact.len() >= 3
        && ['-', '*', '_']
            .iter()
            .any(|symbol| compact.chars().all(|c| c == *symbol))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ListKind {
    Bullet,
    Ordered(i64),
}

/// `(indent, kind, offset of the item's content)` if `line` opens a list item.
pub(super) fn list_marker(line: &str) -> Option<(usize, ListKind, usize)> {
    let indent = indent_of(line);
    let rest = &line[indent..];
    let digits = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
    let (kind, marker_len) = if rest.starts_with(['-', '*', '+']) {
        (ListKind::Bullet, 1)
    } else if (1..=9).contains(&digits) && rest[digits..].starts_with(['.', ')']) {
        (ListKind::Ordered(rest[..digits].parse().ok()?), digits + 1)
    } else {
        return None;
    };
    let after = &rest[marker_len..];
    if !after.is_empty() && !after.starts_with(' ') {
        return None;
    }
    let spaces = after.len() - after.trim_start_matches(' ').len();
    let spaces = if spaces == 0 || spaces > 4 { 1 } else { spaces };
    Some((indent, kind, indent + marker_len + spaces))
}

pub(super) fn same_list(a: ListKind, b: ListKind) -> bool {
    matches!(
        (a, b),
        (ListKind::Bullet, ListKind::Bullet) | (ListKind::Ordered(_), ListKind::Ordered(_))
    )
}

/// Whether `line` interrupts a paragraph.
pub(super) fn interrupts_paragraph(line: &str, lines: &[String], at: usize) -> bool {
    is_blank(line)
        || fence_of(line).is_some()
        || heading_of(line).is_some()
        || is_rule(line)
        || line.trim_start().starts_with('>')
        || list_marker(line).is_some_and(|(_, kind, _)| match kind {
            ListKind::Bullet => true,
            ListKind::Ordered(start) => start == 1,
        })
        || starts_table(lines, at)
}

pub(super) fn parse_blocks(
    lines: &[String],
    image: &mut dyn FnMut(&str) -> Option<ImageFile>,
    depth: usize,
) -> Vec<Value> {
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];
        if is_blank(line) {
            i += 1;
        } else if let Some((symbol, run, language)) = fence_of(line) {
            let mut code = Vec::new();
            i += 1;
            while i < lines.len() {
                let closing = fence_of(&lines[i])
                    .is_some_and(|(s, r, rest)| s == symbol && r >= run && rest.is_empty());
                i += 1;
                if closing {
                    break;
                }
                code.push(lines[i - 1].as_str());
            }
            blocks.push(code_block(&code.join("\n"), language));
        } else if let Some((level, text)) = heading_of(line) {
            let content = parse_inline(text, &[], 0);
            blocks.push(json!({
                "type": "heading",
                "attrs": { "level": level },
                "content": merge(content),
            }));
            i += 1;
        } else if is_rule(line) {
            blocks.push(json!({ "type": "horizontalRule" }));
            i += 1;
        } else if line.trim_start().starts_with('>') && depth < MAX_DEPTH {
            let mut inner = Vec::new();
            while i < lines.len() && lines[i].trim_start().starts_with('>') {
                let stripped = lines[i].trim_start().trim_start_matches('>');
                inner.push(stripped.strip_prefix(' ').unwrap_or(stripped).to_string());
                i += 1;
            }
            let mut content = parse_blocks(&inner, image, depth + 1);
            if content.is_empty() {
                content.push(json!({ "type": "paragraph" }));
            }
            blocks.push(json!({ "type": "blockquote", "content": content }));
        } else if let Some((_, first_kind, _)) = list_marker(line).filter(|_| depth < MAX_DEPTH) {
            let (list, next) = parse_list(lines, i, first_kind, image, depth);
            blocks.push(list);
            i = next;
        } else if starts_table(lines, i) {
            let header = split_row(line);
            let width = header.len();
            let mut rows = vec![(header, true)];
            i += 2;
            while i < lines.len() && !is_blank(&lines[i]) && lines[i].contains('|') {
                rows.push((split_row(&lines[i]), false));
                i += 1;
            }
            blocks.push(table_node(rows, width));
        } else {
            let start = i;
            i += 1;
            while i < lines.len() && !interrupts_paragraph(&lines[i], lines, i) {
                i += 1;
            }
            blocks.extend(paragraph_nodes(&lines[start..i], image));
        }
    }
    blocks
}

pub(super) fn code_block(code: &str, language: &str) -> Value {
    let language = language.split_whitespace().next().unwrap_or("");
    let attrs = if language.is_empty() {
        json!({ "language": null })
    } else {
        json!({ "language": language })
    };
    if code.is_empty() {
        json!({ "type": "codeBlock", "attrs": attrs })
    } else {
        json!({
            "type": "codeBlock",
            "attrs": attrs,
            "content": [{ "type": "text", "text": code }],
        })
    }
}

pub(super) fn parse_list(
    lines: &[String],
    start: usize,
    first_kind: ListKind,
    image: &mut dyn FnMut(&str) -> Option<ImageFile>,
    depth: usize,
) -> (Value, usize) {
    let mut items: Vec<(Vec<String>, Option<bool>)> = Vec::new();
    let mut i = start;
    let mut first_offset = 0;
    let ordered_start = match first_kind {
        ListKind::Ordered(n) => n,
        ListKind::Bullet => 1,
    };
    while i < lines.len() {
        let Some((indent, item_kind, offset)) = list_marker(&lines[i]) else {
            break;
        };
        if items.is_empty() {
            first_offset = offset;
        } else if !same_list(first_kind, item_kind) || indent >= first_offset {
            break;
        }
        let mut body = vec![lines[i][offset.min(lines[i].len())..].to_string()];
        i += 1;
        while i < lines.len() {
            let line = &lines[i];
            if is_blank(line) {
                let next = lines[i + 1..].iter().find(|l| !is_blank(l));
                if next.is_some_and(|next| indent_of(next) >= offset) {
                    body.push(String::new());
                    i += 1;
                    continue;
                }
                break;
            }
            if indent_of(line) >= offset {
                body.push(line[offset.min(line.len())..].to_string());
            } else if list_marker(line).is_some()
                || interrupts_paragraph(line, lines, i)
                || body.last().is_some_and(|last| is_blank(last))
            {
                break;
            } else {
                body.push(line.trim_start().to_string());
            }
            i += 1;
        }
        let mut checked = None;
        for prefix in ["[ ] ", "[x] ", "[X] "] {
            if let Some(rest) = body[0].strip_prefix(prefix) {
                checked = Some(prefix != "[ ] ");
                body[0] = rest.to_string();
                break;
            }
        }
        items.push((body, checked));
    }
    let is_task_list = items.first().is_some_and(|(_, checked)| checked.is_some());
    let nodes: Vec<Value> = items
        .into_iter()
        .map(|(body, checked)| {
            let mut content = parse_blocks(&body, image, depth + 1);
            if content
                .first()
                .is_none_or(|first| kind(first) != "paragraph")
            {
                content.insert(0, json!({ "type": "paragraph" }));
            }
            if is_task_list {
                json!({
                    "type": "taskItem",
                    "attrs": { "checked": checked.unwrap_or(false) },
                    "content": content,
                })
            } else {
                json!({ "type": "listItem", "content": content })
            }
        })
        .collect();
    let list = match (is_task_list, first_kind) {
        (true, _) => json!({ "type": "taskList", "content": nodes }),
        (false, ListKind::Bullet) => json!({ "type": "bulletList", "content": nodes }),
        (false, ListKind::Ordered(_)) => json!({
            "type": "orderedList",
            "attrs": { "start": ordered_start },
            "content": nodes,
        }),
    };
    (list, i)
}

/// One paragraph of text, or several blocks if images interrupt it.
pub(super) fn paragraph_nodes(
    lines: &[String],
    image: &mut dyn FnMut(&str) -> Option<ImageFile>,
) -> Vec<Value> {
    let mut inline = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let mut text = line.trim();
        if text.ends_with('\\') && !text.ends_with("\\\\") {
            text = &text[..text.len() - 1];
        }
        if index > 0 {
            inline.push(json!({ "type": "hardBreak" }));
        }
        inline.extend(parse_inline(text, &[], 0));
    }
    let inline = merge(inline);
    let mut blocks = Vec::new();
    let mut run: Vec<Value> = Vec::new();
    let flush = |run: &mut Vec<Value>, blocks: &mut Vec<Value>| {
        while run.first().is_some_and(|n| kind(n) == "hardBreak") {
            run.remove(0);
        }
        while run.last().is_some_and(|n| kind(n) == "hardBreak") {
            run.pop();
        }
        if !run.is_empty() {
            blocks.push(json!({ "type": "paragraph", "content": std::mem::take(run) }));
        }
    };
    for node in inline {
        if kind(&node) == "__image" {
            flush(&mut run, &mut blocks);
            let src = node.get("src").and_then(Value::as_str).unwrap_or("");
            if let Some(file) = image(src) {
                blocks.push(json!({
                    "type": "image",
                    "attrs": { "attachmentId": file.attachment_id, "mime": file.mime },
                }));
            }
        } else {
            run.push(node);
        }
    }
    flush(&mut run, &mut blocks);
    blocks
}

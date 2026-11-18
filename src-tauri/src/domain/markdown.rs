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

// ---------------------------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------------------------

/// The Markdown of a whole document, without a trailing newline.
pub fn to_markdown(doc: &Value, labels: &Labels<'_>) -> String {
    blocks_md(children(doc), labels)
}

fn blocks_md(blocks: &[Value], labels: &Labels<'_>) -> String {
    blocks
        .iter()
        .map(|block| block_md(block, labels))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn block_md(node: &Value, labels: &Labels<'_>) -> String {
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

fn code_block_md(node: &Value) -> String {
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

fn longest_run(text: &str, symbol: char) -> usize {
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

fn list_md(list: &Value, labels: &Labels<'_>) -> String {
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

fn table_md(table: &Value, labels: &Labels<'_>) -> String {
    let rows: Vec<Vec<String>> = children(table)
        .iter()
        .map(|row| {
            children(row)
                .iter()
                .map(|cell| {
                    let text = children(cell)
                        .iter()
                        .map(|block| inline_md(children(block), labels, "<br>"))
                        .collect::<Vec<_>>()
                        .join("<br>")
                        .replace('|', "\\|")
                        .replace('\n', "<br>");
                    if text.is_empty() {
                        " ".to_string()
                    } else {
                        text
                    }
                })
                .collect()
        })
        .collect();
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    if width == 0 {
        return String::new();
    }
    let line = |row: &Vec<String>| {
        let mut cells = row.clone();
        cells.resize(width, " ".to_string());
        format!("| {} |", cells.join(" | "))
    };
    let mut lines = vec![line(&rows[0])];
    lines.push(format!("|{}|", vec![" --- "; width].join("|")));
    lines.extend(rows[1..].iter().map(line));
    lines.join("\n")
}

fn inline_md(nodes: &[Value], labels: &Labels<'_>, line_break: &str) -> String {
    let mut out = String::new();
    for node in nodes {
        match kind(node) {
            "text" => {
                let text = node.get("text").and_then(Value::as_str).unwrap_or("");
                let marks: Vec<&Value> = node
                    .get("marks")
                    .and_then(Value::as_array)
                    .map(|marks| marks.iter().collect())
                    .unwrap_or_default();
                out.push_str(&marked(text, &marks));
            }
            "hardBreak" => out.push_str(line_break),
            "noteLink" => {
                let id = attr(node, "noteId").and_then(Value::as_str).unwrap_or("");
                out.push_str(&format!("[[{}]]", escape_text(&(labels.note_label)(id))));
            }
            _ => out.push_str(&inline_md(children(node), labels, line_break)),
        }
    }
    out
}

fn has_mark(marks: &[&Value], name: &str) -> bool {
    marks.iter().any(|mark| kind(mark) == name)
}

fn marked(text: &str, marks: &[&Value]) -> String {
    if text.is_empty() {
        return String::new();
    }
    let link = marks
        .iter()
        .find(|mark| kind(mark) == "link")
        .and_then(|mark| attr(mark, "href"))
        .and_then(Value::as_str);
    let core = if has_mark(marks, "code") {
        let fence = "`".repeat(longest_run(text, '`') + 1);
        let pad = if text.starts_with('`') || text.ends_with('`') {
            " "
        } else {
            ""
        };
        format!("{fence}{pad}{text}{pad}{fence}")
    } else {
        let trimmed_start = text.trim_start();
        let leading = &text[..text.len() - trimmed_start.len()];
        let body = trimmed_start.trim_end();
        let trailing = &trimmed_start[body.len()..];
        if body.is_empty() {
            return text.to_string();
        }
        let mut wrapped = escape_text(body);
        for (name, delimiter) in [
            ("strike", "~~"),
            ("highlight", "=="),
            ("italic", "*"),
            ("bold", "**"),
        ] {
            if has_mark(marks, name) {
                wrapped = format!("{delimiter}{wrapped}{delimiter}");
            }
        }
        // Markdown has no underline: the HTML tag is the usual way to write it.
        if has_mark(marks, "underline") {
            wrapped = format!("<u>{wrapped}</u>");
        }
        format!("{leading}{wrapped}{trailing}")
    };
    match link {
        Some(href) => {
            let trimmed_start = core.trim_start();
            let leading = &core[..core.len() - trimmed_start.len()];
            let body = trimmed_start.trim_end();
            let trailing = &trimmed_start[body.len()..];
            format!("{leading}[{body}]({}){trailing}", encode_url(href))
        }
        None => core,
    }
}

fn starts_with_tag(chars: &[char], tag: &str) -> bool {
    let wanted: Vec<char> = tag.chars().collect();
    chars.len() >= wanted.len()
        && chars
            .iter()
            .zip(&wanted)
            .all(|(found, expected)| found.eq_ignore_ascii_case(expected))
}

/// Index of the next `tag` at or after `from`.
fn find_tag(chars: &[char], from: usize, tag: &str) -> Option<usize> {
    (from..chars.len()).find(|&index| starts_with_tag(&chars[index..], tag))
}

fn encode_url(url: &str) -> String {
    url.replace(' ', "%20")
        .replace('(', "%28")
        .replace(')', "%29")
}

fn escape_text(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (index, &c) in chars.iter().enumerate() {
        let doubled_equals = c == '=' && chars.get(index + 1) == Some(&'=');
        if matches!(c, '\\' | '*' | '_' | '`' | '[' | ']' | '<' | '~') || doubled_equals {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// A paragraph that would read as a heading, list or quote gets its first character escaped.
fn escape_block_start(text: &str) -> String {
    let first_line = text.lines().next().unwrap_or("");
    let digits = first_line.chars().take_while(char::is_ascii_digit).count();
    let after_digits = first_line[digits..].chars().next();
    let looks_ordered = digits > 0 && matches!(after_digits, Some('.' | ')'));
    let needs = first_line.starts_with(['#', '>', '+', '-']) || looks_ordered;
    if !needs {
        return text.to_string();
    }
    let at = if looks_ordered { digits } else { 0 };
    format!("{}\\{}", &text[..at], &text[at..])
}

// ---------------------------------------------------------------------------------------------
// Reading
// ---------------------------------------------------------------------------------------------

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

fn unquote(value: &str) -> String {
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

fn parse_front_matter(lines: &[&str]) -> FrontMatter {
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

fn expand_tabs(line: &str) -> String {
    let leading = line.len() - line.trim_start_matches(['\t', ' ']).len();
    let mut out = String::with_capacity(line.len());
    for c in line[..leading].chars() {
        out.push_str(if c == '\t' { "    " } else { " " });
    }
    out.push_str(&line[leading..]);
    out
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

fn fence_of(line: &str) -> Option<(char, usize, &str)> {
    let trimmed = line.trim_start();
    if indent_of(line) > 3 {
        return None;
    }
    let symbol = trimmed.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let run = trimmed.chars().take_while(|c| *c == symbol).count();
    (run >= 3).then(|| (symbol, run, trimmed[run..].trim()))
}

fn heading_of(line: &str) -> Option<(usize, &str)> {
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

fn is_rule(line: &str) -> bool {
    let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    indent_of(line) <= 3
        && compact.len() >= 3
        && ['-', '*', '_']
            .iter()
            .any(|symbol| compact.chars().all(|c| c == *symbol))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ListKind {
    Bullet,
    Ordered(i64),
}

/// `(indent, kind, offset of the item's content)` if `line` opens a list item.
fn list_marker(line: &str) -> Option<(usize, ListKind, usize)> {
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

fn same_list(a: ListKind, b: ListKind) -> bool {
    matches!(
        (a, b),
        (ListKind::Bullet, ListKind::Bullet) | (ListKind::Ordered(_), ListKind::Ordered(_))
    )
}

fn split_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let trimmed = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut chars = trimmed.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'|') => {
                current.push_str("\\|");
                chars.next();
            }
            '|' => cells.push(std::mem::take(&mut current).trim().to_string()),
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        cells.push(current.trim().to_string());
    }
    cells
}

fn is_delimiter_row(line: &str) -> bool {
    line.contains('-') && {
        let cells = split_row(line);
        !cells.is_empty()
            && cells.iter().all(|cell| {
                let core = cell.trim_matches(':');
                !core.is_empty() && core.chars().all(|c| c == '-')
            })
    }
}

fn starts_table(lines: &[String], at: usize) -> bool {
    lines[at].contains('|') && lines.get(at + 1).is_some_and(|next| is_delimiter_row(next))
}

/// Whether `line` interrupts a paragraph.
fn interrupts_paragraph(line: &str, lines: &[String], at: usize) -> bool {
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

fn parse_blocks(
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

fn code_block(code: &str, language: &str) -> Value {
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

fn table_node(rows: Vec<(Vec<String>, bool)>, width: usize) -> Value {
    let rows: Vec<Value> = rows
        .into_iter()
        .map(|(mut cells, header)| {
            cells.resize(width, String::new());
            let cells: Vec<Value> = cells
                .iter()
                .map(|cell| {
                    let content = merge(parse_inline(cell, &[], 0));
                    let paragraph = if content.is_empty() {
                        json!({ "type": "paragraph" })
                    } else {
                        // An image inside a cell has no block to live in: it is dropped here.
                        let inline: Vec<Value> = content
                            .into_iter()
                            .filter(|node| kind(node) != "__image")
                            .collect();
                        json!({ "type": "paragraph", "content": inline })
                    };
                    json!({
                        "type": if header { "tableHeader" } else { "tableCell" },
                        "content": [paragraph],
                    })
                })
                .collect();
            json!({ "type": "tableRow", "content": cells })
        })
        .collect();
    json!({ "type": "table", "content": rows })
}

fn parse_list(
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
fn paragraph_nodes(
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

fn text_node(text: &str, marks: &[Value]) -> Value {
    let mut node = Map::new();
    node.insert("type".into(), json!("text"));
    node.insert("text".into(), json!(text));
    if !marks.is_empty() {
        node.insert("marks".into(), Value::Array(marks.to_vec()));
    }
    Value::Object(node)
}

/// Joins neighbouring text nodes that carry the same marks.
fn merge(nodes: Vec<Value>) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::new();
    for node in nodes {
        if let Some(last) = out.last_mut()
            && kind(last) == "text"
            && kind(&node) == "text"
            && last.get("marks") == node.get("marks")
        {
            let joined = format!(
                "{}{}",
                last.get("text").and_then(Value::as_str).unwrap_or(""),
                node.get("text").and_then(Value::as_str).unwrap_or("")
            );
            last["text"] = json!(joined);
            continue;
        }
        out.push(node);
    }
    out
}

fn closing_bracket(chars: &[char], open: usize) -> Option<usize> {
    let mut level = 0;
    let mut i = open;
    while i < chars.len() {
        match chars[i] {
            '\\' => i += 1,
            '[' => level += 1,
            ']' => {
                level -= 1;
                if level == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// `(target, index after the closing parenthesis)` of `(target "title")` starting at `open`.
fn link_target(chars: &[char], open: usize) -> Option<(String, usize)> {
    let mut level = 0;
    let mut i = open;
    while i < chars.len() {
        match chars[i] {
            '\\' => i += 1,
            '(' => level += 1,
            ')' => {
                level -= 1;
                if level == 0 {
                    let inside: String = chars[open + 1..i].iter().collect();
                    let inside = inside.trim();
                    let target = if let Some(rest) = inside.strip_prefix('<')
                        && let Some(end) = rest.find('>')
                    {
                        &rest[..end]
                    } else {
                        inside.split_whitespace().next().unwrap_or("")
                    };
                    return Some((target.to_string(), i + 1));
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn find_closing(chars: &[char], from: usize, delimiter: &[char]) -> Option<usize> {
    let mut i = from;
    while i + delimiter.len() <= chars.len() {
        if chars[i] == '\\' {
            i += 2;
            continue;
        }
        if chars[i] == '`' {
            // Code spans hide delimiters.
            let run = chars[i..].iter().take_while(|c| **c == '`').count();
            let mut j = i + run;
            let mut found = None;
            while j + run <= chars.len() {
                if chars[j..j + run].iter().all(|c| *c == '`') && chars.get(j + run) != Some(&'`') {
                    found = Some(j + run);
                    break;
                }
                j += 1;
            }
            i = found.unwrap_or(i + run);
            continue;
        }
        if chars[i..i + delimiter.len()] == *delimiter
            && i > from
            && !chars[i - 1].is_whitespace()
            && (delimiter.len() > 1
                || (chars.get(i + 1) != Some(&delimiter[0])
                    && (delimiter[0] != '_'
                        || !chars.get(i + 1).is_some_and(|c| c.is_alphanumeric()))))
        {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn add_mark(marks: &[Value], mark: Value) -> Vec<Value> {
    let mut next = marks.to_vec();
    if !next.iter().any(|existing| existing == &mark) {
        next.push(mark);
    }
    next
}

fn parse_inline(text: &str, marks: &[Value], depth: usize) -> Vec<Value> {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<Value> = Vec::new();
    let mut buffer = String::new();
    let mut i = 0;
    let flush = |buffer: &mut String, out: &mut Vec<Value>| {
        if !buffer.is_empty() {
            out.push(text_node(buffer, marks));
            buffer.clear();
        }
    };
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '\\' && next.is_some_and(|n| n.is_ascii_punctuation()) {
            buffer.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if c == '`' {
            let run = chars[i..].iter().take_while(|ch| **ch == '`').count();
            let mut j = i + run;
            let mut end = None;
            while j + run <= chars.len() {
                if chars[j..j + run].iter().all(|ch| *ch == '`')
                    && chars.get(j + run) != Some(&'`')
                    && (j == 0 || chars[j - 1] != '`')
                {
                    end = Some(j);
                    break;
                }
                j += 1;
            }
            if let Some(end) = end {
                flush(&mut buffer, &mut out);
                let mut code: String = chars[i + run..end].iter().collect();
                if code.len() > 2 && code.starts_with(' ') && code.ends_with(' ') {
                    code = code[1..code.len() - 1].to_string();
                }
                if !code.is_empty() {
                    out.push(text_node(&code, &[json!({ "type": "code" })]));
                }
                i = end + run;
            } else {
                buffer.push_str(&"`".repeat(run));
                i += run;
            }
            continue;
        }
        if c == '!'
            && next == Some('[')
            && let Some(close) = closing_bracket(&chars, i + 1)
            && chars.get(close + 1) == Some(&'(')
            && let Some((target, after)) = link_target(&chars, close + 1)
        {
            flush(&mut buffer, &mut out);
            out.push(json!({ "type": "__image", "src": target }));
            i = after;
            continue;
        }
        if c == '['
            && depth < MAX_INLINE_DEPTH
            && let Some(close) = closing_bracket(&chars, i)
            && chars.get(close + 1) == Some(&'(')
            && let Some((target, after)) = link_target(&chars, close + 1)
        {
            flush(&mut buffer, &mut out);
            let label: String = chars[i + 1..close].iter().collect();
            let inner = if is_allowed_link(&target) {
                add_mark(
                    marks,
                    json!({ "type": "link", "attrs": { "href": target } }),
                )
            } else {
                marks.to_vec()
            };
            out.extend(parse_inline(&label, &inner, depth + 1));
            i = after;
            continue;
        }
        if (c == '*' || c == '_') && next == Some(c) && depth < MAX_INLINE_DEPTH {
            let boundary_ok = c == '*' || i == 0 || !chars[i - 1].is_alphanumeric();
            if boundary_ok
                && chars.get(i + 2).is_some_and(|n| !n.is_whitespace())
                && let Some(end) = find_closing(&chars, i + 2, &[c, c])
            {
                flush(&mut buffer, &mut out);
                let inner: String = chars[i + 2..end].iter().collect();
                out.extend(parse_inline(
                    &inner,
                    &add_mark(marks, json!({ "type": "bold" })),
                    depth + 1,
                ));
                i = end + 2;
                continue;
            }
        }
        if (c == '*' || c == '_') && depth < MAX_INLINE_DEPTH {
            let boundary_ok = c == '*' || i == 0 || !chars[i - 1].is_alphanumeric();
            if boundary_ok
                && next.is_some_and(|n| !n.is_whitespace() && n != c)
                && let Some(end) = find_closing(&chars, i + 1, &[c])
            {
                flush(&mut buffer, &mut out);
                let inner: String = chars[i + 1..end].iter().collect();
                out.extend(parse_inline(
                    &inner,
                    &add_mark(marks, json!({ "type": "italic" })),
                    depth + 1,
                ));
                i = end + 1;
                continue;
            }
        }
        if (c == '=' || c == '~')
            && next == Some(c)
            && depth < MAX_INLINE_DEPTH
            && chars.get(i + 2).is_some_and(|n| !n.is_whitespace())
            && let Some(end) = find_closing(&chars, i + 2, &[c, c])
        {
            flush(&mut buffer, &mut out);
            let inner: String = chars[i + 2..end].iter().collect();
            let mark = if c == '=' { "highlight" } else { "strike" };
            let inner_marks = add_mark(marks, json!({ "type": mark }));
            out.extend(parse_inline(&inner, &inner_marks, depth + 1));
            i = end + 2;
            continue;
        }
        if c == '<'
            && depth < MAX_INLINE_DEPTH
            && starts_with_tag(&chars[i..], "<u>")
            && let Some(end) = find_tag(&chars, i + 3, "</u>")
        {
            flush(&mut buffer, &mut out);
            let inner: String = chars[i + 3..end].iter().collect();
            out.extend(parse_inline(
                &inner,
                &add_mark(marks, json!({ "type": "underline" })),
                depth + 1,
            ));
            i = end + 4;
            continue;
        }
        if c == '<' {
            let rest: String = chars[i..chars.len().min(i + 6)].iter().collect();
            let lower = rest.to_ascii_lowercase();
            let break_len = ["<br>", "<br/>", "<br />"]
                .iter()
                .find(|tag| lower.starts_with(**tag))
                .map(|tag| tag.len());
            if let Some(len) = break_len {
                flush(&mut buffer, &mut out);
                out.push(json!({ "type": "hardBreak" }));
                i += len;
                continue;
            }
            if let Some(end) = chars[i..].iter().position(|ch| *ch == '>') {
                let target: String = chars[i + 1..i + end].iter().collect();
                if is_allowed_link(&target) && !target.contains(char::is_whitespace) {
                    flush(&mut buffer, &mut out);
                    out.push(text_node(
                        &target,
                        &add_mark(
                            marks,
                            json!({ "type": "link", "attrs": { "href": target } }),
                        ),
                    ));
                    i += end + 1;
                    continue;
                }
            }
        }
        let at_word_start = i == 0 || !chars[i - 1].is_alphanumeric();
        if (c == 'h' || c == 'H') && at_word_start && marks.iter().all(|m| kind(m) != "link") {
            let rest: String = chars[i..].iter().collect();
            let lower = rest.to_ascii_lowercase();
            if lower.starts_with("http://") || lower.starts_with("https://") {
                let raw: String = rest
                    .chars()
                    .take_while(|ch| !ch.is_whitespace() && *ch != '<')
                    .collect();
                let url = raw.trim_end_matches(['.', ',', ';', ':', '!', '?', ')', '\'', '"']);
                if is_allowed_link(url) {
                    flush(&mut buffer, &mut out);
                    out.push(text_node(
                        url,
                        &add_mark(marks, json!({ "type": "link", "attrs": { "href": url } })),
                    ));
                    i += url.chars().count();
                    continue;
                }
            }
        }
        buffer.push(c);
        i += 1;
    }
    flush(&mut buffer, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels() -> Labels<'static> {
        Labels {
            image_path: &|id| Some(format!("adjuntos/{id}.png")),
            note_label: &|id| format!("nota {id}"),
            task_label: &|id| format!("tarea {id}"),
        }
    }

    fn write(doc: &Value) -> String {
        to_markdown(doc, &labels())
    }

    fn read(text: &str) -> Value {
        from_markdown(text, &mut |src| {
            src.ends_with(".png").then(|| ImageFile {
                attachment_id: "a".repeat(64),
                mime: "image/png".into(),
            })
        })
    }

    fn doc(blocks: Value) -> Value {
        json!({ "type": "doc", "content": blocks })
    }

    fn paragraph(text: &str) -> Value {
        json!({ "type": "paragraph", "content": [{ "type": "text", "text": text }] })
    }

    #[test]
    fn writes_headings_and_paragraphs_separated_by_a_blank_line() {
        let document = doc(json!([
            { "type": "heading", "attrs": { "level": 2 }, "content": [{ "type": "text", "text": "Título" }] },
            paragraph("Hola"),
        ]));
        assert_eq!(write(&document), "## Título\n\nHola");
    }

    #[test]
    fn writes_marks_keeping_white_space_outside_the_markers() {
        let document = doc(json!([{ "type": "paragraph", "content": [
            { "type": "text", "text": "a ", "marks": [] },
            { "type": "text", "text": "fuerte ", "marks": [{ "type": "bold" }] },
            { "type": "text", "text": "web", "marks": [{ "type": "link", "attrs": { "href": "https://x.es/a b" } }] },
            { "type": "text", "text": "x_y", "marks": [{ "type": "italic" }, { "type": "highlight" }] },
            { "type": "text", "text": "a`b", "marks": [{ "type": "code" }] },
        ]}]));
        assert_eq!(
            write(&document),
            "a **fuerte** [web](https://x.es/a%20b)*==x\\_y==*``a`b``"
        );
    }

    #[test]
    fn writes_nested_lists_and_task_lists() {
        let document = doc(json!([
            { "type": "bulletList", "content": [
                { "type": "listItem", "content": [paragraph("uno"),
                    { "type": "orderedList", "attrs": { "start": 3 }, "content": [
                        { "type": "listItem", "content": [paragraph("tres")] }] }] },
                { "type": "listItem", "content": [paragraph("dos")] }] },
            { "type": "taskList", "content": [
                { "type": "taskItem", "attrs": { "checked": true }, "content": [paragraph("hecho")] },
                { "type": "taskItem", "attrs": { "checked": false }, "content": [paragraph("falta")] }] },
        ]));
        assert_eq!(
            write(&document),
            "- uno\n  3. tres\n- dos\n\n- [x] hecho\n- [ ] falta"
        );
    }

    #[test]
    fn writes_quotes_code_rules_tables_images_and_references() {
        let cell = |kind: &str, text: &str| json!({ "type": kind, "content": [paragraph(text)] });
        let document = doc(json!([
            { "type": "blockquote", "content": [paragraph("cita")] },
            { "type": "codeBlock", "attrs": { "language": "rust" }, "content": [{ "type": "text", "text": "let a = 1;" }] },
            { "type": "horizontalRule" },
            { "type": "table", "content": [
                { "type": "tableRow", "content": [cell("tableHeader", "A"), cell("tableHeader", "B|C")] },
                { "type": "tableRow", "content": [cell("tableCell", "1"), cell("tableCell", "2")] }] },
            { "type": "image", "attrs": { "attachmentId": "abc" } },
            { "type": "taskRef", "attrs": { "taskId": "t1" } },
            { "type": "paragraph", "content": [{ "type": "noteLink", "attrs": { "noteId": "n1" } }] },
        ]));
        assert_eq!(
            write(&document),
            "> cita\n\n```rust\nlet a = 1;\n```\n\n---\n\n| A | B\\|C |\n| --- | --- |\n| 1 | 2 |\n\n![imagen](adjuntos/abc.png)\n\ntarea t1\n\n[[nota n1]]"
        );
    }

    #[test]
    fn a_paragraph_that_looks_like_a_list_is_escaped() {
        assert_eq!(
            write(&doc(json!([paragraph("- no es lista")]))),
            "\\- no es lista"
        );
        assert_eq!(write(&doc(json!([paragraph("1. no")]))), "1\\. no");
        assert_eq!(write(&doc(json!([paragraph("# no")]))), "\\# no");
    }

    #[test]
    fn a_missing_image_is_left_out() {
        let none = Labels {
            image_path: &|_| None,
            note_label: &|_| String::new(),
            task_label: &|_| String::new(),
        };
        let document =
            doc(json!([{ "type": "image", "attrs": { "attachmentId": "x" } }, paragraph("t")]));
        assert_eq!(to_markdown(&document, &none), "t");
    }

    #[test]
    fn reads_what_it_writes() {
        let original = doc(json!([
            { "type": "heading", "attrs": { "level": 3 }, "content": [{ "type": "text", "text": "T" }] },
            { "type": "paragraph", "content": [
                { "type": "text", "text": "a " },
                { "type": "text", "text": "b", "marks": [{ "type": "bold" }] },
                { "type": "text", "text": " c" }] },
            { "type": "bulletList", "content": [
                { "type": "listItem", "content": [paragraph("uno"),
                    { "type": "bulletList", "content": [
                        { "type": "listItem", "content": [paragraph("hijo")] }] }] },
                { "type": "listItem", "content": [paragraph("dos")] }] },
            { "type": "orderedList", "attrs": { "start": 1 }, "content": [
                { "type": "listItem", "content": [paragraph("x")] },
                { "type": "listItem", "content": [paragraph("y")] }] },
            { "type": "taskList", "content": [
                { "type": "taskItem", "attrs": { "checked": true }, "content": [paragraph("ok")] }] },
            { "type": "blockquote", "content": [paragraph("q")] },
            { "type": "codeBlock", "attrs": { "language": "js" }, "content": [{ "type": "text", "text": "a\n\nb" }] },
            { "type": "horizontalRule" },
        ]));
        assert_eq!(read(&write(&original)), original);
    }

    #[test]
    fn reads_inline_syntax() {
        let parsed = read("**a** *b* _c_ `d` ==e== [f](https://x.es) ~~g~~ snake_case_name");
        let content = parsed["content"][0]["content"]
            .as_array()
            .expect("inline")
            .clone();
        let texts: Vec<(String, Vec<String>)> = content
            .iter()
            .map(|node| {
                (
                    node["text"].as_str().unwrap_or("").to_string(),
                    node["marks"]
                        .as_array()
                        .map(|marks| marks.iter().map(|m| kind(m).to_string()).collect())
                        .unwrap_or_default(),
                )
            })
            .collect();
        assert_eq!(texts[0], ("a".into(), vec!["bold".into()]));
        assert_eq!(texts[2], ("b".into(), vec!["italic".into()]));
        assert_eq!(texts[4], ("c".into(), vec!["italic".into()]));
        assert_eq!(texts[6], ("d".into(), vec!["code".into()]));
        assert_eq!(texts[8], ("e".into(), vec!["highlight".into()]));
        assert_eq!(texts[10], ("f".into(), vec!["link".into()]));
        assert_eq!(texts[11], (" ".into(), vec![]));
        assert_eq!(texts[12], ("g".into(), vec!["strike".into()]));
        assert_eq!(texts[13], (" snake_case_name".into(), vec![]));
    }

    #[test]
    fn strike_and_underline_survive_a_round_trip() {
        let original = doc(json!([{ "type": "paragraph", "content": [
            { "type": "text", "text": "a ", "marks": [] },
            { "type": "text", "text": "tachado", "marks": [{ "type": "strike" }] },
            { "type": "text", "text": " y ", "marks": [] },
            { "type": "text", "text": "subrayado", "marks": [{ "type": "underline" }] },
            { "type": "text", "text": " ", "marks": [] },
            { "type": "text", "text": "ambos", "marks": [{ "type": "bold" }, { "type": "strike" }, { "type": "underline" }] },
        ]}]));
        let text = write(&original);
        assert_eq!(
            text,
            "a ~~tachado~~ y <u>subrayado</u> <u>**~~ambos~~**</u>"
        );
        let again = read(&text);
        let flat = |value: &Value| -> Vec<(String, Vec<String>)> {
            value["content"][0]["content"]
                .as_array()
                .map(|nodes| {
                    nodes
                        .iter()
                        .map(|node| {
                            let mut names: Vec<String> = node["marks"]
                                .as_array()
                                .map(|marks| marks.iter().map(|m| kind(m).to_string()).collect())
                                .unwrap_or_default();
                            names.sort();
                            (node["text"].as_str().unwrap_or("").to_string(), names)
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        assert_eq!(flat(&again), flat(&original));
    }

    #[test]
    fn a_literal_underline_tag_in_text_stays_text() {
        let original = doc(json!([paragraph("escribe <u>así</u> sin efecto")]));
        assert_eq!(read(&write(&original)), original);
    }

    #[test]
    fn unsafe_links_lose_their_target_and_keep_their_text() {
        let parsed = read("[clic](javascript:alert(1))");
        assert_eq!(parsed, doc(json!([paragraph("clic")])));
    }

    #[test]
    fn bare_urls_become_links() {
        let parsed = read("ver https://example.com/a.");
        let content = &parsed["content"][0]["content"];
        assert_eq!(content[1]["text"], "https://example.com/a");
        assert_eq!(
            content[1]["marks"][0]["attrs"]["href"],
            "https://example.com/a"
        );
        assert_eq!(content[2]["text"], ".");
    }

    #[test]
    fn reads_tables_with_missing_cells() {
        let parsed = read("| A | B |\n|:--|--:|\n| 1 |\n| 3 | 4 |");
        let rows = parsed["content"][0]["content"].as_array().expect("rows");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0]["content"][0]["type"], "tableHeader");
        assert_eq!(rows[1]["content"].as_array().map(Vec::len), Some(2));
        assert_eq!(
            rows[1]["content"][1]["content"][0],
            json!({ "type": "paragraph" })
        );
    }

    #[test]
    fn images_become_blocks_and_unusable_ones_are_dropped() {
        let parsed = read("antes ![x](foto.png) después\n\n![y](web.gif)");
        let blocks = parsed["content"].as_array().expect("blocks");
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[1]["type"], "image");
        assert_eq!(blocks[1]["attrs"]["mime"], "image/png");
    }

    #[test]
    fn single_line_breaks_inside_a_paragraph_are_kept() {
        let parsed = read("uno\ndos");
        assert_eq!(
            parsed["content"][0]["content"],
            json!([
                { "type": "text", "text": "uno" },
                { "type": "hardBreak" },
                { "type": "text", "text": "dos" }
            ])
        );
    }

    #[test]
    fn deeply_nested_input_does_not_overflow() {
        let hostile = format!("{}x", "> ".repeat(5000));
        let parsed = read(&hostile);
        assert_eq!(parsed["type"], "doc");
        let lists = format!("{}x", "- ".repeat(5000));
        assert_eq!(read(&lists)["type"], "doc");
    }

    #[test]
    fn front_matter_is_split_off() {
        let (data, body) = split_front_matter(
            "---\ntitle: \"Mi nota\"\ntags: [a, #b]\ncreated: 2026-01-02\n---\ncuerpo",
        );
        assert_eq!(data.title.as_deref(), Some("Mi nota"));
        assert_eq!(data.tags, ["a", "b"]);
        assert_eq!(data.created.as_deref(), Some("2026-01-02"));
        assert_eq!(body, "cuerpo");

        let (listed, _) = split_front_matter("---\ntags:\n  - x\n  - y\n---\n");
        assert_eq!(listed.tags, ["x", "y"]);

        let (none, body) = split_front_matter("---\nsin cierre\n");
        assert_eq!(none, FrontMatter::default());
        assert_eq!(body, "---\nsin cierre\n");
    }

    #[test]
    fn plain_text_keeps_paragraphs_and_lines() {
        let parsed = from_plain_text("uno\ndos\r\n\r\ntres\n");
        assert_eq!(parsed["content"].as_array().map(Vec::len), Some(2));
        assert_eq!(
            parsed["content"][0]["content"][1],
            json!({ "type": "hardBreak" })
        );
    }

    #[test]
    fn an_empty_document_has_no_blocks() {
        assert_eq!(read(""), doc(json!([])));
        assert_eq!(write(&doc(json!([]))), "");
    }
}

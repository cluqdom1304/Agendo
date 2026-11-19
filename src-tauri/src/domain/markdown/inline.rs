//! Text with marks in both directions: escaping, emphasis, links and images.

use super::write::longest_run;
use super::*;

pub(super) fn inline_md(nodes: &[Value], labels: &Labels<'_>, line_break: &str) -> String {
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

pub(super) fn has_mark(marks: &[&Value], name: &str) -> bool {
    marks.iter().any(|mark| kind(mark) == name)
}

pub(super) fn marked(text: &str, marks: &[&Value]) -> String {
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

pub(super) fn starts_with_tag(chars: &[char], tag: &str) -> bool {
    let wanted: Vec<char> = tag.chars().collect();
    chars.len() >= wanted.len()
        && chars
            .iter()
            .zip(&wanted)
            .all(|(found, expected)| found.eq_ignore_ascii_case(expected))
}

/// Index of the next `tag` at or after `from`.
pub(super) fn find_tag(chars: &[char], from: usize, tag: &str) -> Option<usize> {
    (from..chars.len()).find(|&index| starts_with_tag(&chars[index..], tag))
}

pub(super) fn encode_url(url: &str) -> String {
    url.replace(' ', "%20")
        .replace('(', "%28")
        .replace(')', "%29")
}

pub(super) fn escape_text(text: &str) -> String {
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
pub(super) fn escape_block_start(text: &str) -> String {
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

pub(super) fn text_node(text: &str, marks: &[Value]) -> Value {
    let mut node = Map::new();
    node.insert("type".into(), json!("text"));
    node.insert("text".into(), json!(text));
    if !marks.is_empty() {
        node.insert("marks".into(), Value::Array(marks.to_vec()));
    }
    Value::Object(node)
}

/// Joins neighbouring text nodes that carry the same marks.
pub(super) fn merge(nodes: Vec<Value>) -> Vec<Value> {
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

pub(super) fn closing_bracket(chars: &[char], open: usize) -> Option<usize> {
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
pub(super) fn link_target(chars: &[char], open: usize) -> Option<(String, usize)> {
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

pub(super) fn find_closing(chars: &[char], from: usize, delimiter: &[char]) -> Option<usize> {
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

pub(super) fn add_mark(marks: &[Value], mark: Value) -> Vec<Value> {
    let mut next = marks.to_vec();
    if !next.iter().any(|existing| existing == &mark) {
        next.push(mark);
    }
    next
}

pub(super) fn parse_inline(text: &str, marks: &[Value], depth: usize) -> Vec<Value> {
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

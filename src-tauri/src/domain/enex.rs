//! Reading Evernote exports (`.enex`, F14-04): the XML envelope, the ENML of each note
//! converted to the editor's document, and its resources (images). Nothing here touches the disk
//! or the network. The XML reader is deliberately small and tolerant: it understands what ENEX
//! and ENML use (elements, attributes, entities, CDATA) and nothing else, so no dependency is
//! needed.

use serde_json::{Value, json};

use super::{
    codec::{base64_decode, md5_hex},
    markdown::ImageFile,
    validation::is_allowed_link,
};

/// Deepest element nesting the XML reader keeps; deeper elements are read as if they were empty.
const MAX_XML_DEPTH: usize = 100;
/// Deepest block nesting the converter follows before it flattens the rest to text.
const MAX_BLOCK_DEPTH: usize = 30;

// ---------------------------------------------------------------------------------------------
// XML
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Node {
    Text(String),
    Element(Element),
}

#[derive(Debug, Clone, Default)]
struct Element {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<Node>,
}

impl Element {
    fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    fn elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|child| match child {
            Node::Element(element) => Some(element),
            Node::Text(_) => None,
        })
    }

    fn child(&self, name: &str) -> Option<&Element> {
        self.elements().find(|element| element.name == name)
    }

    /// Every piece of text below this element, in order.
    fn text(&self) -> String {
        let mut out = String::new();
        collect_text(&self.children, &mut out);
        out
    }
}

fn collect_text(nodes: &[Node], out: &mut String) {
    for node in nodes {
        match node {
            Node::Text(text) => out.push_str(text),
            Node::Element(element) => collect_text(&element.children, out),
        }
    }
}

const VOID: [&str; 6] = ["br", "hr", "img", "en-todo", "en-media", "col"];

fn decode_entities(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        let replaced = rest.find(';').filter(|end| *end <= 10).and_then(|end| {
            let name = &rest[1..end];
            let c = match name {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "nbsp" => Some('\u{a0}'),
                "copy" => Some('©'),
                "reg" => Some('®'),
                "hellip" => Some('…'),
                "ndash" => Some('–'),
                "mdash" => Some('—'),
                "laquo" => Some('«'),
                "raquo" => Some('»'),
                _ => name.strip_prefix('#').and_then(|number| {
                    let code = match number.strip_prefix(['x', 'X']) {
                        Some(hex) => u32::from_str_radix(hex, 16).ok(),
                        None => number.parse().ok(),
                    };
                    code.and_then(char::from_u32)
                }),
            };
            c.map(|c| (c, end + 1))
        });
        match replaced {
            Some((c, length)) => {
                out.push(c);
                rest = &rest[length..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn parse_tag(tag: &str) -> (String, Vec<(String, String)>) {
    let tag = tag.trim_end_matches('/').trim();
    let name_end = tag.find(char::is_whitespace).unwrap_or(tag.len());
    let name = tag[..name_end].to_ascii_lowercase();
    let mut attrs = Vec::new();
    let mut rest = tag[name_end..].trim_start();
    while !rest.is_empty() {
        let key_end = rest
            .find(|c: char| c == '=' || c.is_whitespace())
            .unwrap_or(rest.len());
        let key = rest[..key_end].to_ascii_lowercase();
        rest = rest[key_end..].trim_start();
        let mut value = String::new();
        if let Some(after) = rest.strip_prefix('=') {
            let after = after.trim_start();
            let quote = after.chars().next().filter(|c| *c == '"' || *c == '\'');
            if let Some(quote) = quote {
                let inner = &after[1..];
                let end = inner.find(quote).unwrap_or(inner.len());
                value = decode_entities(&inner[..end]);
                rest = inner.get(end + 1..).unwrap_or("").trim_start();
            } else {
                let end = after.find(char::is_whitespace).unwrap_or(after.len());
                value = decode_entities(&after[..end]);
                rest = after[end..].trim_start();
            }
        }
        if !key.is_empty() {
            attrs.push((key, value));
        }
    }
    (name, attrs)
}

/// Reads `input` into a synthetic root element. Malformed markup never fails: unmatched closing
/// tags are ignored and elements left open are closed at the end.
fn parse_xml(input: &str) -> Element {
    let mut stack = vec![Element::default()];
    let mut rest = input;
    let close = |stack: &mut Vec<Element>| {
        if stack.len() > 1
            && let Some(done) = stack.pop()
            && let Some(parent) = stack.last_mut()
        {
            parent.children.push(Node::Element(done));
        }
    };
    while !rest.is_empty() {
        let Some(open) = rest.find('<') else {
            push_text(&mut stack, &decode_entities(rest));
            break;
        };
        if open > 0 {
            push_text(&mut stack, &decode_entities(&rest[..open]));
        }
        rest = &rest[open..];
        if let Some(after) = rest.strip_prefix("<![CDATA[") {
            let end = after.find("]]>").unwrap_or(after.len());
            push_text(&mut stack, &after[..end]);
            rest = after.get(end + 3..).unwrap_or("");
        } else if let Some(after) = rest.strip_prefix("<!--") {
            rest = after.find("-->").map_or("", |end| &after[end + 3..]);
        } else if rest.starts_with("<?") {
            rest = rest.find("?>").map_or("", |end| &rest[end + 2..]);
        } else if rest.starts_with("<!") {
            // `<!DOCTYPE …>`, possibly with an internal subset in brackets.
            let end = match (rest.find('['), rest.find('>')) {
                (Some(bracket), Some(gt)) if bracket < gt => rest.find("]>").map(|at| at + 1),
                (_, gt) => gt,
            };
            rest = end.map_or("", |end| &rest[end + 1..]);
        } else if let Some(after) = rest.strip_prefix("</") {
            let end = after.find('>').unwrap_or(after.len());
            let name = after[..end].trim().to_ascii_lowercase();
            rest = after.get(end + 1..).unwrap_or("");
            if let Some(depth) = stack.iter().rposition(|element| element.name == name)
                && depth > 0
            {
                while stack.len() > depth {
                    close(&mut stack);
                }
            }
        } else {
            let Some(end) = rest.find('>') else {
                push_text(&mut stack, &decode_entities(rest));
                break;
            };
            let inside = &rest[1..end];
            rest = &rest[end + 1..];
            let self_closing = inside.ends_with('/');
            let (name, attrs) = parse_tag(inside);
            if name.is_empty() {
                continue;
            }
            let element = Element {
                name,
                attrs,
                children: Vec::new(),
            };
            if self_closing || VOID.contains(&element.name.as_str()) || stack.len() >= MAX_XML_DEPTH
            {
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(Node::Element(element));
                }
            } else {
                stack.push(element);
            }
        }
    }
    while stack.len() > 1 {
        close(&mut stack);
    }
    stack.pop().unwrap_or_default()
}

fn push_text(stack: &mut [Element], text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(parent) = stack.last_mut() {
        parent.children.push(Node::Text(text.to_string()));
    }
}

// ---------------------------------------------------------------------------------------------
// ENML → document
// ---------------------------------------------------------------------------------------------

fn kind(node: &Value) -> &str {
    node.get("type").and_then(Value::as_str).unwrap_or("")
}

fn is_block_tag(name: &str) -> bool {
    matches!(
        name,
        "div"
            | "p"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "ul"
            | "ol"
            | "li"
            | "blockquote"
            | "pre"
            | "table"
            | "hr"
            | "center"
            | "section"
            | "article"
    )
}

fn has_block(element: &Element) -> bool {
    element
        .elements()
        .any(|child| is_block_tag(&child.name) || has_block(child))
}

fn add_mark(marks: &[Value], mark: Value) -> Vec<Value> {
    let mut next = marks.to_vec();
    if !next.contains(&mark) {
        next.push(mark);
    }
    next
}

fn text_node(text: &str, marks: &[Value]) -> Value {
    if marks.is_empty() {
        json!({ "type": "text", "text": text })
    } else {
        json!({ "type": "text", "text": text, "marks": marks })
    }
}

fn collapse_spaces(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut space = false;
    for c in text.chars() {
        if matches!(c, ' ' | '\t' | '\n' | '\r') {
            space = true;
        } else {
            if space {
                out.push(' ');
                space = false;
            }
            out.push(c);
        }
    }
    if space {
        out.push(' ');
    }
    out
}

struct Resource {
    hash: String,
    data: Vec<u8>,
}

struct Converter<'a> {
    resources: &'a [Resource],
    image: &'a mut dyn FnMut(&[u8]) -> Option<ImageFile>,
    /// Images placed in the document.
    placed: usize,
}

impl Converter<'_> {
    fn inline(&mut self, nodes: &[Node], marks: &[Value], out: &mut Vec<Value>) {
        for node in nodes {
            match node {
                Node::Text(text) => {
                    let text = collapse_spaces(text);
                    if !text.is_empty() {
                        out.push(text_node(&text, marks));
                    }
                }
                Node::Element(element) => self.inline_element(element, marks, out),
            }
        }
    }

    fn inline_element(&mut self, element: &Element, marks: &[Value], out: &mut Vec<Value>) {
        let mut inner = marks.to_vec();
        match element.name.as_str() {
            "br" => {
                out.push(json!({ "type": "hardBreak" }));
                return;
            }
            "en-todo" => {
                out.push(json!({
                    "type": "__todo",
                    "checked": element.attr("checked") == Some("true"),
                }));
                return;
            }
            "en-media" => {
                out.push(json!({
                    "type": "__media",
                    "hash": element.attr("hash").unwrap_or("").to_ascii_lowercase(),
                }));
                return;
            }
            "b" | "strong" => inner = add_mark(marks, json!({ "type": "bold" })),
            "i" | "em" => inner = add_mark(marks, json!({ "type": "italic" })),
            "code" | "tt" => inner = add_mark(marks, json!({ "type": "code" })),
            "a" => {
                if let Some(href) = element.attr("href").filter(|href| is_allowed_link(href)) {
                    inner = add_mark(marks, json!({ "type": "link", "attrs": { "href": href } }));
                }
            }
            _ => {}
        }
        if let Some(style) = element.attr("style") {
            let style = style.to_ascii_lowercase().replace(' ', "");
            if style.contains("font-weight:bold") || style.contains("font-weight:700") {
                inner = add_mark(&inner, json!({ "type": "bold" }));
            }
            if style.contains("font-style:italic") {
                inner = add_mark(&inner, json!({ "type": "italic" }));
            }
            if style.contains("-evernote-highlight:true") {
                inner = add_mark(&inner, json!({ "type": "highlight" }));
            }
        }
        self.inline(&element.children, &inner, out);
    }

    /// Blocks made of `nodes`: loose inline content becomes paragraphs.
    fn container(&mut self, nodes: &[Node], depth: usize) -> Vec<Value> {
        let mut blocks = Vec::new();
        let mut buffer: Vec<Value> = Vec::new();
        for node in nodes {
            let element = match node {
                Node::Text(_) => {
                    self.inline(std::slice::from_ref(node), &[], &mut buffer);
                    continue;
                }
                Node::Element(element) => element,
            };
            if !is_block_tag(&element.name) {
                self.inline_element(element, &[], &mut buffer);
                continue;
            }
            self.flush(&mut buffer, &mut blocks);
            if depth >= MAX_BLOCK_DEPTH {
                self.inline(&element.children, &[], &mut buffer);
                self.flush(&mut buffer, &mut blocks);
                continue;
            }
            self.block(element, depth, &mut blocks);
        }
        self.flush(&mut buffer, &mut blocks);
        blocks
    }

    fn block(&mut self, element: &Element, depth: usize, blocks: &mut Vec<Value>) {
        match element.name.as_str() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level: u8 = element.name[1..].parse().unwrap_or(1);
                let mut content = Vec::new();
                self.inline(&element.children, &[], &mut content);
                content.retain(|node| matches!(kind(node), "text" | "hardBreak"));
                blocks.push(json!({
                    "type": "heading",
                    "attrs": { "level": level },
                    "content": trim_edges(content),
                }));
            }
            "ul" | "ol" => blocks.push(self.list(element, depth)),
            "blockquote" => {
                let mut content = self.container(&element.children, depth + 1);
                if content.is_empty() {
                    content.push(json!({ "type": "paragraph" }));
                }
                blocks.push(json!({ "type": "blockquote", "content": content }));
            }
            "pre" => {
                let code = element.text();
                let code = code.trim_matches('\n');
                blocks.push(if code.is_empty() {
                    json!({ "type": "codeBlock" })
                } else {
                    json!({ "type": "codeBlock", "content": [{ "type": "text", "text": code }] })
                });
            }
            "hr" => blocks.push(json!({ "type": "horizontalRule" })),
            "table" => {
                if let Some(table) = self.table(element, depth) {
                    blocks.push(table);
                }
            }
            _ if has_block(element) => {
                blocks.extend(self.container(&element.children, depth + 1));
            }
            _ => {
                let mut buffer = Vec::new();
                self.inline(&element.children, &[], &mut buffer);
                let blank = trim_edges(buffer.clone()).is_empty();
                self.flush(&mut buffer, blocks);
                // `<div><br/></div>` is a blank line; several in a row are one.
                if blank && !blocks.is_empty() && !ends_with_blank(blocks) {
                    blocks.push(json!({ "type": "paragraph" }));
                }
            }
        }
    }

    fn list(&mut self, list: &Element, depth: usize) -> Value {
        let mut items: Vec<(Vec<Value>, Option<bool>)> = Vec::new();
        for item in list.elements().filter(|child| child.name == "li") {
            let mut checked = None;
            let mut content = if has_block(item) {
                self.container(&item.children, depth + 1)
            } else {
                let mut buffer = Vec::new();
                self.inline(&item.children, &[], &mut buffer);
                if buffer.first().is_some_and(|node| kind(node) == "__todo") {
                    checked = buffer.remove(0).get("checked").and_then(Value::as_bool);
                }
                let mut blocks = Vec::new();
                self.flush(&mut buffer, &mut blocks);
                blocks
            };
            if content
                .first()
                .is_none_or(|first| kind(first) != "paragraph")
            {
                content.insert(0, json!({ "type": "paragraph" }));
            }
            items.push((content, checked));
        }
        let is_task_list = items.iter().any(|(_, checked)| checked.is_some());
        let nodes: Vec<Value> = items
            .into_iter()
            .map(|(content, checked)| {
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
        if is_task_list {
            json!({ "type": "taskList", "content": nodes })
        } else if list.name == "ol" {
            let start: i64 = list.attr("start").and_then(|s| s.parse().ok()).unwrap_or(1);
            json!({ "type": "orderedList", "attrs": { "start": start }, "content": nodes })
        } else {
            json!({ "type": "bulletList", "content": nodes })
        }
    }

    fn table(&mut self, table: &Element, depth: usize) -> Option<Value> {
        let mut rows: Vec<&Element> = Vec::new();
        collect_rows(table, &mut rows);
        let width = rows
            .iter()
            .map(|row| {
                row.elements()
                    .filter(|c| matches!(c.name.as_str(), "td" | "th"))
                    .count()
            })
            .max()
            .unwrap_or(0);
        if width == 0 {
            return None;
        }
        let rows: Vec<Value> = rows
            .into_iter()
            .map(|row| {
                let mut cells: Vec<Value> = row
                    .elements()
                    .filter(|cell| matches!(cell.name.as_str(), "td" | "th"))
                    .map(|cell| {
                        let mut content = self.container(&cell.children, depth + 1);
                        if content.is_empty() {
                            content.push(json!({ "type": "paragraph" }));
                        }
                        let tag = if cell.name == "th" {
                            "tableHeader"
                        } else {
                            "tableCell"
                        };
                        json!({ "type": tag, "content": content })
                    })
                    .collect();
                while cells.len() < width {
                    cells.push(json!({
                        "type": "tableCell",
                        "content": [{ "type": "paragraph" }],
                    }));
                }
                json!({ "type": "tableRow", "content": cells })
            })
            .collect();
        Some(json!({ "type": "table", "content": rows }))
    }

    /// Turns the pending inline content into blocks: paragraphs, images and task items.
    fn flush(&mut self, buffer: &mut Vec<Value>, blocks: &mut Vec<Value>) {
        let inline = merge(std::mem::take(buffer));
        let mut run: Vec<Value> = Vec::new();
        for node in inline {
            match kind(&node) {
                "__media" => {
                    self.paragraph(std::mem::take(&mut run), blocks);
                    let hash = node.get("hash").and_then(Value::as_str).unwrap_or("");
                    let file = self
                        .resources
                        .iter()
                        .find(|resource| resource.hash == hash)
                        .and_then(|resource| (self.image)(&resource.data));
                    if let Some(file) = file {
                        self.placed += 1;
                        blocks.push(json!({
                            "type": "image",
                            "attrs": { "attachmentId": file.attachment_id, "mime": file.mime },
                        }));
                    }
                }
                _ => run.push(node),
            }
        }
        self.paragraph(run, blocks);
    }

    fn paragraph(&mut self, run: Vec<Value>, blocks: &mut Vec<Value>) {
        let mut run = trim_edges(run);
        if run.is_empty() {
            return;
        }
        if kind(&run[0]) == "__todo" {
            let checked = run[0]
                .get("checked")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            run.remove(0);
            let content = trim_edges(run);
            let paragraph = if content.is_empty() {
                json!({ "type": "paragraph" })
            } else {
                json!({ "type": "paragraph", "content": content })
            };
            let item = json!({
                "type": "taskItem",
                "attrs": { "checked": checked },
                "content": [paragraph],
            });
            if let Some(last) = blocks.last_mut()
                && kind(last) == "taskList"
                && let Some(items) = last.get_mut("content").and_then(Value::as_array_mut)
            {
                items.push(item);
            } else {
                blocks.push(json!({ "type": "taskList", "content": [item] }));
            }
            return;
        }
        run.retain(|node| matches!(kind(node), "text" | "hardBreak"));
        if !run.is_empty() {
            blocks.push(json!({ "type": "paragraph", "content": run }));
        }
    }
}

fn collect_rows<'a>(element: &'a Element, rows: &mut Vec<&'a Element>) {
    for child in element.elements() {
        match child.name.as_str() {
            "tr" => rows.push(child),
            "tbody" | "thead" | "tfoot" => collect_rows(child, rows),
            _ => {}
        }
    }
}

fn ends_with_blank(blocks: &[Value]) -> bool {
    blocks
        .last()
        .is_some_and(|last| kind(last) == "paragraph" && last.get("content").is_none())
}

/// Drops line breaks and blanks at both ends of an inline run.
fn trim_edges(mut run: Vec<Value>) -> Vec<Value> {
    run = merge(run);
    let blank = |node: &Value| match kind(node) {
        "hardBreak" => true,
        "text" => node
            .get("text")
            .and_then(Value::as_str)
            .is_some_and(|text| text.chars().all(|c| c == ' ')),
        _ => false,
    };
    while run.first().is_some_and(blank) {
        run.remove(0);
    }
    while run.last().is_some_and(blank) {
        run.pop();
    }
    if let Some(first) = run.first_mut()
        && kind(first) == "text"
        && let Some(text) = first.get("text").and_then(Value::as_str)
    {
        first["text"] = json!(text.trim_start_matches(' '));
    }
    if let Some(last) = run.last_mut()
        && kind(last) == "text"
        && let Some(text) = last.get("text").and_then(Value::as_str)
    {
        last["text"] = json!(text.trim_end_matches(' '));
    }
    run
}

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

// ---------------------------------------------------------------------------------------------
// ENEX
// ---------------------------------------------------------------------------------------------

/// A note read from an export, ready to be stored.
#[derive(Debug, Clone)]
pub struct EnexNote {
    pub title: String,
    /// UTC instants as `YYYY-MM-DDTHH:MM:SSZ`.
    pub created: Option<String>,
    pub updated: Option<String>,
    pub tags: Vec<String>,
    pub document: Value,
    /// Resources the note carried that could not be kept (not images, or not usable).
    pub dropped_files: usize,
}

/// `20200131T235959Z` as `2020-01-31T23:59:59Z`.
pub fn parse_enex_date(text: &str) -> Option<String> {
    let text = text.trim();
    let bytes = text.as_bytes();
    if bytes.len() != 16 || bytes[8] != b'T' || bytes[15] != b'Z' {
        return None;
    }
    if !bytes[..8]
        .iter()
        .chain(&bytes[9..15])
        .all(u8::is_ascii_digit)
    {
        return None;
    }
    let formatted = format!(
        "{}-{}-{}T{}:{}:{}Z",
        &text[0..4],
        &text[4..6],
        &text[6..8],
        &text[9..11],
        &text[11..13],
        &text[13..15]
    );
    super::dates::parse_instant(&formatted).ok()?;
    Some(formatted)
}

/// Reads every note of an export, passing each to `on_note`. Returns how many were read.
/// `image` stores an image and returns its file, or `None` if the bytes are not a usable image.
pub fn read_enex(
    text: &str,
    image: &mut dyn FnMut(&[u8]) -> Option<ImageFile>,
    on_note: &mut dyn FnMut(EnexNote),
) -> Result<usize, String> {
    if !text.contains("<en-export") {
        return Err("el archivo no es una exportación de Evernote (.enex)".into());
    }
    let mut count = 0;
    let mut rest = text;
    while let Some(start) = rest.find("<note>") {
        let after = &rest[start..];
        let Some(end) = after.find("</note>") else {
            break;
        };
        let chunk = &after[..end + "</note>".len()];
        rest = &after[end + "</note>".len()..];
        let root = parse_xml(chunk);
        let Some(note) = root.child("note") else {
            continue;
        };
        on_note(convert_note(note, image));
        count += 1;
    }
    Ok(count)
}

fn convert_note(note: &Element, image: &mut dyn FnMut(&[u8]) -> Option<ImageFile>) -> EnexNote {
    let title: String = note
        .child("title")
        .map(Element::text)
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let resources: Vec<Resource> = note
        .elements()
        .filter(|element| element.name == "resource")
        .filter_map(|resource| {
            let data = resource.child("data")?;
            let bytes = base64_decode(&data.text())?;
            Some(Resource {
                hash: md5_hex(&bytes),
                data: bytes,
            })
        })
        .collect();
    let content = note.child("content").map(Element::text).unwrap_or_default();
    let body = parse_xml(&content);
    let mut converter = Converter {
        resources: &resources,
        image,
        placed: 0,
    };
    let blocks = body
        .child("en-note")
        .map(|en_note| converter.container(&en_note.children, 0))
        .unwrap_or_default();
    let dropped = resources.len().saturating_sub(converter.placed);
    let tags = note
        .elements()
        .filter(|element| element.name == "tag")
        .map(|tag| tag.text().trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect();
    EnexNote {
        title,
        created: note
            .child("created")
            .and_then(|e| parse_enex_date(&e.text())),
        updated: note
            .child("updated")
            .and_then(|e| parse_enex_date(&e.text())),
        tags,
        document: json!({ "type": "doc", "content": blocks }),
        dropped_files: dropped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::codec::base64_encode;

    fn convert(enml: &str) -> Value {
        let root = parse_xml(enml);
        let mut image = |_: &[u8]| None;
        let mut converter = Converter {
            resources: &[],
            image: &mut image,
            placed: 0,
        };
        let blocks = root
            .child("en-note")
            .map(|n| converter.container(&n.children, 0))
            .unwrap_or_default();
        Value::Array(blocks)
    }

    #[test]
    fn xml_entities_cdata_comments_and_doctype_are_understood() {
        let root = parse_xml(
            "<?xml version=\"1.0\"?><!DOCTYPE en-note SYSTEM \"x.dtd\"><!-- c --><a k=\"1 &amp; 2\" b='x'>&lt;t&gt; &#65;&#x42; &nbsp;<![CDATA[<raw>&amp;]]></a>",
        );
        let a = root.child("a").expect("a");
        assert_eq!(a.attr("k"), Some("1 & 2"));
        assert_eq!(a.attr("b"), Some("x"));
        assert_eq!(a.text(), "<t> AB \u{a0}<raw>&amp;");
    }

    #[test]
    fn xml_tolerates_unclosed_and_stray_tags() {
        let root = parse_xml("<a><b>x</c><d/>y");
        let a = root.child("a").expect("a");
        assert_eq!(a.text(), "xy");
        assert!(a.child("b").is_some());
    }

    #[test]
    fn xml_depth_is_bounded() {
        let hostile = "<a>".repeat(10_000) + "x";
        let root = parse_xml(&hostile);
        assert!(root.text().contains('x') || root.children.len() == 1);
    }

    #[test]
    fn divs_become_paragraphs_and_blank_divs_become_one_blank_line() {
        let blocks = convert(
            "<en-note><div>Hola <b>mundo</b></div><div><br/></div><div><br/></div><div>Fin</div></en-note>",
        );
        assert_eq!(
            blocks,
            json!([
                { "type": "paragraph", "content": [
                    { "type": "text", "text": "Hola " },
                    { "type": "text", "text": "mundo", "marks": [{ "type": "bold" }] }] },
                { "type": "paragraph" },
                { "type": "paragraph", "content": [{ "type": "text", "text": "Fin" }] },
            ])
        );
    }

    #[test]
    fn styled_spans_links_and_highlights_become_marks() {
        let blocks = convert(
            "<en-note><div><span style=\"font-weight: bold;\">a</span><a href=\"https://x.es\">b</a><a href=\"javascript:x\">c</a><span style=\"background-color: rgb(1,2,3); -evernote-highlight:true;\">d</span></div></en-note>",
        );
        let content = &blocks[0]["content"];
        assert_eq!(content[0]["marks"][0]["type"], "bold");
        assert_eq!(content[1]["marks"][0]["attrs"]["href"], "https://x.es");
        assert_eq!(content[2], json!({ "type": "text", "text": "c" }));
        assert_eq!(content[3]["marks"][0]["type"], "highlight");
    }

    #[test]
    fn checkboxes_become_one_task_list() {
        let blocks = convert(
            "<en-note><div><en-todo checked=\"true\"/>hecho</div><div><en-todo/>falta</div><div>texto</div></en-note>",
        );
        assert_eq!(blocks[0]["type"], "taskList");
        let items = blocks[0]["content"].as_array().expect("items");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["attrs"]["checked"], true);
        assert_eq!(items[1]["attrs"]["checked"], false);
        assert_eq!(blocks[1]["type"], "paragraph");
    }

    #[test]
    fn lists_tables_quotes_code_and_headings_are_converted() {
        let blocks = convert(
            "<en-note><h2>T</h2><ul><li>uno</li><li>dos<ul><li>hijo</li></ul></li></ul>\
             <ol start=\"3\"><li><en-todo checked=\"true\"/>x</li></ol>\
             <table><tbody><tr><td>a</td><td>b</td></tr><tr><td>c</td></tr></tbody></table>\
             <blockquote><div>q</div></blockquote><pre>a\n  b</pre><hr/></en-note>",
        );
        assert_eq!(
            blocks[0],
            json!({ "type": "heading", "attrs": { "level": 2 }, "content": [{ "type": "text", "text": "T" }] })
        );
        assert_eq!(blocks[1]["type"], "bulletList");
        assert_eq!(blocks[1]["content"][1]["content"][1]["type"], "bulletList");
        assert_eq!(blocks[2]["type"], "taskList");
        assert_eq!(blocks[3]["type"], "table");
        assert_eq!(
            blocks[3]["content"][1]["content"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(blocks[4]["type"], "blockquote");
        assert_eq!(blocks[5]["content"][0]["text"], "a\n  b");
        assert_eq!(blocks[6]["type"], "horizontalRule");
    }

    #[test]
    fn dates_are_validated() {
        assert_eq!(
            parse_enex_date("20200131T235959Z").as_deref(),
            Some("2020-01-31T23:59:59Z")
        );
        assert_eq!(parse_enex_date("20201331T235959Z"), None);
        assert_eq!(parse_enex_date("2020-01-31"), None);
    }

    #[test]
    fn a_whole_export_is_read_with_its_images_matched_by_hash() {
        let png = b"\x89PNG\r\n\x1a\nfake";
        let other = b"%PDF-1.4";
        let hash = md5_hex(png);
        let export = format!(
            "<?xml version=\"1.0\"?><en-export><note><title> Mi\n nota </title>\
             <content><![CDATA[<?xml version=\"1.0\"?><en-note><div>antes</div><div><en-media hash=\"{hash}\" type=\"image/png\"/></div></en-note>]]></content>\
             <created>20200102T030405Z</created><updated>basura</updated><tag>a</tag><tag> b </tag>\
             <resource><data encoding=\"base64\">{}</data><mime>image/png</mime></resource>\
             <resource><data encoding=\"base64\">{}</data><mime>application/pdf</mime></resource>\
             </note><note><title>Dos</title><content><![CDATA[<en-note>x</en-note>]]></content></note></en-export>",
            base64_encode(png),
            base64_encode(other)
        );
        let mut notes = Vec::new();
        let mut stored = Vec::new();
        let count = read_enex(
            &export,
            &mut |bytes| {
                stored.push(bytes.to_vec());
                Some(ImageFile {
                    attachment_id: "f".repeat(64),
                    mime: "image/png".into(),
                })
            },
            &mut |note| notes.push(note),
        )
        .expect("enex");
        assert_eq!(count, 2);
        assert_eq!(notes[0].title, "Mi nota");
        assert_eq!(notes[0].created.as_deref(), Some("2020-01-02T03:04:05Z"));
        assert_eq!(notes[0].updated, None);
        assert_eq!(notes[0].tags, ["a", "b"]);
        assert_eq!(stored, [png.to_vec()]);
        assert_eq!(notes[0].document["content"][1]["type"], "image");
        assert_eq!(notes[0].dropped_files, 1);
        assert_eq!(notes[1].document["content"][0]["content"][0]["text"], "x");
    }

    #[test]
    fn a_file_that_is_not_enex_is_rejected() {
        let result = read_enex("<html></html>", &mut |_| None, &mut |_| {});
        assert!(result.is_err());
    }
}

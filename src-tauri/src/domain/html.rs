//! HTML export of a note (F14-01): one self-contained page, with no script and no remote
//! resource. Text is always escaped; only `http`, `https` and `mailto` links are written.

use serde_json::Value;

use super::validation::is_allowed_link;

/// What the writer needs to know about the world outside the document.
pub struct Labels<'a> {
    /// Value for the `src` of an image (a `data:` URI), `None` if the file is not available.
    pub image_src: &'a dyn Fn(&str) -> Option<String>,
    pub note_label: &'a dyn Fn(&str) -> String,
    pub task_label: &'a dyn Fn(&str) -> String,
}

const STYLE: &str = "body{font:16px/1.6 system-ui,sans-serif;max-width:46rem;margin:2rem auto;padding:0 1rem;color:#1f2328}\
img{max-width:100%}pre{background:#f3f4f6;padding:.75rem;overflow:auto}code{background:#f3f4f6;padding:0 .2em}\
blockquote{margin-left:0;padding-left:1rem;border-left:3px solid #c9ced6;color:#555}\
table{border-collapse:collapse}td,th{border:1px solid #c9ced6;padding:.3rem .6rem}\
ul.tasks{list-style:none;padding-left:1rem}mark{background:#fff3a3}";

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

pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// A complete HTML page with `title` as its heading.
pub fn to_html(doc: &Value, title: &str, labels: &Labels<'_>) -> String {
    let heading = escape(title);
    format!(
        "<!DOCTYPE html>\n<html lang=\"es\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{heading}</title>\n<style>{STYLE}</style>\n</head>\n<body>\n<h1>{heading}</h1>\n{}\n</body>\n</html>\n",
        blocks(children(doc), labels)
    )
}

fn blocks(nodes: &[Value], labels: &Labels<'_>) -> String {
    nodes
        .iter()
        .map(|node| block(node, labels))
        .collect::<Vec<_>>()
        .join("\n")
}

fn block(node: &Value, labels: &Labels<'_>) -> String {
    match kind(node) {
        "paragraph" => format!("<p>{}</p>", inline(children(node), labels)),
        "heading" => {
            // The page title is the only `h1`: a note's own headings start at `h2`.
            let level = attr(node, "level")
                .and_then(Value::as_u64)
                .unwrap_or(1)
                .clamp(1, 5)
                + 1;
            format!("<h{level}>{}</h{level}>", inline(children(node), labels))
        }
        "bulletList" => format!("<ul>\n{}\n</ul>", items(node, labels)),
        "orderedList" => {
            let start = attr(node, "start").and_then(Value::as_i64).unwrap_or(1);
            format!("<ol start=\"{start}\">\n{}\n</ol>", items(node, labels))
        }
        "taskList" => format!("<ul class=\"tasks\">\n{}\n</ul>", items(node, labels)),
        "blockquote" => format!(
            "<blockquote>\n{}\n</blockquote>",
            blocks(children(node), labels)
        ),
        "codeBlock" => {
            let code: String = children(node)
                .iter()
                .filter_map(|child| child.get("text").and_then(Value::as_str))
                .collect();
            format!("<pre><code>{}</code></pre>", escape(&code))
        }
        "horizontalRule" => "<hr>".to_string(),
        "table" => table(node, labels),
        "image" => attr(node, "attachmentId")
            .and_then(Value::as_str)
            .and_then(|id| (labels.image_src)(id))
            .map(|src| format!("<p><img src=\"{}\" alt=\"\"></p>", escape(&src)))
            .unwrap_or_default(),
        "taskRef" => {
            let id = attr(node, "taskId").and_then(Value::as_str).unwrap_or("");
            format!("<p>{}</p>", escape(&(labels.task_label)(id)))
        }
        _ => inline(children(node), labels),
    }
}

fn items(list: &Value, labels: &Labels<'_>) -> String {
    let task = kind(list) == "taskList";
    children(list)
        .iter()
        .map(|item| {
            let checkbox = if task {
                let checked = attr(item, "checked").and_then(Value::as_bool) == Some(true);
                format!(
                    "<input type=\"checkbox\" disabled{}> ",
                    if checked { " checked" } else { "" }
                )
            } else {
                String::new()
            };
            // A paragraph inside an item is written without its `<p>`, as editors show it.
            let body = children(item)
                .iter()
                .map(|child| match kind(child) {
                    "paragraph" => inline(children(child), labels),
                    _ => block(child, labels),
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("<li>{checkbox}{body}</li>")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn table(node: &Value, labels: &Labels<'_>) -> String {
    let rows: Vec<String> = children(node)
        .iter()
        .map(|row| {
            let cells: String = children(row)
                .iter()
                .map(|cell| {
                    let tag = if kind(cell) == "tableHeader" {
                        "th"
                    } else {
                        "td"
                    };
                    let body = children(cell)
                        .iter()
                        .map(|paragraph| inline(children(paragraph), labels))
                        .collect::<Vec<_>>()
                        .join("<br>");
                    format!("<{tag}>{body}</{tag}>")
                })
                .collect();
            format!("<tr>{cells}</tr>")
        })
        .collect();
    format!("<table>\n{}\n</table>", rows.join("\n"))
}

fn inline(nodes: &[Value], labels: &Labels<'_>) -> String {
    let mut out = String::new();
    for node in nodes {
        match kind(node) {
            "text" => {
                let text = escape(node.get("text").and_then(Value::as_str).unwrap_or(""));
                let marks = node
                    .get("marks")
                    .and_then(Value::as_array)
                    .map_or(&[][..], Vec::as_slice);
                out.push_str(&wrap(text, marks));
            }
            "hardBreak" => out.push_str("<br>"),
            "noteLink" => {
                let id = attr(node, "noteId").and_then(Value::as_str).unwrap_or("");
                out.push_str(&escape(&(labels.note_label)(id)));
            }
            _ => out.push_str(&inline(children(node), labels)),
        }
    }
    out
}

fn wrap(mut html: String, marks: &[Value]) -> String {
    for (name, tag) in [
        ("code", "code"),
        ("strike", "s"),
        ("underline", "u"),
        ("highlight", "mark"),
        ("italic", "em"),
        ("bold", "strong"),
    ] {
        if marks.iter().any(|mark| kind(mark) == name) {
            html = format!("<{tag}>{html}</{tag}>");
        }
    }
    let href = marks
        .iter()
        .find(|mark| kind(mark) == "link")
        .and_then(|mark| attr(mark, "href"))
        .and_then(Value::as_str)
        .filter(|href| is_allowed_link(href));
    match href {
        Some(href) => format!(
            "<a href=\"{}\" rel=\"noopener noreferrer\">{html}</a>",
            escape(href)
        ),
        None => html,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn labels() -> Labels<'static> {
        Labels {
            image_src: &|id| (id == "img").then(|| "data:image/png;base64,AAAA".to_string()),
            note_label: &|id| format!("nota {id}"),
            task_label: &|id| format!("tarea {id}"),
        }
    }

    fn paragraph(text: &str) -> Value {
        json!({ "type": "paragraph", "content": [{ "type": "text", "text": text }] })
    }

    #[test]
    fn the_page_escapes_the_title_and_every_text() {
        let doc =
            json!({ "type": "doc", "content": [paragraph("<script>alert(\"x\")</script> & más")] });
        let page = to_html(&doc, "A <b>&</b>", &labels());
        assert!(page.contains("<title>A &lt;b&gt;&amp;&lt;/b&gt;</title>"));
        assert!(
            page.contains("<p>&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt; &amp; más</p>")
        );
        assert!(!page.contains("<script>"));
    }

    #[test]
    fn marks_and_safe_links_are_written_and_unsafe_ones_dropped() {
        let doc = json!({ "type": "doc", "content": [{ "type": "paragraph", "content": [
            { "type": "text", "text": "a", "marks": [{ "type": "bold" }, { "type": "italic" }] },
            { "type": "text", "text": "web", "marks": [{ "type": "link", "attrs": { "href": "https://x.es/?a=1&b=\"2\"" } }] },
            { "type": "text", "text": "mal", "marks": [{ "type": "link", "attrs": { "href": "javascript:alert(1)" } }] },
        ]}]});
        let page = to_html(&doc, "t", &labels());
        assert!(page.contains("<strong><em>a</em></strong>"));
        let marked = json!({ "type": "doc", "content": [{ "type": "paragraph", "content": [
            { "type": "text", "text": "x", "marks": [{ "type": "strike" }, { "type": "underline" }] },
        ]}]});
        assert!(to_html(&marked, "t", &labels()).contains("<u><s>x</s></u>"));
        assert!(page.contains(
            "<a href=\"https://x.es/?a=1&amp;b=&quot;2&quot;\" rel=\"noopener noreferrer\">web</a>"
        ));
        assert!(page.contains("mal</p>"));
        assert!(!page.contains("javascript"));
    }

    #[test]
    fn blocks_become_their_elements() {
        let cell = |kind: &str, text: &str| json!({ "type": kind, "content": [paragraph(text)] });
        let doc = json!({ "type": "doc", "content": [
            { "type": "heading", "attrs": { "level": 1 }, "content": [{ "type": "text", "text": "H" }] },
            { "type": "taskList", "content": [
                { "type": "taskItem", "attrs": { "checked": true }, "content": [paragraph("ok")] }] },
            { "type": "orderedList", "attrs": { "start": 4 }, "content": [
                { "type": "listItem", "content": [paragraph("x")] }] },
            { "type": "codeBlock", "content": [{ "type": "text", "text": "a<b" }] },
            { "type": "table", "content": [{ "type": "tableRow", "content": [cell("tableHeader", "A"), cell("tableCell", "B")] }] },
            { "type": "image", "attrs": { "attachmentId": "img" } },
            { "type": "image", "attrs": { "attachmentId": "gone" } },
            { "type": "horizontalRule" },
        ]});
        let page = to_html(&doc, "t", &labels());
        assert!(page.contains("<h2>H</h2>"));
        assert!(page.contains("<li><input type=\"checkbox\" disabled checked> ok</li>"));
        assert!(page.contains("<ol start=\"4\">"));
        assert!(page.contains("<pre><code>a&lt;b</code></pre>"));
        assert!(page.contains("<tr><th>A</th><td>B</td></tr>"));
        assert!(page.contains("<img src=\"data:image/png;base64,AAAA\" alt=\"\">"));
        assert_eq!(page.matches("<img").count(), 1);
        assert!(page.contains("<hr>"));
    }
}

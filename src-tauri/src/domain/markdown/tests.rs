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
    for unit in ["**", "*", "~~", "[", "![", "`"] {
        let inline = format!("{}x", unit.repeat(5000));
        assert_eq!(read(&inline)["type"], "doc", "inline unit {unit}");
    }
    let links = format!("{}x{}", "[".repeat(3000), "](u)".repeat(3000));
    assert_eq!(read(&links)["type"], "doc");
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

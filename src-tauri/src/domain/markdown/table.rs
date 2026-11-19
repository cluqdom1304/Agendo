//! Tables in both directions.

use super::inline::*;
use super::*;

pub(super) fn table_md(table: &Value, labels: &Labels<'_>) -> String {
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

pub(super) fn split_row(line: &str) -> Vec<String> {
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

pub(super) fn is_delimiter_row(line: &str) -> bool {
    line.contains('-') && {
        let cells = split_row(line);
        !cells.is_empty()
            && cells.iter().all(|cell| {
                let core = cell.trim_matches(':');
                !core.is_empty() && core.chars().all(|c| c == '-')
            })
    }
}

pub(super) fn starts_table(lines: &[String], at: usize) -> bool {
    lines[at].contains('|') && lines.get(at + 1).is_some_and(|next| is_delimiter_row(next))
}

pub(super) fn table_node(rows: Vec<(Vec<String>, bool)>, width: usize) -> Value {
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

//! Input rules that need no database: lengths, formats and tag hierarchy depth (D-06).
//!
//! The limits mirror the `CHECK` constraints of `001_init.sql`; the database stays the last line
//! of defence, but these checks give a readable message before a statement is attempted.
//! Messages are Spanish because they are shown to the user (as `VALIDATION: …`, D-08).

use std::fmt;

use chrono::{NaiveDate, NaiveTime, Timelike};

use super::dates::{self, DateError};

/// `tasks.title` limit, in characters.
pub const MAX_TITLE_CHARS: usize = 500;
/// `tags.name` limit, in characters, after trimming.
pub const MAX_TAG_NAME_CHARS: usize = 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError(String);

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub fn message(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ValidationError {}

impl From<DateError> for ValidationError {
    fn from(error: DateError) -> Self {
        Self(error.to_string())
    }
}

pub fn validate_title(title: &str) -> Result<(), ValidationError> {
    let length = title.chars().count();
    if length > MAX_TITLE_CHARS {
        return Err(ValidationError::new(format!(
            "el título no puede superar {MAX_TITLE_CHARS} caracteres (tiene {length})"
        )));
    }
    Ok(())
}

/// Returns the name to store: trimmed, 1 to 60 characters.
pub fn validate_tag_name(name: &str) -> Result<&str, ValidationError> {
    let trimmed = name.trim();
    let length = trimmed.chars().count();
    if length == 0 {
        return Err(ValidationError::new(
            "el nombre de la etiqueta no puede estar vacío",
        ));
    }
    if length > MAX_TAG_NAME_CHARS {
        return Err(ValidationError::new(format!(
            "el nombre de la etiqueta no puede superar {MAX_TAG_NAME_CHARS} caracteres (tiene {length})"
        )));
    }
    Ok(trimmed)
}

/// `#RRGGBB`, upper or lower case.
pub fn validate_color_hex(color: &str) -> Result<(), ValidationError> {
    let valid = color.len() == 7
        && color.starts_with('#')
        && color.bytes().skip(1).all(|byte| byte.is_ascii_hexdigit());
    if valid {
        Ok(())
    } else {
        Err(ValidationError::new(
            "el color debe tener el formato #RRGGBB",
        ))
    }
}

/// The notes document must be a ProseMirror document: a JSON object of type `doc`.
pub fn validate_content_json(content: &str) -> Result<(), ValidationError> {
    let invalid = || ValidationError::new("las notas no tienen un formato válido");
    let value: serde_json::Value = serde_json::from_str(content).map_err(|_| invalid())?;
    match value.get("type").and_then(serde_json::Value::as_str) {
        Some("doc") => {}
        _ => return Err(invalid()),
    }
    if has_unsafe_link(&value) {
        return Err(ValidationError::new(
            "las notas contienen un enlace no permitido (solo http, https y mailto)",
        ));
    }
    Ok(())
}

/// Whether `href` may be stored and opened: only `http`, `https` and `mailto` (SPEC F-08).
pub fn is_allowed_link(href: &str) -> bool {
    let Some((scheme, rest)) = href.trim().split_once(':') else {
        return false;
    };
    match scheme.to_ascii_lowercase().as_str() {
        "http" | "https" => rest.len() > 2 && rest.starts_with("//"),
        "mailto" => !rest.is_empty(),
        _ => false,
    }
}

fn has_unsafe_link(node: &serde_json::Value) -> bool {
    let bad_mark = node
        .get("marks")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|marks| {
            marks.iter().any(|mark| {
                mark.get("type").and_then(serde_json::Value::as_str) == Some("link")
                    && !mark
                        .get("attrs")
                        .and_then(|attrs| attrs.get("href"))
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(is_allowed_link)
            })
        });
    bad_mark
        || node
            .get("content")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|children| children.iter().any(has_unsafe_link))
}

/// Validates a task's date and time together: both well formed, and no time without a date
/// (a task without date lives in POR HACER, which has no schedule).
pub fn validate_schedule(
    due_date: Option<&str>,
    due_time: Option<&str>,
) -> Result<(Option<NaiveDate>, Option<NaiveTime>), ValidationError> {
    let date = due_date.map(dates::parse_date).transpose()?;
    let time = due_time.map(dates::parse_time).transpose()?;
    if time.is_some() && date.is_none() {
        return Err(ValidationError::new("una hora requiere una fecha"));
    }
    Ok((date, time))
}

/// A duration needs a start time, lasts at least a minute and ends before midnight (D-33).
pub fn validate_duration(
    due_time: Option<&str>,
    duration_minutes: Option<i64>,
) -> Result<(), ValidationError> {
    let Some(duration) = duration_minutes else {
        return Ok(());
    };
    let Some(time) = due_time else {
        return Err(ValidationError::new("una duración requiere una hora"));
    };
    if !(1..=24 * 60).contains(&duration) {
        return Err(ValidationError::new(
            "la duración debe estar entre 1 minuto y 24 horas",
        ));
    }
    let start = dates::parse_time(time)?;
    let start_minutes = i64::from(start.hour()) * 60 + i64::from(start.minute());
    if start_minutes + duration > 24 * 60 {
        return Err(ValidationError::new(
            "la tarea tiene que terminar antes de medianoche",
        ));
    }
    Ok(())
}

/// The tag a tag is being attached to as a child.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParentTag<'a> {
    pub id: &'a str,
    pub parent_id: Option<&'a str>,
}

/// D-06: tags have at most one level of hierarchy.
///
/// `parent` is the tag chosen as parent (already loaded by the caller; `None` for a top-level
/// tag) and `has_children` tells whether `tag_id` already is the parent of other tags.
pub fn validate_tag_parent(
    tag_id: &str,
    parent: Option<ParentTag<'_>>,
    has_children: bool,
) -> Result<(), ValidationError> {
    let Some(parent) = parent else {
        return Ok(());
    };
    if parent.id == tag_id {
        return Err(ValidationError::new(
            "una etiqueta no puede ser su propia etiqueta padre",
        ));
    }
    if parent.parent_id.is_some() {
        return Err(ValidationError::new(
            "solo se permite un nivel de etiquetas: la etiqueta padre ya es subetiqueta de otra",
        ));
    }
    if has_children {
        return Err(ValidationError::new(
            "una etiqueta con subetiquetas no puede pasar a ser subetiqueta",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message<T: fmt::Debug>(result: Result<T, ValidationError>) -> String {
        result.expect_err("must be rejected").to_string()
    }

    #[test]
    fn a_duration_needs_a_time_and_ends_before_midnight() {
        assert!(validate_duration(None, None).is_ok());
        assert!(validate_duration(Some("09:00"), Some(60)).is_ok());
        assert!(validate_duration(Some("00:00"), Some(1440)).is_ok());
        assert!(validate_duration(Some("23:30"), Some(30)).is_ok());
        assert!(validate_duration(None, Some(30)).is_err());
        assert!(validate_duration(Some("09:00"), Some(0)).is_err());
        assert!(validate_duration(Some("09:00"), Some(1441)).is_err());
        assert!(validate_duration(Some("23:30"), Some(31)).is_err());
    }

    #[test]
    fn title_accepts_up_to_the_limit_counting_characters() {
        assert!(validate_title("").is_ok());
        assert!(validate_title("Comprar pan").is_ok());
        assert!(validate_title(&"x".repeat(MAX_TITLE_CHARS)).is_ok());
        // 500 two-byte characters are 1000 bytes but still within the limit.
        assert!(validate_title(&"ñ".repeat(MAX_TITLE_CHARS)).is_ok());
    }

    #[test]
    fn title_over_the_limit_is_rejected_with_its_length() {
        let text = message(validate_title(&"x".repeat(MAX_TITLE_CHARS + 1)));
        assert_eq!(
            text,
            "el título no puede superar 500 caracteres (tiene 501)"
        );
        assert!(validate_title(&"ñ".repeat(MAX_TITLE_CHARS + 1)).is_err());
    }

    #[test]
    fn tag_name_is_trimmed_and_checked() {
        assert_eq!(validate_tag_name("Física"), Ok("Física"));
        assert_eq!(validate_tag_name("  Grado  \t"), Ok("Grado"));
        assert_eq!(validate_tag_name("a"), Ok("a"));
        let longest = "n".repeat(MAX_TAG_NAME_CHARS);
        assert_eq!(
            validate_tag_name(&format!(" {longest} ")),
            Ok(longest.as_str())
        );
    }

    #[test]
    fn tag_name_rejects_blank_and_too_long_names() {
        for blank in ["", "   ", "\t\n"] {
            assert_eq!(
                message(validate_tag_name(blank)),
                "el nombre de la etiqueta no puede estar vacío",
                "{blank:?}"
            );
        }
        let text = message(validate_tag_name(&"n".repeat(MAX_TAG_NAME_CHARS + 1)));
        assert_eq!(
            text,
            "el nombre de la etiqueta no puede superar 60 caracteres (tiene 61)"
        );
    }

    #[test]
    fn color_hex_table() {
        for valid in ["#000000", "#FFFFFF", "#0a84ff", "#A1b2C3"] {
            assert!(validate_color_hex(valid).is_ok(), "{valid}");
        }
        for invalid in [
            "", "#", "000000", "#00000", "#0000000", "#GGGGGG", "#12345G", "red", " #000000",
            "#00000é", "#ñ00000", "#0000é",
        ] {
            assert_eq!(
                message(validate_color_hex(invalid)),
                "el color debe tener el formato #RRGGBB",
                "{invalid:?}"
            );
        }
    }

    #[test]
    fn content_json_must_be_a_document() {
        for valid in [
            r#"{"type":"doc","content":[]}"#,
            r#"{"type":"doc"}"#,
            r#"{ "type": "doc", "content": [{"type":"paragraph"}] }"#,
        ] {
            assert!(validate_content_json(valid).is_ok(), "{valid}");
        }
        for invalid in [
            "",
            "not json",
            "[]",
            "null",
            "\"doc\"",
            "{}",
            r#"{"type":"paragraph"}"#,
            r#"{"type":1}"#,
            r#"{"type":"doc""#,
        ] {
            assert_eq!(
                message(validate_content_json(invalid)),
                "las notas no tienen un formato válido",
                "{invalid:?}"
            );
        }
    }

    #[test]
    fn schedule_parses_date_and_time() {
        assert_eq!(validate_schedule(None, None), Ok((None, None)));

        let (date, time) = validate_schedule(Some("2026-03-10"), Some("09:30")).expect("valid");
        assert_eq!(date.map(dates::format_date).as_deref(), Some("2026-03-10"));
        assert_eq!(time.map(dates::format_time).as_deref(), Some("09:30"));

        let (date, time) = validate_schedule(Some("2026-03-10"), None).expect("date only");
        assert!(date.is_some() && time.is_none());
    }

    #[test]
    fn schedule_rejects_bad_formats_and_a_time_without_a_date() {
        assert_eq!(
            message(validate_schedule(Some("2026-02-30"), None)),
            "fecha no válida «2026-02-30»: se espera AAAA-MM-DD"
        );
        assert_eq!(
            message(validate_schedule(Some("2026-03-10"), Some("25:00"))),
            "hora no válida «25:00»: se espera HH:MM"
        );
        assert_eq!(
            message(validate_schedule(None, Some("09:30"))),
            "una hora requiere una fecha"
        );
    }

    fn parent<'a>(id: &'a str, parent_id: Option<&'a str>) -> Option<ParentTag<'a>> {
        Some(ParentTag { id, parent_id })
    }

    #[test]
    fn tag_parent_table() {
        let cases: [(&str, Option<ParentTag<'_>>, bool, Option<&str>); 7] = [
            ("top-level tag", None, false, None),
            ("top-level tag that has children", None, true, None),
            (
                "child of a top-level tag",
                parent("grado", None),
                false,
                None,
            ),
            (
                "own parent",
                parent("a", None),
                false,
                Some("una etiqueta no puede ser su propia etiqueta padre"),
            ),
            (
                "parent is itself a child",
                parent("asignatura", Some("grado")),
                false,
                Some(
                    "solo se permite un nivel de etiquetas: la etiqueta padre ya es subetiqueta de otra",
                ),
            ),
            (
                "tag with children becoming a child",
                parent("otra", None),
                true,
                Some("una etiqueta con subetiquetas no puede pasar a ser subetiqueta"),
            ),
            (
                "own parent wins over the other reasons",
                parent("a", Some("x")),
                true,
                Some("una etiqueta no puede ser su propia etiqueta padre"),
            ),
        ];

        for (label, parent, has_children, expected) in cases {
            let result = validate_tag_parent("a", parent, has_children);
            match expected {
                None => assert_eq!(result, Ok(()), "{label}"),
                Some(text) => assert_eq!(message(result), text, "{label}"),
            }
        }
    }

    #[test]
    fn link_scheme_table() {
        for (href, allowed) in [
            ("https://example.com", true),
            ("HTTP://example.com/a?b=1", true),
            ("mailto:ana@example.com", true),
            ("javascript:alert(1)", false),
            ("file:///c:/secret.txt", false),
            ("data:text/html,<b>x</b>", false),
            (" javascript:alert(1)", false),
            ("https:", false),
            ("example.com", false),
            ("", false),
        ] {
            assert_eq!(is_allowed_link(href), allowed, "{href}");
        }
    }

    #[test]
    fn content_with_an_unsafe_link_is_rejected_even_when_nested() {
        let doc = |href: &str| {
            format!(
                r#"{{"type":"doc","content":[{{"type":"bulletList","content":[{{"type":"listItem","content":[{{"type":"paragraph","content":[{{"type":"text","text":"x","marks":[{{"type":"link","attrs":{{"href":"{href}"}}}}]}}]}}]}}]}}]}}"#
            )
        };
        assert!(validate_content_json(&doc("https://example.com")).is_ok());
        assert!(validate_content_json(&doc("javascript:alert(1)")).is_err());
        assert!(validate_content_json(&doc("file:///x")).is_err());
    }

    #[test]
    fn validation_error_exposes_its_message() {
        let error = ValidationError::new("algo falló");
        assert_eq!(error.message(), "algo falló");
        assert_eq!(error.to_string(), "algo falló");
        let from_date = ValidationError::from(DateError::OutOfRange);
        assert_eq!(
            from_date.message(),
            "la fecha queda fuera del rango admitido"
        );
    }
}

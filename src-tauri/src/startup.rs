use std::time::Duration;
#[cfg(debug_assertions)]
use std::{sync::OnceLock, time::Instant};

pub const MAX_LABEL_LEN: usize = 64;

#[cfg(debug_assertions)]
static START: OnceLock<Instant> = OnceLock::new();

/// Fixes the time origin of the startup log. Call it as early as possible in `run()`.
pub fn init() {
    #[cfg(debug_assertions)]
    {
        let _ = START.set(Instant::now());
        mark("process_start");
    }
}

/// Prints `[startup +X.XXms] label` to stderr. Compiled to a no-op in release builds.
pub fn mark(label: &str) {
    #[cfg(debug_assertions)]
    log_mark(label);
    #[cfg(not(debug_assertions))]
    let _ = label;
}

#[cfg(debug_assertions)]
fn log_mark(label: &str) {
    if let Some(start) = START.get() {
        eprintln!("{}", format_mark(label, start.elapsed()));
    }
}

pub fn validate_label(label: &str) -> Result<(), String> {
    if label.is_empty() || label.len() > MAX_LABEL_LEN {
        return Err(format!(
            "VALIDATION: label must be 1 to {MAX_LABEL_LEN} bytes"
        ));
    }
    if !label
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("VALIDATION: label only allows ASCII letters, digits, `_` and `-`".to_string());
    }
    Ok(())
}

#[cfg_attr(not(any(debug_assertions, test)), allow(dead_code))]
fn format_mark(label: &str, elapsed: Duration) -> String {
    format!("[startup +{:.2}ms] {label}", elapsed.as_secs_f64() * 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_mark_shows_milliseconds_with_two_decimals() {
        let line = format_mark("setup_done", Duration::from_micros(12_345));
        assert_eq!(line, "[startup +12.35ms] setup_done");
    }

    #[test]
    fn format_mark_handles_zero_elapsed() {
        assert_eq!(format_mark("x", Duration::ZERO), "[startup +0.00ms] x");
    }

    #[test]
    fn valid_labels_are_accepted() {
        assert!(validate_label("first_paint").is_ok());
        assert!(validate_label("page-load-1").is_ok());
        assert!(validate_label(&"a".repeat(MAX_LABEL_LEN)).is_ok());
    }

    #[test]
    fn invalid_labels_are_rejected_with_validation_code() {
        for label in [
            "",
            "has space",
            "line\nbreak",
            "ñandú",
            &"a".repeat(MAX_LABEL_LEN + 1),
        ] {
            let error = validate_label(label).expect_err("must be rejected");
            assert!(
                error.starts_with("VALIDATION: "),
                "unexpected error: {error}"
            );
        }
    }

    #[test]
    fn mark_before_init_does_not_panic() {
        mark("before_init");
    }
}

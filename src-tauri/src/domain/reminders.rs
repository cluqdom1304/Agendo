//! Reminder rules (SPEC §5.3, §6, D-26). Pure: the time zone is a parameter.

use chrono::{DateTime, Duration, TimeZone, Utc};

use super::{
    dates::{self, DateError},
    validation::ValidationError,
};

/// Largest "before" offset accepted: 30 days.
pub const MAX_OFFSET_MINUTES: i64 = 30 * 24 * 60;

/// A reminder is stale once it is this many hours overdue (F-14).
pub const STALE_AFTER_HOURS: i64 = 24;

pub fn validate_offset(minutes: i64) -> Result<(), ValidationError> {
    if (0..=MAX_OFFSET_MINUTES).contains(&minutes) {
        Ok(())
    } else {
        Err(ValidationError::new(
            "el recordatorio debe ser entre 0 minutos y 30 días antes",
        ))
    }
}

/// UTC instant of "`offset_minutes` before the task's local date and time".
pub fn offset_notify_at<Tz: TimeZone>(
    zone: &Tz,
    due_date: &str,
    due_time: &str,
    offset_minutes: i64,
) -> Result<DateTime<Utc>, DateError> {
    let date = dates::parse_date(due_date)?;
    let time = dates::parse_time(due_time)?;
    let due = dates::local_to_utc(zone, date, time)?;
    due.checked_sub_signed(Duration::minutes(offset_minutes))
        .ok_or(DateError::OutOfRange)
}

/// Instant before which an unsent reminder counts as lost for good at `now`.
pub fn stale_cutoff(now: DateTime<Utc>) -> DateTime<Utc> {
    now - Duration::hours(STALE_AFTER_HOURS)
}

#[cfg(test)]
mod tests {
    use chrono::FixedOffset;

    use super::*;

    fn instant(text: &str) -> DateTime<Utc> {
        dates::parse_instant(text).expect("instant")
    }

    #[test]
    fn offsets_are_taken_from_the_local_due_time() {
        let zone = FixedOffset::east_opt(2 * 3600).expect("zone");
        let at = |offset| offset_notify_at(&zone, "2026-03-10", "09:30", offset).expect("at");
        assert_eq!(at(0), instant("2026-03-10T07:30:00Z"));
        assert_eq!(at(15), instant("2026-03-10T07:15:00Z"));
        assert_eq!(at(60), instant("2026-03-10T06:30:00Z"));
        assert_eq!(at(1440), instant("2026-03-09T07:30:00Z"));
    }

    #[test]
    fn invalid_dates_and_times_are_rejected() {
        assert!(offset_notify_at(&Utc, "2026-02-30", "09:30", 0).is_err());
        assert!(offset_notify_at(&Utc, "2026-03-10", "25:00", 0).is_err());
    }

    #[test]
    fn offsets_outside_the_range_are_rejected() {
        assert!(validate_offset(0).is_ok());
        assert!(validate_offset(MAX_OFFSET_MINUTES).is_ok());
        assert!(validate_offset(-1).is_err());
        assert!(validate_offset(MAX_OFFSET_MINUTES + 1).is_err());
    }

    #[test]
    fn the_stale_cutoff_is_24_hours_back() {
        assert_eq!(
            stale_cutoff(instant("2026-03-10T10:00:00Z")),
            instant("2026-03-09T10:00:00Z")
        );
    }
}

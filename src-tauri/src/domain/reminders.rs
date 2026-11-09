//! Reminder rules (SPEC §5.3, §6, D-26). Pure: the time zone is a parameter.

use chrono::{DateTime, Duration, Months, TimeZone, Utc};
use serde::{Deserialize, Serialize};

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

/// Longest interval (in its unit) and series a repeating reminder may have.
pub const MAX_REPEAT_EVERY: u32 = 365;
pub const MAX_REPEAT_COUNT: u32 = 1000;

/// Upper bound on steps taken to catch up after a long absence; beyond it the series ends.
const MAX_CATCH_UP_STEPS: u32 = 20_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepeatUnit {
    Hour,
    Day,
    Week,
    Month,
}

impl RepeatUnit {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [Self::Hour, Self::Day, Self::Week, Self::Month]
            .into_iter()
            .find(|unit| unit.as_str() == text)
    }
}

/// How a custom-time reminder repeats: every `every` units, until it has sounded `count` times
/// in total or its local date passes `until` (inclusive). Never both; neither means forever.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repeat {
    pub every: u32,
    pub unit: RepeatUnit,
    #[serde(default)]
    pub count: Option<u32>,
    /// Local date, `YYYY-MM-DD`.
    #[serde(default)]
    pub until: Option<String>,
    /// Which sounding of the series this reminder is, starting at 1.
    #[serde(default = "first_occurrence")]
    pub occurrence: u32,
}

fn first_occurrence() -> u32 {
    1
}

/// Checks a repeat rule typed by the user. `first` is when the first sounding happens.
pub fn validate_repeat<Tz: TimeZone>(
    zone: &Tz,
    repeat: &Repeat,
    first: DateTime<Utc>,
) -> Result<(), ValidationError> {
    if !(1..=MAX_REPEAT_EVERY).contains(&repeat.every) {
        return Err(ValidationError::new(
            "la repetición debe ser cada 1 a 365 unidades",
        ));
    }
    if repeat.count.is_some() && repeat.until.is_some() {
        return Err(ValidationError::new(
            "la repetición termina por número de veces o por fecha, no por las dos",
        ));
    }
    if let Some(count) = repeat.count
        && !(2..=MAX_REPEAT_COUNT).contains(&count)
    {
        return Err(ValidationError::new(
            "el número de repeticiones debe estar entre 2 y 1000",
        ));
    }
    if repeat.occurrence == 0 || repeat.count.is_some_and(|count| repeat.occurrence > count) {
        return Err(ValidationError::new(
            "el número de aviso dentro de la repetición no es válido",
        ));
    }
    if let Some(until) = &repeat.until {
        let until = dates::parse_date(until)
            .map_err(|_| ValidationError::new("la fecha final de la repetición no es válida"))?;
        if until < dates::utc_to_local(zone, first).date() {
            return Err(ValidationError::new(
                "la repetición no puede terminar antes del primer aviso",
            ));
        }
    }
    Ok(())
}

fn step<Tz: TimeZone>(
    zone: &Tz,
    instant: DateTime<Utc>,
    every: u32,
    unit: RepeatUnit,
) -> Result<DateTime<Utc>, DateError> {
    if unit == RepeatUnit::Hour {
        return instant
            .checked_add_signed(Duration::hours(i64::from(every)))
            .ok_or(DateError::OutOfRange);
    }
    // Days, weeks and months keep the wall-clock time across daylight-saving changes.
    let local = dates::utc_to_local(zone, instant);
    let date = match unit {
        RepeatUnit::Day => local
            .date()
            .checked_add_signed(Duration::days(i64::from(every))),
        RepeatUnit::Week => local
            .date()
            .checked_add_signed(Duration::weeks(i64::from(every))),
        _ => local.date().checked_add_months(Months::new(every)),
    }
    .ok_or(DateError::OutOfRange)?;
    dates::local_to_utc(zone, date, local.time())
}

/// The next sounding of a series strictly after `after`, with its occurrence number, or `None`
/// when the series has ended. Soundings that fell before `after` (the app was closed) are
/// skipped but still count towards `count`.
pub fn next_occurrence<Tz: TimeZone>(
    zone: &Tz,
    current: DateTime<Utc>,
    repeat: &Repeat,
    after: DateTime<Utc>,
) -> Result<Option<(DateTime<Utc>, u32)>, DateError> {
    let until = match &repeat.until {
        Some(text) => Some(dates::parse_date(text)?),
        None => None,
    };
    let mut at = current;
    let mut occurrence = repeat.occurrence;
    for _ in 0..MAX_CATCH_UP_STEPS {
        occurrence += 1;
        if repeat.count.is_some_and(|count| occurrence > count) {
            return Ok(None);
        }
        at = step(zone, at, repeat.every, repeat.unit)?;
        if until.is_some_and(|last| dates::utc_to_local(zone, at).date() > last) {
            return Ok(None);
        }
        if at > after {
            return Ok(Some((at, occurrence)));
        }
    }
    Ok(None)
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

    fn repeat(every: u32, unit: RepeatUnit) -> Repeat {
        Repeat {
            every,
            unit,
            count: None,
            until: None,
            occurrence: 1,
        }
    }

    fn next(zone: &FixedOffset, from: &str, rule: &Repeat, after: &str) -> Option<(String, u32)> {
        next_occurrence(zone, instant(from), rule, instant(after))
            .expect("next")
            .map(|(at, n)| (dates::format_instant(at), n))
    }

    #[test]
    fn each_unit_steps_by_its_interval() {
        let zone = FixedOffset::east_opt(0).expect("zone");
        let from = "2026-01-31T09:00:00Z";
        let cases = [
            (repeat(3, RepeatUnit::Hour), "2026-01-31T12:00:00Z"),
            (repeat(2, RepeatUnit::Day), "2026-02-02T09:00:00Z"),
            (repeat(1, RepeatUnit::Week), "2026-02-07T09:00:00Z"),
            (repeat(1, RepeatUnit::Month), "2026-02-28T09:00:00Z"),
        ];
        for (rule, expected) in cases {
            assert_eq!(
                next(&zone, from, &rule, from),
                Some((expected.to_string(), 2)),
                "{rule:?}"
            );
        }
    }

    #[test]
    fn days_keep_the_local_wall_clock_time() {
        let zone = FixedOffset::east_opt(2 * 3600).expect("zone");
        assert_eq!(
            next(
                &zone,
                "2026-06-10T07:30:00Z",
                &repeat(1, RepeatUnit::Day),
                "2026-06-10T07:30:00Z"
            ),
            Some(("2026-06-11T07:30:00Z".to_string(), 2))
        );
    }

    #[test]
    fn missed_soundings_are_skipped_and_still_counted() {
        let zone = FixedOffset::east_opt(0).expect("zone");
        let rule = Repeat {
            count: Some(5),
            ..repeat(1, RepeatUnit::Day)
        };
        assert_eq!(
            next(&zone, "2026-03-01T09:00:00Z", &rule, "2026-03-03T12:00:00Z"),
            Some(("2026-03-04T09:00:00Z".to_string(), 4))
        );
        assert_eq!(
            next(&zone, "2026-03-01T09:00:00Z", &rule, "2026-03-06T12:00:00Z"),
            None,
            "the fifth sounding was missed and the series is over"
        );
    }

    #[test]
    fn a_series_ends_after_its_count() {
        let zone = FixedOffset::east_opt(0).expect("zone");
        let rule = Repeat {
            count: Some(3),
            occurrence: 3,
            ..repeat(1, RepeatUnit::Week)
        };
        assert_eq!(
            next(&zone, "2026-03-15T09:00:00Z", &rule, "2026-03-15T09:00:00Z"),
            None
        );
    }

    #[test]
    fn a_series_ends_after_its_last_date_inclusive_in_local_time() {
        let zone = FixedOffset::east_opt(2 * 3600).expect("zone");
        let rule = Repeat {
            until: Some("2026-03-12".into()),
            ..repeat(1, RepeatUnit::Day)
        };
        assert_eq!(
            next(&zone, "2026-03-11T08:00:00Z", &rule, "2026-03-11T08:00:00Z"),
            Some(("2026-03-12T08:00:00Z".to_string(), 2))
        );
        assert_eq!(
            next(&zone, "2026-03-12T08:00:00Z", &rule, "2026-03-12T08:00:00Z"),
            None
        );
    }

    #[test]
    fn repeat_rules_are_validated() {
        let zone = FixedOffset::east_opt(0).expect("zone");
        let first = instant("2026-03-10T09:00:00Z");
        let ok = |rule: Repeat| validate_repeat(&zone, &rule, first).is_ok();
        let daily = || repeat(1, RepeatUnit::Day);

        assert!(ok(daily()));
        assert!(!ok(repeat(0, RepeatUnit::Day)));
        assert!(!ok(repeat(366, RepeatUnit::Day)));
        assert!(ok(Repeat {
            count: Some(2),
            ..daily()
        }));
        assert!(!ok(Repeat {
            count: Some(1),
            ..daily()
        }));
        assert!(!ok(Repeat {
            count: Some(1001),
            ..daily()
        }));
        assert!(ok(Repeat {
            until: Some("2026-03-10".into()),
            ..daily()
        }));
        assert!(!ok(Repeat {
            until: Some("2026-03-09".into()),
            ..daily()
        }));
        assert!(!ok(Repeat {
            until: Some("no-fecha".into()),
            ..daily()
        }));
        assert!(!ok(Repeat {
            count: Some(3),
            until: Some("2026-04-01".into()),
            ..daily()
        }));
    }

    #[test]
    fn units_round_trip_through_their_names() {
        for unit in [
            RepeatUnit::Hour,
            RepeatUnit::Day,
            RepeatUnit::Week,
            RepeatUnit::Month,
        ] {
            assert_eq!(RepeatUnit::parse(unit.as_str()), Some(unit));
        }
        assert_eq!(RepeatUnit::parse("year"), None);
    }
}

//! Date and time rules (SPEC §5.1).
//!
//! * Business dates and times are local and zone-less: `due_date` is `YYYY-MM-DD`, `due_time` is
//!   `HH:MM`.
//! * Technical instants (`created_at`, `notify_at`, …) are UTC, `YYYY-MM-DDTHH:MM:SSZ`.
//!
//! This is the only place where a local date/time becomes UTC or the other way round. It never
//! reads the clock; the time zone is a parameter (`chrono::Local` in production), so DST
//! behaviour can be tested with a simulated zone.

use std::fmt;

use chrono::{
    DateTime, Datelike, Duration, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, Offset,
    TimeZone, Utc,
};

/// Years accepted by [`local_to_utc`]; `parse_date` only produces 0000 to 9999.
const MIN_YEAR: i32 = 1;
const MAX_YEAR: i32 = 9998;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateError {
    InvalidDate(String),
    InvalidTime(String),
    InvalidInstant(String),
    /// The result falls outside the range `chrono` can represent.
    OutOfRange,
}

impl fmt::Display for DateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDate(value) => {
                write!(f, "fecha no válida «{value}»: se espera AAAA-MM-DD")
            }
            Self::InvalidTime(value) => write!(f, "hora no válida «{value}»: se espera HH:MM"),
            Self::InvalidInstant(value) => write!(
                f,
                "instante no válido «{value}»: se espera AAAA-MM-DDTHH:MM:SSZ"
            ),
            Self::OutOfRange => write!(f, "la fecha queda fuera del rango admitido"),
        }
    }
}

impl std::error::Error for DateError {}

/// Reads exactly `digits` ASCII digits as a number, or `None` for anything else (signs,
/// spaces and non-ASCII digits included).
fn number(text: &str, digits: usize) -> Option<u32> {
    if text.len() != digits || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// Splits `text` at ASCII `separator` into exactly two parts, if it is ASCII at all.
fn split_ascii(text: &str, at: usize, separator: char) -> Option<(&str, &str)> {
    if !text.is_ascii() || text.as_bytes().get(at) != Some(&(separator as u8)) {
        return None;
    }
    Some((&text[..at], &text[at + 1..]))
}

/// Strict `YYYY-MM-DD` that must also be a real calendar date (no `2026-02-30`).
pub fn parse_date(text: &str) -> Result<NaiveDate, DateError> {
    let invalid = || DateError::InvalidDate(text.to_string());
    let (year, rest) = split_ascii(text, 4, '-').ok_or_else(invalid)?;
    let (month, day) = split_ascii(rest, 2, '-').ok_or_else(invalid)?;
    let year = number(year, 4).and_then(|y| i32::try_from(y).ok());
    match (year, number(month, 2), number(day, 2)) {
        (Some(year), Some(month), Some(day)) => {
            NaiveDate::from_ymd_opt(year, month, day).ok_or_else(invalid)
        }
        _ => Err(invalid()),
    }
}

/// Strict `HH:MM`, 00:00 to 23:59.
pub fn parse_time(text: &str) -> Result<NaiveTime, DateError> {
    let invalid = || DateError::InvalidTime(text.to_string());
    let (hour, minute) = split_ascii(text, 2, ':').ok_or_else(invalid)?;
    match (number(hour, 2), number(minute, 2)) {
        (Some(hour), Some(minute)) => NaiveTime::from_hms_opt(hour, minute, 0).ok_or_else(invalid),
        _ => Err(invalid()),
    }
}

/// Strict UTC instant `YYYY-MM-DDTHH:MM:SSZ`.
pub fn parse_instant(text: &str) -> Result<DateTime<Utc>, DateError> {
    let invalid = || DateError::InvalidInstant(text.to_string());
    let body = text.strip_suffix('Z').ok_or_else(invalid)?;
    let (date, time) = split_ascii(body, 10, 'T').ok_or_else(invalid)?;
    let date = parse_date(date).map_err(|_| invalid())?;
    let (clock, seconds) = time
        .len()
        .checked_sub(3)
        .and_then(|at| split_ascii(time, at, ':'))
        .ok_or_else(invalid)?;
    let time = parse_time(clock).map_err(|_| invalid())?;
    let seconds = number(seconds, 2).filter(|s| *s < 60).ok_or_else(invalid)?;
    let naive = date.and_time(time) + Duration::seconds(i64::from(seconds));
    Ok(naive.and_utc())
}

pub fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub fn format_time(time: NaiveTime) -> String {
    time.format("%H:%M").to_string()
}

pub fn format_instant(instant: DateTime<Utc>) -> String {
    instant.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Converts a local date and time in `zone` to a UTC instant.
///
/// DST edge cases are resolved deterministically:
/// * A local time that does not exist (clocks jump forward) is read with the offset in effect
///   before the jump, so `02:30` becomes `03:30` on the new clock.
/// * A local time that happens twice (clocks go back) means its first occurrence.
pub fn local_to_utc<Tz: TimeZone>(
    zone: &Tz,
    date: NaiveDate,
    time: NaiveTime,
) -> Result<DateTime<Utc>, DateError> {
    // `chrono` panics converting dates near its own limits; stay well inside them.
    if !(MIN_YEAR..=MAX_YEAR).contains(&date.year()) {
        return Err(DateError::OutOfRange);
    }
    let local = date.and_time(time);
    match zone.from_local_datetime(&local) {
        LocalResult::Single(instant) => Ok(instant.with_timezone(&Utc)),
        LocalResult::Ambiguous(first, second) => Ok(first.min(second).with_timezone(&Utc)),
        LocalResult::None => {
            // A day earlier is safely before the jump: no zone changes offset twice in 24 h.
            let before = local
                .checked_sub_signed(Duration::days(1))
                .ok_or(DateError::OutOfRange)?;
            let offset = zone.offset_from_local_datetime(&before).earliest();
            let seconds = offset.map_or(0, |offset| i64::from(offset.fix().local_minus_utc()));
            let utc = local
                .checked_sub_signed(Duration::seconds(seconds))
                .ok_or(DateError::OutOfRange)?;
            Ok(utc.and_utc())
        }
    }
}

/// The local date and time in `zone` at the UTC instant `instant`.
pub fn utc_to_local<Tz: TimeZone>(zone: &Tz, instant: DateTime<Utc>) -> NaiveDateTime {
    instant.with_timezone(zone).naive_local()
}

#[cfg(test)]
mod tests {
    use chrono::{FixedOffset, Weekday};

    use super::*;

    /// Simulated Central European zone: UTC+1, UTC+2 from the last Sunday of March to the last
    /// Sunday of October, both transitions at 01:00 UTC.
    #[derive(Clone, Copy, Debug)]
    struct TestZone;

    fn last_sunday(year: i32, month: u32) -> NaiveDate {
        let mut day = NaiveDate::from_ymd_opt(year, month, 31).expect("31-day month");
        while day.weekday() != Weekday::Sun {
            day -= Duration::days(1);
        }
        day
    }

    fn hours(count: i32) -> FixedOffset {
        FixedOffset::east_opt(count * 3600).expect("offset")
    }

    impl TimeZone for TestZone {
        type Offset = FixedOffset;

        fn from_offset(_: &FixedOffset) -> Self {
            TestZone
        }

        fn offset_from_local_date(&self, local: &NaiveDate) -> LocalResult<FixedOffset> {
            self.offset_from_local_datetime(&local.and_hms_opt(12, 0, 0).expect("noon"))
        }

        fn offset_from_local_datetime(&self, local: &NaiveDateTime) -> LocalResult<FixedOffset> {
            let valid: Vec<FixedOffset> = [2, 1]
                .into_iter()
                .map(hours)
                .filter(|offset| {
                    let utc = *local - Duration::seconds(i64::from(offset.local_minus_utc()));
                    self.offset_from_utc_datetime(&utc) == *offset
                })
                .collect();
            match valid.as_slice() {
                [] => LocalResult::None,
                [one] => LocalResult::Single(*one),
                [earlier, later] => LocalResult::Ambiguous(*earlier, *later),
                _ => LocalResult::None,
            }
        }

        fn offset_from_utc_date(&self, utc: &NaiveDate) -> FixedOffset {
            self.offset_from_utc_datetime(&utc.and_hms_opt(12, 0, 0).expect("noon"))
        }

        fn offset_from_utc_datetime(&self, utc: &NaiveDateTime) -> FixedOffset {
            let start = last_sunday(utc.year(), 3).and_hms_opt(1, 0, 0);
            let end = last_sunday(utc.year(), 10).and_hms_opt(1, 0, 0);
            match (start, end) {
                (Some(start), Some(end)) if *utc >= start && *utc < end => hours(2),
                _ => hours(1),
            }
        }
    }

    fn date(text: &str) -> NaiveDate {
        parse_date(text).expect("date")
    }

    fn time(text: &str) -> NaiveTime {
        parse_time(text).expect("time")
    }

    fn utc(text: &str) -> DateTime<Utc> {
        parse_instant(text).expect("instant")
    }

    #[test]
    fn parse_date_accepts_real_calendar_dates() {
        assert_eq!(format_date(date("2026-03-09")), "2026-03-09");
        assert_eq!(format_date(date("2028-02-29")), "2028-02-29");
        assert_eq!(format_date(date("1999-12-31")), "1999-12-31");
    }

    #[test]
    fn parse_date_rejects_everything_else() {
        for text in [
            "",
            "2026-3-09",
            "2026-03-9",
            "26-03-09",
            "2026/03/09",
            "09/03/2026",
            "2026-13-01",
            "2026-00-10",
            "2026-02-30",
            "2027-02-29",
            "2026-04-31",
            "2026-03-00",
            "2026-03-091",
            " 2026-03-09",
            "2026-03-09 ",
            "+026-03-09",
            "2026-+3-09",
            "2026-03-0a",
            "２０２６-03-09",
            "2026-03-09T10:00",
            "hoy",
        ] {
            assert_eq!(
                parse_date(text),
                Err(DateError::InvalidDate(text.to_string())),
                "{text:?}"
            );
        }
    }

    #[test]
    fn parse_time_accepts_the_whole_day() {
        assert_eq!(format_time(time("00:00")), "00:00");
        assert_eq!(format_time(time("09:05")), "09:05");
        assert_eq!(format_time(time("23:59")), "23:59");
    }

    #[test]
    fn parse_time_rejects_everything_else() {
        for text in [
            "", "9:05", "09:5", "24:00", "23:60", "12:00:00", "12-00", "1200", "ab:cd", " 9:05",
            "+9:05", "09:0٣", "noon",
        ] {
            assert_eq!(
                parse_time(text),
                Err(DateError::InvalidTime(text.to_string())),
                "{text:?}"
            );
        }
    }

    #[test]
    fn instants_round_trip() {
        for text in [
            "2026-03-09T10:15:30Z",
            "2026-12-31T23:59:59Z",
            "2026-01-01T00:00:00Z",
            "2028-02-29T12:00:00Z",
        ] {
            assert_eq!(format_instant(utc(text)), text);
        }
    }

    #[test]
    fn parse_instant_rejects_everything_else() {
        for text in [
            "",
            "2026-03-09T10:15:30",
            "2026-03-09 10:15:30Z",
            "2026-03-09T10:15Z",
            "2026-03-09T10:15:60Z",
            "2026-03-09T24:00:00Z",
            "2026-02-30T10:00:00Z",
            "2026-03-09T10:15:30+01:00",
            "2026-03-09T10:15:3Z",
            "2026-03-09T10:15:300Z",
            "2026-03-09t10:15:30Z",
            "2026-3-09T10:15:30Z",
            "ñ026-03-09T10:15:30Z",
        ] {
            assert_eq!(
                parse_instant(text),
                Err(DateError::InvalidInstant(text.to_string())),
                "{text:?}"
            );
        }
    }

    #[test]
    fn utc_zone_is_the_identity() {
        let result = local_to_utc(&Utc, date("2026-03-09"), time("10:30")).expect("convert");
        assert_eq!(format_instant(result), "2026-03-09T10:30:00Z");
    }

    #[test]
    fn the_system_zone_round_trips_an_ordinary_instant() {
        let instant =
            local_to_utc(&chrono::Local, date("2026-07-01"), time("12:00")).expect("convert");
        let local = utc_to_local(&chrono::Local, instant);
        assert_eq!(
            (format_date(local.date()), format_time(local.time())),
            ("2026-07-01".to_string(), "12:00".to_string())
        );
    }

    #[test]
    fn fixed_offsets_are_applied_in_both_directions() {
        let zone = FixedOffset::east_opt(5 * 3600 + 1800).expect("offset");
        let instant = local_to_utc(&zone, date("2026-03-09"), time("03:00")).expect("convert");
        assert_eq!(format_instant(instant), "2026-03-08T21:30:00Z");

        let local = utc_to_local(&zone, instant);
        assert_eq!(
            (format_date(local.date()), format_time(local.time())),
            ("2026-03-09".to_string(), "03:00".to_string())
        );
    }

    #[test]
    fn winter_and_summer_use_different_offsets() {
        let winter = local_to_utc(&TestZone, date("2026-01-15"), time("09:00")).expect("winter");
        let summer = local_to_utc(&TestZone, date("2026-07-15"), time("09:00")).expect("summer");
        assert_eq!(format_instant(winter), "2026-01-15T08:00:00Z");
        assert_eq!(format_instant(summer), "2026-07-15T07:00:00Z");
    }

    #[test]
    fn a_local_time_skipped_by_the_spring_jump_moves_forward() {
        // 2026-03-29: 02:00 → 03:00 local. 02:30 does not exist.
        let instant = local_to_utc(&TestZone, date("2026-03-29"), time("02:30")).expect("gap");
        assert_eq!(format_instant(instant), "2026-03-29T01:30:00Z");
        let local = utc_to_local(&TestZone, instant);
        assert_eq!(format_time(local.time()), "03:30");
    }

    #[test]
    fn times_around_the_spring_jump_are_unaffected() {
        let before = local_to_utc(&TestZone, date("2026-03-29"), time("01:59")).expect("before");
        let after = local_to_utc(&TestZone, date("2026-03-29"), time("03:00")).expect("after");
        assert_eq!(format_instant(before), "2026-03-29T00:59:00Z");
        assert_eq!(format_instant(after), "2026-03-29T01:00:00Z");
    }

    #[test]
    fn a_local_time_repeated_by_the_autumn_jump_means_its_first_occurrence() {
        // 2026-10-25: 03:00 → 02:00 local. 02:30 happens at 00:30Z and again at 01:30Z.
        let instant = local_to_utc(&TestZone, date("2026-10-25"), time("02:30")).expect("fold");
        assert_eq!(format_instant(instant), "2026-10-25T00:30:00Z");
    }

    #[test]
    fn utc_to_local_shows_both_occurrences_of_the_repeated_hour() {
        let first = utc_to_local(&TestZone, utc("2026-10-25T00:30:00Z"));
        let second = utc_to_local(&TestZone, utc("2026-10-25T01:30:00Z"));
        assert_eq!(format_time(first.time()), "02:30");
        assert_eq!(format_time(second.time()), "02:30");
        assert_eq!(first.date(), second.date());
    }

    #[test]
    fn local_to_utc_to_local_round_trips_outside_the_transitions() {
        let mut instant = utc("2026-01-01T00:00:00Z");
        let end = utc("2027-01-01T00:00:00Z");
        while instant < end {
            let local = utc_to_local(&TestZone, instant);
            let back = local_to_utc(&TestZone, local.date(), local.time()).expect("convert");
            // Only the second pass through the repeated autumn hour maps to an earlier instant.
            if back != instant {
                assert_eq!(back + Duration::hours(1), instant, "{instant}");
            }
            instant += Duration::minutes(30);
        }
    }

    #[test]
    fn extreme_dates_report_out_of_range_instead_of_panicking() {
        let zone = FixedOffset::east_opt(12 * 3600).expect("offset");
        for extreme in [
            NaiveDate::MIN,
            NaiveDate::MAX,
            date("0000-06-01"),
            date("9999-06-01"),
        ] {
            assert_eq!(
                local_to_utc(&zone, extreme, NaiveTime::MIN),
                Err(DateError::OutOfRange),
                "{extreme}"
            );
        }
        assert!(local_to_utc(&zone, date("0001-06-01"), NaiveTime::MIN).is_ok());
        assert!(local_to_utc(&zone, date("9998-06-01"), NaiveTime::MIN).is_ok());
    }

    #[test]
    fn error_messages_are_readable_spanish() {
        assert_eq!(
            DateError::InvalidDate("hoy".into()).to_string(),
            "fecha no válida «hoy»: se espera AAAA-MM-DD"
        );
        assert_eq!(
            DateError::InvalidTime("25:00".into()).to_string(),
            "hora no válida «25:00»: se espera HH:MM"
        );
        assert_eq!(
            DateError::InvalidInstant("x".into()).to_string(),
            "instante no válido «x»: se espera AAAA-MM-DDTHH:MM:SSZ"
        );
        assert_eq!(
            DateError::OutOfRange.to_string(),
            "la fecha queda fuera del rango admitido"
        );
    }
}

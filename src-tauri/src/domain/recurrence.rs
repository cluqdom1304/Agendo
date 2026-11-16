//! Repeating tasks (F13-05, B-05). The rule lives in `tasks.rrule` as a small subset of RFC 5545:
//! `FREQ=DAILY|WEEKLY|MONTHLY|YEARLY`, `INTERVAL=n`, `UNTIL=YYYYMMDD`, plus `BYMONTHDAY` and
//! `BYMONTH`, which the app fills in from the task's date so that «every month on the 31st»
//! lands on the last day of short months without drifting to the 28th afterwards.
//!
//! Everything here works on calendar dates: no time zone, no clock.

use chrono::{Datelike, Duration, NaiveDate};

use super::validation::ValidationError;

pub const MAX_INTERVAL: u32 = 365;
/// Steps tried before giving up when a task is completed long after its date.
const MAX_STEPS: u32 = 20_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freq {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Freq {
    fn as_str(self) -> &'static str {
        match self {
            Self::Daily => "DAILY",
            Self::Weekly => "WEEKLY",
            Self::Monthly => "MONTHLY",
            Self::Yearly => "YEARLY",
        }
    }

    fn parse(text: &str) -> Option<Self> {
        match text {
            "DAILY" => Some(Self::Daily),
            "WEEKLY" => Some(Self::Weekly),
            "MONTHLY" => Some(Self::Monthly),
            "YEARLY" => Some(Self::Yearly),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub freq: Freq,
    pub interval: u32,
    pub until: Option<NaiveDate>,
    /// Day of the month the series sticks to (monthly and yearly).
    pub month_day: Option<u32>,
    /// Month the series sticks to (yearly).
    pub month: Option<u32>,
}

fn invalid(detail: &str) -> ValidationError {
    ValidationError::new(format!("regla de repetición no válida: {detail}"))
}

impl Rule {
    pub fn parse(text: &str) -> Result<Self, ValidationError> {
        let mut freq = None;
        let mut interval = 1;
        let mut until = None;
        let mut month_day = None;
        let mut month = None;
        for part in text.split(';') {
            let (key, value) = part
                .split_once('=')
                .ok_or_else(|| invalid("se esperaba CLAVE=valor"))?;
            match key {
                "FREQ" => {
                    freq =
                        Some(Freq::parse(value).ok_or_else(|| invalid("frecuencia desconocida"))?);
                }
                "INTERVAL" => {
                    interval = value
                        .parse::<u32>()
                        .ok()
                        .filter(|n| (1..=MAX_INTERVAL).contains(n))
                        .ok_or_else(|| invalid("el intervalo va de 1 a 365"))?;
                }
                "UNTIL" => {
                    until = Some(
                        NaiveDate::parse_from_str(value, "%Y%m%d")
                            .map_err(|_| invalid("UNTIL debe ser AAAAMMDD"))?,
                    );
                }
                "BYMONTHDAY" => {
                    month_day = Some(
                        value
                            .parse::<u32>()
                            .ok()
                            .filter(|n| (1..=31).contains(n))
                            .ok_or_else(|| invalid("BYMONTHDAY va de 1 a 31"))?,
                    );
                }
                "BYMONTH" => {
                    month = Some(
                        value
                            .parse::<u32>()
                            .ok()
                            .filter(|n| (1..=12).contains(n))
                            .ok_or_else(|| invalid("BYMONTH va de 1 a 12"))?,
                    );
                }
                _ => return Err(invalid("parte no admitida")),
            }
        }
        let freq = freq.ok_or_else(|| invalid("falta FREQ"))?;
        Ok(Self {
            freq,
            interval,
            until,
            month_day,
            month,
        })
    }

    pub fn format(&self) -> String {
        let mut text = format!("FREQ={};INTERVAL={}", self.freq.as_str(), self.interval);
        if let Some(day) = self.month_day {
            text.push_str(&format!(";BYMONTHDAY={day}"));
        }
        if let Some(month) = self.month {
            text.push_str(&format!(";BYMONTH={month}"));
        }
        if let Some(until) = self.until {
            text.push_str(&format!(";UNTIL={}", until.format("%Y%m%d")));
        }
        text
    }

    /// The rule as stored for a task that starts on `start`: the anchors come from that date.
    pub fn anchored(mut self, start: NaiveDate) -> Result<Self, ValidationError> {
        if self.until.is_some_and(|until| until < start) {
            return Err(invalid("termina antes de la fecha de la tarea"));
        }
        self.month_day = matches!(self.freq, Freq::Monthly | Freq::Yearly).then(|| start.day());
        self.month = (self.freq == Freq::Yearly).then(|| start.month());
        Ok(self)
    }

    /// The first occurrence after `current` that is also after `floor` (the day the task was
    /// actually completed, so a task finished late does not come back already overdue), or
    /// `None` when the series has ended.
    pub fn next_after(&self, current: NaiveDate, floor: NaiveDate) -> Option<NaiveDate> {
        let mut date = current;
        for _ in 0..MAX_STEPS {
            date = self.step(date)?;
            if date > floor && date > current {
                return self.until.is_none_or(|until| date <= until).then_some(date);
            }
            if self.until.is_some_and(|until| date > until) {
                return None;
            }
        }
        None
    }

    fn step(&self, date: NaiveDate) -> Option<NaiveDate> {
        let n = i64::from(self.interval);
        match self.freq {
            Freq::Daily => date.checked_add_signed(Duration::days(n)),
            Freq::Weekly => date.checked_add_signed(Duration::days(7 * n)),
            Freq::Monthly => {
                let index = i64::from(date.year()) * 12 + i64::from(date.month0()) + n;
                self.month_date(
                    index.div_euclid(12),
                    index.rem_euclid(12) as u32 + 1,
                    date.day(),
                )
            }
            Freq::Yearly => {
                let year = i64::from(date.year()) + n;
                self.month_date(year, self.month.unwrap_or_else(|| date.month()), date.day())
            }
        }
    }

    /// `day` (or the series' anchor day) in that month, or the month's last day if it is shorter.
    fn month_date(&self, year: i64, month: u32, fallback_day: u32) -> Option<NaiveDate> {
        let year = i32::try_from(year).ok()?;
        let wanted = self.month_day.unwrap_or(fallback_day);
        (1..=wanted.min(31))
            .rev()
            .find_map(|day| NaiveDate::from_ymd_opt(year, month, day))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(text: &str) -> NaiveDate {
        NaiveDate::parse_from_str(text, "%Y-%m-%d").expect("date")
    }

    fn rule(text: &str, start: &str) -> Rule {
        Rule::parse(text)
            .expect("rule")
            .anchored(d(start))
            .expect("anchored")
    }

    #[test]
    fn parses_and_formats_the_supported_subset() {
        let parsed = Rule::parse("FREQ=WEEKLY;INTERVAL=2;UNTIL=20261231").expect("parse");
        assert_eq!(parsed.freq, Freq::Weekly);
        assert_eq!(parsed.interval, 2);
        assert_eq!(parsed.until, Some(d("2026-12-31")));
        assert_eq!(parsed.format(), "FREQ=WEEKLY;INTERVAL=2;UNTIL=20261231");
        assert_eq!(Rule::parse("FREQ=DAILY").expect("default").interval, 1);
    }

    #[test]
    fn rejects_what_it_does_not_understand() {
        for bad in [
            "",
            "FREQ=HOURLY",
            "INTERVAL=2",
            "FREQ=DAILY;INTERVAL=0",
            "FREQ=DAILY;INTERVAL=366",
            "FREQ=DAILY;COUNT=3",
            "FREQ=DAILY;UNTIL=2026-12-31",
            "FREQ=DAILY;BYDAY=MO",
            "nada",
        ] {
            assert!(Rule::parse(bad).is_err(), "{bad}");
        }
        assert!(
            Rule::parse("FREQ=DAILY;UNTIL=20260101")
                .expect("parse")
                .anchored(d("2026-02-01"))
                .is_err()
        );
    }

    #[test]
    fn daily_and_weekly_steps_skip_what_has_already_passed() {
        let daily = rule("FREQ=DAILY;INTERVAL=3", "2026-03-10");
        assert_eq!(
            daily.next_after(d("2026-03-10"), d("2026-03-10")),
            Some(d("2026-03-13"))
        );
        // Finished 10 days late: the next one is the first after that day, still on the cycle.
        assert_eq!(
            daily.next_after(d("2026-03-10"), d("2026-03-20")),
            Some(d("2026-03-22"))
        );
        let weekly = rule("FREQ=WEEKLY", "2026-03-10");
        assert_eq!(
            weekly.next_after(d("2026-03-10"), d("2026-03-10")),
            Some(d("2026-03-17"))
        );
    }

    #[test]
    fn monthly_sticks_to_the_day_and_clamps_in_short_months_without_drifting() {
        let monthly = rule("FREQ=MONTHLY", "2026-01-31");
        assert_eq!(monthly.month_day, Some(31));
        let feb = monthly
            .next_after(d("2026-01-31"), d("2026-01-31"))
            .expect("feb");
        assert_eq!(feb, d("2026-02-28"));
        let mar = monthly.next_after(feb, feb).expect("mar");
        assert_eq!(mar, d("2026-03-31"));
        let apr = monthly.next_after(mar, mar).expect("apr");
        assert_eq!(apr, d("2026-04-30"));
        assert_eq!(
            monthly.next_after(d("2027-12-15"), d("2027-12-15")),
            Some(d("2028-01-31")),
            "the year rolls over"
        );
    }

    #[test]
    fn a_leap_day_comes_back_on_the_28th_in_common_years() {
        let yearly = rule("FREQ=YEARLY", "2028-02-29");
        assert_eq!((yearly.month, yearly.month_day), (Some(2), Some(29)));
        let y2029 = yearly
            .next_after(d("2028-02-29"), d("2028-02-29"))
            .expect("2029");
        assert_eq!(y2029, d("2029-02-28"));
        let y2032 = rule("FREQ=YEARLY;INTERVAL=4", "2028-02-29")
            .next_after(d("2028-02-29"), d("2028-02-29"))
            .expect("2032");
        assert_eq!(y2032, d("2032-02-29"));
    }

    #[test]
    fn the_series_ends_after_until() {
        let limited = rule("FREQ=DAILY;UNTIL=20260312", "2026-03-10");
        assert_eq!(
            limited.next_after(d("2026-03-10"), d("2026-03-10")),
            Some(d("2026-03-11"))
        );
        assert_eq!(limited.next_after(d("2026-03-12"), d("2026-03-12")), None);
        assert_eq!(limited.next_after(d("2026-03-10"), d("2026-03-20")), None);
    }

    #[test]
    fn the_end_of_the_calendar_does_not_panic() {
        let max = NaiveDate::MAX;
        assert_eq!(rule("FREQ=DAILY", "2026-03-10").next_after(max, max), None);
        assert_eq!(rule("FREQ=YEARLY", "2026-03-10").next_after(max, max), None);
        assert_eq!(
            rule("FREQ=MONTHLY", "2026-03-10").next_after(max, max),
            None
        );
    }
}

//! Repeating tasks (F13-05, B-05). The rule lives in `tasks.rrule` as a small subset of RFC 5545:
//! `FREQ=DAILY|WEEKLY|MONTHLY|YEARLY`, `INTERVAL=n`, `UNTIL=YYYYMMDD`, `COUNT=n` (occurrences
//! left, counting the task that carries the rule), `BYDAY` (weekly only) plus `BYMONTHDAY` and
//! `BYMONTH`, which the app fills in from the task's date so that «every month on the 31st»
//! lands on the last day of short months without drifting to the 28th afterwards.
//!
//! Everything here works on calendar dates: no time zone, no clock.

use chrono::{Datelike, Duration, NaiveDate};

use super::validation::ValidationError;

pub const MAX_INTERVAL: u32 = 365;
pub const MAX_COUNT: u32 = 999;
const DAY_CODES: [&str; 7] = ["MO", "TU", "WE", "TH", "FR", "SA", "SU"];
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
    /// Weekly only: weekdays as a bit mask, bit 0 = Monday. Zero means «the task's own weekday».
    pub by_day: u8,
    /// Occurrences left including the current task; `None` for no limit.
    pub count: Option<u32>,
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
        let mut by_day = 0u8;
        let mut count = None;
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
                "COUNT" => {
                    count = Some(
                        value
                            .parse::<u32>()
                            .ok()
                            .filter(|n| (1..=MAX_COUNT).contains(n))
                            .ok_or_else(|| invalid("COUNT va de 1 a 999"))?,
                    );
                }
                "BYDAY" => {
                    for code in value.split(',') {
                        let index = DAY_CODES
                            .iter()
                            .position(|known| *known == code)
                            .ok_or_else(|| invalid("día de la semana desconocido"))?;
                        by_day |= 1 << index;
                    }
                }
                _ => return Err(invalid("parte no admitida")),
            }
        }
        let freq = freq.ok_or_else(|| invalid("falta FREQ"))?;
        if by_day != 0 && freq != Freq::Weekly {
            return Err(invalid("BYDAY solo se admite con FREQ=WEEKLY"));
        }
        if until.is_some() && count.is_some() {
            return Err(invalid("UNTIL y COUNT no pueden ir juntos"));
        }
        Ok(Self {
            freq,
            interval,
            until,
            month_day,
            month,
            by_day,
            count,
        })
    }

    pub fn format(&self) -> String {
        let mut text = format!("FREQ={};INTERVAL={}", self.freq.as_str(), self.interval);
        if self.by_day != 0 {
            let codes: Vec<&str> = DAY_CODES
                .iter()
                .enumerate()
                .filter(|(index, _)| self.by_day & (1 << index) != 0)
                .map(|(_, code)| *code)
                .collect();
            text.push_str(&format!(";BYDAY={}", codes.join(",")));
        }
        if let Some(day) = self.month_day {
            text.push_str(&format!(";BYMONTHDAY={day}"));
        }
        if let Some(month) = self.month {
            text.push_str(&format!(";BYMONTH={month}"));
        }
        if let Some(count) = self.count {
            text.push_str(&format!(";COUNT={count}"));
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
        if self.count.is_some_and(|left| left <= 1) {
            return None;
        }
        if self.freq == Freq::Weekly && self.by_day != 0 {
            return self.next_weekday(current, floor);
        }
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

    /// The rule for the task that follows this one: one occurrence fewer to go.
    pub fn successor(&self) -> Self {
        Self {
            count: self.count.map(|left| left.saturating_sub(1)),
            ..self.clone()
        }
    }

    /// Next selected weekday, in a week that is a whole number of intervals after the current one
    /// (weeks start on Monday), so a late completion stays on the cycle.
    fn next_weekday(&self, current: NaiveDate, floor: NaiveDate) -> Option<NaiveDate> {
        let base_week = week_index(current);
        let mut date = current.max(floor);
        for _ in 0..MAX_STEPS {
            date = date.checked_add_signed(Duration::days(1))?;
            if self.until.is_some_and(|until| date > until) {
                return None;
            }
            let selected = self.by_day & (1 << date.weekday().num_days_from_monday()) != 0;
            if selected && (week_index(date) - base_week) % i64::from(self.interval) == 0 {
                return Some(date);
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

/// Most occurrences looked at in one expansion, so a rule from long ago cannot make a query
/// walk for ever (a daily series needs 20 000 steps to cover 55 years).
pub const MAX_SCANNED_OCCURRENCES: usize = 20_000;

/// The dates of a series, in order, starting with its first one (D-94).
pub struct Occurrences {
    rule: Rule,
    next: Option<NaiveDate>,
}

impl Iterator for Occurrences {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<NaiveDate> {
        let current = self.next?;
        self.next = self.rule.next_after(current, current);
        self.rule = self.rule.successor();
        Some(current)
    }
}

impl Rule {
    /// Every date of the series that starts on `start`, which must be the date the rule was
    /// anchored to. Without `UNTIL` or `COUNT` the series never ends: take what you need.
    pub fn occurrences(&self, start: NaiveDate) -> Occurrences {
        Occurrences {
            rule: self.clone(),
            next: Some(start),
        }
    }

    /// The dates of the series from `start` that fall in `from..=to`.
    pub fn occurrences_in(
        &self,
        start: NaiveDate,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Vec<NaiveDate> {
        self.occurrences(start)
            .take(MAX_SCANNED_OCCURRENCES)
            .take_while(|date| *date <= to)
            .filter(|date| *date >= from)
            .collect()
    }

    /// The first date of the series on or after `from`.
    pub fn first_on_or_after(&self, start: NaiveDate, from: NaiveDate) -> Option<NaiveDate> {
        self.occurrences(start)
            .take(MAX_SCANNED_OCCURRENCES)
            .find(|date| *date >= from)
    }
}

/// Whole weeks since a fixed Monday, so that two dates in the same Monday-to-Sunday week match.
fn week_index(date: NaiveDate) -> i64 {
    let monday = date - Duration::days(i64::from(date.weekday().num_days_from_monday()));
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 5).unwrap_or(NaiveDate::MIN);
    monday.signed_duration_since(epoch).num_days().div_euclid(7)
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
            "FREQ=DAILY;COUNT=0",
            "FREQ=DAILY;COUNT=1000",
            "FREQ=DAILY;COUNT=3;UNTIL=20261231",
            "FREQ=MONTHLY;BYDAY=MO",
            "FREQ=WEEKLY;BYDAY=XX",
            "FREQ=DAILY;UNTIL=2026-12-31",
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
    fn occurrences_expand_a_series_inside_a_window() {
        let weekly = rule("FREQ=WEEKLY;INTERVAL=1", "2026-03-02");
        assert_eq!(
            weekly.occurrences_in(d("2026-03-02"), d("2026-03-10"), d("2026-03-24")),
            [d("2026-03-16"), d("2026-03-23")]
        );

        let counted = rule("FREQ=DAILY;INTERVAL=1;COUNT=3", "2026-03-30");
        assert_eq!(
            counted.occurrences_in(d("2026-03-30"), d("2026-01-01"), d("2026-12-31")),
            [d("2026-03-30"), d("2026-03-31"), d("2026-04-01")]
        );

        let until = rule("FREQ=DAILY;INTERVAL=2;UNTIL=20260307", "2026-03-01");
        assert_eq!(
            until.occurrences_in(d("2026-03-01"), d("2026-03-02"), d("2026-04-01")),
            [d("2026-03-03"), d("2026-03-05"), d("2026-03-07")]
        );

        let days = rule("FREQ=WEEKLY;INTERVAL=1;BYDAY=MO,WE", "2026-03-02");
        assert_eq!(
            days.occurrences_in(d("2026-03-02"), d("2026-03-02"), d("2026-03-11")),
            [
                d("2026-03-02"),
                d("2026-03-04"),
                d("2026-03-09"),
                d("2026-03-11")
            ]
        );

        let month_end = rule("FREQ=MONTHLY;INTERVAL=1", "2026-01-31");
        assert_eq!(
            month_end.occurrences_in(d("2026-01-31"), d("2026-02-01"), d("2026-04-30")),
            [d("2026-02-28"), d("2026-03-31"), d("2026-04-30")]
        );
        assert!(
            weekly
                .occurrences_in(d("2026-03-02"), d("2026-03-20"), d("2026-03-01"))
                .is_empty()
        );
    }

    #[test]
    fn first_on_or_after_finds_the_next_date_of_a_series() {
        let daily = rule("FREQ=DAILY;INTERVAL=1;COUNT=3", "2026-03-01");
        assert_eq!(
            daily.first_on_or_after(d("2026-03-01"), d("2026-03-02")),
            Some(d("2026-03-02"))
        );
        assert_eq!(
            daily.first_on_or_after(d("2026-03-01"), d("2026-03-04")),
            None
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

    #[test]
    fn weekly_by_day_visits_the_chosen_weekdays_every_n_weeks() {
        // 2026-03-09 is a Monday.
        let weekly = rule("FREQ=WEEKLY;BYDAY=MO,WE,FR", "2026-03-09");
        assert_eq!(weekly.format(), "FREQ=WEEKLY;INTERVAL=1;BYDAY=MO,WE,FR");
        let wed = weekly
            .next_after(d("2026-03-09"), d("2026-03-09"))
            .expect("wed");
        assert_eq!(wed, d("2026-03-11"));
        let fri = weekly.next_after(wed, wed).expect("fri");
        assert_eq!(fri, d("2026-03-13"));
        assert_eq!(weekly.next_after(fri, fri), Some(d("2026-03-16")));
        // Completed on the Saturday: the next one is the following Monday, not a past day.
        assert_eq!(
            weekly.next_after(d("2026-03-09"), d("2026-03-14")),
            Some(d("2026-03-16"))
        );
        let biweekly = rule("FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,TU", "2026-03-09");
        assert_eq!(
            biweekly.next_after(d("2026-03-09"), d("2026-03-09")),
            Some(d("2026-03-10"))
        );
        assert_eq!(
            biweekly.next_after(d("2026-03-10"), d("2026-03-10")),
            Some(d("2026-03-23"))
        );
        assert_eq!(
            biweekly.next_after(d("2026-03-10"), d("2026-03-30")),
            Some(d("2026-04-06")),
            "late completion stays on the two-week cycle"
        );
        // Sunday belongs to the week that started on the previous Monday.
        let sunday = rule("FREQ=WEEKLY;INTERVAL=2;BYDAY=SU,MO", "2026-03-15");
        assert_eq!(
            sunday.next_after(d("2026-03-15"), d("2026-03-15")),
            Some(d("2026-03-23"))
        );
    }

    #[test]
    fn count_runs_out_and_successor_counts_down() {
        let counted = rule("FREQ=DAILY;COUNT=3", "2026-03-10");
        assert_eq!(counted.count, Some(3));
        let second = counted
            .next_after(d("2026-03-10"), d("2026-03-10"))
            .expect("second");
        let rest = counted.successor();
        assert_eq!(rest.count, Some(2));
        let third = rest.next_after(second, second).expect("third");
        assert_eq!(third, d("2026-03-12"));
        let last = rest.successor();
        assert_eq!(last.count, Some(1));
        assert_eq!(last.next_after(third, third), None);
        assert_eq!(last.format(), "FREQ=DAILY;INTERVAL=1;COUNT=1");
        assert_eq!(rule("FREQ=DAILY", "2026-03-10").successor().count, None);
    }

    #[test]
    fn by_day_respects_until() {
        let limited = rule("FREQ=WEEKLY;BYDAY=MO;UNTIL=20260316", "2026-03-09");
        assert_eq!(
            limited.next_after(d("2026-03-09"), d("2026-03-09")),
            Some(d("2026-03-16"))
        );
        assert_eq!(limited.next_after(d("2026-03-16"), d("2026-03-16")), None);
    }
}

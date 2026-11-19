//! Front/back parity (A9-01): the TypeScript layer re-implements some domain rules (repeat rules,
//! display order, local/UTC conversion, reminder instants, durations). Both sides read the same
//! vectors from `fixtures/parity/*.json`; `src/lib/parity.test.ts` runs them against the
//! TypeScript code. Change a vector and both suites must still agree.

use std::{fs, path::PathBuf};

use chrono::{
    DateTime, Datelike, Duration, FixedOffset, LocalResult, NaiveDate, NaiveDateTime, TimeZone,
    Utc, Weekday,
};
use serde::Deserialize;
use serde_json::Value;

use crate::domain::{
    dates,
    ordering::{self, Sortable},
    recurrence::{Freq, Rule},
    reminders,
    validation::validate_duration,
};

fn load(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("parity")
        .join(name);
    let text = fs::read_to_string(&path).expect("parity fixture must exist");
    serde_json::from_str(&text).expect("parity fixture is valid JSON")
}

fn field<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get(key).expect("fixture field")
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    field(value, key).as_str().expect("fixture text")
}

/// Europe/Madrid: UTC+1, UTC+2 from the last Sunday of March to the last Sunday of October,
/// both changes at 01:00 UTC.
#[derive(Clone, Copy, Debug)]
struct Madrid;

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

impl TimeZone for Madrid {
    type Offset = FixedOffset;

    fn from_offset(_: &FixedOffset) -> Self {
        Madrid
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

fn instant(text: &str) -> DateTime<Utc> {
    dates::parse_instant(text).expect("instant")
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Form {
    freq: String,
    interval: u32,
    until: String,
    days: Vec<String>,
    count: Option<u32>,
}

const DAY_CODES: [&str; 7] = ["MO", "TU", "WE", "TH", "FR", "SA", "SU"];

fn freq_of(code: &str) -> Freq {
    match code {
        "DAILY" => Freq::Daily,
        "WEEKLY" => Freq::Weekly,
        "MONTHLY" => Freq::Monthly,
        _ => Freq::Yearly,
    }
}

fn form_of(rule: &Rule) -> Form {
    let freq = ["DAILY", "WEEKLY", "MONTHLY", "YEARLY"]
        .into_iter()
        .find(|code| freq_of(code) == rule.freq)
        .expect("freq");
    Form {
        freq: freq.to_string(),
        interval: rule.interval,
        until: rule.until.map(dates::format_date).unwrap_or_default(),
        days: DAY_CODES
            .iter()
            .enumerate()
            .filter(|(index, _)| rule.by_day & (1 << index) != 0)
            .map(|(_, code)| (*code).to_string())
            .collect(),
        count: rule.count,
    }
}

fn same_form(actual: &Form, expected: &Value) -> bool {
    let wanted: Form = serde_json::from_value(expected.clone()).expect("form");
    actual.freq == wanted.freq
        && actual.interval == wanted.interval
        && actual.until == wanted.until
        && actual.days == wanted.days
        && actual.count == wanted.count
}

#[test]
fn repeat_rules_parse_like_the_frontend() {
    let data = load("recurrence.json");
    for case in field(&data, "parse").as_array().expect("cases") {
        let rule = Rule::parse(text(case, "rule")).expect("valid rule");
        assert!(same_form(&form_of(&rule), field(case, "form")), "{case}");
    }
    for rule in field(&data, "invalid").as_array().expect("invalid") {
        let rule = rule.as_str().expect("rule text");
        assert!(Rule::parse(rule).is_err(), "{rule:?} must be rejected");
    }
}

#[test]
fn repeat_rules_are_built_like_the_frontend() {
    let data = load("recurrence.json");
    for case in field(&data, "build").as_array().expect("cases") {
        let form: Form = serde_json::from_value(field(case, "form").clone()).expect("form");
        let mut by_day = 0u8;
        for code in &form.days {
            let index = DAY_CODES.iter().position(|known| known == code);
            by_day |= 1 << index.expect("day code");
        }
        let rule = Rule {
            freq: freq_of(&form.freq),
            interval: form.interval,
            until: (!form.until.is_empty()).then(|| dates::parse_date(&form.until).expect("date")),
            month_day: None,
            month: None,
            by_day,
            count: form.count,
        };
        assert_eq!(rule.format(), text(case, "rule"), "{case}");
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    id: String,
    due_date: Option<String>,
    due_time: Option<String>,
    is_completed: bool,
    completed_at: Option<String>,
    position: i64,
}

impl Sortable for Item {
    fn id(&self) -> &str {
        &self.id
    }
    fn due_date(&self) -> Option<&str> {
        self.due_date.as_deref()
    }
    fn due_time(&self) -> Option<&str> {
        self.due_time.as_deref()
    }
    fn is_completed(&self) -> bool {
        self.is_completed
    }
    fn completed_at(&self) -> Option<&str> {
        self.completed_at.as_deref()
    }
    fn position(&self) -> i64 {
        self.position
    }
}

fn items(case: &Value) -> Vec<Item> {
    serde_json::from_value(field(case, "tasks").clone()).expect("tasks")
}

fn ids(items: &[Item]) -> Vec<String> {
    items.iter().map(|item| item.id.clone()).collect()
}

fn id_list(value: &Value) -> Vec<String> {
    serde_json::from_value(value.clone()).expect("id list")
}

#[test]
fn task_order_matches_the_shared_vectors() {
    let data = load("ordering.json");
    for case in field(&data, "todo").as_array().expect("cases") {
        let mut tasks = items(case);
        ordering::sort_todo(&mut tasks);
        assert_eq!(ids(&tasks), id_list(field(case, "expected")), "{case}");
    }
    for case in field(&data, "day").as_array().expect("cases") {
        let mut tasks = items(case);
        ordering::sort_day(&mut tasks);
        assert_eq!(ids(&tasks), id_list(field(case, "expected")), "{case}");
    }
    for case in field(&data, "groups").as_array().expect("cases") {
        let groups = ordering::group_by_day(items(case));
        let actual: Vec<(String, Vec<String>)> = groups
            .iter()
            .map(|group| (group.date.clone(), ids(&group.tasks)))
            .collect();
        let expected: Vec<(String, Vec<String>)> = field(case, "expected")
            .as_array()
            .expect("groups")
            .iter()
            .map(|group| {
                (
                    text(group, "date").to_string(),
                    id_list(field(group, "ids")),
                )
            })
            .collect();
        assert_eq!(actual, expected, "{case}");
    }
}

#[test]
fn local_and_utc_conversions_match_the_shared_vectors() {
    let data = load("dates.json");
    for case in field(&data, "localToUtc").as_array().expect("cases") {
        let (date, time) = text(case, "local").split_once('T').expect("local");
        let utc = dates::local_to_utc(
            &Madrid,
            dates::parse_date(date).expect("date"),
            dates::parse_time(time).expect("time"),
        )
        .expect("utc");
        assert_eq!(dates::format_instant(utc), text(case, "utc"), "{case}");
    }
    for case in field(&data, "utcToLocalDate").as_array().expect("cases") {
        let local = dates::utc_to_local(&Madrid, instant(text(case, "utc")));
        assert_eq!(
            dates::format_date(local.date()),
            text(case, "date"),
            "{case}"
        );
        assert_eq!(
            dates::format_time(local.time()),
            text(case, "time"),
            "{case}"
        );
    }
}

#[test]
fn reminder_instants_match_the_shared_vectors() {
    let data = load("reminders.json");
    for case in field(&data, "offsetNotifyAt").as_array().expect("cases") {
        let offset = field(case, "offset").as_i64().expect("offset");
        let at = reminders::offset_notify_at(
            &Madrid,
            text(case, "dueDate"),
            text(case, "dueTime"),
            offset,
        )
        .expect("notify at");
        assert_eq!(dates::format_instant(at), text(case, "notifyAt"), "{case}");
    }
}

#[test]
fn durations_match_the_shared_vectors() {
    let data = load("duration.json");
    for case in field(&data, "cases").as_array().expect("cases") {
        let duration = field(case, "duration").as_i64().expect("duration");
        let valid = field(case, "valid").as_bool().expect("valid");
        let result = validate_duration(Some(text(case, "start")), Some(duration));
        assert_eq!(result.is_ok(), valid, "{case}");
    }
}

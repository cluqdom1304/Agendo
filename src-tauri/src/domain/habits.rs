//! Habit rules (D-95): which days a habit is due and how long its streak is. Pure: «today» is a
//! parameter (the service takes it from the `Clock`), and the streak is always derived from the
//! days that were marked, never stored.

use std::collections::BTreeSet;

use chrono::{Datelike, Duration, NaiveDate};

/// Every day of the week.
pub const EVERY_DAY: u8 = 0b111_1111;

/// Days looked at at most when computing a streak: ten years.
const MAX_SCAN_DAYS: i64 = 3_660;

/// Whether `days` (bit 0 = Monday ... bit 6 = Sunday) includes the weekday of `date`.
pub fn is_scheduled(days: u8, date: NaiveDate) -> bool {
    days & (1 << date.weekday().num_days_from_monday()) != 0
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Streak {
    /// Scheduled days in a row, up to today, that were marked. A scheduled day that is still
    /// to be marked today does not break it.
    pub current: u32,
    pub best: u32,
}

/// The streaks of a habit scheduled on `days` whose marked days are `done`. Days the habit is
/// not scheduled are skipped: they neither count nor break the streak, and a mark on one of
/// them is ignored. Marks after `today` are ignored too.
pub fn streak(days: u8, done: &BTreeSet<NaiveDate>, today: NaiveDate) -> Streak {
    let Some(first) = done.iter().find(|date| **date <= today).copied() else {
        return Streak::default();
    };
    let first = first.max(today - Duration::days(MAX_SCAN_DAYS));
    let mut run = 0_u32;
    let mut best = 0_u32;
    let mut date = first;
    while date <= today {
        if is_scheduled(days, date) {
            if done.contains(&date) {
                run += 1;
                best = best.max(run);
            } else if date != today {
                run = 0;
            }
        }
        date += Duration::days(1);
    }
    Streak { current: run, best }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(text: &str) -> NaiveDate {
        NaiveDate::parse_from_str(text, "%Y-%m-%d").expect("date")
    }

    fn marks(days: &[&str]) -> BTreeSet<NaiveDate> {
        days.iter().map(|day| d(day)).collect()
    }

    #[test]
    fn schedule_follows_the_weekday_mask() {
        // 2026-03-09 is a Monday.
        assert!(is_scheduled(EVERY_DAY, d("2026-03-15")));
        let weekdays = 0b001_1111;
        assert!(is_scheduled(weekdays, d("2026-03-09")));
        assert!(is_scheduled(weekdays, d("2026-03-13")));
        assert!(!is_scheduled(weekdays, d("2026-03-14")));
        assert!(!is_scheduled(weekdays, d("2026-03-15")));
    }

    #[test]
    fn a_daily_streak_counts_consecutive_marked_days() {
        let done = marks(&["2026-03-07", "2026-03-08", "2026-03-09", "2026-03-10"]);
        assert_eq!(
            streak(EVERY_DAY, &done, d("2026-03-10")),
            Streak {
                current: 4,
                best: 4
            }
        );
    }

    #[test]
    fn today_still_to_do_does_not_break_the_streak_but_a_missed_yesterday_does() {
        let done = marks(&["2026-03-08", "2026-03-09"]);
        assert_eq!(
            streak(EVERY_DAY, &done, d("2026-03-10")),
            Streak {
                current: 2,
                best: 2
            }
        );
        assert_eq!(
            streak(EVERY_DAY, &done, d("2026-03-11")),
            Streak {
                current: 0,
                best: 2
            }
        );
    }

    #[test]
    fn the_best_streak_survives_a_later_gap() {
        let done = marks(&[
            "2026-03-01",
            "2026-03-02",
            "2026-03-03",
            "2026-03-04",
            "2026-03-08",
            "2026-03-09",
        ]);
        assert_eq!(
            streak(EVERY_DAY, &done, d("2026-03-09")),
            Streak {
                current: 2,
                best: 4
            }
        );
    }

    #[test]
    fn days_off_neither_count_nor_break_the_streak() {
        // Monday to Friday; the weekend of the 14th and 15th is skipped.
        let weekdays = 0b001_1111;
        let done = marks(&[
            "2026-03-12",
            "2026-03-13",
            "2026-03-14", // a mark on a day off is ignored
            "2026-03-16",
            "2026-03-17",
        ]);
        assert_eq!(
            streak(weekdays, &done, d("2026-03-17")),
            Streak {
                current: 4,
                best: 4
            }
        );
        // On a day off, nothing is pending and the streak stays.
        assert_eq!(
            streak(weekdays, &done, d("2026-03-15")),
            Streak {
                current: 2,
                best: 2
            }
        );
    }

    #[test]
    fn marks_in_the_future_and_an_empty_history_give_no_streak() {
        assert_eq!(
            streak(EVERY_DAY, &BTreeSet::new(), d("2026-03-10")),
            Streak::default()
        );
        assert_eq!(
            streak(EVERY_DAY, &marks(&["2026-03-12"]), d("2026-03-10")),
            Streak::default()
        );
    }

    #[test]
    fn a_very_old_history_is_bounded() {
        let done = marks(&["1990-01-01", "2026-03-10"]);
        let found = streak(EVERY_DAY, &done, d("2026-03-10"));
        assert_eq!(
            found,
            Streak {
                current: 1,
                best: 1
            }
        );
    }
}

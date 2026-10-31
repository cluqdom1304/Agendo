//! Display order of tasks (D-05). Pure functions over anything that exposes the fields the
//! rules need, so they are tested without a database.
//!
//! * POR HACER: `position` ascending.
//! * AGENDA: days with the latest `due_date` first.
//! * Inside a day: tasks with a time first (time ascending), then tasks without time
//!   (`position` ascending).
//! * Completed tasks form a "Completadas" subsection at the end of their block (the "to do"
//!   list or a day), ordered by `completed_at` ascending.
//!
//! Dates, times and instants compare as text because their formats are fixed-width and
//! zero-padded (`dates.rs` validates them). `id` breaks every tie, so the result does not depend
//! on the input order.

use std::{cmp::Ordering, collections::BTreeMap};

use serde::Serialize;

/// What the ordering rules read from a task.
pub trait Sortable {
    fn id(&self) -> &str;
    fn due_date(&self) -> Option<&str>;
    fn due_time(&self) -> Option<&str>;
    fn is_completed(&self) -> bool;
    fn completed_at(&self) -> Option<&str>;
    fn position(&self) -> i64;
}

/// Tasks of one day, already in display order.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayGroup<T> {
    /// `YYYY-MM-DD`.
    pub date: String,
    pub tasks: Vec<T>,
}

fn completed_order<T: Sortable>(a: &T, b: &T) -> Ordering {
    a.completed_at()
        .cmp(&b.completed_at())
        .then_with(|| a.id().cmp(b.id()))
}

fn position_order<T: Sortable>(a: &T, b: &T) -> Ordering {
    a.position()
        .cmp(&b.position())
        .then_with(|| a.id().cmp(b.id()))
}

/// Pending tasks by position, then the completed ones.
pub fn sort_todo<T: Sortable>(tasks: &mut [T]) {
    tasks.sort_by(|a, b| {
        a.is_completed()
            .cmp(&b.is_completed())
            .then_with(|| match a.is_completed() {
                true => completed_order(a, b),
                false => position_order(a, b),
            })
    });
}

/// Orders the tasks of a single day: timed, untimed, then completed.
pub fn sort_day<T: Sortable>(tasks: &mut [T]) {
    tasks.sort_by(|a, b| {
        a.is_completed().cmp(&b.is_completed()).then_with(|| {
            if a.is_completed() {
                return completed_order(a, b);
            }
            match (a.due_time(), b.due_time()) {
                (Some(x), Some(y)) => x.cmp(y).then_with(|| position_order(a, b)),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => position_order(a, b),
            }
        })
    });
}

/// Groups dated tasks by day, latest day first, each day in the order of [`sort_day`].
/// Tasks without a date belong to no day and are left out.
pub fn group_by_day<T: Sortable>(tasks: Vec<T>) -> Vec<DayGroup<T>> {
    let mut days: BTreeMap<String, Vec<T>> = BTreeMap::new();
    for task in tasks {
        if let Some(date) = task.due_date().map(str::to_string) {
            days.entry(date).or_default().push(task);
        }
    }
    days.into_iter()
        .rev()
        .map(|(date, mut tasks)| {
            sort_day(&mut tasks);
            DayGroup { date, tasks }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Item {
        id: &'static str,
        date: Option<&'static str>,
        time: Option<&'static str>,
        completed_at: Option<&'static str>,
        position: i64,
    }

    impl Sortable for Item {
        fn id(&self) -> &str {
            self.id
        }
        fn due_date(&self) -> Option<&str> {
            self.date
        }
        fn due_time(&self) -> Option<&str> {
            self.time
        }
        fn is_completed(&self) -> bool {
            self.completed_at.is_some()
        }
        fn completed_at(&self) -> Option<&str> {
            self.completed_at
        }
        fn position(&self) -> i64 {
            self.position
        }
    }

    /// Pending task: `id`, optional time, position.
    fn open(id: &'static str, time: Option<&'static str>, position: i64) -> Item {
        Item {
            id,
            date: Some("2026-03-10"),
            time,
            completed_at: None,
            position,
        }
    }

    /// Completed task: `id`, completion instant, plus a position and time that must not matter.
    fn done(id: &'static str, at: &'static str, position: i64, time: Option<&'static str>) -> Item {
        Item {
            completed_at: Some(at),
            ..open(id, time, position)
        }
    }

    fn undated(item: Item) -> Item {
        Item { date: None, ..item }
    }

    fn ids<T: Sortable>(tasks: &[T]) -> Vec<&str> {
        tasks.iter().map(Sortable::id).collect()
    }

    fn permutations(items: &[Item]) -> Vec<Vec<Item>> {
        if items.len() <= 1 {
            return vec![items.to_vec()];
        }
        let mut result = Vec::new();
        for index in 0..items.len() {
            let mut rest = items.to_vec();
            let head = rest.remove(index);
            for mut tail in permutations(&rest) {
                tail.insert(0, head.clone());
                result.push(tail);
            }
        }
        result
    }

    #[test]
    fn sort_day_table() {
        let cases: Vec<(&str, Vec<Item>, Vec<&str>)> = vec![
            ("empty", vec![], vec![]),
            ("single", vec![open("a", None, 0)], vec!["a"]),
            (
                "untimed by position",
                vec![open("c", None, 2), open("a", None, 0), open("b", None, 1)],
                vec!["a", "b", "c"],
            ),
            (
                "timed before untimed even with a worse position",
                vec![open("untimed", None, 0), open("timed", Some("23:59"), 9)],
                vec!["timed", "untimed"],
            ),
            (
                "timed by time ascending",
                vec![
                    open("noon", Some("12:00"), 0),
                    open("early", Some("07:30"), 5),
                    open("late", Some("18:45"), 1),
                ],
                vec!["early", "noon", "late"],
            ),
            (
                "same time falls back to position then id",
                vec![
                    open("z", Some("09:00"), 1),
                    open("b", Some("09:00"), 2),
                    open("a", Some("09:00"), 2),
                ],
                vec!["z", "a", "b"],
            ),
            (
                "same position falls back to id",
                vec![open("b", None, 0), open("a", None, 0)],
                vec!["a", "b"],
            ),
            (
                "completed go last regardless of time and position",
                vec![
                    done("d1", "2026-03-10T08:00:00Z", 0, Some("06:00")),
                    open("p1", None, 7),
                    open("p2", Some("20:00"), 8),
                ],
                vec!["p2", "p1", "d1"],
            ),
            (
                "completed by completion instant ascending",
                vec![
                    done("late", "2026-03-10T18:00:00Z", 0, None),
                    done("early", "2026-03-10T07:00:00Z", 5, Some("23:00")),
                    done("mid", "2026-03-10T12:00:00Z", 2, Some("01:00")),
                ],
                vec!["early", "mid", "late"],
            ),
            (
                "same completion instant falls back to id",
                vec![
                    done("b", "2026-03-10T07:00:00Z", 0, None),
                    done("a", "2026-03-10T07:00:00Z", 1, None),
                ],
                vec!["a", "b"],
            ),
            (
                "everything at once",
                vec![
                    done("done-2", "2026-03-10T09:00:00Z", 0, None),
                    open("untimed-2", None, 4),
                    open("timed-2", Some("14:00"), 0),
                    done("done-1", "2026-03-10T08:00:00Z", 3, Some("08:00")),
                    open("untimed-1", None, 1),
                    open("timed-1", Some("08:15"), 9),
                ],
                vec![
                    "timed-1",
                    "timed-2",
                    "untimed-1",
                    "untimed-2",
                    "done-1",
                    "done-2",
                ],
            ),
        ];

        for (label, mut tasks, expected) in cases {
            sort_day(&mut tasks);
            assert_eq!(ids(&tasks), expected, "{label}");
        }
    }

    #[test]
    fn sort_todo_table() {
        let cases: Vec<(&str, Vec<Item>, Vec<&str>)> = vec![
            ("empty", vec![], vec![]),
            (
                "by position ascending",
                vec![
                    undated(open("c", None, 2)),
                    undated(open("a", None, 0)),
                    undated(open("b", None, 1)),
                ],
                vec!["a", "b", "c"],
            ),
            (
                "time is irrelevant in the to-do list",
                vec![
                    undated(open("late", Some("20:00"), 0)),
                    undated(open("early", Some("06:00"), 1)),
                ],
                vec!["late", "early"],
            ),
            (
                "position ties fall back to id",
                vec![undated(open("b", None, 3)), undated(open("a", None, 3))],
                vec!["a", "b"],
            ),
            (
                "completed go to the end by completion instant",
                vec![
                    undated(done("second", "2026-03-10T10:00:00Z", 0, None)),
                    undated(open("p2", None, 5)),
                    undated(done("first", "2026-03-09T10:00:00Z", 9, None)),
                    undated(open("p1", None, 1)),
                ],
                vec!["p1", "p2", "first", "second"],
            ),
        ];

        for (label, mut tasks, expected) in cases {
            sort_todo(&mut tasks);
            assert_eq!(ids(&tasks), expected, "{label}");
        }
    }

    #[test]
    fn result_does_not_depend_on_input_order() {
        let day = vec![
            open("untimed-1", None, 0),
            open("untimed-2", None, 1),
            open("timed", Some("09:00"), 5),
            done("done-1", "2026-03-10T08:00:00Z", 2, None),
            done("done-2", "2026-03-10T09:00:00Z", 3, Some("07:00")),
        ];
        let mut canonical = day.clone();
        sort_day(&mut canonical);

        for mut shuffled in permutations(&day) {
            sort_day(&mut shuffled);
            assert_eq!(shuffled, canonical);
        }

        let todo: Vec<Item> = day.into_iter().map(undated).collect();
        let mut canonical = todo.clone();
        sort_todo(&mut canonical);
        for mut shuffled in permutations(&todo) {
            sort_todo(&mut shuffled);
            assert_eq!(shuffled, canonical);
        }
    }

    #[test]
    fn group_by_day_puts_the_latest_day_first_and_orders_each_day() {
        let day = |id, date, time, position| Item {
            date: Some(date),
            ..open(id, time, position)
        };
        let tasks = vec![
            day("old-b", "2026-01-05", None, 1),
            day("new-untimed", "2026-03-20", None, 0),
            day("mid", "2026-02-14", Some("10:00"), 0),
            day("old-a", "2026-01-05", Some("23:00"), 9),
            day("new-timed", "2026-03-20", Some("08:00"), 4),
            Item {
                date: Some("2026-03-20"),
                ..done("new-done", "2026-03-20T07:00:00Z", 0, None)
            },
        ];

        let groups = group_by_day(tasks);

        let summary: Vec<(&str, Vec<&str>)> = groups
            .iter()
            .map(|group| (group.date.as_str(), ids(&group.tasks)))
            .collect();
        assert_eq!(
            summary,
            [
                ("2026-03-20", vec!["new-timed", "new-untimed", "new-done"]),
                ("2026-02-14", vec!["mid"]),
                ("2026-01-05", vec!["old-a", "old-b"]),
            ]
        );
    }

    #[test]
    fn group_by_day_leaves_out_undated_tasks_and_handles_empty_input() {
        assert!(group_by_day::<Item>(vec![]).is_empty());
        assert!(group_by_day(vec![undated(open("a", None, 0))]).is_empty());

        let groups = group_by_day(vec![undated(open("a", None, 0)), open("b", None, 0)]);
        assert_eq!(groups.len(), 1);
        assert_eq!(ids(&groups[0].tasks), ["b"]);
    }

    #[test]
    fn day_group_serializes_in_camel_case() {
        let group = DayGroup {
            date: "2026-03-10".to_string(),
            tasks: vec![1, 2],
        };
        let json = serde_json::to_value(&group).expect("json");
        assert_eq!(
            json,
            serde_json::json!({ "date": "2026-03-10", "tasks": [1, 2] })
        );
    }
}

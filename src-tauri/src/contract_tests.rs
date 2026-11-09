//! IPC contract (SPEC §13): the JSON the backend sends must equal `fixtures/dto.json`, which the
//! frontend test `src/lib/types.test.ts` checks against the TypeScript types. Change a DTO and
//! both sides fail until the fixture and `src/lib/types.ts` are updated together.
//! Regenerate the fixture with `UPDATE_DTO_FIXTURE=1 cargo test contract`.

use std::{fs, path::PathBuf};

use serde_json::{Value, json};

use crate::{
    commands::tasks::{Sections, TasksListOutput},
    domain::{
        ordering::DayGroup,
        reminders::{Repeat, RepeatUnit},
    },
    repo::{
        model::{Reminder, Tag, TagWithCount, Task},
        search::SearchHit,
    },
    window::effects::{EffectApplied, EffectMode},
};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("dto.json")
}

fn task(with_values: bool) -> Task {
    Task {
        id: "task-1".into(),
        title: "Entregar práctica".into(),
        content_json: r#"{"type":"doc","content":[]}"#.into(),
        due_date: with_values.then(|| "2026-03-12".to_string()),
        due_time: with_values.then(|| "09:30".to_string()),
        is_completed: with_values,
        completed_at: with_values.then(|| "2026-03-12T08:00:00Z".to_string()),
        position: 3,
        tag_ids: vec!["tag-1".into()],
        reminders: vec![Reminder {
            id: "rem-1".into(),
            notify_at: "2026-03-12T08:30:00Z".into(),
            offset_minutes: with_values.then_some(60),
            is_sent: false,
            repeat: with_values.then_some(Repeat {
                every: 2,
                unit: RepeatUnit::Week,
                count: Some(5),
                until: None,
                occurrence: 3,
            }),
        }],
        created_at: "2026-03-10T10:00:00Z".into(),
        updated_at: "2026-03-10T11:00:00Z".into(),
    }
}

fn tag(parent: Option<&str>) -> Tag {
    Tag {
        id: "tag-1".into(),
        name: "Física".into(),
        color_hex: "#0A84FF".into(),
        parent_id: parent.map(str::to_string),
        position: 0,
    }
}

fn current() -> Value {
    let day = DayGroup {
        date: "2026-03-12".into(),
        tasks: vec![task(true)],
    };
    json!({
        "task": task(true),
        "taskWithoutOptionals": task(false),
        "tag": tag(Some("tag-0")),
        "tagWithoutParent": tag(None),
        "tagWithCount": TagWithCount { tag: tag(None), pending_count: 4 },
        "dayGroup": day,
        "sectionsOutput": TasksListOutput::Sections(Sections {
            todo: vec![task(false)],
            agenda: vec![day],
        }),
        "rangeOutput": TasksListOutput::Range(vec![task(true)]),
        "searchHit": SearchHit {
            task_id: "task-1".into(),
            title: "Entregar práctica".into(),
            snippet: "…revisar la práctica…".into(),
            due_date: None,
            is_completed: false,
        },
        "effectApplied": EffectApplied {
            requested: EffectMode::Mica,
            applied: EffectMode::Acrylic,
        },
    })
}

#[test]
fn serialized_dtos_match_the_shared_fixture() {
    let actual = current();
    let path = fixture_path();

    if std::env::var_os("UPDATE_DTO_FIXTURE").is_some() {
        let text = serde_json::to_string_pretty(&actual).expect("serialize fixture");
        fs::create_dir_all(path.parent().expect("fixture dir")).expect("create dir");
        fs::write(&path, format!("{text}\n")).expect("write fixture");
    }

    let stored = fs::read_to_string(&path).expect("fixtures/dto.json must exist");
    let expected: Value = serde_json::from_str(&stored).expect("fixture is valid JSON");
    assert_eq!(
        actual, expected,
        "DTO serialization changed: update fixtures/dto.json and src/lib/types.ts together"
    );
}

use chrono::Duration;

use super::*;
use crate::{repo::model::EMPTY_DOC, state::test_state};

fn state() -> AppState {
    test_state().0
}

fn create_titled(state: &AppState, title: &str, date: Option<&str>) -> Task {
    create(
        state,
        TaskCreateInput {
            title: Some(title.to_string()),
            due_date: date.map(str::to_string),
            ..TaskCreateInput::default()
        },
    )
    .expect("create")
}

fn view(state: &AppState, view: ListView) -> TasksListOutput {
    list(
        state,
        TasksListInput {
            view,
            from: None,
            to: None,
            tag_id: None,
        },
    )
    .expect("list")
}

fn sections(output: TasksListOutput) -> Sections {
    match output {
        TasksListOutput::Sections(sections) => sections,
        TasksListOutput::Range(_) => panic!("expected sections"),
    }
}

fn titles(tasks: &[Task]) -> Vec<&str> {
    tasks.iter().map(|task| task.title.as_str()).collect()
}

#[test]
fn list_all_returns_both_sections_like_the_two_separate_views() {
    let state = state();
    create_titled(&state, "Sin fecha", None);
    create_titled(&state, "Mañana", Some("2026-03-12"));
    create_titled(&state, "Antes", Some("2026-03-10"));
    let trashed = create_titled(&state, "Borrada", None);
    delete(&state, &trashed.id).expect("trash");

    let all = list_all(&state, None).expect("list all");
    let todo_view = sections(view(&state, ListView::Todo));
    let agenda_view = sections(view(&state, ListView::Agenda));
    assert_eq!(all.todo, todo_view.todo);
    assert_eq!(all.agenda, agenda_view.agenda);
    assert_eq!(titles(&all.todo), ["Sin fecha"]);
    assert_eq!(all.agenda.len(), 2);
}

#[test]
fn list_all_filters_by_tag() {
    let state = state();
    create_titled(&state, "Una", None);
    let none = list_all(&state, Some("no-such-tag")).expect("list all");
    assert!(none.todo.is_empty() && none.agenda.is_empty());
}

fn patch(json: &str) -> TaskPatchInput {
    serde_json::from_str(json).expect("patch json")
}

fn code_of<T: std::fmt::Debug>(result: AppResult<T>) -> String {
    result.expect_err("must fail").to_string()
}

fn local_utc(date: &str, time: &str, offset: i64) -> String {
    dates::format_instant(
        crate::domain::reminders::offset_notify_at(&Local, date, time, offset).expect("at"),
    )
}

fn reminders_of(state: &AppState, id: &str) -> Vec<crate::repo::model::Reminder> {
    state
        .db
        .read(|conn| Ok(reminders::for_task(conn, id)?))
        .expect("reminders")
}

fn timed_task_with_reminder(state: &AppState) -> Task {
    use crate::commands::reminders::{ReminderInput, set};
    let task = create(
        state,
        TaskCreateInput {
            due_date: Some("2030-05-01".into()),
            due_time: Some("09:00".into()),
            ..TaskCreateInput::default()
        },
    )
    .expect("task");
    let relative = ReminderInput {
        offset_minutes: Some(15),
        notify_at: None,
        repeat: None,
    };
    set(state, &task.id, &[relative], &Local).expect("reminder");
    task
}

#[test]
fn changing_the_time_of_a_task_reschedules_its_relative_reminders() {
    let state = state();
    let task = timed_task_with_reminder(&state);
    assert_eq!(
        reminders_of(&state, &task.id)[0].notify_at,
        local_utc("2030-05-01", "09:00", 15)
    );

    let updated = update(&state, &task.id, patch(r#"{"dueTime":"18:30"}"#)).expect("change time");
    assert_eq!(
        updated.reminders[0].notify_at,
        local_utc("2030-05-01", "18:30", 15)
    );

    let moved = update(&state, &task.id, patch(r#"{"dueDate":"2030-05-03"}"#)).expect("date");
    assert_eq!(
        moved.reminders[0].notify_at,
        local_utc("2030-05-03", "18:30", 15)
    );
}

#[test]
fn dragging_a_task_to_another_day_reschedules_its_reminders() {
    let state = state();
    let task = timed_task_with_reminder(&state);
    let moved = move_task(
        &state,
        &task.id,
        Section::Agenda,
        Some("2030-06-10".into()),
        None,
    )
    .expect("move");
    assert_eq!(
        moved.reminders[0].notify_at,
        local_utc("2030-06-10", "09:00", 15)
    );
}

#[test]
fn removing_the_time_removes_unsent_relative_reminders_but_not_custom_ones() {
    use crate::commands::reminders::{ReminderInput, set};
    let state = state();
    let task = timed_task_with_reminder(&state);
    let custom = ReminderInput {
        offset_minutes: None,
        notify_at: Some("2030-04-30T08:00:00Z".into()),
        repeat: None,
    };
    let relative = ReminderInput {
        offset_minutes: Some(15),
        notify_at: None,
        repeat: None,
    };
    set(&state, &task.id, &[relative, custom], &Local).expect("both");

    let updated = update(&state, &task.id, patch(r#"{"dueTime":null}"#)).expect("clear");

    assert_eq!(updated.reminders.len(), 1);
    assert_eq!(updated.reminders[0].offset_minutes, None);
}

#[test]
fn sent_reminders_are_not_touched_when_the_time_changes() {
    let state = state();
    let task = timed_task_with_reminder(&state);
    let before = reminders_of(&state, &task.id)[0].clone();
    let now = state.clock.now_string();
    state
        .db
        .write(|tx| Ok(reminders::mark_sent(tx, &before.id, &now)?))
        .expect("sent");

    update(&state, &task.id, patch(r#"{"dueTime":"18:30"}"#)).expect("change time");

    let after = reminders_of(&state, &task.id);
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].notify_at, before.notify_at);
    assert!(after[0].is_sent);
}

#[test]
fn create_uses_generated_id_and_clock_and_defaults() {
    let state = state();
    let task = create(&state, TaskCreateInput::default()).expect("create");

    assert_eq!(task.id, "id-1");
    assert_eq!(task.title, "");
    assert_eq!(task.content_json, EMPTY_DOC);
    assert_eq!(task.created_at, "2026-03-10T10:00:00Z");
    assert_eq!(task.position, 0);
}

#[test]
fn create_validates_every_field() {
    let state = state();
    let cases = [
        (
            TaskCreateInput {
                title: Some("x".repeat(501)),
                ..TaskCreateInput::default()
            },
            "VALIDATION: el título no puede superar 500 caracteres (tiene 501)",
        ),
        (
            TaskCreateInput {
                due_date: Some("2026-02-30".into()),
                ..TaskCreateInput::default()
            },
            "VALIDATION: fecha no válida «2026-02-30»: se espera AAAA-MM-DD",
        ),
        (
            TaskCreateInput {
                due_time: Some("09:00".into()),
                ..TaskCreateInput::default()
            },
            "VALIDATION: una hora requiere una fecha",
        ),
        (
            TaskCreateInput {
                content_json: Some("not json".into()),
                ..TaskCreateInput::default()
            },
            "VALIDATION: las notas no tienen un formato válido",
        ),
        (
            TaskCreateInput {
                tag_ids: Some(vec!["missing".into()]),
                ..TaskCreateInput::default()
            },
            "NOT_FOUND: el elemento referenciado no existe",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(code_of(create(&state, input)), expected);
    }
    assert!(sections(view(&state, ListView::Todo)).todo.is_empty());
}

#[test]
fn create_derives_content_text_for_search() {
    let state = state();
    let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"apuntes de álgebra"}]}]}"#;
    let task = create(
        &state,
        TaskCreateInput {
            content_json: Some(json.to_string()),
            ..TaskCreateInput::default()
        },
    )
    .expect("create");

    let indexed: i64 = state
        .db
        .read(|conn| {
            Ok(conn.query_row(
                "SELECT COUNT(*) FROM tasks_fts WHERE tasks_fts MATCH 'algebra' AND task_id = ?1",
                [&task.id],
                |row| row.get(0),
            )?)
        })
        .expect("fts");
    assert_eq!(indexed, 1);
}

#[test]
fn todo_and_agenda_views_are_ordered_by_the_domain_rules() {
    let state = state();
    create_titled(&state, "todo 1", None);
    let done = create_titled(&state, "todo 2", None);
    create_titled(&state, "todo 3", None);
    set_completed(&state, &done.id, true).expect("complete");
    create_titled(&state, "old day", Some("2026-03-01"));
    create_titled(&state, "new day", Some("2026-03-20"));

    let todo = sections(view(&state, ListView::Todo));
    assert_eq!(titles(&todo.todo), ["todo 1", "todo 3", "todo 2"]);
    assert!(todo.agenda.is_empty());

    let agenda = sections(view(&state, ListView::Agenda));
    let days: Vec<(&str, Vec<&str>)> = agenda
        .agenda
        .iter()
        .map(|day| (day.date.as_str(), titles(&day.tasks)))
        .collect();
    assert_eq!(
        days,
        [
            ("2026-03-20", vec!["new day"]),
            ("2026-03-01", vec!["old day"])
        ]
    );
    assert!(agenda.todo.is_empty());
}

#[test]
fn range_view_is_flat_ascending_and_validated() {
    let state = state();
    create_titled(&state, "outside", Some("2026-04-02"));
    create_titled(&state, "late", Some("2026-03-20"));
    create_titled(&state, "early", Some("2026-03-02"));

    let request = |from: Option<&str>, to: Option<&str>| TasksListInput {
        view: ListView::Range,
        from: from.map(str::to_string),
        to: to.map(str::to_string),
        tag_id: None,
    };

    let TasksListOutput::Range(found) =
        list(&state, request(Some("2026-03-01"), Some("2026-03-31"))).expect("range")
    else {
        panic!("expected a flat list");
    };
    assert_eq!(titles(&found), ["early", "late"]);

    for (input, expected) in [
        (
            request(None, Some("2026-03-31")),
            "VALIDATION: la vista de rango necesita las fechas «from» y «to»",
        ),
        (
            request(Some("2026-03-31"), Some("2026-03-01")),
            "VALIDATION: la fecha inicial del rango es posterior a la final",
        ),
        (
            request(Some("ayer"), Some("2026-03-01")),
            "VALIDATION: fecha no válida «ayer»: se espera AAAA-MM-DD",
        ),
    ] {
        assert_eq!(code_of(list(&state, input)), expected);
    }
}

#[test]
fn list_filters_by_tag() {
    let state = state();
    let tag = crate::commands::tags::upsert(
        &state,
        crate::commands::tags::TagUpsertInput {
            id: None,
            name: "Física".into(),
            color_hex: "#112233".into(),
            parent_id: None,
        },
    )
    .expect("tag");
    create(
        &state,
        TaskCreateInput {
            title: Some("tagged".into()),
            tag_ids: Some(vec![tag.id.clone()]),
            ..TaskCreateInput::default()
        },
    )
    .expect("tagged");
    create_titled(&state, "plain", None);

    let found = list(
        &state,
        TasksListInput {
            view: ListView::Todo,
            from: None,
            to: None,
            tag_id: Some(tag.id),
        },
    )
    .expect("list");

    assert_eq!(titles(&sections(found).todo), ["tagged"]);
}

#[test]
fn update_changes_only_the_given_fields_and_stamps_the_clock() {
    let (state, clock) = test_state();
    let task = create_titled(&state, "antes", Some("2026-03-12"));
    clock.advance(Duration::hours(2));

    let updated = update(&state, &task.id, patch(r#"{"title":"después"}"#)).expect("update");

    assert_eq!(updated.title, "después");
    assert_eq!(updated.due_date.as_deref(), Some("2026-03-12"));
    assert_eq!(updated.updated_at, "2026-03-10T12:00:00Z");
    assert_eq!(updated.created_at, "2026-03-10T10:00:00Z");
}

#[test]
fn update_distinguishes_absent_fields_from_null() {
    let state = state();
    let task = create(
        &state,
        TaskCreateInput {
            title: Some("t".into()),
            due_date: Some("2026-03-12".into()),
            due_time: Some("08:30".into()),
            ..TaskCreateInput::default()
        },
    )
    .expect("create");

    let only_time = update(&state, &task.id, patch(r#"{"dueTime":null}"#)).expect("clear time");
    assert_eq!(only_time.due_date.as_deref(), Some("2026-03-12"));
    assert_eq!(only_time.due_time, None);

    let retimed = update(&state, &task.id, patch(r#"{"dueTime":"17:45"}"#)).expect("set time");
    assert_eq!(retimed.due_time.as_deref(), Some("17:45"));
    assert_eq!(retimed.due_date.as_deref(), Some("2026-03-12"));
}

#[test]
fn a_duration_is_stored_kept_cleared_and_validated() {
    let state = state();
    let task = create(
        &state,
        TaskCreateInput {
            due_date: Some("2026-03-12".into()),
            due_time: Some("09:00".into()),
            duration_minutes: Some(45),
            ..TaskCreateInput::default()
        },
    )
    .expect("create");
    assert_eq!(task.duration_minutes, Some(45));

    let moved = update(&state, &task.id, patch(r#"{"dueTime":"10:00"}"#)).expect("retime");
    assert_eq!(moved.duration_minutes, Some(45));

    let longer = update(&state, &task.id, patch(r#"{"durationMinutes":90}"#)).expect("longer");
    assert_eq!(longer.duration_minutes, Some(90));

    assert_eq!(
        code_of(update(&state, &task.id, patch(r#"{"dueTime":"23:00"}"#))),
        "VALIDATION: la tarea tiene que terminar antes de medianoche"
    );
    assert_eq!(
        update(&state, &task.id, patch(r#"{}"#))
            .expect("noop")
            .duration_minutes,
        Some(90)
    );

    let cleared = update(&state, &task.id, patch(r#"{"durationMinutes":null}"#)).expect("clear");
    assert_eq!(cleared.duration_minutes, None);

    update(&state, &task.id, patch(r#"{"durationMinutes":30}"#)).expect("again");
    let untimed = update(&state, &task.id, patch(r#"{"dueTime":null}"#)).expect("no time");
    assert_eq!((untimed.due_time, untimed.duration_minutes), (None, None));
}

#[test]
fn a_duration_without_a_time_is_rejected_on_create() {
    let state = state();
    let result = create(
        &state,
        TaskCreateInput {
            due_date: Some("2026-03-12".into()),
            duration_minutes: Some(30),
            ..TaskCreateInput::default()
        },
    );
    assert_eq!(
        code_of(result),
        "VALIDATION: una duración requiere una hora"
    );
}

#[test]
fn clearing_the_date_clears_the_time_too() {
    let state = state();
    let task = create(
        &state,
        TaskCreateInput {
            due_date: Some("2026-03-12".into()),
            due_time: Some("08:30".into()),
            ..TaskCreateInput::default()
        },
    )
    .expect("create");

    let cleared = update(&state, &task.id, patch(r#"{"dueDate":null}"#)).expect("clear date");

    assert_eq!((cleared.due_date, cleared.due_time), (None, None));
}

#[test]
fn update_rejects_invalid_results_without_changing_the_task() {
    let state = state();
    let undated = create_titled(&state, "sin fecha", None);
    let dated = create_titled(&state, "con fecha", Some("2026-03-12"));

    assert_eq!(
        code_of(update(&state, &undated.id, patch(r#"{"dueTime":"09:00"}"#))),
        "VALIDATION: una hora requiere una fecha"
    );
    assert_eq!(
        code_of(update(
            &state,
            &dated.id,
            patch(r#"{"dueDate":null,"dueTime":"09:00"}"#)
        )),
        "VALIDATION: una hora requiere una fecha"
    );
    assert_eq!(
        code_of(update(
            &state,
            &dated.id,
            patch(r#"{"dueDate":"2026-13-01"}"#)
        )),
        "VALIDATION: fecha no válida «2026-13-01»: se espera AAAA-MM-DD"
    );
    let long = format!(r#"{{"title":"{}"}}"#, "x".repeat(501));
    assert!(code_of(update(&state, &dated.id, patch(&long))).starts_with("VALIDATION: "));
    assert!(
        code_of(update(&state, &dated.id, patch(r#"{"contentJson":"[]"}"#)))
            .starts_with("VALIDATION: ")
    );

    let unchanged = sections(view(&state, ListView::Agenda));
    assert_eq!(
        unchanged.agenda[0].tasks[0].due_date.as_deref(),
        Some("2026-03-12")
    );
}

#[test]
fn update_with_an_empty_patch_changes_nothing() {
    let (state, clock) = test_state();
    let task = create_titled(&state, "igual", None);
    clock.advance(Duration::hours(1));

    let same = update(&state, &task.id, patch("{}")).expect("update");

    assert_eq!(same, task);
}

#[test]
fn update_reindexes_title_and_notes() {
    let state = state();
    let task = create_titled(&state, "borrador", None);
    let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"cálculo diferencial"}]}]}"#;
    let body = serde_json::json!({ "title": "informe", "contentJson": json });

    update(&state, &task.id, patch(&body.to_string())).expect("update");

    let hits = |term: &str| -> i64 {
        state
            .db
            .read(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM tasks_fts WHERE tasks_fts MATCH ?1",
                    [term],
                    |row| row.get(0),
                )?)
            })
            .expect("fts")
    };
    assert_eq!(
        (hits("informe"), hits("calculo"), hits("borrador")),
        (1, 1, 0)
    );
}

#[test]
fn commands_report_missing_tasks() {
    let state = state();
    let missing = "NOT_FOUND: tarea nope no encontrada";

    assert_eq!(code_of(update(&state, "nope", patch("{}"))), missing);
    assert_eq!(code_of(set_completed(&state, "nope", true)), missing);
    assert_eq!(code_of(delete(&state, "nope")), missing);
    assert_eq!(code_of(restore(&state, "nope")), missing);
    assert_eq!(
        code_of(move_task(&state, "nope", Section::Todo, None, None)),
        missing
    );
}

#[test]
fn completing_a_task_moves_it_to_the_completion_day() {
    let state = state();
    let undated = create_titled(&state, "sin fecha", None);
    let dated = create_titled(&state, "con fecha", Some("2026-03-12"));

    let done = complete(&state, &undated.id, true, Some("2026-03-20")).expect("complete");
    assert_eq!(done.due_date.as_deref(), Some("2026-03-20"));
    assert!(done.is_completed);

    let moved = complete(&state, &dated.id, true, Some("2026-03-20")).expect("complete");
    assert_eq!(moved.due_date.as_deref(), Some("2026-03-20"));

    let reopened = complete(&state, &undated.id, false, Some("2026-03-25")).expect("reopen");
    assert_eq!(reopened.due_date.as_deref(), Some("2026-03-20"));
    assert!(!reopened.is_completed);
}

#[test]
fn completing_without_a_local_date_keeps_the_task_undated() {
    let state = state();
    let task = create_titled(&state, "t", None);
    let done = set_completed(&state, &task.id, true).expect("complete");
    assert_eq!(done.due_date, None);
}

#[test]
fn completing_rejects_a_malformed_local_date() {
    let state = state();
    let task = create_titled(&state, "t", None);
    assert!(
        code_of(complete(&state, &task.id, true, Some("20-03-2026"))).starts_with("VALIDATION:")
    );
    assert!(
        !state
            .db
            .read(|c| Ok(tasks::get(c, &task.id)?))
            .expect("get")
            .expect("task")
            .is_completed
    );
}

#[test]
fn complete_and_reopen_use_the_clock() {
    let (state, clock) = test_state();
    let task = create_titled(&state, "t", None);
    clock.advance(Duration::minutes(5));

    let done = set_completed(&state, &task.id, true).expect("complete");
    assert!(done.is_completed);
    assert_eq!(done.completed_at.as_deref(), Some("2026-03-10T10:05:00Z"));

    let reopened = set_completed(&state, &task.id, false).expect("reopen");
    assert!(!reopened.is_completed);
    assert_eq!(reopened.completed_at, None);
}

#[test]
fn delete_trash_and_restore_round_trip() {
    let state = state();
    create_titled(&state, "keep", None);
    let gone = create_titled(&state, "gone", None);

    delete(&state, &gone.id).expect("delete");

    assert_eq!(
        titles(&sections(view(&state, ListView::Todo)).todo),
        ["keep"]
    );
    assert_eq!(titles(&trash(&state).expect("trash")), ["gone"]);
    assert_eq!(
        code_of(delete(&state, &gone.id)),
        format!("NOT_FOUND: tarea {} no encontrada", gone.id)
    );

    let restored = restore(&state, &gone.id).expect("restore");
    assert_eq!(restored.id, gone.id);
    assert!(trash(&state).expect("trash").is_empty());
    assert_eq!(
        titles(&sections(view(&state, ListView::Todo)).todo),
        ["keep", "gone"]
    );
}

#[test]
fn move_reorders_within_todo_and_between_sections() {
    let state = state();
    let a = create_titled(&state, "a", None);
    let b = create_titled(&state, "b", None);
    let c = create_titled(&state, "c", None);

    move_task(&state, &c.id, Section::Todo, None, Some(a.id.clone())).expect("reorder");
    assert_eq!(
        titles(&sections(view(&state, ListView::Todo)).todo),
        ["c", "a", "b"]
    );

    let dated = move_task(
        &state,
        &a.id,
        Section::Agenda,
        Some("2026-03-15".into()),
        None,
    )
    .expect("to agenda");
    assert_eq!(dated.due_date.as_deref(), Some("2026-03-15"));
    assert_eq!(
        titles(&sections(view(&state, ListView::Todo)).todo),
        ["c", "b"]
    );

    let back = move_task(&state, &a.id, Section::Todo, None, Some(b.id)).expect("to todo");
    assert_eq!(back.due_date, None);
    assert_eq!(
        titles(&sections(view(&state, ListView::Todo)).todo),
        ["c", "a", "b"]
    );
}

#[test]
fn move_validates_destination_and_anchor() {
    let state = state();
    let todo = create_titled(&state, "todo", None);
    let dated = create_titled(&state, "dated", Some("2026-03-15"));

    let cases = [
        (
            move_task(&state, &todo.id, Section::Agenda, None, None),
            "VALIDATION: mover a la agenda necesita una fecha".to_string(),
        ),
        (
            move_task(
                &state,
                &todo.id,
                Section::Todo,
                Some("2026-03-15".into()),
                None,
            ),
            "VALIDATION: la sección «por hacer» no admite fecha".to_string(),
        ),
        (
            move_task(&state, &todo.id, Section::Agenda, Some("hoy".into()), None),
            "VALIDATION: fecha no válida «hoy»: se espera AAAA-MM-DD".to_string(),
        ),
        (
            move_task(
                &state,
                &todo.id,
                Section::Todo,
                None,
                Some(dated.id.clone()),
            ),
            "VALIDATION: la tarea de referencia no está en la sección de destino".to_string(),
        ),
        (
            move_task(&state, &todo.id, Section::Todo, None, Some("nope".into())),
            "NOT_FOUND: tarea nope no encontrada".to_string(),
        ),
    ];
    for (result, expected) in cases {
        assert_eq!(code_of(result), expected);
    }
    assert_eq!(
        titles(&sections(view(&state, ListView::Todo)).todo),
        ["todo"]
    );
}

#[test]
fn inputs_deserialize_from_the_frontend_shape() {
    let request: ListView = serde_json::from_str("\"agenda\"").expect("view");
    assert_eq!(request, ListView::Agenda);
    let section: Section = serde_json::from_str("\"todo\"").expect("section");
    assert_eq!(section, Section::Todo);
    assert!(serde_json::from_str::<ListView>("\"week\"").is_err());

    let full = patch(r#"{"title":"t","contentJson":"{}","dueDate":"2026-03-12","dueTime":null}"#);
    assert_eq!(full.title.as_deref(), Some("t"));
    assert_eq!(full.due_date, Some(Some("2026-03-12".to_string())));
    assert_eq!(full.due_time, Some(None));
    let empty = patch("{}");
    assert_eq!((empty.due_date, empty.due_time), (None, None));
}

#[test]
fn list_output_serializes_as_documented() {
    let state = state();
    create_titled(&state, "t", Some("2026-03-12"));

    let agenda = serde_json::to_value(view(&state, ListView::Agenda)).expect("json");
    assert_eq!(agenda["todo"], serde_json::json!([]));
    assert_eq!(agenda["agenda"][0]["date"], "2026-03-12");
    assert_eq!(agenda["agenda"][0]["tasks"][0]["title"], "t");

    let range = serde_json::to_value(TasksListOutput::Range(vec![])).expect("json");
    assert_eq!(range, serde_json::json!([]));
}
fn repeating(state: &AppState, date: &str, rule: &str) -> Task {
    create(
        state,
        TaskCreateInput {
            title: Some("Regar".into()),
            due_date: Some(date.into()),
            due_time: Some("09:00".into()),
            rrule: Some(rule.into()),
            ..TaskCreateInput::default()
        },
    )
    .expect("repeating")
}

fn open_tasks(state: &AppState) -> Vec<Task> {
    let mut found: Vec<Task> = state
        .db
        .read(|conn| Ok(tasks::list(conn, tasks::Query::Agenda, None)?))
        .expect("list")
        .into_iter()
        .filter(|task| !task.is_completed)
        .collect();
    found.sort_by(|a, b| a.due_date.cmp(&b.due_date));
    found
}

fn open_dates(state: &AppState) -> Vec<String> {
    open_tasks(state)
        .into_iter()
        .filter_map(|task| task.due_date)
        .collect()
}

#[test]
fn a_rule_is_validated_and_anchored_to_the_date() {
    let (state, _) = test_state();
    let made = repeating(&state, "2026-01-31", "FREQ=MONTHLY");
    assert_eq!(
        made.rrule.as_deref(),
        Some("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=31")
    );

    let code = |r: AppResult<Task>| r.expect_err("fails").to_string();
    let undated = TaskCreateInput {
        rrule: Some("FREQ=DAILY".into()),
        ..TaskCreateInput::default()
    };
    assert!(code(create(&state, undated)).starts_with("VALIDATION"));
    let bad = TaskCreateInput {
        due_date: Some("2026-03-10".into()),
        rrule: Some("FREQ=SOMETIMES".into()),
        ..TaskCreateInput::default()
    };
    assert!(code(create(&state, bad)).starts_with("VALIDATION"));
}

#[test]
fn completing_a_repeating_task_creates_the_next_one_and_hands_over_the_series() {
    let (state, _) = test_state();
    let tag = crate::commands::tags::upsert(
        &state,
        crate::commands::tags::TagUpsertInput {
            id: None,
            name: "Casa".into(),
            color_hex: "#112233".into(),
            parent_id: None,
        },
    )
    .expect("tag");
    let first = create(
        &state,
        TaskCreateInput {
            title: Some("Regar".into()),
            due_date: Some("2026-03-10".into()),
            due_time: Some("09:00".into()),
            duration_minutes: Some(30),
            tag_ids: Some(vec![tag.id.clone()]),
            rrule: Some("FREQ=WEEKLY".into()),
            ..TaskCreateInput::default()
        },
    )
    .expect("first");

    let done = complete(&state, &first.id, true, Some("2026-03-10")).expect("complete");
    assert!(done.is_completed);
    assert_eq!(done.rrule, None, "the series moves on");

    let open = open_tasks(&state);
    assert_eq!(open.len(), 1);
    let next = &open[0];
    assert_eq!(next.due_date.as_deref(), Some("2026-03-17"));
    assert_eq!(next.title, "Regar");
    assert_eq!(next.due_time.as_deref(), Some("09:00"));
    assert_eq!(next.duration_minutes, Some(30));
    assert_eq!(next.tag_ids, [tag.id]);
    assert_eq!(next.rrule.as_deref(), first.rrule.as_deref());
    assert_ne!(next.id, first.id);
}

#[test]
fn a_counted_weekday_series_stops_after_its_last_occurrence() {
    let (state, _) = test_state();
    // 2026-03-09 is a Monday.
    let first = repeating(&state, "2026-03-09", "FREQ=WEEKLY;BYDAY=MO,WE;COUNT=3");
    complete(&state, &first.id, true, Some("2026-03-09")).expect("first");
    let second = open_tasks(&state).remove(0);
    assert_eq!(second.due_date.as_deref(), Some("2026-03-11"));
    assert_eq!(
        second.rrule.as_deref(),
        Some("FREQ=WEEKLY;INTERVAL=1;BYDAY=MO,WE;COUNT=2")
    );
    complete(&state, &second.id, true, Some("2026-03-11")).expect("second");
    let third = open_tasks(&state).remove(0);
    assert_eq!(third.due_date.as_deref(), Some("2026-03-16"));
    complete(&state, &third.id, true, Some("2026-03-16")).expect("third");
    assert!(open_tasks(&state).is_empty(), "the series is over");
}

#[test]
fn reopening_and_completing_again_does_not_duplicate_and_late_completion_skips_ahead() {
    let (state, _) = test_state();
    let first = repeating(&state, "2026-03-10", "FREQ=DAILY");
    complete(&state, &first.id, true, Some("2026-03-10")).expect("done");
    complete(&state, &first.id, false, None).expect("reopen");
    complete(&state, &first.id, true, Some("2026-03-10")).expect("done again");
    let after_reopen = open_dates(&state);
    assert_eq!(after_reopen, ["2026-03-11"], "one next task, no copy");

    let (state, _) = test_state();
    let late = repeating(&state, "2026-03-01", "FREQ=WEEKLY");
    complete(&state, &late.id, true, Some("2026-03-10")).expect("late");
    assert_eq!(
        open_dates(&state),
        ["2026-03-15"],
        "still on its weekly cycle"
    );
}

#[test]
fn the_series_ends_at_until_and_a_plain_task_creates_nothing() {
    let (state, _) = test_state();
    let last = repeating(&state, "2026-03-10", "FREQ=DAILY;UNTIL=20260310");
    complete(&state, &last.id, true, Some("2026-03-10")).expect("done");
    assert!(open_dates(&state).is_empty());

    let plain = create(
        &state,
        TaskCreateInput {
            title: Some("Suelta".into()),
            due_date: Some("2026-03-10".into()),
            ..TaskCreateInput::default()
        },
    )
    .expect("plain");
    complete(&state, &plain.id, true, Some("2026-03-10")).expect("done");
    assert!(open_dates(&state).is_empty());
}

#[test]
fn relative_reminders_follow_the_series() {
    let (state, _) = test_state();
    let first = repeating(&state, "2026-03-20", "FREQ=WEEKLY");
    crate::commands::reminders::set(
        &state,
        &first.id,
        &[crate::commands::reminders::ReminderInput {
            offset_minutes: Some(60),
            notify_at: None,
            repeat: None,
        }],
        &chrono::Local,
    )
    .expect("reminder");
    complete(&state, &first.id, true, Some("2026-03-20")).expect("done");
    let open = open_tasks(&state);
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].due_date.as_deref(), Some("2026-03-27"));
    assert_eq!(open[0].reminders.len(), 1);
    assert_eq!(open[0].reminders[0].offset_minutes, Some(60));
    assert!(!open[0].reminders[0].is_sent);
}

#[test]
fn changing_or_clearing_the_date_keeps_the_rule_coherent() {
    let (state, _) = test_state();
    let task = repeating(&state, "2026-01-31", "FREQ=MONTHLY");
    let moved = update(
        &state,
        &task.id,
        TaskPatchInput {
            due_date: Some(Some("2026-02-05".into())),
            ..TaskPatchInput::default()
        },
    )
    .expect("moved");
    assert_eq!(
        moved.rrule.as_deref(),
        Some("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=5")
    );

    let off = update(
        &state,
        &task.id,
        TaskPatchInput {
            rrule: Some(None),
            ..TaskPatchInput::default()
        },
    )
    .expect("off");
    assert_eq!(off.rrule, None);

    let again = update(
        &state,
        &task.id,
        TaskPatchInput {
            rrule: Some(Some("FREQ=DAILY".into())),
            ..TaskPatchInput::default()
        },
    )
    .expect("again");
    assert!(again.rrule.is_some());
    let undated = update(
        &state,
        &task.id,
        TaskPatchInput {
            due_date: Some(None),
            ..TaskPatchInput::default()
        },
    )
    .expect("undated");
    assert_eq!(undated.rrule, None, "no date, no repetition");
}

#[test]
fn dragging_a_repeating_task_re_anchors_it_and_the_to_do_list_ends_it() {
    let (state, _) = test_state();
    let task = repeating(&state, "2026-01-31", "FREQ=MONTHLY");
    let moved = move_task(
        &state,
        &task.id,
        Section::Agenda,
        Some("2026-02-09".into()),
        None,
    )
    .expect("moved");
    assert_eq!(
        moved.rrule.as_deref(),
        Some("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=9")
    );
    let parked = move_task(&state, &task.id, Section::Todo, None, None).expect("parked");
    assert_eq!(parked.rrule, None);
}

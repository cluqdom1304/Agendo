use super::*;

fn reminder(title: &str, tags: &[&str], time: Option<&str>) -> DueReminder {
    DueReminder {
        id: "r".into(),
        notify_at: "2026-03-10T10:00:00Z".into(),
        task_id: Some("t".into()),
        note_id: None,
        event_id: None,
        title: title.into(),
        due_time: time.map(str::to_string),
        notes: String::new(),
        tag_names: tags.iter().map(|tag| (*tag).to_string()).collect(),
    }
}

#[test]
fn the_body_lists_tags_and_time() {
    let n = Notification::for_reminder(&reminder("Entregar", &["Mates", "Examen"], Some("09:30")));
    assert_eq!(n.title, "Entregar");
    assert_eq!(n.body, "Mates, Examen · 09:30");
}

#[test]
fn the_body_leaves_out_what_is_missing() {
    assert_eq!(
        Notification::for_reminder(&reminder("A", &[], Some("08:00"))).body,
        "08:00"
    );
    assert_eq!(
        Notification::for_reminder(&reminder("A", &["Mates"], None)).body,
        "Mates"
    );
    assert_eq!(
        Notification::for_reminder(&reminder("A", &[], None)).body,
        ""
    );
}

#[test]
fn an_untitled_task_still_has_a_title() {
    assert_eq!(
        Notification::for_reminder(&reminder("  ", &[], None)).title,
        UNTITLED
    );
}

#[test]
fn notes_add_a_second_line_flattened_to_one_line() {
    let mut r = reminder("A", &["Mates"], Some("09:00"));
    r.notes = "Primera línea

  segunda   línea"
        .into();
    assert_eq!(
        Notification::for_reminder(&r).body,
        "Mates · 09:00
Primera línea segunda línea"
    );
    r.tag_names.clear();
    r.due_time = None;
    assert_eq!(
        Notification::for_reminder(&r).body,
        "Primera línea segunda línea"
    );
}

#[test]
fn long_notes_are_cut_at_a_word_with_an_ellipsis() {
    let mut r = reminder("A", &[], None);
    r.notes = "palabra ".repeat(40);
    let body = Notification::for_reminder(&r).body;
    assert!(body.ends_with("palabra…"), "{body}");
    assert!(body.chars().count() <= SNIPPET_CHARS + 1);
}

#[test]
fn the_fired_payload_is_camel_case() {
    let json = serde_json::to_value(Fired {
        task_id: Some("t".into()),
        note_id: None,
        event_id: None,
        reminder_id: "r".into(),
    })
    .expect("json");
    assert_eq!(
        json,
        serde_json::json!({ "taskId": "t", "reminderId": "r" })
    );
}
fn task_in(state: &AppState, title: &str) -> String {
    tasks::create(
        state,
        tasks::TaskCreateInput {
            title: Some(title.into()),
            ..tasks::TaskCreateInput::default()
        },
    )
    .expect("task")
    .id
}

fn unsent_times(state: &AppState, id: &str) -> Vec<String> {
    let mut times: Vec<String> = state
        .db
        .read(|conn| Ok(reminder_repo::for_task(conn, id)?))
        .expect("reminders")
        .into_iter()
        .filter(|reminder| !reminder.is_sent)
        .map(|reminder| reminder.notify_at)
        .collect();
    times.sort();
    times
}

#[test]
fn snooze_buttons_add_a_reminder_later_and_keep_the_others() {
    let (state, _) = crate::state::test_state();
    let id = task_in(&state, "Entregar");
    assert_eq!(
        perform_action(&state, ACTION_SNOOZE_10, Some(&id), None, None),
        ActionOutcome::Snoozed(id.clone())
    );
    assert_eq!(
        perform_action(&state, ACTION_SNOOZE_HOUR, Some(&id), None, None),
        ActionOutcome::Snoozed(id.clone())
    );
    assert_eq!(
        unsent_times(&state, &id),
        ["2026-03-10T10:10:00Z", "2026-03-10T11:00:00Z"]
    );
}

#[test]
fn snoozing_to_tomorrow_uses_nine_local_time_and_refuses_unknown_or_finished_tasks() {
    let (state, _) = crate::state::test_state();
    let id = task_in(&state, "Entregar");
    let madrid = chrono::FixedOffset::east_opt(3600).expect("offset");
    snooze(&state, &id, ACTION_SNOOZE_TOMORROW, &madrid).expect("snooze");
    assert_eq!(unsent_times(&state, &id), ["2026-03-11T08:00:00Z"]);

    assert_eq!(
        perform_action(&state, ACTION_SNOOZE_10, None, None, None),
        ActionOutcome::Ignored
    );
    assert_eq!(
        perform_action(&state, ACTION_SNOOZE_10, Some("no-existe"), None, None),
        ActionOutcome::Ignored
    );
    tasks::complete(&state, &id, true, None).expect("done");
    assert_eq!(
        perform_action(&state, ACTION_SNOOZE_10, Some(&id), None, None),
        ActionOutcome::Ignored,
        "a finished task is not brought back"
    );
    assert!(snooze(&state, &id, "otra", &madrid).is_err());
}

#[test]
fn the_notification_remembers_what_it_is_about() {
    let task = Notification::for_reminder(&reminder("Entregar", &[], None));
    assert_eq!(task.task_id.as_deref(), Some("t"));
    assert_eq!(task.note_id, None);
    let mut for_note = reminder("Reunión", &[], None);
    for_note.task_id = None;
    for_note.note_id = Some("n".into());
    let note = Notification::for_reminder(&for_note);
    assert_eq!((note.task_id, note.note_id.as_deref()), (None, Some("n")));
}

#[test]
fn the_done_button_completes_the_task_of_the_notification() {
    let (state, _) = crate::state::test_state();
    let id = task_in(&state, "Entregar");
    assert_eq!(
        perform_action(&state, ACTION_DONE, Some(&id), None, None),
        ActionOutcome::Completed(id.clone())
    );
    let done = state
        .db
        .read(|conn| Ok(crate::repo::tasks::get(conn, &id)?))
        .expect("read")
        .expect("task");
    assert!(done.is_completed);
}

#[test]
fn the_done_button_never_guesses_which_task() {
    let (state, _) = crate::state::test_state();
    let id = task_in(&state, "Entregar");
    assert_eq!(
        perform_action(&state, ACTION_DONE, None, None, None),
        ActionOutcome::Ignored
    );
    assert_eq!(
        perform_action(&state, ACTION_DONE, Some("no-existe"), None, None),
        ActionOutcome::Ignored
    );
    let open = state
        .db
        .read(|conn| Ok(crate::repo::tasks::get(conn, &id)?))
        .expect("read")
        .expect("task");
    assert!(!open.is_completed);
}

#[test]
fn a_click_opens_the_task_or_the_note_and_does_not_complete_anything() {
    let (state, _) = crate::state::test_state();
    let id = task_in(&state, "Entregar");
    assert_eq!(
        perform_action(&state, "tap", Some(&id), None, None),
        ActionOutcome::Open {
            task_id: Some(id.clone()),
            note_id: None,
            event_id: None
        }
    );
    assert_eq!(
        perform_action(&state, "tap", None, Some("n1"), None),
        ActionOutcome::Open {
            task_id: None,
            note_id: Some("n1".into()),
            event_id: None
        }
    );
    assert_eq!(
        perform_action(&state, "tap", None, None, Some("e1")),
        ActionOutcome::Open {
            task_id: None,
            note_id: None,
            event_id: Some("e1".into())
        }
    );
    assert_eq!(
        perform_action(&state, "tap", None, None, None),
        ActionOutcome::Ignored
    );
    let still_open = state
        .db
        .read(|conn| Ok(crate::repo::tasks::get(conn, &id)?))
        .expect("read")
        .expect("task");
    assert!(!still_open.is_completed);
}

use super::*;
use crate::{
    services::tasks::{TaskCreateInput, create},
    state::test_state,
};

fn input(name: &str, parent: Option<&str>) -> TagUpsertInput {
    TagUpsertInput {
        id: None,
        name: name.to_string(),
        color_hex: "#0A84FF".to_string(),
        parent_id: parent.map(str::to_string),
    }
}

fn make(state: &AppState, name: &str, parent: Option<&str>) -> Tag {
    upsert(state, input(name, parent)).expect("upsert")
}

fn code_of<T: std::fmt::Debug>(result: AppResult<T>) -> String {
    result.expect_err("must fail").to_string()
}

fn new_task(state: &AppState, tag_ids: Vec<String>) -> Task {
    create(
        state,
        TaskCreateInput {
            tag_ids: Some(tag_ids),
            ..TaskCreateInput::default()
        },
    )
    .expect("task")
}

#[test]
fn upsert_creates_with_generated_id_and_trimmed_name() {
    let (state, _) = test_state();
    let tag = make(&state, "  Matemáticas  ", None);

    assert_eq!(tag.id, "id-1");
    assert_eq!(tag.name, "Matemáticas");
    assert_eq!(tag.color_hex, "#0A84FF");
    assert_eq!(tag.position, 0);
    assert_eq!(make(&state, "Física", None).position, 1);
}

#[test]
fn upsert_updates_an_existing_tag_in_place() {
    let (state, _) = test_state();
    let tag = make(&state, "Física", None);

    let renamed = upsert(
        &state,
        TagUpsertInput {
            id: Some(tag.id.clone()),
            name: "Física II".into(),
            color_hex: "#ff0000".into(),
            parent_id: None,
        },
    )
    .expect("rename");

    assert_eq!(renamed.id, tag.id);
    assert_eq!(renamed.name, "Física II");
    assert_eq!(renamed.color_hex, "#ff0000");
    assert_eq!(list(&state).expect("list").len(), 1);
}

#[test]
fn upsert_validates_name_color_and_ids() {
    let (state, _) = test_state();
    let existing = make(&state, "Grado", None);
    let with = |change: &dyn Fn(&mut TagUpsertInput)| {
        let mut value = input("Nueva", None);
        change(&mut value);
        upsert(&state, value)
    };

    assert_eq!(
        code_of(with(&|i| i.name = "   ".into())),
        "VALIDATION: el nombre de la etiqueta no puede estar vacío"
    );
    assert_eq!(
        code_of(with(&|i| i.color_hex = "azul".into())),
        "VALIDATION: el color debe tener el formato #RRGGBB"
    );
    assert_eq!(
        code_of(with(&|i| i.name = "GRADO".into())),
        "CONFLICT: ya existe una etiqueta con ese nombre"
    );
    assert_eq!(
        code_of(with(&|i| i.id = Some("nope".into()))),
        "NOT_FOUND: etiqueta nope no encontrada"
    );
    assert_eq!(
        code_of(with(&|i| i.parent_id = Some("nope".into()))),
        "NOT_FOUND: etiqueta padre nope no encontrada"
    );
    assert_eq!(list(&state).expect("list").len(), 1);
    assert_eq!(existing.name, "Grado");
}

#[test]
fn hierarchy_is_limited_to_one_level() {
    let (state, _) = test_state();
    let grade = make(&state, "Grado", None);
    let subject = make(&state, "Álgebra", Some(&grade.id));
    assert_eq!(subject.parent_id.as_deref(), Some(grade.id.as_str()));

    let grandchild = upsert(&state, input("Tema 1", Some(&subject.id)));
    assert!(code_of(grandchild).starts_with("VALIDATION: solo se permite un nivel"));

    let other = make(&state, "Otro", None);
    let adopt_parent = upsert(
        &state,
        TagUpsertInput {
            id: Some(grade.id.clone()),
            name: "Grado".into(),
            color_hex: "#112233".into(),
            parent_id: Some(other.id.clone()),
        },
    );
    assert_eq!(
        code_of(adopt_parent),
        "VALIDATION: una etiqueta con subetiquetas no puede pasar a ser subetiqueta"
    );

    let own_parent = upsert(
        &state,
        TagUpsertInput {
            id: Some(other.id.clone()),
            name: "Otro".into(),
            color_hex: "#112233".into(),
            parent_id: Some(other.id),
        },
    );
    assert_eq!(
        code_of(own_parent),
        "VALIDATION: una etiqueta no puede ser su propia etiqueta padre"
    );
}

#[test]
fn merge_rejects_the_same_tag_and_unknown_tags() {
    let (state, _) = test_state();
    let one = make(&state, "Uno", None);
    let two = make(&state, "Dos", None);
    assert_eq!(
        code_of(merge(&state, &one.id, &one.id)).split(':').next(),
        Some("VALIDATION")
    );
    assert_eq!(
        code_of(merge(&state, &one.id, "zzz")).split(':').next(),
        Some("NOT_FOUND")
    );
    merge(&state, &one.id, &two.id).expect("merge");
    let ids: Vec<_> = list(&state)
        .expect("list")
        .into_iter()
        .map(|t| t.tag.id)
        .collect();
    assert_eq!(ids, vec![two.id]);
}

#[test]
fn a_child_can_be_detached_by_clearing_its_parent() {
    let (state, _) = test_state();
    let grade = make(&state, "Grado", None);
    let subject = make(&state, "Álgebra", Some(&grade.id));

    let detached = upsert(
        &state,
        TagUpsertInput {
            id: Some(subject.id),
            name: "Álgebra".into(),
            color_hex: "#0A84FF".into(),
            parent_id: None,
        },
    )
    .expect("detach");

    assert_eq!(detached.parent_id, None);
}

#[test]
fn list_counts_pending_tasks_per_tag() {
    let (state, _) = test_state();
    let tag = make(&state, "Física", None);
    let empty = make(&state, "Vacía", None);
    new_task(&state, vec![tag.id.clone()]);
    let done = new_task(&state, vec![tag.id.clone()]);
    crate::services::tasks::set_completed(&state, &done.id, true).expect("complete");

    let tags = list(&state).expect("list");

    let counts: Vec<(&str, i64)> = tags
        .iter()
        .map(|t| (t.tag.id.as_str(), t.pending_count))
        .collect();
    assert_eq!(counts, [(tag.id.as_str(), 1), (empty.id.as_str(), 0)]);
}

#[test]
fn delete_unlinks_tasks_and_reports_missing_tags() {
    let (state, _) = test_state();
    let tag = make(&state, "Física", None);
    let task = new_task(&state, vec![tag.id.clone()]);

    delete(&state, &tag.id).expect("delete");

    let after = set_task_tags(&state, &task.id, &[]).expect("task");
    assert!(after.tag_ids.is_empty());
    assert_eq!(
        code_of(delete(&state, &tag.id)),
        format!("NOT_FOUND: etiqueta {} no encontrada", tag.id)
    );
}

#[test]
fn set_task_tags_replaces_the_set_and_returns_the_task() {
    let (state, clock) = test_state();
    let a = make(&state, "A", None);
    let b = make(&state, "B", None);
    let c = make(&state, "C", None);
    let task = new_task(&state, vec![a.id.clone()]);
    clock.advance(chrono::Duration::minutes(1));

    let updated = set_task_tags(&state, &task.id, &[c.id.clone(), b.id.clone()]).expect("replace");

    let mut expected = vec![b.id, c.id];
    expected.sort();
    assert_eq!(updated.tag_ids, expected);
    assert_eq!(updated.updated_at, "2026-03-10T10:01:00Z");
}

#[test]
fn set_task_tags_rejects_unknown_tasks_and_tags() {
    let (state, _) = test_state();
    let tag = make(&state, "A", None);
    let task = new_task(&state, vec![tag.id.clone()]);

    assert_eq!(
        code_of(set_task_tags(&state, "nope", std::slice::from_ref(&tag.id))),
        "NOT_FOUND: tarea nope no encontrada"
    );
    assert_eq!(
        code_of(set_task_tags(&state, &task.id, &["nope".to_string()])),
        "NOT_FOUND: el elemento referenciado no existe"
    );
    let unchanged = set_task_tags(&state, &task.id, std::slice::from_ref(&tag.id)).expect("same");
    assert_eq!(unchanged.tag_ids, [tag.id]);
}

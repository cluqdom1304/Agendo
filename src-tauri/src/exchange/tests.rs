use super::*;
use super::{export::*, import::*, paths::*};
use crate::{
    commands::notes::{NoteCreateInput, create},
    domain::codec::md5_hex,
    state::test_state,
    test_support::TempDir,
};
use serde_json::json;

const PNG: &[u8] = b"\x89PNG\r\n\x1a\nfake image";

fn paragraph(text: &str) -> Value {
    json!({ "type": "paragraph", "content": [{ "type": "text", "text": text }] })
}

fn doc(blocks: Value) -> String {
    json!({ "type": "doc", "content": blocks }).to_string()
}

fn new_note(state: &AppState, folder: Option<&str>, title: &str, json: String) -> Note {
    create(
        state,
        NoteCreateInput {
            folder_id: folder.map(str::to_string),
            title: Some(title.into()),
            content_json: Some(json),
            tag_ids: None,
        },
    )
    .expect("note")
}

fn folder(state: &AppState, name: &str, parent: Option<&str>) -> String {
    folder_commands::upsert(
        state,
        FolderUpsertInput {
            id: None,
            name: name.into(),
            parent_id: parent.map(str::to_string),
        },
    )
    .expect("folder")
    .id
}

fn all_notes(state: &AppState) -> Vec<Note> {
    state
        .db
        .read(|conn| {
            let list = notes::list(conn, Scope::All, None, Sort::Title)?;
            list.iter()
                .filter_map(|n| notes::get(conn, &n.id).transpose())
                .collect::<Result<Vec<_>, _>>()
                .map_err(AppError::from)
        })
        .expect("notes")
}

#[test]
fn names_are_safe_and_do_not_repeat() {
    assert_eq!(safe_name("a/b:c?", "x"), "a_b_c_");
    assert_eq!(safe_name("  ..  ", "x"), "x");
    assert_eq!(safe_name("CON", "x"), "_CON");
    assert_eq!(safe_name("com1.txt", "x"), "_com1.txt");
    assert_eq!(safe_name("COM0", "x"), "COM0");
    assert_eq!(safe_name(&"a".repeat(300), "x").chars().count(), 100);
    let mut namer = Namer::default();
    let dir = Path::new("d");
    assert_eq!(namer.claim(dir, "Nota", "md"), "Nota.md");
    assert_eq!(namer.claim(dir, "nota", "md"), "nota (2).md");
    assert_eq!(namer.claim(dir, "Nota", "md"), "Nota (3).md");
}

#[test]
fn markdown_export_writes_folders_front_matter_and_images() {
    let (state, _) = test_state();
    let dir = TempDir::new();
    let base = dir.database().parent().expect("parent").to_path_buf();
    let files = base.join("files");
    let image = attachments::add(&state, &files, PNG).expect("image");
    let docs = folder(&state, "Docs", None);
    new_note(&state, None, "Raíz", doc(json!([paragraph("hola")])));
    new_note(
        &state,
        Some(&docs),
        "Con imagen",
        doc(
            json!([{ "type": "image", "attrs": { "attachmentId": image.id, "mime": "image/png" } }]),
        ),
    );

    let target = base.join("out");
    let report =
        export_notes(&state, &files, &target, Format::Markdown, &Selection::All).expect("export");
    assert_eq!(report, ExportReport { notes: 2, files: 1 });

    let root = fs::read_to_string(target.join("Raíz.md")).expect("root note");
    assert!(root.starts_with("---\ntitle: \"Raíz\"\ntags: []\n"));
    assert!(root.ends_with("# Raíz\n\nhola\n"));
    let nested = fs::read_to_string(target.join("Docs").join("Con imagen.md")).expect("nested");
    assert!(nested.contains(&format!("![imagen](../adjuntos/{}.png)", image.id)));
    assert_eq!(
        fs::read(target.join(ASSETS_DIR).join(format!("{}.png", image.id))).expect("copied"),
        PNG
    );
}

#[test]
fn selecting_a_folder_or_a_note_limits_the_export() {
    let (state, _) = test_state();
    let dir = TempDir::new();
    let base = dir.database().parent().expect("parent").to_path_buf();
    let files = base.join("files");
    let outer = folder(&state, "Fuera", None);
    let inner = folder(&state, "Dentro", Some(&outer));
    new_note(&state, Some(&outer), "A", doc(json!([])));
    let deep = new_note(&state, Some(&inner), "B", doc(json!([])));
    new_note(&state, None, "C", doc(json!([])));

    let target = base.join("folder");
    let report = export_notes(
        &state,
        &files,
        &target,
        Format::Html,
        &Selection::Folder(outer),
    )
    .expect("export");
    assert_eq!(report.notes, 2);
    assert!(target.join("Fuera").join("A.html").exists());
    assert!(target.join("Fuera").join("Dentro").join("B.html").exists());
    assert!(!target.join("C.html").exists());

    let single = base.join("single");
    export_notes(
        &state,
        &files,
        &single,
        Format::Markdown,
        &Selection::Note(deep.id),
    )
    .expect("export");
    assert!(single.join("B.md").exists());
    assert_eq!(fs::read_dir(&single).expect("dir").count(), 1);

    let missing = export_notes(
        &state,
        &files,
        &base.join("none"),
        Format::Markdown,
        &Selection::Note("nope".into()),
    );
    assert!(matches!(missing, Err(AppError::NotFound(_))));
}

#[test]
fn exported_markdown_imports_back_with_its_structure_tags_and_images() {
    let (source, _) = test_state();
    let dir = TempDir::new();
    let base = dir.database().parent().expect("parent").to_path_buf();
    let files = base.join("files");
    let image = attachments::add(&source, &files, PNG).expect("image");
    let docs = folder(&source, "Docs", None);
    let tag = tag_commands::upsert(
        &source,
        TagUpsertInput {
            id: None,
            name: "Trabajo".into(),
            color_hex: "#4C8DFF".into(),
            parent_id: None,
        },
    )
    .expect("tag");
    let original = new_note(
        &source,
        Some(&docs),
        "Plan",
        doc(json!([
            { "type": "heading", "attrs": { "level": 2 }, "content": [{ "type": "text", "text": "Meta" }] },
            paragraph("Texto"),
            { "type": "image", "attrs": { "attachmentId": image.id, "mime": "image/png" } },
        ])),
    );
    crate::commands::notes::set_note_tags(&source, &original.id, &[tag.id]).expect("tags");
    let out = base.join("out");
    export_notes(&source, &files, &out, Format::Markdown, &Selection::All).expect("export");

    let (target, _) = test_state();
    let target_files = base.join("target-files");
    let report = import_text_files(&target, &target_files, &out).expect("import");
    assert_eq!(
        report,
        ImportReport {
            notes: 1,
            folders: 1,
            tags: 1,
            skipped: 0,
            duplicates: 0
        }
    );
    let imported = &all_notes(&target)[0];
    assert_eq!(imported.title, "Plan");
    assert_eq!(imported.created_at, original.created_at);
    let document: Value = serde_json::from_str(&imported.content_json).expect("json");
    assert_eq!(document["content"][0]["attrs"]["level"], 2);
    assert_eq!(document["content"][2]["type"], "image");
    assert_eq!(document["content"][2]["attrs"]["attachmentId"], image.id);
    assert!(attachments::path_of(&target_files, &image.id).exists());
    assert_eq!(imported.tag_ids.len(), 1);
    assert!(imported.folder_id.is_some());
}

#[test]
fn importing_the_same_folder_again_skips_what_it_already_has() {
    let (state, _) = test_state();
    let dir = TempDir::new();
    let base = dir.database().parent().expect("parent").to_path_buf();
    let source = base.join("src");
    fs::create_dir_all(source.join("a")).expect("dirs");
    fs::write(source.join("uno.md"), "primera").expect("uno");
    fs::write(source.join("a").join("dos.txt"), "segunda").expect("dos");
    let files = base.join("files");

    let first = import_text_files(&state, &files, &source).expect("first");
    assert_eq!((first.notes, first.duplicates), (2, 0));
    let again = import_text_files(&state, &files, &source).expect("again");
    assert_eq!((again.notes, again.duplicates), (0, 2));
    assert_eq!(all_notes(&state).len(), 2);

    // A file that changed is new content: it comes in as its own note.
    fs::write(source.join("uno.md"), "primera, corregida").expect("edit");
    let edited = import_text_files(&state, &files, &source).expect("edited");
    assert_eq!((edited.notes, edited.duplicates), (1, 1));
    assert_eq!(all_notes(&state).len(), 3);

    // A note the user deleted can be imported again.
    let gone = all_notes(&state)
        .into_iter()
        .find(|note| note.content_json.contains("segunda"))
        .expect("note");
    crate::commands::notes::delete(&state, &gone.id).expect("delete");
    let back = import_text_files(&state, &files, &source).expect("back");
    assert_eq!((back.notes, back.duplicates), (1, 1));
}

#[test]
fn the_same_file_in_two_places_is_not_a_duplicate() {
    let (state, _) = test_state();
    let dir = TempDir::new();
    let base = dir.database().parent().expect("parent").to_path_buf();
    let source = base.join("src");
    fs::create_dir_all(source.join("a")).expect("dirs");
    fs::write(source.join("x.md"), "igual").expect("x");
    fs::write(source.join("a").join("x.md"), "igual").expect("y");
    let report = import_text_files(&state, &base.join("files"), &source).expect("import");
    assert_eq!((report.notes, report.duplicates), (2, 0));
}

#[test]
fn plain_files_and_folders_are_imported_and_unsafe_images_are_refused() {
    let (state, _) = test_state();
    let dir = TempDir::new();
    let base = dir.database().parent().expect("parent").to_path_buf();
    let source = base.join("src");
    fs::create_dir_all(source.join("a").join("b")).expect("dirs");
    fs::write(source.join("suelto.txt"), "uno\ndos\n\ntres").expect("txt");
    fs::write(
        source.join("a").join("b").join("deep.md"),
        "# Título H1\n\ncuerpo",
    )
    .expect("md");
    fs::write(source.join(".oculto.md"), "x").expect("hidden");
    fs::write(
        source.join("nota.md"),
        "![x](../secreto.png) y ![w](https://x.es/a.png)",
    )
    .expect("md");
    fs::write(base.join("secreto.png"), PNG).expect("outside");

    let report = import_text_files(&state, &base.join("files"), &source).expect("import");
    assert_eq!(report.notes, 3);
    assert_eq!(report.folders, 2);
    assert_eq!(report.skipped, 2);
    let titles: Vec<String> = all_notes(&state).into_iter().map(|n| n.title).collect();
    assert!(titles.contains(&"suelto".to_string()));
    assert!(titles.contains(&"Título H1".to_string()));
    assert!(titles.contains(&"nota".to_string()));
    assert!(!base.join("files").exists());

    let wrong = import_text_files(&state, &base.join("files"), &base.join("secreto.png"));
    assert!(matches!(wrong, Err(AppError::Validation(_))));
}

#[test]
fn json_export_holds_every_note() {
    let (state, _) = test_state();
    let dir = TempDir::new();
    let base = dir.database().parent().expect("parent").to_path_buf();
    fs::create_dir_all(&base).expect("dir");
    new_note(&state, None, "Una", doc(json!([paragraph("x")])));
    let target = base.join("todo.json");
    let report = export_json(&state, &target).expect("export");
    assert_eq!(report.notes, 1);
    let data: Value =
        serde_json::from_str(&fs::read_to_string(&target).expect("file")).expect("json");
    assert_eq!(data["notes"][0]["title"], "Una");
    assert!(!target.with_extension("json.part").exists());
}

#[test]
fn an_evernote_export_becomes_a_folder_of_notes() {
    let (state, _) = test_state();
    let dir = TempDir::new();
    let base = dir.database().parent().expect("parent").to_path_buf();
    fs::create_dir_all(&base).expect("dir");
    let hash = md5_hex(PNG);
    let export = format!(
        "<en-export><note><title>Receta</title><content><![CDATA[<en-note><div>Harina</div><en-media hash=\"{hash}\" type=\"image/png\"/></en-note>]]></content>\
         <created>20200102T030405Z</created><updated>20210102T030405Z</updated><tag>cocina</tag>\
         <resource><data encoding=\"base64\">{}</data><mime>image/png</mime></resource></note>\
         <note><title></title><content><![CDATA[<en-note><div>sin título</div></en-note>]]></content></note></en-export>",
        base64_encode(PNG)
    );
    let file = base.join("Cocina.enex");
    fs::write(&file, export).expect("enex");

    let report = import_enex_files(&state, &base.join("files"), &[file]).expect("import");
    assert_eq!(
        report,
        ImportReport {
            notes: 2,
            folders: 1,
            tags: 1,
            skipped: 0,
            duplicates: 0
        }
    );
    let notes = all_notes(&state);
    let recipe = notes.iter().find(|n| n.title == "Receta").expect("recipe");
    assert_eq!(recipe.created_at, "2020-01-02T03:04:05Z");
    assert_eq!(recipe.updated_at, "2021-01-02T03:04:05Z");
    assert_eq!(recipe.tag_ids.len(), 1);
    assert!(notes.iter().any(|n| n.title == "Sin título"));

    let again = import_enex_files(&state, &base.join("files"), &[base.join("Cocina.enex")]);
    let again = again.expect("again");
    assert_eq!((again.notes, again.duplicates), (0, 2));
    assert_eq!(all_notes(&state).len(), 2);
    assert!(recipe.content_json.contains("\"image\""));

    let bad = base.join("mal.enex");
    fs::write(&bad, "<html/>").expect("bad");
    assert!(matches!(
        import_enex_files(&state, &base.join("files"), &[bad]),
        Err(AppError::Validation(_))
    ));
}

#[test]
fn dates_of_a_front_matter_accept_instants_and_plain_days() {
    assert_eq!(
        instant_of("2026-01-02T03:04:05Z").as_deref(),
        Some("2026-01-02T03:04:05Z")
    );
    assert_eq!(
        instant_of("2026-01-02").as_deref(),
        Some("2026-01-02T00:00:00Z")
    );
    assert_eq!(instant_of("ayer"), None);
}

#[test]
fn percent_encoding_in_image_paths_is_decoded() {
    assert_eq!(percent_decode("a%20b%2Fc%zz"), "a b/c%zz");
}

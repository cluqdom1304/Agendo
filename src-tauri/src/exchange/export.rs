//! Export of notes to Markdown or HTML files and of everything to a JSON dump.

use super::paths::*;
use super::*;

pub(super) struct Snapshot {
    pub(super) folders: Vec<Folder>,
    pub(super) notes: Vec<Note>,
    pub(super) tag_names: HashMap<String, String>,
    pub(super) note_titles: HashMap<String, String>,
    pub(super) task_titles: HashMap<String, String>,
}

pub(super) fn snapshot(state: &AppState, selection: &Selection) -> AppResult<Snapshot> {
    state.db.read(|conn| {
        let folders: Vec<Folder> = repo::folders::list(conn)?
            .into_iter()
            .map(|entry| entry.folder)
            .collect();
        let everything = notes::list(conn, Scope::All, None, Sort::Title)?;
        let note_titles = everything
            .iter()
            .map(|note| (note.id.clone(), note.title.clone()))
            .collect();
        let ids: Vec<String> = match selection {
            Selection::All => everything.iter().map(|note| note.id.clone()).collect(),
            Selection::Note(id) => {
                if !everything.iter().any(|note| &note.id == id) {
                    return Err(AppError::not_found("nota", id));
                }
                vec![id.clone()]
            }
            Selection::Folder(id) => {
                if !folders.iter().any(|folder| &folder.id == id) {
                    return Err(AppError::not_found("carpeta", id));
                }
                notes::list(conn, Scope::Folder(id), None, Sort::Title)?
                    .into_iter()
                    .map(|note| note.id)
                    .collect()
            }
        };
        let mut selected = Vec::with_capacity(ids.len());
        for id in &ids {
            if let Some(note) = notes::get(conn, id)? {
                selected.push(note);
            }
        }
        let tag_names = tags::list(conn)?
            .into_iter()
            .map(|entry| (entry.tag.id, entry.tag.name))
            .collect();
        Ok(Snapshot {
            folders,
            notes: selected,
            tag_names,
            note_titles,
            task_titles: repo_export::task_titles(conn)?,
        })
    })
}

/// The directory (relative to the export root) of each folder to write, creating no name twice.
pub(super) fn folder_dirs(
    folders: &[Folder],
    root: Option<&str>,
    namer: &mut Namer,
) -> HashMap<String, PathBuf> {
    let by_id: HashMap<&str, &Folder> = folders.iter().map(|f| (f.id.as_str(), f)).collect();
    let mut dirs: HashMap<String, PathBuf> = HashMap::new();
    // Parents come before children when sorted by depth, so each parent's directory exists.
    let depth = |start: &str| {
        let mut id = start.to_string();
        let mut levels = 0;
        while let Some(parent) = by_id.get(id.as_str()).and_then(|f| f.parent_id.clone()) {
            levels += 1;
            id = parent;
            if levels > 64 {
                break;
            }
        }
        levels
    };
    let mut ordered: Vec<&Folder> = folders.iter().collect();
    ordered.sort_by_key(|folder| (depth(&folder.id), folder.position, folder.id.clone()));
    for folder in ordered {
        let parent_dir = match folder.parent_id.as_deref() {
            Some(parent) if Some(folder.id.as_str()) != root => dirs.get(parent).cloned(),
            _ => Some(PathBuf::new()),
        };
        let Some(parent_dir) = parent_dir else {
            continue;
        };
        let name = namer.claim(&parent_dir, &safe_name(&folder.name, "Carpeta"), "");
        dirs.insert(folder.id.clone(), parent_dir.join(name));
    }
    dirs
}

pub(super) fn quote(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_default()
}

pub(super) fn front_matter(note: &Note, tag_names: &HashMap<String, String>) -> String {
    let mut tags: Vec<String> = note
        .tag_ids
        .iter()
        .filter_map(|id| tag_names.get(id))
        .map(|name| quote(&name.replace(',', " ")))
        .collect();
    tags.sort();
    format!(
        "---\ntitle: {}\ntags: [{}]\ncreated: {}\nupdated: {}\n---\n\n",
        quote(&note.title),
        tags.join(", "),
        note.created_at,
        note.updated_at
    )
}

/// Writes the selected notes below `target`. Images are copied to `target/adjuntos` and linked
/// with relative paths (Markdown) or embedded (HTML).
pub fn export_notes(
    state: &AppState,
    attachments_dir: &Path,
    target: &Path,
    format: Format,
    selection: &Selection,
) -> AppResult<ExportReport> {
    let data = snapshot(state, selection)?;
    fs::create_dir_all(target).map_err(|e| io("no se pudo crear la carpeta de destino", e))?;

    let mut namer = Namer::default();
    let root_folder = match selection {
        Selection::Folder(id) => Some(id.as_str()),
        _ => None,
    };
    let dirs = folder_dirs(&data.folders, root_folder, &mut namer);
    let images: RefCell<HashMap<String, Option<String>>> = RefCell::new(HashMap::new());
    let failure: RefCell<Option<AppError>> = RefCell::new(None);
    let mut report = ExportReport::default();

    for note in &data.notes {
        let dir = match (&note.folder_id, selection) {
            (Some(folder), Selection::All | Selection::Folder(_)) => {
                dirs.get(folder).cloned().unwrap_or_default()
            }
            // One note goes straight into the chosen folder, whatever folder it lives in.
            _ => PathBuf::new(),
        };
        let directory = target.join(&dir);
        fs::create_dir_all(&directory).map_err(|e| io("no se pudo crear una carpeta", e))?;
        let name = namer.claim(
            &dir,
            &safe_name(&note.title, "Sin título"),
            format.extension(),
        );
        let depth = dir.components().count();

        let read_image = |id: &str| -> Option<(Vec<u8>, &'static str)> {
            if !attachments::is_valid_id(id) {
                return None;
            }
            let bytes = fs::read(attachments::path_of(attachments_dir, id)).ok()?;
            let mime = attachments::sniff_image(&bytes)?;
            Some((bytes, mime))
        };
        // An image used by several notes is read and copied once: later uses find its file name
        // in `images` (`None` records an image that is missing or not usable).
        let image_path = |id: &str| -> Option<String> {
            if !images.borrow().contains_key(id) {
                match copy_image(attachments_dir, target, id) {
                    Ok(file) => images.borrow_mut().insert(id.to_string(), file),
                    Err(error) => {
                        *failure.borrow_mut() = Some(error);
                        return None;
                    }
                };
            }
            let file = images.borrow().get(id)?.clone()?;
            Some(format!("{}{ASSETS_DIR}/{file}", "../".repeat(depth)))
        };
        let image_src = |id: &str| -> Option<String> {
            let (bytes, mime) = read_image(id)?;
            Some(format!("data:{mime};base64,{}", base64_encode(&bytes)))
        };
        let note_label = |id: &str| -> String {
            data.note_titles
                .get(id)
                .cloned()
                .unwrap_or_else(|| "nota eliminada".to_string())
        };
        let task_label = |id: &str| -> String {
            data.task_titles.get(id).map_or_else(
                || "Tarea eliminada".to_string(),
                |title| format!("Tarea: {title}"),
            )
        };

        let document: Value = serde_json::from_str(&note.content_json)
            .map_err(|e| AppError::Internal(format!("documento no válido: {e}")))?;
        let text = match format {
            Format::Markdown => {
                let body = markdown::to_markdown(
                    &document,
                    &markdown::Labels {
                        image_path: &image_path,
                        note_label: &note_label,
                        task_label: &task_label,
                    },
                );
                let heading = format!("# {}\n\n", note.title.replace('\n', " "));
                format!("{}{heading}{body}\n", front_matter(note, &data.tag_names))
            }
            Format::Html => html::to_html(
                &document,
                &note.title,
                &html::Labels {
                    image_src: &image_src,
                    note_label: &note_label,
                    task_label: &task_label,
                },
            ),
        };
        if let Some(error) = failure.borrow_mut().take() {
            return Err(error);
        }
        fs::write(directory.join(name), text).map_err(|e| io("no se pudo guardar la nota", e))?;
        report.notes += 1;
    }
    report.files = images.borrow().values().flatten().count();
    Ok(report)
}

/// Copies the stored image `id` into the export's images folder and returns the name it has
/// there, or `None` if the file is missing or is not an image. Only the first bytes are read to
/// recognise the type; the file itself is copied by the system without passing through memory.
pub(super) fn copy_image(
    attachments_dir: &Path,
    target: &Path,
    id: &str,
) -> AppResult<Option<String>> {
    if !attachments::is_valid_id(id) {
        return Ok(None);
    }
    let source = attachments::path_of(attachments_dir, id);
    let mut head = [0_u8; 16];
    let read = fs::File::open(&source).and_then(|mut file| {
        let mut filled = 0;
        while filled < head.len() {
            match file.read(&mut head[filled..])? {
                0 => break,
                count => filled += count,
            }
        }
        Ok(filled)
    });
    let Some(mime) = read
        .ok()
        .and_then(|filled| attachments::sniff_image(&head[..filled]))
    else {
        return Ok(None);
    };
    let file = format!("{id}.{}", extension_of(mime));
    let folder = target.join(ASSETS_DIR);
    fs::create_dir_all(&folder)
        .and_then(|()| fs::copy(&source, folder.join(&file)))
        .map_err(|error| io("no se pudo guardar una imagen", error))?;
    Ok(Some(file))
}

/// Writes everything the application holds as one JSON file. The target is written beside
/// itself first, so an older file there is replaced only by a whole one.
pub fn export_json(state: &AppState, target: &Path) -> AppResult<ExportReport> {
    let exported_at = state.clock.now_string();
    let data = state
        .db
        .read(|conn| Ok(repo_export::everything(conn, &exported_at)?))?;
    let count = data["notes"].as_array().map_or(0, Vec::len);
    let text = serde_json::to_string_pretty(&data)
        .map_err(|e| AppError::Internal(format!("no se pudo generar el JSON: {e}")))?;
    write_whole(target, &text)?;
    Ok(ExportReport {
        notes: count,
        files: 0,
    })
}

/// Writes `text` to `target` through a `.part` file beside it, so an older file there is replaced
/// only by a whole one.
pub(super) fn write_whole(target: &Path, text: &str) -> AppResult<()> {
    let mut partial = target.as_os_str().to_os_string();
    partial.push(".part");
    let partial = PathBuf::from(partial);
    fs::write(&partial, text).map_err(|e| io("no se pudo guardar el archivo", e))?;
    fs::rename(&partial, target).map_err(|e| {
        let _ = fs::remove_file(&partial);
        io("no se pudo guardar el archivo", e)
    })
}

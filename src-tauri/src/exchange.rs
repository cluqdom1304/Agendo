//! Moving notes in and out of the application (F14): Markdown and HTML files, a JSON dump of
//! everything, folders of Markdown and `.txt`, and Evernote exports. The formats themselves live
//! in `domain/`; this module reads and writes the files and talks to the database. Nothing here
//! uses the network: images come only from files the user chose.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_json::Value;

use crate::{
    attachments::{self, MAX_BYTES},
    commands::{
        folders::{self as folder_commands, FolderUpsertInput},
        notes::derive,
        tags::{self as tag_commands, TagUpsertInput},
    },
    domain::{
        codec::base64_encode,
        dates, enex, html,
        markdown::{self, ImageFile},
        validation::{self, MAX_FOLDER_DEPTH},
    },
    error::{AppError, AppResult},
    repo::{
        self, attachments as attachment_repo, export,
        model::{Folder, NewNote, Note},
        notes::{self, Scope, Sort},
        tags,
    },
    state::AppState,
};

/// Folder, next to the exported notes, that holds their images.
pub const ASSETS_DIR: &str = "adjuntos";
/// Largest text file read when importing a folder.
const MAX_TEXT_BYTES: u64 = 10 * 1024 * 1024;
/// Largest Evernote export read. It is read in a stream, one note at a time, so memory depends
/// on the largest note (`enex::MAX_NOTE_BYTES`) and not on this limit, which only bounds time.
const MAX_ENEX_BYTES: u64 = 4 * 1024 * 1024 * 1024;
/// Most files taken from one folder.
const MAX_IMPORT_FILES: usize = 50_000;
/// Colours given to the tags an import creates, in turn.
const TAG_COLORS: [&str; 6] = [
    "#4C8DFF", "#34B36B", "#F08A3C", "#E5588A", "#8E6CE0", "#2BB5B8",
];

fn io(action: &str, error: impl std::fmt::Display) -> AppError {
    AppError::Io(format!("{action}: {error}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Markdown,
    Html,
}

impl Format {
    fn extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Html => "html",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    /// Every live note.
    All,
    /// A folder with its subfolders.
    Folder(String),
    Note(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportReport {
    pub notes: usize,
    /// Images written next to the notes.
    pub files: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub notes: usize,
    pub folders: usize,
    pub tags: usize,
    /// Files, notes or images that could not be read and were left out.
    pub skipped: usize,
    /// Notes already imported earlier (same origin and content), left out (F17-06).
    pub duplicates: usize,
}

// ---------------------------------------------------------------------------------------------
// Names
// ---------------------------------------------------------------------------------------------

/// A name that is valid on every system: no separators or reserved characters, not a reserved
/// device name, no trailing dot or space, at most 100 characters.
fn safe_name(raw: &str, fallback: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| {
            if c.is_control() || "<>:\"/\\|?*".contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    let truncated: String = cleaned.trim().chars().take(100).collect();
    let name = truncated.trim_end_matches(['.', ' ']).to_string();
    if name.is_empty() {
        return fallback.to_string();
    }
    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem.ends_with(|c: char| c.is_ascii_digit() && c != '0'));
    if reserved { format!("_{name}") } else { name }
}

/// Hands out names that do not repeat inside a directory (without regard to case).
#[derive(Default)]
struct Namer {
    used: HashMap<PathBuf, HashSet<String>>,
}

impl Namer {
    fn claim(&mut self, dir: &Path, base: &str, extension: &str) -> String {
        let used = self.used.entry(dir.to_path_buf()).or_default();
        let dotted = if extension.is_empty() {
            String::new()
        } else {
            format!(".{extension}")
        };
        let mut candidate = format!("{base}{dotted}");
        let mut number = 2;
        while !used.insert(candidate.to_lowercase()) {
            candidate = format!("{base} ({number}){dotted}");
            number += 1;
        }
        candidate
    }
}

fn extension_of(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "png",
    }
}

// ---------------------------------------------------------------------------------------------
// Export of notes
// ---------------------------------------------------------------------------------------------

struct Snapshot {
    folders: Vec<Folder>,
    notes: Vec<Note>,
    tag_names: HashMap<String, String>,
    note_titles: HashMap<String, String>,
    task_titles: HashMap<String, String>,
}

fn snapshot(state: &AppState, selection: &Selection) -> AppResult<Snapshot> {
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
                    return Err(AppError::NotFound(format!("nota {id} no encontrada")));
                }
                vec![id.clone()]
            }
            Selection::Folder(id) => {
                if !folders.iter().any(|folder| &folder.id == id) {
                    return Err(AppError::NotFound(format!("carpeta {id} no encontrada")));
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
            task_titles: export::task_titles(conn)?,
        })
    })
}

/// The directory (relative to the export root) of each folder to write, creating no name twice.
fn folder_dirs(
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

fn quote(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_default()
}

fn front_matter(note: &Note, tag_names: &HashMap<String, String>) -> String {
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
fn copy_image(attachments_dir: &Path, target: &Path, id: &str) -> AppResult<Option<String>> {
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
        .read(|conn| Ok(export::everything(conn, &exported_at)?))?;
    let count = data["notes"].as_array().map_or(0, Vec::len);
    let text = serde_json::to_string_pretty(&data)
        .map_err(|e| AppError::Internal(format!("no se pudo generar el JSON: {e}")))?;
    let mut partial = target.as_os_str().to_os_string();
    partial.push(".part");
    let partial = PathBuf::from(partial);
    fs::write(&partial, text).map_err(|e| io("no se pudo guardar el archivo", e))?;
    fs::rename(&partial, target).map_err(|e| {
        let _ = fs::remove_file(&partial);
        io("no se pudo guardar el archivo", e)
    })?;
    Ok(ExportReport {
        notes: count,
        files: 0,
    })
}

// ---------------------------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------------------------

/// Finds or creates the folders and tags an import needs, remembering what it made.
struct Catalog {
    folders: HashMap<(Option<String>, String), String>,
    tags: HashMap<String, String>,
    new_folders: usize,
    new_tags: usize,
    /// Origin and content hash of the notes imported before, and of those imported now.
    known: HashSet<(String, String)>,
}

impl Catalog {
    fn load(state: &AppState) -> AppResult<Self> {
        let folders = state.db.read(|conn| Ok(repo::folders::list(conn)?))?;
        let tag_list = state.db.read(|conn| Ok(tags::list(conn)?))?;
        let known = state
            .db
            .read(|conn| Ok(notes::import_sources(conn)?))?
            .into_iter()
            .collect();
        Ok(Self {
            known,
            folders: folders
                .into_iter()
                .map(|entry| {
                    (
                        (entry.folder.parent_id, entry.folder.name.to_lowercase()),
                        entry.folder.id,
                    )
                })
                .collect(),
            tags: tag_list
                .into_iter()
                .map(|entry| (entry.tag.name.to_lowercase(), entry.tag.id))
                .collect(),
            new_folders: 0,
            new_tags: 0,
        })
    }

    /// The folder at the end of `path`, below the top level; `None` for an empty path. Levels
    /// past the fifth are folded into the fifth.
    fn folder(&mut self, state: &AppState, path: &[String]) -> AppResult<Option<String>> {
        let mut parent: Option<String> = None;
        for raw in path.iter().take(MAX_FOLDER_DEPTH) {
            let name: String = raw.trim().chars().take(100).collect();
            if validation::validate_folder_name(&name).is_err() {
                continue;
            }
            let key = (parent.clone(), name.to_lowercase());
            let id = match self.folders.get(&key) {
                Some(id) => id.clone(),
                None => {
                    let created = folder_commands::upsert(
                        state,
                        FolderUpsertInput {
                            id: None,
                            name: name.clone(),
                            parent_id: parent.clone(),
                        },
                    )?;
                    self.new_folders += 1;
                    self.folders.insert(key, created.id.clone());
                    created.id
                }
            };
            parent = Some(id);
        }
        Ok(parent)
    }

    fn tag(&mut self, state: &AppState, raw: &str) -> Option<String> {
        let name = validation::validate_tag_name(raw).ok()?.to_string();
        if let Some(id) = self.tags.get(&name.to_lowercase()) {
            return Some(id.clone());
        }
        let color = TAG_COLORS[self.new_tags % TAG_COLORS.len()];
        let created = tag_commands::upsert(
            state,
            TagUpsertInput {
                id: None,
                name: name.clone(),
                color_hex: color.to_string(),
                parent_id: None,
            },
        )
        .ok()?;
        self.new_tags += 1;
        self.tags.insert(name.to_lowercase(), created.id.clone());
        Some(created.id)
    }
}

struct Incoming {
    title: String,
    document: Value,
    tags: Vec<String>,
    created: Option<String>,
    updated: Option<String>,
    /// Where it came from and a hash of what it said (F17-06).
    source: Option<(String, String)>,
}

fn store_note(
    state: &AppState,
    catalog: &mut Catalog,
    folder_id: Option<&str>,
    incoming: Incoming,
) -> AppResult<()> {
    let title: String = incoming
        .title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(validation::MAX_TITLE_CHARS)
        .collect();
    let title = if title.is_empty() {
        "Sin título".to_string()
    } else {
        title
    };
    let json = serde_json::to_string(&incoming.document)
        .map_err(|e| AppError::Internal(format!("documento no válido: {e}")))?;
    let (document, attachment_ids) = derive(json)?;
    let source = incoming.source;
    let tag_ids: Vec<String> = incoming
        .tags
        .iter()
        .filter_map(|name| catalog.tag(state, name))
        .collect();
    let now = state.clock.now_string();
    let created = incoming.created.unwrap_or_else(|| now.clone());
    let updated = incoming.updated.unwrap_or_else(|| created.clone());
    let new = NewNote {
        id: state.ids.new_id(),
        folder_id: folder_id.map(str::to_string),
        title,
        content_json: Some(document.json),
        content_text: document.text,
        task_ids: Vec::new(),
        note_ids: Vec::new(),
        tag_ids,
        now: created.clone(),
    };
    state.db.write(|tx| {
        let note = notes::create(tx, &new)?;
        attachment_repo::sync_note(tx, &note.id, &attachment_ids)?;
        if updated != created {
            notes::set_updated_at(tx, &note.id, &updated)?;
        }
        if let Some((origin, hash)) = &source {
            notes::set_import_source(tx, &note.id, origin, hash)?;
        }
        Ok(())
    })?;
    if let Some(source) = source {
        catalog.known.insert(source);
    }
    Ok(())
}

fn content_hash(parts: &[&str]) -> String {
    attachments::hash_of(parts.join("|").as_bytes())
}

fn stored_image(state: &AppState, dir: &Path, bytes: &[u8]) -> Option<ImageFile> {
    attachments::add(state, dir, bytes)
        .ok()
        .map(|attachment| ImageFile {
            attachment_id: attachment.id,
            mime: attachment.mime,
        })
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = |at: usize| {
            bytes
                .get(at)
                .and_then(|b| char::from(*b).to_digit(16))
                .and_then(|d| u8::try_from(d).ok())
        };
        if bytes[i] == b'%'
            && let (Some(high), Some(low)) = (hex(i + 1), hex(i + 2))
        {
            out.push(high * 16 + low);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// A date of a front matter as a UTC instant: full instants as they are, plain dates at 00:00.
fn instant_of(text: &str) -> Option<String> {
    if let Ok(instant) = dates::parse_instant(text) {
        return Some(dates::format_instant(instant));
    }
    let date = dates::parse_date(text.get(..10)?).ok()?;
    Some(format!("{}T00:00:00Z", dates::format_date(date)))
}

struct TextFile {
    folder: Vec<String>,
    path: PathBuf,
}

fn collect_files(
    dir: &Path,
    folder: &mut Vec<String>,
    files: &mut Vec<TextFile>,
    skipped: &mut usize,
    depth: usize,
) {
    if depth > 64 {
        return;
    }
    let Ok(read) = fs::read_dir(dir) else {
        *skipped += 1;
        return;
    };
    let mut entries: Vec<_> = read.filter_map(Result::ok).collect();
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        // Links could lead out of the chosen folder or loop; hidden entries are not notes.
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if name.starts_with('.') || kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            folder.push(name);
            collect_files(&entry.path(), folder, files, skipped, depth + 1);
            folder.pop();
        } else if is_text_note(&entry.path()) {
            if files.len() >= MAX_IMPORT_FILES {
                *skipped += 1;
            } else {
                files.push(TextFile {
                    folder: folder.clone(),
                    path: entry.path(),
                });
            }
        }
    }
}

/// The file's place inside the imported folder, with `/` separators on every system, or just its
/// name when a single file was chosen.
fn origin_of(path: &Path, source: &Path) -> String {
    let relative = path
        .strip_prefix(source)
        .ok()
        .filter(|r| !r.as_os_str().is_empty());
    let shown = relative
        .or_else(|| path.file_name().map(Path::new))
        .unwrap_or(path);
    shown
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn is_text_note(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        ["md", "markdown", "txt"]
            .iter()
            .any(|known| e.eq_ignore_ascii_case(known))
    })
}

/// Imports the `.md`, `.markdown` and `.txt` files of a folder (or one such file), keeping the
/// folder structure (up to five levels) and reading front matter for title, tags and dates.
pub fn import_text_files(
    state: &AppState,
    attachments_dir: &Path,
    source: &Path,
) -> AppResult<ImportReport> {
    let mut report = ImportReport::default();
    let mut files = Vec::new();
    let root = if source.is_dir() {
        collect_files(source, &mut Vec::new(), &mut files, &mut report.skipped, 0);
        source.to_path_buf()
    } else if is_text_note(source) {
        files.push(TextFile {
            folder: Vec::new(),
            path: source.to_path_buf(),
        });
        source.parent().map_or_else(PathBuf::new, Path::to_path_buf)
    } else {
        return Err(AppError::Validation(
            "elige una carpeta o un archivo .md, .markdown o .txt".into(),
        ));
    };
    let canonical_root =
        fs::canonicalize(&root).map_err(|e| io("no se pudo leer la carpeta", e))?;
    let mut catalog = Catalog::load(state)?;

    for file in files {
        let too_big = fs::metadata(&file.path).map_or(true, |meta| meta.len() > MAX_TEXT_BYTES);
        let Ok(bytes) = (if too_big {
            Err(())
        } else {
            fs::read(&file.path).map_err(|_| ())
        }) else {
            report.skipped += 1;
            continue;
        };
        let origin = format!("file:{}", origin_of(&file.path, source));
        let hash = attachments::hash_of(&bytes);
        if catalog.known.contains(&(origin.clone(), hash.clone())) {
            report.duplicates += 1;
            continue;
        }
        let text = String::from_utf8_lossy(&bytes);
        let stem = file
            .path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let base = file
            .path
            .parent()
            .map_or_else(PathBuf::new, Path::to_path_buf);
        let mut missing_images = 0usize;

        let incoming = if file
            .path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("txt"))
        {
            Incoming {
                title: stem,
                document: markdown::from_plain_text(&text),
                tags: Vec::new(),
                created: None,
                updated: None,
                source: Some((origin, hash)),
            }
        } else {
            let (front, body) = markdown::split_front_matter(&text);
            let mut resolve = |src: &str| -> Option<ImageFile> {
                let found = resolve_image(&canonical_root, &base, src)
                    .and_then(|path| fs::read(path).ok())
                    .filter(|bytes| bytes.len() <= MAX_BYTES)
                    .and_then(|bytes| stored_image(state, attachments_dir, &bytes));
                if found.is_none() {
                    missing_images += 1;
                }
                found
            };
            let mut document = markdown::from_markdown(body, &mut resolve);
            let heading = first_heading(&document);
            let title = match (&front.title, &heading) {
                (Some(title), Some(heading)) if title == heading => {
                    drop_first_block(&mut document);
                    title.clone()
                }
                (Some(title), _) => title.clone(),
                (None, Some(heading)) => {
                    drop_first_block(&mut document);
                    heading.clone()
                }
                (None, None) => stem,
            };
            Incoming {
                title,
                document,
                tags: front.tags,
                created: front.created.as_deref().and_then(instant_of),
                updated: front.updated.as_deref().and_then(instant_of),
                source: Some((origin, hash)),
            }
        };
        report.skipped += missing_images;
        match catalog
            .folder(state, &file.folder)
            .and_then(|folder| store_note(state, &mut catalog, folder.as_deref(), incoming))
        {
            Ok(()) => report.notes += 1,
            Err(_) => report.skipped += 1,
        }
    }
    report.folders = catalog.new_folders;
    report.tags = catalog.new_tags;
    if report.notes > 0 {
        state.db.analyze();
    }
    Ok(report)
}

/// The file an image link points to, only if it lies inside the folder being imported and is
/// not a web address.
fn resolve_image(root: &Path, base: &Path, src: &str) -> Option<PathBuf> {
    let src = src.trim();
    if src.is_empty() || src.contains("://") || src.starts_with("data:") || src.starts_with('/') {
        return None;
    }
    let decoded = percent_decode(src.split(['#', '?']).next().unwrap_or(src));
    let path = fs::canonicalize(base.join(decoded)).ok()?;
    path.starts_with(root).then_some(path)
}

fn first_heading(document: &Value) -> Option<String> {
    let first = document.get("content")?.as_array()?.first()?;
    if first.get("type")?.as_str()? != "heading" || first.get("attrs")?.get("level")?.as_u64()? != 1
    {
        return None;
    }
    let text: String = first
        .get("content")?
        .as_array()?
        .iter()
        .filter_map(|node| node.get("text").and_then(Value::as_str))
        .collect();
    (!text.trim().is_empty()).then(|| text.trim().to_string())
}

fn drop_first_block(document: &mut Value) {
    if let Some(blocks) = document.get_mut("content").and_then(Value::as_array_mut)
        && !blocks.is_empty()
    {
        blocks.remove(0);
    }
}

/// Imports Evernote exports: each file becomes a folder named after it (Evernote names the file
/// after the notebook), and its notes go inside with their tags, images and dates.
pub fn import_enex_files(
    state: &AppState,
    attachments_dir: &Path,
    files: &[PathBuf],
) -> AppResult<ImportReport> {
    let mut report = ImportReport::default();
    let mut catalog = Catalog::load(state)?;
    for file in files {
        let size = fs::metadata(file)
            .map_err(|e| io("no se pudo leer el archivo", e))?
            .len();
        if size > MAX_ENEX_BYTES {
            return Err(AppError::Validation(
                "el archivo .enex es demasiado grande (máximo 4 GB)".into(),
            ));
        }
        let input = fs::File::open(file).map_err(|e| io("no se pudo leer el archivo", e))?;
        let notebook = file
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let folder = catalog.folder(state, std::slice::from_ref(&notebook))?;
        let mut failed = 0usize;
        let mut dropped = 0usize;
        let mut stored = 0usize;
        read_notes(input, state, attachments_dir, &mut |note| {
            dropped += note.dropped_files;
            let origin = format!(
                "enex:{notebook}:{}:{}",
                note.created.as_deref().unwrap_or(""),
                note.title
            );
            let hash = content_hash(&[&note.title, &note.document.to_string()]);
            if catalog.known.contains(&(origin.clone(), hash.clone())) {
                report.duplicates += 1;
                return;
            }
            let incoming = Incoming {
                title: note.title,
                document: note.document,
                tags: note.tags,
                created: note.created,
                updated: note.updated,
                source: Some((origin, hash)),
            };
            match store_note(state, &mut catalog, folder.as_deref(), incoming) {
                Ok(()) => stored += 1,
                Err(_) => failed += 1,
            }
        })?;
        report.notes += stored;
        report.skipped += failed + dropped;
    }
    report.folders = catalog.new_folders;
    report.tags = catalog.new_tags;
    if report.notes > 0 {
        state.db.analyze();
    }
    Ok(report)
}

fn read_notes(
    input: impl std::io::Read,
    state: &AppState,
    attachments_dir: &Path,
    on_note: &mut dyn FnMut(enex::EnexNote),
) -> AppResult<usize> {
    enex::read_enex_stream(
        input,
        &mut |bytes| stored_image(state, attachments_dir, bytes),
        on_note,
    )
    .map_err(AppError::Validation)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let report = export_notes(&state, &files, &target, Format::Markdown, &Selection::All)
            .expect("export");
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
}

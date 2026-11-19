//! Import of Markdown, text and Evernote files.

use super::*;
use super::{catalog::*, paths::*};

/// A date of a front matter as a UTC instant: full instants as they are, plain dates at 00:00.
pub(super) fn instant_of(text: &str) -> Option<String> {
    if let Ok(instant) = dates::parse_instant(text) {
        return Some(dates::format_instant(instant));
    }
    let date = dates::parse_date(text.get(..10)?).ok()?;
    Some(format!("{}T00:00:00Z", dates::format_date(date)))
}

pub(super) struct TextFile {
    pub(super) folder: Vec<String>,
    pub(super) path: PathBuf,
}

pub(super) fn collect_files(
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

pub(super) fn is_text_note(path: &Path) -> bool {
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

pub(super) fn first_heading(document: &Value) -> Option<String> {
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

pub(super) fn drop_first_block(document: &mut Value) {
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

pub(super) fn read_notes(
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

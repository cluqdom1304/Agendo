//! The `.agendo` copy (D-47): a zip with the database (`agenda.sqlite`, a consistent snapshot)
//! and the files of the notes (`attachments/<sha256>`). Restoring also accepts the plain
//! `.sqlite` copies of earlier versions (see `commands::backup`). Nothing is trusted from the
//! archive: only the three kinds of entry below are read, files are re-hashed, and sizes are
//! capped while copying, whatever the archive says about itself.

use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
    attachments::{DIR_NAME, MAX_BYTES, hash_of, is_valid_id, path_of, sniff_image},
    db::{self, MIGRATIONS},
    error::{AppError, AppResult},
    repo::attachments as attachment_repo,
    state::AppState,
};

const DATABASE_ENTRY: &str = "agenda.sqlite";
const MANIFEST_ENTRY: &str = "manifest.json";
const MANIFEST: &str = "{\"format\":\"agendo\",\"version\":1}";
/// Largest database accepted from an archive.
const MAX_DATABASE_BYTES: u64 = 1024 * 1024 * 1024;

/// Whether the file at `path` is a zip (it starts with `PK`), as opposed to a SQLite copy.
pub fn is_archive(path: &Path) -> bool {
    let mut magic = [0u8; 4];
    File::open(path)
        .and_then(|mut file| file.read_exact(&mut magic))
        .is_ok()
        && (magic == *b"PK\x03\x04" || magic == *b"PK\x05\x06")
}

fn io_error(action: &str, error: impl std::fmt::Display) -> AppError {
    AppError::Io(format!("{action}: {error}"))
}

fn invalid(reason: &str) -> AppError {
    AppError::Validation(format!("el archivo no es una copia válida: {reason}"))
}

fn sibling(path: &Path, name: &str) -> PathBuf {
    path.with_file_name(name)
}

/// Writes the copy at `target`; an older file there is replaced only when the new one is whole.
pub fn create(
    state: &AppState,
    database: &Path,
    attachments_dir: &Path,
    target: &Path,
) -> AppResult<()> {
    let protected = crate::commands::backup::protected_files(database);
    if protected.iter().any(|path| path == target) {
        return Err(AppError::Validation(
            "no se puede guardar la copia sobre la base de datos en uso".into(),
        ));
    }
    let snapshot = sibling(target, ".agendo-snapshot.tmp");
    let result = write_archive(state, attachments_dir, &snapshot, target);
    let _ = fs::remove_file(&snapshot);
    result
}

fn write_archive(
    state: &AppState,
    attachments_dir: &Path,
    snapshot: &Path,
    target: &Path,
) -> AppResult<()> {
    state
        .db
        .with_writer(|conn| db::backup::create(conn, snapshot, &[]))?;
    let files = state.db.read(|conn| Ok(attachment_repo::all(conn)?))?;

    let partial = sibling(target, ".agendo.tmp");
    let written = (|| -> AppResult<()> {
        let output =
            File::create(&partial).map_err(|e| io_error("no se pudo crear la copia", e))?;
        let mut zip = ZipWriter::new(output);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        // Images are already compressed: they are stored as they are.
        let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        zip.start_file(MANIFEST_ENTRY, options)
            .map_err(|e| io_error("no se pudo escribir la copia", e))?;
        zip.write_all(MANIFEST.as_bytes())
            .map_err(|e| io_error("no se pudo escribir la copia", e))?;

        zip.start_file(DATABASE_ENTRY, options.large_file(true))
            .map_err(|e| io_error("no se pudo escribir la copia", e))?;
        let mut database = File::open(snapshot)
            .map_err(|e| io_error("no se pudo leer la copia de la base de datos", e))?;
        io::copy(&mut database, &mut zip)
            .map_err(|e| io_error("no se pudo escribir la copia", e))?;

        for file in files {
            // A registered file that is gone from the disk is left out: the note shows it as
            // unavailable, as it did before.
            let Ok(mut source) = File::open(path_of(attachments_dir, &file.id)) else {
                continue;
            };
            zip.start_file(format!("{DIR_NAME}/{}", file.id), stored)
                .map_err(|e| io_error("no se pudo escribir la copia", e))?;
            io::copy(&mut source, &mut zip)
                .map_err(|e| io_error("no se pudo escribir la copia", e))?;
        }
        zip.finish()
            .map_err(|e| io_error("no se pudo cerrar la copia", e))?;
        Ok(())
    })();
    if let Err(error) = written {
        let _ = fs::remove_file(&partial);
        return Err(error);
    }
    fs::rename(&partial, target).map_err(|e| {
        let _ = fs::remove_file(&partial);
        io_error("no se pudo guardar la copia", e)
    })
}

/// Copies at most `limit` bytes of `reader`; `None` if there is more.
fn read_capped(reader: &mut impl Read, limit: u64) -> io::Result<Option<Vec<u8>>> {
    let mut bytes = Vec::new();
    reader.take(limit + 1).read_to_end(&mut bytes)?;
    Ok((bytes.len() as u64 <= limit).then_some(bytes))
}

/// What an archive holds, after checking the names of its entries.
struct Contents {
    has_database: bool,
    attachment_ids: Vec<String>,
}

fn inspect(zip: &mut ZipArchive<File>) -> AppResult<Contents> {
    let mut contents = Contents {
        has_database: false,
        attachment_ids: Vec::new(),
    };
    for index in 0..zip.len() {
        let entry = zip
            .by_index_raw(index)
            .map_err(|_| invalid("está dañado"))?;
        let name = entry
            .name()
            .map_err(|_| invalid("contiene un nombre no válido"))?
            .into_owned();
        if entry.is_dir() {
            continue;
        }
        if name == DATABASE_ENTRY {
            contents.has_database = true;
        } else if name == MANIFEST_ENTRY {
            // Informative only.
        } else if let Some(id) = name.strip_prefix(&format!("{DIR_NAME}/")) {
            if !is_valid_id(id) {
                return Err(invalid("contiene un adjunto con un nombre no válido"));
            }
            contents.attachment_ids.push(id.to_string());
        } else {
            return Err(invalid("contiene entradas desconocidas"));
        }
    }
    if !contents.has_database {
        return Err(invalid("no contiene la base de datos"));
    }
    Ok(contents)
}

/// Replaces the data with the archive's. The database goes through the same checks and the
/// same safety copy as a plain `.sqlite` restore; the files are added afterwards, and a file
/// that does not match its name or is not an image is skipped.
pub fn restore(
    state: &AppState,
    database: &Path,
    safety_copy: &Path,
    attachments_dir: &Path,
    source: &Path,
) -> AppResult<()> {
    if !source.is_file() {
        return Err(AppError::NotFound(
            "el archivo de la copia no existe".into(),
        ));
    }
    let file = File::open(source).map_err(|e| io_error("no se pudo abrir la copia", e))?;
    let mut zip = ZipArchive::new(file).map_err(|_| invalid("no es un archivo .agendo"))?;
    let contents = inspect(&mut zip)?;

    let staged = sibling(database, "agenda.restore.tmp");
    let outcome = (|| -> AppResult<()> {
        let mut entry = zip
            .by_name(DATABASE_ENTRY)
            .map_err(|_| invalid("no contiene la base de datos"))?;
        let bytes = read_capped(&mut entry, MAX_DATABASE_BYTES)
            .map_err(|_| invalid("está dañado"))?
            .ok_or_else(|| invalid("la base de datos es demasiado grande"))?;
        drop(entry);
        fs::write(&staged, bytes)
            .map_err(|e| io_error("no se pudo preparar la restauración", e))?;
        state.db.with_writer(|conn| {
            db::backup::restore(
                conn,
                &staged,
                safety_copy,
                &crate::commands::backup::protected_files(database),
                &MIGRATIONS,
            )
        })
    })();
    let _ = fs::remove_file(&staged);
    outcome?;

    fs::create_dir_all(attachments_dir)
        .map_err(|e| io_error("no se pudo crear la carpeta de adjuntos", e))?;
    for id in contents.attachment_ids {
        let target = path_of(attachments_dir, &id);
        if target.exists() {
            continue;
        }
        let Ok(mut entry) = zip.by_name(&format!("{DIR_NAME}/{id}")) else {
            continue;
        };
        let Ok(Some(bytes)) = read_capped(&mut entry, MAX_BYTES as u64) else {
            continue;
        };
        if hash_of(&bytes) != id || sniff_image(&bytes).is_none() {
            continue;
        }
        let partial = attachments_dir.join(format!("{id}.part"));
        if fs::write(&partial, &bytes).is_ok() && fs::rename(&partial, &target).is_err() {
            let _ = fs::remove_file(&partial);
        }
    }
    // Registered files whose bytes did not come: nothing to do, the notes show them as gone.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        attachments,
        commands::notes::{NoteCreateInput, create as create_note},
        repo::notes,
        state::test_state,
        test_support::TempDir,
    };

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nimage-bytes";

    struct Paths {
        _temp: TempDir,
        database: PathBuf,
        safety: PathBuf,
        attachments: PathBuf,
    }

    fn paths() -> Paths {
        let temp = TempDir::new();
        let database = temp.database();
        fs::create_dir_all(database.parent().expect("parent")).expect("dir");
        Paths {
            safety: sibling(&database, "safety.sqlite"),
            attachments: sibling(&database, DIR_NAME),
            database,
            _temp: temp,
        }
    }

    fn note_with_image(state: &AppState, dir: &Path) -> (String, String) {
        let image = attachments::add(state, dir, PNG).expect("add");
        let doc = format!(
            r#"{{"type":"doc","content":[{{"type":"image","attrs":{{"attachmentId":"{}"}}}}]}}"#,
            image.id
        );
        let note = create_note(
            state,
            NoteCreateInput {
                title: Some("con imagen".into()),
                content_json: Some(doc),
                ..NoteCreateInput::default()
            },
        )
        .expect("note");
        (note.id, image.id)
    }

    #[test]
    fn an_archive_carries_the_database_and_the_files_back_to_a_clean_install() {
        let (state, _) = test_state();
        let from = paths();
        let (note_id, image_id) = note_with_image(&state, &from.attachments);
        let archive = sibling(&from.database, "copia.agendo");
        create(&state, &from.database, &from.attachments, &archive).expect("create");
        assert!(is_archive(&archive));

        // Another installation: empty database, no files.
        let (other, _) = test_state();
        let to = paths();
        restore(&other, &to.database, &to.safety, &to.attachments, &archive).expect("restore");
        let found = other
            .db
            .read(|conn| Ok(notes::get(conn, &note_id)?))
            .expect("read");
        assert_eq!(found.expect("the note came back").title, "con imagen");
        let (_, bytes) = attachments::read(&other, &to.attachments, &image_id).expect("image");
        assert_eq!(bytes, PNG);
    }

    #[test]
    fn restoring_over_existing_files_keeps_them() {
        let (state, _) = test_state();
        let from = paths();
        let (_, image_id) = note_with_image(&state, &from.attachments);
        let archive = sibling(&from.database, "copia.agendo");
        create(&state, &from.database, &from.attachments, &archive).expect("create");
        restore(
            &state,
            &from.database,
            &from.safety,
            &from.attachments,
            &archive,
        )
        .expect("restore");
        assert!(path_of(&from.attachments, &image_id).is_file());
    }

    fn zip_with(path: &Path, entries: &[(&str, &[u8])]) {
        let mut zip = ZipWriter::new(File::create(path).expect("file"));
        for (name, bytes) in entries {
            zip.start_file(*name, SimpleFileOptions::default())
                .expect("start");
            zip.write_all(bytes).expect("write");
        }
        zip.finish().expect("finish");
    }

    #[test]
    fn a_bad_archive_is_rejected_before_anything_changes() {
        let (state, _) = test_state();
        let at = paths();
        let good = sibling(&at.database, "good.agendo");
        create(&state, &at.database, &at.attachments, &good).expect("create");
        let database_bytes = {
            let mut zip = ZipArchive::new(File::open(&good).expect("open")).expect("zip");
            let mut entry = zip.by_name(DATABASE_ENTRY).expect("db");
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).expect("read");
            bytes
        };

        type Entries<'a> = Vec<(&'a str, &'a [u8])>;
        let cases: Vec<(&str, Entries)> = vec![
            ("not-a-zip.agendo", vec![]),
            ("no-db.agendo", vec![("manifest.json", b"{}")]),
            (
                "stranger.agendo",
                vec![("agenda.sqlite", &database_bytes), ("../evil.exe", b"MZ")],
            ),
            (
                "bad-name.agendo",
                vec![
                    ("agenda.sqlite", &database_bytes),
                    ("attachments/../x", b"1"),
                ],
            ),
            ("garbage-db.agendo", vec![("agenda.sqlite", b"nope")]),
        ];
        for (name, entries) in cases {
            let path = sibling(&at.database, name);
            if entries.is_empty() {
                fs::write(&path, b"PK\x03\x04 not really").expect("write");
            } else {
                zip_with(&path, &entries);
            }
            let result = restore(&state, &at.database, &at.safety, &at.attachments, &path);
            assert!(
                matches!(result, Err(AppError::Validation(_))),
                "{name}: {result:?}"
            );
        }
        assert!(!at.safety.exists(), "nothing was saved aside");
        assert!(matches!(
            restore(
                &state,
                &at.database,
                &at.safety,
                &at.attachments,
                &sibling(&at.database, "missing.agendo")
            ),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn files_that_do_not_match_their_name_or_are_not_images_are_skipped() {
        let (state, _) = test_state();
        let at = paths();
        let good = sibling(&at.database, "good.agendo");
        create(&state, &at.database, &at.attachments, &good).expect("create");
        let mut zip = ZipArchive::new(File::open(&good).expect("open")).expect("zip");
        let mut database = Vec::new();
        zip.by_name(DATABASE_ENTRY)
            .expect("db")
            .read_to_end(&mut database)
            .expect("read");

        let real = hash_of(PNG);
        let lie = "0".repeat(64);
        let script = b"<script>alert(1)</script>";
        let script_id = hash_of(script);
        let path = sibling(&at.database, "mixed.agendo");
        zip_with(
            &path,
            &[
                ("agenda.sqlite", &database),
                (&format!("attachments/{real}"), PNG),
                (&format!("attachments/{lie}"), PNG),
                (&format!("attachments/{script_id}"), script),
            ],
        );
        restore(&state, &at.database, &at.safety, &at.attachments, &path).expect("restore");
        assert!(path_of(&at.attachments, &real).is_file());
        assert!(!path_of(&at.attachments, &lie).exists());
        assert!(!path_of(&at.attachments, &script_id).exists());
    }

    #[test]
    fn the_copy_cannot_overwrite_the_live_database() {
        let (state, _) = test_state();
        let at = paths();
        fs::write(&at.database, b"x").expect("live");
        let result = create(&state, &at.database, &at.attachments, &at.database);
        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}

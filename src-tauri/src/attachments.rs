//! The files of the notes (D-47): stored in `attachments/`, named by the SHA-256 of their
//! content, so the same image pasted twice is one file. What is accepted is decided by the
//! bytes (their signature), never by the name or the type the caller claims.

use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::{
    error::{AppError, AppResult},
    repo::attachments::{self, Attachment},
    state::AppState,
};

/// Folder of the files, inside the data directory.
pub const DIR_NAME: &str = "attachments";
/// Largest file accepted.
pub const MAX_BYTES: usize = 20 * 1024 * 1024;
/// How long an unused file is kept before it is collected.
const GRACE_HOURS: u32 = 24;

/// The image type of `bytes`, or `None` if it is not a supported one.
pub fn sniff_image(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

pub fn hash_of(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        })
}

/// Whether `id` can be the name of a stored file: 64 lowercase hex digits. Anything else is
/// refused before it gets near a path.
pub fn is_valid_id(id: &str) -> bool {
    id.len() == 64
        && id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn path_of(dir: &Path, id: &str) -> PathBuf {
    dir.join(id)
}

fn io(action: &str, error: std::io::Error) -> AppError {
    AppError::Io(format!("{action}: {error}"))
}

/// Stores an image and registers it. Adding the same content again changes nothing.
pub fn add(state: &AppState, dir: &Path, bytes: &[u8]) -> AppResult<Attachment> {
    if bytes.is_empty() {
        return Err(AppError::Validation("el fichero está vacío".into()));
    }
    if bytes.len() > MAX_BYTES {
        return Err(AppError::Validation(
            "el fichero supera el tamaño máximo (20 MB)".into(),
        ));
    }
    let mime = sniff_image(bytes).ok_or_else(|| {
        AppError::Validation("solo se admiten imágenes PNG, JPEG, GIF o WebP".into())
    })?;
    let attachment = Attachment {
        id: hash_of(bytes),
        mime: mime.into(),
        size_bytes: i64::try_from(bytes.len()).unwrap_or(i64::MAX),
    };
    fs::create_dir_all(dir).map_err(|e| io("no se pudo crear la carpeta de adjuntos", e))?;
    let target = path_of(dir, &attachment.id);
    if !target.exists() {
        // Written aside and renamed: a crash never leaves half a file under its final name.
        let partial = dir.join(format!("{}.part", attachment.id));
        fs::write(&partial, bytes).map_err(|e| io("no se pudo guardar el adjunto", e))?;
        fs::rename(&partial, &target).map_err(|e| io("no se pudo guardar el adjunto", e))?;
    }
    let now = state.clock.now_string();
    state
        .db
        .write(|tx| Ok(attachments::insert_if_absent(tx, &attachment, &now)?))?;
    Ok(attachment)
}

/// The bytes and type of a stored file.
pub fn read(state: &AppState, dir: &Path, id: &str) -> AppResult<(Attachment, Vec<u8>)> {
    if !is_valid_id(id) {
        return Err(AppError::Validation(
            "identificador de adjunto no válido".into(),
        ));
    }
    let attachment = state
        .db
        .read(|conn| Ok(attachments::get(conn, id)?))?
        .ok_or_else(|| AppError::NotFound(format!("adjunto {id} no encontrado")))?;
    let bytes = fs::read(path_of(dir, id))
        .map_err(|_| AppError::NotFound(format!("el fichero del adjunto {id} no está")))?;
    Ok((attachment, bytes))
}

/// Deletes the files no note uses (past the grace period). Returns how many went.
pub fn collect_orphans(state: &AppState, dir: &Path) -> AppResult<usize> {
    let now = state.clock.now_string();
    let orphans = state
        .db
        .read(|conn| Ok(attachments::orphans(conn, &now, GRACE_HOURS)?))?;
    let mut gone = Vec::new();
    for id in orphans {
        if !is_valid_id(&id) {
            continue;
        }
        // The file first: if it cannot be deleted the row stays and the next start tries again.
        match fs::remove_file(path_of(dir, &id)) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => continue,
        }
        gone.push(id);
    }
    // One transaction for all the rows, instead of one write (and one commit) per file.
    state.db.write(|tx| {
        for id in &gone {
            attachments::delete(tx, id)?;
        }
        Ok(gone.len())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{state::test_state, test_support::TempDir};

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nrest-of-the-image";

    fn dir() -> (TempDir, PathBuf) {
        let temp = TempDir::new();
        let path = temp.database().with_file_name(DIR_NAME);
        (temp, path)
    }

    #[test]
    fn signatures_decide_what_is_an_image() {
        assert_eq!(sniff_image(PNG), Some("image/png"));
        assert_eq!(sniff_image(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("image/jpeg"));
        assert_eq!(sniff_image(b"GIF89a..."), Some("image/gif"));
        assert_eq!(sniff_image(b"RIFF\0\0\0\0WEBPVP8 "), Some("image/webp"));
        assert_eq!(sniff_image(b"<svg xmlns='...'></svg>"), None);
        assert_eq!(sniff_image(b"MZ\x90\0"), None);
    }

    #[test]
    fn ids_are_64_lowercase_hex_digits_and_nothing_else() {
        assert!(is_valid_id(&hash_of(b"x")));
        let short = "a".repeat(63);
        let upper = "A".repeat(64);
        let beyond = "g".repeat(64);
        for bad in ["", "../x", &short, &upper, &beyond] {
            assert!(!is_valid_id(bad), "{bad}");
        }
        assert_eq!(
            hash_of(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn the_same_content_is_stored_once_and_read_back() {
        let (state, _) = test_state();
        let (_temp, dir) = dir();
        let first = add(&state, &dir, PNG).expect("add");
        let second = add(&state, &dir, PNG).expect("again");
        assert_eq!(first, second);
        assert_eq!(first.mime, "image/png");
        assert_eq!(fs::read_dir(&dir).expect("dir").count(), 1);
        let (found, bytes) = read(&state, &dir, &first.id).expect("read");
        assert_eq!(found, first);
        assert_eq!(bytes, PNG);
    }

    #[test]
    fn orphans_are_collected_in_one_batch_after_the_grace_period() {
        let (state, clock) = test_state();
        let (_temp, dir) = dir();
        let mut ids = Vec::new();
        for index in 0..3_u8 {
            let mut bytes = PNG.to_vec();
            bytes.push(index);
            ids.push(add(&state, &dir, &bytes).expect("add").id);
        }
        assert_eq!(collect_orphans(&state, &dir).expect("too early"), 0);

        clock.advance(chrono::Duration::hours(i64::from(GRACE_HOURS) + 1));
        assert_eq!(collect_orphans(&state, &dir).expect("collect"), 3);
        assert_eq!(fs::read_dir(&dir).expect("dir").count(), 0);
        for id in ids {
            assert!(matches!(
                read(&state, &dir, &id),
                Err(AppError::NotFound(_))
            ));
        }
        assert_eq!(collect_orphans(&state, &dir).expect("again"), 0);
    }

    #[test]
    fn bad_input_is_refused_before_anything_is_written() {
        let (state, _) = test_state();
        let (_temp, dir) = dir();
        let too_big = vec![0u8; MAX_BYTES + 1];
        for bytes in [&b""[..], b"not an image", &too_big] {
            let result = add(&state, &dir, bytes);
            assert!(matches!(result, Err(AppError::Validation(_))));
        }
        assert!(!dir.exists());
        assert!(matches!(
            read(&state, &dir, "../../agenda.sqlite"),
            Err(AppError::Validation(_))
        ));
        assert!(matches!(
            read(&state, &dir, &"a".repeat(64)),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn unused_files_are_collected_only_after_the_grace_period() {
        let (state, clock) = test_state();
        let (_temp, dir) = dir();
        let kept = add(&state, &dir, PNG).expect("add");
        assert_eq!(collect_orphans(&state, &dir).expect("collect"), 0);
        clock.advance(chrono::Duration::hours(25));
        assert_eq!(collect_orphans(&state, &dir).expect("collect"), 1);
        assert!(!path_of(&dir, &kept.id).exists());
        assert!(read(&state, &dir, &kept.id).is_err());
    }

    #[test]
    fn saving_a_note_keeps_the_files_it_shows_and_releases_the_rest() {
        use crate::services::notes::{NoteCreateInput, NotePatchInput, create, update};

        let (state, clock) = test_state();
        let (_temp, dir) = dir();
        let image = add(&state, &dir, PNG).expect("add");
        let doc = format!(
            r#"{{"type":"doc","content":[{{"type":"image","attrs":{{"attachmentId":"{}"}}}}]}}"#,
            image.id
        );
        let note = create(
            &state,
            NoteCreateInput {
                content_json: Some(doc),
                ..NoteCreateInput::default()
            },
        )
        .expect("create");
        clock.advance(chrono::Duration::hours(48));
        assert_eq!(collect_orphans(&state, &dir).expect("collect"), 0);

        update(
            &state,
            &note.id,
            NotePatchInput {
                content_json: Some(r#"{"type":"doc","content":[{"type":"paragraph"}]}"#.into()),
                ..NotePatchInput::default()
            },
        )
        .expect("update");
        assert_eq!(collect_orphans(&state, &dir).expect("collect"), 1);
    }
}

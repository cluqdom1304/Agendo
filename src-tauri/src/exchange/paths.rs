//! File and folder names for export and import: safe names, unique names and import origins.

use super::*;

/// A name that is valid on every system: no separators or reserved characters, not a reserved
/// device name, no trailing dot or space, at most 100 characters.
pub(super) fn safe_name(raw: &str, fallback: &str) -> String {
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
pub(super) struct Namer {
    pub(super) used: HashMap<PathBuf, HashSet<String>>,
}

impl Namer {
    pub(super) fn claim(&mut self, dir: &Path, base: &str, extension: &str) -> String {
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

pub(super) fn extension_of(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "png",
    }
}

pub(super) fn percent_decode(text: &str) -> String {
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

/// The file's place inside the imported folder, with `/` separators on every system, or just its
/// name when a single file was chosen.
pub(super) fn origin_of(path: &Path, source: &Path) -> String {
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

/// The file an image link points to, only if it lies inside the folder being imported and is
/// not a web address.
pub(super) fn resolve_image(root: &Path, base: &Path, src: &str) -> Option<PathBuf> {
    let src = src.trim();
    if src.is_empty() || src.contains("://") || src.starts_with("data:") || src.starts_with('/') {
        return None;
    }
    let decoded = percent_decode(src.split(['#', '?']).next().unwrap_or(src));
    let path = fs::canonicalize(base.join(decoded)).ok()?;
    path.starts_with(root).then_some(path)
}

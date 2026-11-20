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
    commands::folders::{self as folder_commands, FolderUpsertInput},
    domain::{
        codec::base64_encode,
        dates, enex, html,
        markdown::{self, ImageFile},
        validation::{self, MAX_FOLDER_DEPTH},
    },
    error::{AppError, AppResult},
    repo::{
        self, attachments as attachment_repo, export as repo_export,
        model::{Folder, NewNote, Note},
        notes::{self, Scope, Sort},
        tags,
    },
    services::{
        notes::derive,
        tags::{self as tag_commands, TagUpsertInput},
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

mod catalog;
mod export;
mod import;
mod paths;
mod summary;
#[cfg(test)]
mod tests;

pub use export::{export_json, export_notes};
pub use import::{import_enex_files, import_text_files};
pub use summary::export_summary;

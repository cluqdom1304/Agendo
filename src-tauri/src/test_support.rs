//! Helpers shared by the tests of several modules.

use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU32, Ordering},
};

static NEXT_DIR: AtomicU32 = AtomicU32::new(0);

/// A fresh directory under the system temp dir, removed on drop.
pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new() -> Self {
        let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        Self(std::env::temp_dir().join(format!("agendo-test-{}-{id}", std::process::id())))
    }

    /// `<dir>/nested/agenda.sqlite`; neither the file nor `nested` exist yet.
    pub fn database(&self) -> PathBuf {
        self.0.join("nested").join("agenda.sqlite")
    }
}

impl Default for TempDir {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

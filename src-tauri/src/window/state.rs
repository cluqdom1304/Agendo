use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

pub const STATE_FILE: &str = "window-state.json";
pub const MIN_WIDTH: u32 = 720;
pub const MIN_HEIGHT: u32 = 480;

/// Last known geometry of the main window, in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

/// A saved size is only usable when it respects the window minimum.
pub fn is_usable(state: &WindowState) -> bool {
    state.width >= MIN_WIDTH && state.height >= MIN_HEIGHT
}

/// Whether the window's top-left area still falls inside one of the monitors
/// (`x, y, width, height`), so a screen that was unplugged never strands the window.
pub fn is_visible_on(state: &WindowState, monitors: &[(i32, i32, u32, u32)]) -> bool {
    const GRAB: i64 = 100;
    monitors.iter().any(|&(mx, my, mw, mh)| {
        let (left, top) = (i64::from(state.x), i64::from(state.y));
        let (right, bottom) = (left + i64::from(state.width), top + i64::from(state.height));
        let (mright, mbottom) = (i64::from(mx) + i64::from(mw), i64::from(my) + i64::from(mh));
        right - GRAB > i64::from(mx)
            && left + GRAB < mright
            && top >= i64::from(my) - GRAB
            && top + GRAB < mbottom
            && bottom > i64::from(my)
    })
}

/// Missing or corrupt files simply mean "no saved state".
pub fn load(path: &Path) -> Option<WindowState> {
    let text = fs::read_to_string(path).ok()?;
    let state: WindowState = serde_json::from_str(&text).ok()?;
    is_usable(&state).then_some(state)
}

/// Writes through a temporary file so an interrupted write never leaves a broken state.
pub fn save(path: &Path, state: &WindowState) -> std::io::Result<()> {
    let text = serde_json::to_string(state).map_err(std::io::Error::other)?;
    let temp = path.with_extension("json.part");
    fs::write(&temp, text)?;
    fs::rename(&temp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> WindowState {
        WindowState {
            x: 100,
            y: 80,
            width: 1280,
            height: 800,
            maximized: false,
        }
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = crate::test_support::TempDir::new();
        fs::create_dir_all(dir.path()).expect("dir");
        let path = dir.path().join(STATE_FILE);
        save(&path, &sample()).expect("save");
        assert_eq!(load(&path), Some(sample()));
    }

    #[test]
    fn missing_or_corrupt_file_gives_none() {
        let dir = crate::test_support::TempDir::new();
        fs::create_dir_all(dir.path()).expect("dir");
        let path = dir.path().join(STATE_FILE);
        assert_eq!(load(&path), None);
        fs::write(&path, "not json").expect("write");
        assert_eq!(load(&path), None);
    }

    #[test]
    fn size_below_the_minimum_is_discarded() {
        let dir = crate::test_support::TempDir::new();
        fs::create_dir_all(dir.path()).expect("dir");
        let path = dir.path().join(STATE_FILE);
        save(
            &path,
            &WindowState {
                width: 100,
                ..sample()
            },
        )
        .expect("save");
        assert_eq!(load(&path), None);
    }

    #[test]
    fn position_on_a_monitor_is_visible() {
        assert!(is_visible_on(&sample(), &[(0, 0, 1920, 1080)]));
    }

    #[test]
    fn position_on_a_missing_monitor_is_not_visible() {
        let state = WindowState {
            x: 2500,
            ..sample()
        };
        assert!(!is_visible_on(&state, &[(0, 0, 1920, 1080)]));
        assert!(is_visible_on(
            &state,
            &[(0, 0, 1920, 1080), (1920, 0, 1920, 1080)]
        ));
    }

    #[test]
    fn minimized_sentinel_position_is_not_visible() {
        let state = WindowState {
            x: -32000,
            y: -32000,
            ..sample()
        };
        assert!(!is_visible_on(&state, &[(0, 0, 1920, 1080)]));
    }
}

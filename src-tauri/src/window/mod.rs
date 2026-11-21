pub mod effects;
pub mod state;

use std::path::Path;

use tauri::{PhysicalPosition, PhysicalSize, Runtime, WebviewWindow};

use state::{STATE_FILE, WindowState};

/// Puts the window back where it was when the app was last closed. Any failure is ignored:
/// the window then keeps the size from `tauri.conf.json`.
pub fn restore<R: Runtime>(window: &WebviewWindow<R>, dir: &Path) {
    let Some(saved) = state::load(&dir.join(STATE_FILE)) else {
        return;
    };
    let _ = window.set_size(PhysicalSize::new(saved.width, saved.height));
    let monitors: Vec<_> = window
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m| {
            (
                m.position().x,
                m.position().y,
                m.size().width,
                m.size().height,
            )
        })
        .collect();
    if state::is_visible_on(&saved, &monitors) {
        let _ = window.set_position(PhysicalPosition::new(saved.x, saved.y));
    } else {
        let _ = window.center();
    }
    if saved.maximized {
        let _ = window.maximize();
    }
}

/// Stores the window geometry. While maximized or minimized the previous normal geometry is
/// kept, so un-maximizing after a restart returns to the size the user had chosen.
pub fn remember<R: Runtime>(window: &WebviewWindow<R>, dir: &Path) {
    if window.is_minimized().unwrap_or(false) {
        return;
    }
    let path = dir.join(STATE_FILE);
    let maximized = window.is_maximized().unwrap_or(false);
    let current = match (window.outer_position(), window.inner_size()) {
        (Ok(position), Ok(size)) => Some(WindowState {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
            maximized,
        }),
        _ => None,
    };
    let to_store = if maximized {
        match state::load(&path) {
            Some(previous) => Some(WindowState {
                maximized: true,
                ..previous
            }),
            None => current,
        }
    } else {
        current
    };
    if let Some(value) = to_store
        && let Err(error) = state::save(&path, &value)
    {
        crate::applog::info(
            "window",
            &format!("could not save the window state: {error}"),
        );
    }
}

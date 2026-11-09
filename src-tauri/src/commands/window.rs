use crate::window::effects::{self, EffectApplied, EffectMode, NativeBackend};

/// `theme` is the app's own choice (`light`, `dark` or `system`). The window must follow it, not
/// only the system, or Mica stays dark under a light interface.
#[tauri::command]
pub fn window_set_effect(
    window: tauri::WebviewWindow,
    mode: EffectMode,
    theme: Option<String>,
) -> Result<EffectApplied, String> {
    let (native_theme, dark) = match theme.as_deref() {
        None | Some("system") => (None, None),
        Some("light") => (Some(tauri::Theme::Light), Some(false)),
        Some("dark") => (Some(tauri::Theme::Dark), Some(true)),
        Some(_) => return Err("VALIDATION: el tema debe ser light, dark o system".to_string()),
    };
    window
        .set_theme(native_theme)
        .map_err(|error| format!("INTERNAL: {error}"))?;
    effects::apply_with_fallback(&NativeBackend::new(&window, dark), mode)
}

use crate::window::effects::{self, EffectApplied, EffectMode, NativeBackend};

#[tauri::command]
pub fn window_set_effect(
    window: tauri::WebviewWindow,
    mode: EffectMode,
) -> Result<EffectApplied, String> {
    effects::apply_with_fallback(&NativeBackend::new(&window), mode)
}

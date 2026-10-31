mod commands;
mod window;

// Si Tauri no puede arrancar no hay estado recuperable: abortar con mensaje es correcto.
#[allow(clippy::expect_used)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::window::window_set_effect
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod commands;
mod startup;
mod window;

use tauri::webview::PageLoadEvent;

// Si Tauri no puede arrancar no hay estado recuperable: abortar con mensaje es correcto.
#[allow(clippy::expect_used)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    startup::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            startup::mark("setup_done");
            Ok(())
        })
        .on_page_load(|_webview, payload| match payload.event() {
            PageLoadEvent::Started => startup::mark("page_load_started"),
            PageLoadEvent::Finished => startup::mark("page_load_finished"),
        })
        .invoke_handler(tauri::generate_handler![
            commands::startup::startup_mark,
            commands::window::window_set_effect
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

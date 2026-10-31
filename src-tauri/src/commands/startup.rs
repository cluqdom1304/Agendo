use crate::startup;

#[tauri::command]
pub fn startup_mark(label: String) -> Result<(), String> {
    startup::validate_label(&label)?;
    startup::mark(&label);
    Ok(())
}

// Providers and storage are added incrementally as the project grows — see
// docs/architecture.md and TODO.md for the current step.
pub mod commands;
pub mod domain;
pub mod secrets;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::secrets::save_api_key,
            commands::secrets::delete_api_key,
            commands::secrets::has_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

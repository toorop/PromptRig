// See docs/architecture.md and TODO.md for the current step. `storage`, `providers`, and
// `pricing` aren't wired into the Tauri app yet: no command needs them until Step 5, so they're
// only exercised by their own tests for now.
pub mod commands;
pub mod domain;
pub mod pricing;
pub mod providers;
pub mod secrets;
pub mod storage;

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

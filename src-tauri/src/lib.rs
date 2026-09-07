// Commands, providers, and storage are added incrementally as the project grows — see
// docs/architecture.md and TODO.md for the current step.
pub mod domain;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

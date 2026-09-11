// See docs/architecture.md and TODO.md for the current step.
pub mod commands;
pub mod domain;
pub mod pricing;
pub mod providers;
pub mod secrets;
pub mod storage;

use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

use pricing::ModelsDevCache;
use providers::ProviderRegistry;
use storage::Database;

/// Registers every command with `tauri-specta` once, so the same list drives both the real
/// Tauri `invoke_handler` and the generated TypeScript bindings — there's no separate list to
/// keep in sync by hand.
fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::secrets::save_api_key,
        commands::secrets::delete_api_key,
        commands::secrets::has_api_key,
        commands::providers::list_providers,
        commands::providers::test_provider_connection,
        commands::providers::list_models,
        commands::runs::run_generation,
        commands::experiments::run_experiment,
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder();

    // Regenerated on every debug build so the frontend's types never drift from the backend's;
    // skipped in release builds since there's no `src/` to write into once the app is bundled.
    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/lib/bindings.ts",
        )
        .expect("failed to export TypeScript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Remembers window size/position/maximized-state across restarts (see docs/start.md's
        // deferred-ideas equivalent and TODO.md) — no app-side wiring needed beyond registering
        // it: it hooks window creation/close itself. Matters on traditional (non-tiling) window
        // managers; a tiling WM (this dev machine's own setup) manages sizing itself regardless.
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let db = Database::open(&app_data_dir.join("promptrig.sqlite"))?;

            app.manage(db);
            app.manage(ProviderRegistry::new());
            app.manage(ModelsDevCache::new(app_data_dir));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

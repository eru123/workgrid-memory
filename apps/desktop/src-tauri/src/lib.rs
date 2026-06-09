use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use workgrid_engine::indexer::metadata::MetadataStore;

mod commands;

pub use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize the metadata store in the app data directory
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");

            let db_path = app_data_dir.join("app.sqlite");
            let store = MetadataStore::open(&db_path)
                .expect("failed to open app database");

            app.manage(AppState {
                store: Mutex::new(store),
            });

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_app_version,
            commands::add_workspace,
            commands::list_workspaces,
            commands::get_workspace,
            commands::remove_workspace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running WorkGrid Memory");
}

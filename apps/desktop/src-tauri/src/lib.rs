use std::sync::{Arc, Mutex};
use tauri::Manager;
use workgrid_engine::Engine;
use workgrid_mcp_server::McpServer;

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
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");

            let engine = Engine::open(&app_data_dir).expect("failed to open engine data stores");

            let engine = Arc::new(Mutex::new(engine));
            let mcp_server = McpServer::new(Arc::clone(&engine), 9876);

            app.manage(AppState {
                engine,
                mcp_server: Mutex::new(mcp_server),
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
            commands::scan_workspace,
            commands::index_workspace,
            commands::reindex_workspace,
            commands::cancel_indexing,
            commands::resume_indexing,
            commands::search_workspace,
            commands::get_workspace_stats,
            commands::start_mcp_server,
            commands::stop_mcp_server,
            commands::get_mcp_status,
            commands::get_mcp_token,
            commands::rotate_mcp_token,
            commands::create_profile,
            commands::list_profiles,
            commands::get_profile,
            commands::update_profile,
            commands::delete_profile,
            commands::archive_profile,
            commands::set_profile_mcp,
            commands::search_profiles,
            commands::add_profile_attribute,
            commands::get_profile_attributes,
            commands::add_profile_instruction,
            commands::find_matching_instructions,
            commands::add_profile_relationship,
            commands::get_profile_relationships,
            commands::link_profile_workspace,
            commands::get_profile_workspace_links,
            commands::get_profiles_for_workspace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running WorkGrid Memory");
}

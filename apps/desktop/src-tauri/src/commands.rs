use tauri::{command, State};
use std::sync::Mutex;
use workgrid_engine::indexer::metadata::MetadataStore;
use workgrid_shared::types::Workspace;

/// Application state wrapper for the metadata store.
pub struct AppState {
    pub store: Mutex<MetadataStore>,
}

#[command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! WorkGrid Memory is ready.", name)
}

#[command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[command]
pub fn add_workspace(state: State<AppState>, name: String, path: String) -> Result<Workspace, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;

    // Try to detect git remote
    let git_remote = MetadataStore::detect_git_remote(&path);

    store
        .add_workspace(&name, &path, git_remote.as_deref())
        .map_err(|e| e.to_string())
}

#[command]
pub fn list_workspaces(state: State<AppState>) -> Result<Vec<Workspace>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store.list_workspaces().map_err(|e| e.to_string())
}

#[command]
pub fn get_workspace(state: State<AppState>, id: String) -> Result<Option<Workspace>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store.get_workspace(&id).map_err(|e| e.to_string())
}

#[command]
pub fn remove_workspace(state: State<AppState>, id: String) -> Result<(), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store.remove_workspace(&id).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct ScanResult {
    pub files: Vec<FileEntry>,
    pub total_scanned: u64,
    pub total_ignored: u64,
    pub total_errors: u64,
}

#[derive(serde::Serialize)]
pub struct FileEntry {
    pub relative_path: String,
    pub size: u64,
    pub language: Option<String>,
    pub hash: String,
}

#[command]
pub fn scan_workspace(path: String) -> Result<ScanResult, String> {
    use std::path::Path;
    use workgrid_engine::scanner::file_scanner;

    let result = file_scanner::scan_workspace(Path::new(&path), &[])
        .map_err(|e| e.to_string())?;

    Ok(ScanResult {
        files: result.files.into_iter().map(|f| FileEntry {
            relative_path: f.relative_path,
            size: f.size,
            language: f.language,
            hash: f.hash,
        }).collect(),
        total_scanned: result.total_scanned,
        total_ignored: result.total_ignored,
        total_errors: result.total_errors,
    })
}

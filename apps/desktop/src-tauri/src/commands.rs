use std::sync::{Arc, Mutex};
use tauri::{command, State};
use std::path::Path;
use workgrid_engine::Engine;
use workgrid_mcp_server::McpServer;
use workgrid_shared::types::Workspace;

/// Application state wrapping the engine and MCP server.
pub struct AppState {
    pub engine: Arc<Mutex<Engine>>,
    pub mcp_server: Mutex<McpServer>,
}

#[command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! WorkGrid Memory is ready.", name)
}

#[command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ── Workspace CRUD (delegated to Engine) ──

#[command]
pub fn add_workspace(state: State<AppState>, name: String, path: String) -> Result<Workspace, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.add_workspace(&name, &path).map_err(|e| e.to_string())
}

#[command]
pub fn list_workspaces(state: State<AppState>) -> Result<Vec<Workspace>, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.list_workspaces().map_err(|e| e.to_string())
}

#[command]
pub fn get_workspace(state: State<AppState>, id: String) -> Result<Option<Workspace>, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.get_workspace(&id).map_err(|e| e.to_string())
}

#[command]
pub fn remove_workspace(state: State<AppState>, id: String) -> Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.remove_workspace(&id).map_err(|e| e.to_string())
}

// ── Scanning and Indexing ──

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
    let result = workgrid_engine::scanner::file_scanner::scan_workspace(Path::new(&path), &[])
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

#[derive(serde::Serialize)]
pub struct IndexResult {
    pub file_count: u64,
    pub chunk_count: u64,
    pub symbol_count: u64,
}

#[command]
pub async fn index_workspace(state: State<'_, AppState>, workspace_id: String) -> Result<IndexResult, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let stats = engine.index_workspace(&workspace_id).await.map_err(|e| e.to_string())?;
    Ok(IndexResult {
        file_count: stats.file_count,
        chunk_count: stats.chunk_count,
        symbol_count: stats.symbol_count,
    })
}

#[derive(serde::Serialize)]
pub struct SearchResultItem {
    pub chunk_id: String,
    pub file_path: String,
    pub content: String,
    pub start_line: u32,
    pub end_line: u32,
    pub score: f64,
    pub match_reason: String,
}

#[command]
pub fn search_workspace(
    state: State<AppState>,
    workspace_id: String,
    query: String,
    top_k: Option<usize>,
) -> Result<Vec<SearchResultItem>, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let results = engine
        .search_workspace(&workspace_id, &query, top_k.unwrap_or(10))
        .map_err(|e| e.to_string())?;
    Ok(results
        .into_iter()
        .map(|r| SearchResultItem {
            chunk_id: r.chunk_id,
            file_path: r.file_path,
            content: r.content,
            start_line: r.start_line,
            end_line: r.end_line,
            score: r.score,
            match_reason: r.match_reason,
        })
        .collect())
}

#[command]
pub fn get_workspace_stats(state: State<AppState>, workspace_id: String) -> Result<IndexResult, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let stats = engine.get_workspace_stats(&workspace_id).map_err(|e| e.to_string())?;
    Ok(IndexResult {
        file_count: stats.file_count,
        chunk_count: stats.chunk_count,
        symbol_count: stats.symbol_count,
    })
}

// ── MCP Server Control ──

#[derive(serde::Serialize)]
pub struct McpStatus {
    pub running: bool,
    pub port: u16,
    pub tool_count: usize,
}

#[command]
pub fn start_mcp_server(state: State<AppState>) -> Result<McpStatus, String> {
    let mut server = state.mcp_server.lock().map_err(|e| e.to_string())?;
    server.start().map_err(|e| e.to_string())?;
    Ok(McpStatus {
        running: server.is_running(),
        port: server.port(),
        tool_count: 11,
    })
}

#[command]
pub fn stop_mcp_server(state: State<AppState>) -> Result<McpStatus, String> {
    let mut server = state.mcp_server.lock().map_err(|e| e.to_string())?;
    server.stop();
    Ok(McpStatus {
        running: false,
        port: server.port(),
        tool_count: 11,
    })
}

#[command]
pub fn get_mcp_status(state: State<AppState>) -> Result<McpStatus, String> {
    let server = state.mcp_server.lock().map_err(|e| e.to_string())?;
    Ok(McpStatus {
        running: server.is_running(),
        port: server.port(),
        tool_count: 11,
    })
}
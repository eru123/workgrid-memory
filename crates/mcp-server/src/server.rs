//! MCP server implementation — dispatches tool calls to the engine.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};
use workgrid_engine::Engine;

use crate::tools::{self, McpRequest, ToolCallParams};
use crate::transport::{spawn_http_server, HttpServerHandle};

/// MCP server managing workspace and profile context exposure.
pub struct McpServer {
    engine: Arc<Mutex<Engine>>,
    port: u16,
    server_handle: Option<HttpServerHandle>,
}

impl McpServer {
    /// Create a new MCP server backed by the given engine.
    pub fn new(engine: Arc<Mutex<Engine>>, port: u16) -> Self {
        McpServer {
            engine,
            port,
            server_handle: None,
        }
    }

    /// Port the server is configured for.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Start the MCP HTTP server on 127.0.0.1:{port}.
    /// Returns Ok if the server started, or an error message if it failed.
    pub fn start(&mut self) -> Result<(), String> {
        if self.is_running() {
            warn!("MCP server already running");
            return Ok(());
        }

        let addr = format!("127.0.0.1:{}", self.port);
        let engine = Arc::clone(&self.engine);

        let handler = Arc::new(move |request: McpRequest| -> serde_json::Value {
            let server = McpServer {
                engine: Arc::clone(&engine),
                port: 0,
                server_handle: None,
            };
            server.handle_request(request)
        });

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let handle = spawn_http_server(&addr, handler, running_clone)?;

        self.server_handle = Some(handle);
        info!("MCP server started on {}", addr);
        Ok(())
    }

    /// Stop the MCP HTTP server if running.
    pub fn stop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.shutdown();
            info!("MCP server stopped");
        }
    }

    /// Check if the server is currently running.
    pub fn is_running(&self) -> bool {
        self.server_handle
            .as_ref()
            .map(|h| h.is_running())
            .unwrap_or(false)
    }

    /// Handle an MCP JSON-RPC request and return a response.
    pub fn handle_request(&self, request: McpRequest) -> serde_json::Value {
        match request {
            McpRequest::Initialize(_params) => {
                serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "WorkGrid Memory",
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    "capabilities": {
                        "tools": {}
                    }
                })
            }
            McpRequest::ToolsList => {
                serde_json::json!({
                    "tools": tools::get_tools()
                })
            }
            McpRequest::ToolsCall(call) => self.dispatch_tool(call),
        }
    }

    /// Dispatch a tool call to the appropriate engine method.
    fn dispatch_tool(&self, call: ToolCallParams) -> serde_json::Value {
        let result = match call.name.as_str() {
            "search_workspace" => self.tool_search_workspace(&call.arguments),
            "get_file_context" => self.tool_get_file_context(&call.arguments),
            "explain_symbol" => self.tool_explain_symbol(&call.arguments),
            "find_references" => self.tool_find_references(&call.arguments),
            "get_related_files" => self.tool_get_related_files(&call.arguments),
            "get_workspace_map" => self.tool_get_workspace_map(&call.arguments),
            "verify_claim" => self.tool_verify_claim(&call.arguments),
            "search_profiles" => self.tool_search_profiles(&call.arguments),
            "get_profile_context" => self.tool_get_profile_context(&call.arguments),
            "get_relevant_profiles" => self.tool_get_relevant_profiles(&call.arguments),
            "build_context_pack" => self.tool_build_context_pack(&call.arguments),
            _ => Err(format!("Unknown tool: {}", call.name)),
        };

        match result {
            Ok(value) => serde_json::json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{:?}", value))
                }]
            }),
            Err(e) => serde_json::json!({
                "isError": true,
                "content": [{
                    "type": "text",
                    "text": e
                }]
            }),
        }
    }

    // ── Tool implementations ──

    fn tool_search_workspace(&self, args: &serde_json::Value) -> Result<serde_json::Value, String> {
        let workspace_id = get_str(args, "workspace_id")?;
        let query = get_str(args, "query")?;
        let top_k = args.get("top_k").and_then(|v| v.as_u64()).unwrap_or(8) as usize;

        let engine = lock(&self.engine)?;
        let results = engine
            .search_workspace(&workspace_id, &query, top_k)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_value(results).map_err(|e| e.to_string())?)
    }

    fn tool_get_file_context(&self, args: &serde_json::Value) -> Result<serde_json::Value, String> {
        let workspace_id = get_str(args, "workspace_id")?;
        let file_path = get_str(args, "file_path")?;

        let engine = lock(&self.engine)?;
        let related = engine
            .get_related_files(&workspace_id, &file_path)
            .map_err(|e| e.to_string())?;
        let edges = engine
            .get_edges_for_file(&workspace_id, &file_path)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "file_path": file_path,
            "workspace_id": workspace_id,
            "related_files": related,
            "edges": edges
        }))
    }

    fn tool_explain_symbol(&self, args: &serde_json::Value) -> Result<serde_json::Value, String> {
        let workspace_id = get_str(args, "workspace_id")?;
        let symbol_name = get_str(args, "symbol_name")?;

        let engine = lock(&self.engine)?;
        let results = engine
            .search_workspace(&workspace_id, &symbol_name, 5)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_value(results).map_err(|e| e.to_string())?)
    }

    fn tool_find_references(&self, args: &serde_json::Value) -> Result<serde_json::Value, String> {
        let workspace_id = get_str(args, "workspace_id")?;
        let symbol_name = get_str(args, "symbol_name")?;

        let engine = lock(&self.engine)?;
        let results = engine
            .search_workspace(&workspace_id, &symbol_name, 10)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_value(results).map_err(|e| e.to_string())?)
    }

    fn tool_get_related_files(
        &self,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let workspace_id = get_str(args, "workspace_id")?;
        let file_path = get_str(args, "file_path")?;

        let engine = lock(&self.engine)?;
        let related = engine
            .get_related_files(&workspace_id, &file_path)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_value(related).map_err(|e| e.to_string())?)
    }

    fn tool_get_workspace_map(
        &self,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let workspace_id = get_str(args, "workspace_id")?;

        let engine = lock(&self.engine)?;
        let stats = engine
            .get_workspace_stats(&workspace_id)
            .map_err(|e| e.to_string())?;

        let ws = engine
            .get_workspace(&workspace_id)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "workspace": ws,
            "stats": stats
        }))
    }

    fn tool_verify_claim(&self, args: &serde_json::Value) -> Result<serde_json::Value, String> {
        let workspace_id = get_str(args, "workspace_id")?;
        let claim = get_str(args, "claim")?;

        let engine = lock(&self.engine)?;
        let results = engine
            .search_workspace(&workspace_id, &claim, 5)
            .map_err(|e| e.to_string())?;

        let supported = !results.is_empty();
        Ok(serde_json::json!({
            "claim": claim,
            "supported": supported,
            "evidence_count": results.len(),
            "top_results": results.iter().take(3).map(|r| serde_json::json!({
                "file": r.file_path,
                "lines": format!("{}:{}", r.start_line, r.end_line),
                "score": r.score
            })).collect::<Vec<_>>()
        }))
    }

    fn tool_search_profiles(&self, args: &serde_json::Value) -> Result<serde_json::Value, String> {
        let query = get_str(args, "query")?;

        let engine = lock(&self.engine)?;
        let profiles = engine
            .profile_store()
            .search_profiles_fts(&query)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_value(profiles).map_err(|e| e.to_string())?)
    }

    fn tool_get_profile_context(
        &self,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let profile_id = get_str(args, "profile_id")?;

        let engine = lock(&self.engine)?;
        let profile = engine
            .profile_store()
            .get_profile(&profile_id)
            .map_err(|e| e.to_string())?;
        let attrs = engine
            .profile_store()
            .get_attributes(&profile_id)
            .map_err(|e| e.to_string())?;
        let rels = engine
            .profile_store()
            .get_relationships(&profile_id)
            .map_err(|e| e.to_string())?;
        let links = engine
            .profile_store()
            .get_workspace_links(&profile_id)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "profile": profile,
            "attributes": attrs,
            "relationships": rels,
            "workspace_links": links
        }))
    }

    fn tool_get_relevant_profiles(
        &self,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let _workspace_id = args.get("workspace_id").and_then(|v| v.as_str());
        let task = get_str(args, "task")?;

        let engine = lock(&self.engine)?;
        let matches = engine
            .profile_store()
            .find_matching_instructions(&task)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_value(matches).map_err(|e| e.to_string())?)
    }

    fn tool_build_context_pack(
        &self,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let workspace_id = get_str(args, "workspace_id")?;
        let task = get_str(args, "task")?;

        let engine = lock(&self.engine)?;

        // Search workspace for relevant context
        let workspace_results = engine
            .search_workspace(&workspace_id, &task, 10)
            .map_err(|e| e.to_string())?;

        // Find matching profile instructions
        let profile_matches = engine
            .profile_store()
            .find_matching_instructions(&task)
            .map_err(|e| e.to_string())?;

        let stats = engine
            .get_workspace_stats(&workspace_id)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "task": task,
            "workspace_id": workspace_id,
            "workspace_stats": stats,
            "workspace_evidence": workspace_results,
            "profile_context": profile_matches
        }))
    }
}

// ── Helpers ──

fn get_str(args: &serde_json::Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Missing required parameter: {}", key))
}

fn lock(engine: &Arc<Mutex<Engine>>) -> Result<std::sync::MutexGuard<'_, Engine>, String> {
    engine
        .lock()
        .map_err(|e| format!("Engine lock error: {}", e))
}

//! MCP transport implementations: Streamable HTTP (primary) and stdio (planned).

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tracing::{error, info};

use crate::tools::McpRequest;

/// Runs an HTTP MCP server on the given address in a background thread.
/// Returns a handle that can be used to shut down the server.
pub fn spawn_http_server(
    addr: &str,
    handler: Arc<dyn Fn(McpRequest) -> serde_json::Value + Send + Sync>,
    running: Arc<AtomicBool>,
    auth_token: &str,
) -> Result<HttpServerHandle, String> {
    let listener = TcpListener::bind(addr)
        .map_err(|e| format!("Failed to bind MCP server to {}: {}", addr, e))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("Failed to set non-blocking: {}", e))?;

    let addr_owned = addr.to_string();
    let token_owned = auth_token.to_string();
    info!("MCP HTTP server listening on {}", addr_owned);

    let running_for_thread = Arc::clone(&running);

    let handle = thread::spawn(move || {
        // Accept loop with shutdown check every 250ms
        while running_for_thread.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, peer)) => {
                    // Read the HTTP request
                    let mut buf = [0u8; 65536];
                    let n = match stream.read(&mut buf) {
                        Ok(n) if n > 0 => n,
                        _ => continue,
                    };

                    let request_str = String::from_utf8_lossy(&buf[..n]);
                    let response = handle_http_request(&request_str, &handler, &token_owned);

                    // Send response (ignore errors — client may have disconnected)
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();

                    info!("MCP request handled from {}", peer);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connection ready — sleep briefly then check shutdown
                    thread::sleep(std::time::Duration::from_millis(250));
                }
                Err(e) => {
                    error!("MCP accept error: {}", e);
                    thread::sleep(std::time::Duration::from_millis(250));
                }
            }
        }
        info!("MCP HTTP server shut down");
    });

    Ok(HttpServerHandle { handle, running })
}

/// Handle an HTTP request by extracting the JSON-RPC body and dispatching it.
fn handle_http_request(
    raw: &str,
    handler: &Arc<dyn Fn(McpRequest) -> serde_json::Value + Send + Sync>,
    auth_token: &str,
) -> String {
    // Check Authorization header
    let auth_header = raw
        .lines()
        .find(|l| l.to_lowercase().starts_with("authorization:"))
        .unwrap_or("");
    let provided_token = auth_header
        .trim_start_matches(|c: char| c != 'B' && c != 'b')
        .trim_start_matches("Bearer ")
        .trim_start_matches("bearer ")
        .trim();
    if provided_token != auth_token {
        return http_response(401, &serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": -32001, "message": "Unauthorized: invalid or missing auth token" },
            "id": null
        }).to_string());
    }

    // Extract JSON body from the HTTP request
    let body = extract_json_body(raw);

    // Parse JSON-RPC request
    let request: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            return http_response(
                400,
                &serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": { "code": -32700, "message": format!("Parse error: {}", e) },
                    "id": null
                })
                .to_string(),
            );
        }
    };

    // Extract method and params
    let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = request.get("params");
    let id = request
        .get("id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    // Route to handler
    let mcp_request =
        match method {
            "initialize" => {
                let protocol_version = params
                    .and_then(|p| p.get("protocolVersion"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("2024-11-05")
                    .to_string();
                let client_info = params
                    .and_then(|p| p.get("clientInfo"))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                McpRequest::Initialize(crate::tools::InitializeParams {
                    protocol_version,
                    client_info,
                })
            }
            "tools/list" => McpRequest::ToolsList,
            "tools/call" => {
                let name = params
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let arguments = params
                    .and_then(|p| p.get("arguments"))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                McpRequest::ToolsCall(crate::tools::ToolCallParams { name, arguments })
            }
            _ => {
                return http_response(200, &serde_json::json!({
                "jsonrpc": "2.0",
                "error": { "code": -32601, "message": format!("Method not found: {}", method) },
                "id": id
            }).to_string());
            }
        };

    let result = handler(mcp_request);

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    });

    http_response(200, &response.to_string())
}

/// Extract the JSON body from a raw HTTP request string.
fn extract_json_body(raw: &str) -> String {
    // Find the double-CRLF that separates headers from body
    if let Some(pos) = raw.find("\r\n\r\n") {
        raw[pos + 4..].trim().to_string()
    } else if let Some(pos) = raw.find("\n\n") {
        raw[pos + 2..].trim().to_string()
    } else {
        raw.trim().to_string()
    }
}

/// Build a minimal HTTP/1.1 response with CORS headers for local development.
fn http_response(status: u16, body: &str) -> String {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Error",
    };
    format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: application/json\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        status,
        status_text,
        body.len(),
        body
    )
}

/// Handle for a running HTTP server thread.
pub struct HttpServerHandle {
    handle: JoinHandle<()>,
    running: Arc<AtomicBool>,
}

impl HttpServerHandle {
    /// Shut down the server and wait for the thread to finish.
    pub fn shutdown(self) {
        self.running.store(false, Ordering::Relaxed);
        let _ = self.handle.join();
    }

    /// Check if the server is still running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

/// Transport types supported by the MCP server.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Transport {
    Stdio,
    Http { port: u16 },
}

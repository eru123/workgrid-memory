//! WorkGrid Memory MCP server — exposes workspace and profile context
//! to AI agents via Model Context Protocol.

pub mod server;
pub mod tools;
pub mod transport;



/// MCP server managing workspace and profile context exposure.
pub struct McpServer {
    pub transport: transport::Transport,
    pub tools: Vec<tools::Tool>,
}

impl McpServer {
    pub fn new(transport: transport::Transport) -> Self {
        McpServer {
            transport,
            tools: tools::get_tools(),
        }
    }

    /// Handle an MCP JSON-RPC request and return a response.
    pub async fn handle_request(&self, request: tools::McpRequest) -> serde_json::Value {
        match request {
            tools::McpRequest::Initialize(_params) => {
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
            tools::McpRequest::ToolsList => {
                serde_json::json!({
                    "tools": self.tools
                })
            }
            tools::McpRequest::ToolsCall(call) => {
                serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Tool '{}' called with arguments: {}", call.name, call.arguments)
                    }]
                })
            }
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new(transport::Transport::Stdio)
    }
}

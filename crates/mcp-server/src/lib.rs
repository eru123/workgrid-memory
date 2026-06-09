//! WorkGrid Memory MCP server — exposes workspace and profile context
//! to AI agents via Model Context Protocol.

pub mod server;
pub mod tools;
pub mod transport;
pub struct McpServer {
    // Will be populated in Phase 10
}

impl McpServer {
    pub fn new() -> Self {
        McpServer {}
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

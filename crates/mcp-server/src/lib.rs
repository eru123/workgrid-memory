//! WorkGrid Memory MCP server — exposes workspace and profile context
//! to AI agents via Model Context Protocol.

pub mod server;
pub mod tools;
pub mod transport;

pub use server::McpServer;

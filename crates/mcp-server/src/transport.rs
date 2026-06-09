/// MCP transport types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Transport {
    /// stdio transport (stdin/stdout JSON-RPC)
    Stdio,
    /// Streamable HTTP transport (bound to 127.0.0.1)
    Http { port: u16 },
}

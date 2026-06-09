use thiserror::Error;

#[cfg(feature = "rusqlite")]
impl From<rusqlite::Error> for WorkGridError {
    fn from(e: rusqlite::Error) -> Self {
        WorkGridError::Database(e.to_string())
    }
}

#[derive(Error, Debug)]
pub enum WorkGridError {
    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),

    #[error("Workspace path does not exist: {0}")]
    WorkspacePathNotFound(String),

    #[error("Duplicate workspace: {0}")]
    DuplicateWorkspace(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Database internal error: {0}")]
    DatabaseInternal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Path traversal blocked: {0}")]
    PathTraversal(String),

    #[error("Indexing error: {0}")]
    Indexing(String),

    #[error("Retrieval error: {0}")]
    Retrieval(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("MCP server error: {0}")]
    McpServer(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("{0}")]
    Generic(String),
}

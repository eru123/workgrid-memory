use serde::{Deserialize, Serialize};

/// Represents a workspace registered in the app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub git_remote: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_indexed_at: Option<String>,
    pub status: WorkspaceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceStatus {
    New,
    Indexing,
    Ready,
    Degraded,
    Stale,
    Error,
    Paused,
}

/// A file record in a workspace index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFile {
    pub id: String,
    pub workspace_id: String,
    pub path: String,
    pub language: Option<String>,
    pub hash: String,
    pub size: i64,
    pub mtime: Option<String>,
    pub indexed_at: Option<String>,
    pub ignored: bool,
    pub deleted: bool,
}

/// A code chunk extracted from a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub workspace_id: String,
    pub file_id: String,
    pub symbol_id: Option<String>,
    pub chunk_type: ChunkType,
    pub content: String,
    pub start_line: u32,
    pub end_line: u32,
    pub token_count: Option<u32>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChunkType {
    Symbol,
    Structural,
    Documentation,
    Fallback,
}

/// A code symbol extracted from source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub id: String,
    pub workspace_id: String,
    pub file_id: String,
    pub name: String,
    pub kind: SymbolKind,
    pub signature: Option<String>,
    pub doc: Option<String>,
    pub start_line: u32,
    pub end_line: u32,
    pub parent_symbol_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Class,
    Method,
    Interface,
    Type,
    Constant,
    Import,
    Export,
    Route,
    Controller,
    Model,
    Migration,
    Table,
    ConfigKey,
    EnvKey,
}

/// A graph edge connecting two entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub workspace_id: String,
    pub from_id: String,
    pub to_id: String,
    pub edge_type: EdgeType,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Imports,
    Exports,
    Calls,
    References,
    Defines,
    Extends,
    Implements,
    UsesEnv,
    QueriesTable,
    HandlesRoute,
    BelongsToFile,
    NearSymbol,
    TestedBy,
    Configures,
}

/// An indexing job record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexJob {
    pub id: String,
    pub workspace_id: String,
    pub job_type: String,
    pub status: String,
    pub total_items: u32,
    pub processed_items: u32,
    pub error: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

/// A global profile record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub profile_type: String,
    pub description: Option<String>,
    pub sensitivity: SensitivityLevel,
    pub mcp_exposure: McpExposure,
    pub source: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_reviewed_at: Option<String>,
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SensitivityLevel {
    Public,
    Internal,
    Private,
    Sensitive,
    Secret,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum McpExposure {
    Enabled,
    Disabled,
}

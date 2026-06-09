// Shared type schemas for WorkGrid Memory
// These mirror the Rust types in crates/shared/src/types.rs

export interface Workspace {
  id: string;
  name: string;
  rootPath: string;
  gitRemote?: string;
  createdAt: string;
  updatedAt: string;
  lastIndexedAt?: string;
  status: WorkspaceStatus;
}

export type WorkspaceStatus =
  | "new"
  | "indexing"
  | "ready"
  | "degraded"
  | "stale"
  | "error"
  | "paused";

export interface IndexedFile {
  id: string;
  workspaceId: string;
  path: string;
  language?: string;
  hash: string;
  size: number;
  mtime?: string;
  indexedAt?: string;
  ignored: boolean;
  deleted: boolean;
}

export interface Chunk {
  id: string;
  workspaceId: string;
  fileId: string;
  symbolId?: string;
  chunkType: ChunkType;
  content: string;
  startLine: number;
  endLine: number;
  tokenCount?: number;
  hash: string;
}

export type ChunkType = "symbol" | "structural" | "documentation" | "fallback";

export interface Symbol {
  id: string;
  workspaceId: string;
  fileId: string;
  name: string;
  kind: SymbolKind;
  signature?: string;
  doc?: string;
  startLine: number;
  endLine: number;
  parentSymbolId?: string;
}

export type SymbolKind =
  | "function"
  | "class"
  | "method"
  | "interface"
  | "type"
  | "constant"
  | "import"
  | "export"
  | "route"
  | "controller"
  | "model"
  | "migration"
  | "table"
  | "config_key"
  | "env_key";

export interface Edge {
  id: string;
  workspaceId: string;
  fromId: string;
  toId: string;
  edgeType: EdgeType;
  confidence: number;
}

export type EdgeType =
  | "imports"
  | "exports"
  | "calls"
  | "references"
  | "defines"
  | "extends"
  | "implements"
  | "uses_env"
  | "queries_table"
  | "handles_route"
  | "belongs_to_file"
  | "near_symbol"
  | "tested_by"
  | "configures";

export interface IndexJob {
  id: string;
  workspaceId: string;
  jobType: string;
  status: string;
  totalItems: number;
  processedItems: number;
  error?: string;
  createdAt: string;
  startedAt?: string;
  finishedAt?: string;
}

export interface Profile {
  id: string;
  name: string;
  profileType: string;
  description?: string;
  sensitivity: SensitivityLevel;
  mcpExposure: McpExposure;
  source?: string;
  createdAt: string;
  updatedAt: string;
  lastReviewedAt?: string;
  archived: boolean;
}

export type SensitivityLevel = "public" | "internal" | "private" | "sensitive" | "secret";
export type McpExposure = "enabled" | "disabled";

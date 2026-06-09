//! Core engine for WorkGrid Memory: file scanning, indexing, chunking,
//! symbol extraction, embedding queuing, and retrieval.

pub mod indexer;
pub mod retrieval;
pub mod scanner;

use std::path::{Path, PathBuf};
use tracing::{info, warn};

use indexer::embedding_queue::EmbeddingProvider;
use indexer::fts::WorkspaceMetadataStore;
use indexer::metadata::MetadataStore;
use indexer::profile::ProfileStore;
use retrieval::hybrid_search::{self, HybridSearchResult};
use scanner::chunker::{chunk_text, ChunkConfig};
use scanner::file_scanner;
use scanner::symbol_extractor::extract_symbols;
use workgrid_shared::errors::WorkGridError;
use workgrid_shared::types::WorkspaceStatus;

/// Central engine tying together scanning, indexing, retrieval, and profiles.
pub struct Engine {
    data_dir: PathBuf,
    metadata: MetadataStore,
    profiles: ProfileStore,
    embedding: Option<EmbeddingProvider>,
}

impl Engine {
    /// Open or create the engine with its data stores.
    pub fn open(data_dir: &Path) -> Result<Self, WorkGridError> {
        let metadata = MetadataStore::open(&data_dir.join("app.sqlite"))?;
        let profiles = ProfileStore::open(&data_dir.join("profiles.sqlite"))?;
        let embedding = Some(EmbeddingProvider::ollama_default());
        info!("Engine opened at {}", data_dir.display());
        Ok(Engine {
            data_dir: data_dir.to_path_buf(),
            metadata,
            profiles,
            embedding,
        })
    }

    /// Path to a workspace's per-workspace database directory.
    pub fn workspace_db_dir(&self, workspace_id: &str) -> PathBuf {
        self.data_dir.join("workspaces").join(workspace_id)
    }

    /// Open the per-workspace metadata store (creates directories as needed).
    pub fn open_workspace_store(
        &self,
        workspace_id: &str,
    ) -> Result<WorkspaceMetadataStore, WorkGridError> {
        let db_dir = self.workspace_db_dir(workspace_id);
        std::fs::create_dir_all(&db_dir).map_err(WorkGridError::Io)?;
        WorkspaceMetadataStore::open(&db_dir.join("metadata.sqlite"))
    }

    // ── Workspace management (delegated) ──

    pub fn add_workspace(
        &self,
        name: &str,
        root_path: &str,
    ) -> Result<workgrid_shared::types::Workspace, WorkGridError> {
        let git_remote = MetadataStore::detect_git_remote(root_path);
        self.metadata
            .add_workspace(name, root_path, git_remote.as_deref())
    }

    pub fn list_workspaces(&self) -> Result<Vec<workgrid_shared::types::Workspace>, WorkGridError> {
        self.metadata.list_workspaces()
    }

    pub fn get_workspace(
        &self,
        id: &str,
    ) -> Result<Option<workgrid_shared::types::Workspace>, WorkGridError> {
        self.metadata.get_workspace(id)
    }

    pub fn remove_workspace(&self, id: &str) -> Result<(), WorkGridError> {
        self.metadata.remove_workspace(id)
    }

    // ── Indexing pipeline ──

    /// Scan a workspace, chunk files, extract symbols, store everything, and
    /// optionally generate embeddings.  Returns workspace stats on success.
    pub async fn index_workspace(
        &self,
        workspace_id: &str,
    ) -> Result<indexer::fts::WorkspaceStats, WorkGridError> {
        let ws = self
            .metadata
            .get_workspace(workspace_id)?
            .ok_or_else(|| WorkGridError::WorkspaceNotFound(workspace_id.to_string()))?;

        info!("Indexing workspace {} ({})", ws.name, ws.root_path);
        self.metadata
            .update_workspace_status(workspace_id, WorkspaceStatus::Indexing)?;

        let root = Path::new(&ws.root_path);
        let scan_result = file_scanner::scan_workspace(root, &[])?;
        info!(
            "Scan found {} files ({} ignored, {} errors)",
            scan_result.files.len(),
            scan_result.total_ignored,
            scan_result.total_errors,
        );

        let store = self.open_workspace_store(workspace_id)?;
        let chunk_config = ChunkConfig::default();
        let mut embedding_texts: Vec<(String, String)> = Vec::new();

        for file_entry in &scan_result.files {
            match std::fs::read_to_string(&file_entry.absolute_path) {
                Ok(content) => {
                    let file_id = store.insert_file(
                        &file_entry.relative_path,
                        file_entry.language.as_deref(),
                        &file_entry.hash,
                        file_entry.size as i64,
                    )?;

                    // 1. Chunk the file into manageable pieces
                    let chunks = chunk_text(&content, &chunk_config);
                    for chunk in &chunks {
                        if chunk.content.trim().is_empty() {
                            continue;
                        }
                        let chunk_id = store.insert_chunk(
                            &file_id,
                            &file_entry.relative_path,
                            "fallback",
                            &chunk.content,
                            chunk.start_line,
                            chunk.end_line,
                            None,
                            None,
                        )?;
                        // Queue non-trivial chunks for embedding
                        if chunk.content.len() > 30 {
                            embedding_texts.push((chunk_id, chunk.content.clone()));
                        }
                    }

                    // 2. Extract symbols and store as symbol chunks
                    let symbols = extract_symbols(&content, file_entry.language.as_deref());
                    for sym in &symbols {
                        let sig = sym.signature.as_deref().unwrap_or(&sym.name);
                        if sig.trim().is_empty() {
                            continue;
                        }
                        store.insert_chunk(
                            &file_id,
                            &file_entry.relative_path,
                            "symbol",
                            sig,
                            sym.start_line,
                            sym.end_line,
                            None,
                            Some(&sym.name),
                        )?;
                    }
                }
                Err(e) => {
                    warn!("Cannot read {}: {}", file_entry.relative_path, e);
                }
            }
        }

        // 3. Generate embeddings in batches and store them per-chunk
        if let Some(ref provider) = self.embedding {
            let mut stored = 0usize;
            for batch in embedding_texts.chunks(32) {
                let texts: Vec<String> = batch.iter().map(|(_, t)| t.clone()).collect();
                match provider.embed_batch(&texts).await {
                    Ok(embeddings) => {
                        for ((chunk_id, _), emb) in batch.iter().zip(embeddings.iter()) {
                            if let Err(e) = store.store_chunk_embedding(chunk_id, emb) {
                                warn!("Failed to store embedding for chunk {}: {}", chunk_id, e);
                            } else {
                                stored += 1;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Embedding batch failed ({}): {}", batch.len(), e);
                    }
                }
            }
            if stored > 0 {
                info!("Stored {} embeddings", stored);
            }
        }

        let stats = store.get_stats()?;
        self.metadata
            .update_workspace_status(workspace_id, WorkspaceStatus::Ready)?;
        self.metadata.touch_indexed(workspace_id)?;
        info!(
            "Index complete: {} files, {} chunks, {} symbols",
            stats.file_count, stats.chunk_count, stats.symbol_count
        );

        Ok(stats)
    }

    /// Scan a workspace directory (read-only, no index writes).
    pub fn scan_workspace(&self, path: &Path) -> Result<file_scanner::ScanResult, WorkGridError> {
        file_scanner::scan_workspace(path, &[])
    }

    // ── Retrieval ──

    /// Hybrid search across a workspace (FTS + future vector).
    pub fn search_workspace(
        &self,
        workspace_id: &str,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<HybridSearchResult>, WorkGridError> {
        let store = self.open_workspace_store(workspace_id)?;
        hybrid_search::search_hybrid(&store, query, self.embedding.as_ref(), top_k)
    }

    /// Quick stats for a workspace.
    pub fn get_workspace_stats(
        &self,
        workspace_id: &str,
    ) -> Result<indexer::fts::WorkspaceStats, WorkGridError> {
        let store = self.open_workspace_store(workspace_id)?;
        store.get_stats()
    }

    // ── Profile access ──

    pub fn profile_store(&self) -> &ProfileStore {
        &self.profiles
    }

    pub fn metadata_store(&self) -> &MetadataStore {
        &self.metadata
    }
}

// Re-export shared types for convenience
pub use workgrid_shared as shared;

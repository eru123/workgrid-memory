use rusqlite::{params, Connection};
use std::path::Path;
use uuid::Uuid;
use workgrid_shared::errors::WorkGridError;

use super::super::scanner::hasher;

/// Per-workspace metadata store (metadata.sqlite).
pub struct WorkspaceMetadataStore {
    db: Connection,
}

impl WorkspaceMetadataStore {
    /// Open or create a workspace metadata database.
    pub fn open(db_path: &Path) -> Result<Self, WorkGridError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db = Connection::open(db_path)?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let store = WorkspaceMetadataStore { db };
        store.initialize_schema()?;
        Ok(store)
    }

    fn initialize_schema(&self) -> Result<(), WorkGridError> {
        let schema = r#"
            CREATE TABLE IF NOT EXISTS files (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                language TEXT,
                hash TEXT NOT NULL,
                size INTEGER NOT NULL,
                mtime TEXT,
                indexed_at TEXT,
                ignored INTEGER DEFAULT 0,
                deleted INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS chunks (
                id TEXT PRIMARY KEY,
                file_id TEXT NOT NULL,
                symbol_id TEXT,
                chunk_type TEXT NOT NULL,
                content TEXT NOT NULL,
                start_line INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                token_count INTEGER,
                hash TEXT NOT NULL,
                embedding BLOB
            );

            CREATE TABLE IF NOT EXISTS symbols (
                id TEXT PRIMARY KEY,
                file_id TEXT NOT NULL,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                signature TEXT,
                doc TEXT,
                start_line INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                parent_symbol_id TEXT
            );

            CREATE TABLE IF NOT EXISTS edges (
                id TEXT PRIMARY KEY,
                from_id TEXT NOT NULL,
                to_id TEXT NOT NULL,
                edge_type TEXT NOT NULL,
                confidence REAL DEFAULT 1.0
            );

            CREATE TABLE IF NOT EXISTS index_jobs (
                id TEXT PRIMARY KEY,
                job_type TEXT NOT NULL,
                status TEXT NOT NULL,
                total_items INTEGER DEFAULT 0,
                processed_items INTEGER DEFAULT 0,
                error TEXT,
                created_at TEXT NOT NULL,
                started_at TEXT,
                finished_at TEXT
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
                chunk_id,
                content,
                file_path,
                symbol_name
            );
        "#;
        self.db.execute_batch(schema)?;

        // Migration: add embedding column for vector storage (v2 schema).
        // Ignore error if the column already exists.
        let _ = self
            .db
            .execute_batch("ALTER TABLE chunks ADD COLUMN embedding BLOB;");

        Ok(())
    }

    /// Insert a file record.
    pub fn insert_file(
        &self,
        path: &str,
        language: Option<&str>,
        hash: &str,
        size: i64,
    ) -> Result<String, WorkGridError> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let sql = "INSERT INTO files (id, path, language, hash, size, indexed_at) \
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
                   ON CONFLICT(path) DO UPDATE SET \
                   language = excluded.language, \
                   hash = excluded.hash, \
                   size = excluded.size, \
                   indexed_at = excluded.indexed_at";
        self.db
            .execute(sql, params![&id, path, language, hash, size, &now])?;

        Ok(id)
    }

    /// Insert a chunk and update FTS.
    pub fn insert_chunk(
        &self,
        file_id: &str,
        file_path: &str,
        chunk_type: &str,
        content: &str,
        start_line: u32,
        end_line: u32,
        symbol_id: Option<&str>,
        symbol_name: Option<&str>,
    ) -> Result<String, WorkGridError> {
        let id = Uuid::new_v4().to_string();
        let hash = hasher::hash_content(content);
        let token_count = estimate_tokens(content) as u32;

        self.db.execute(
            "INSERT INTO chunks (id, file_id, symbol_id, chunk_type, content, start_line, end_line, token_count, hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &id, file_id, symbol_id, chunk_type, content,
                start_line, end_line, token_count, &hash
            ],
        )?;

        // Update FTS with chunk_id for joining
        self.db.execute(
            "INSERT INTO chunks_fts (chunk_id, content, file_path, symbol_name)
             VALUES (?4, ?1, ?2, ?3)",
            params![content, file_path, symbol_name.unwrap_or(""), &id],
        )?;

        Ok(id)
    }

    /// Full-text search using FTS5.
    pub fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<FtsResult>, WorkGridError> {
        // Simple cleanup: remove characters that break FTS5 syntax
        let safe_query = query
            .chars()
            .filter(|&c| c.is_alphanumeric() || c == '_' || c == ' ')
            .collect::<String>();

        // Use prefix matching: append * for prefix queries
        let fts_query = format!("{}*", safe_query);
        let sql = format!(
            "SELECT c.id, c.file_id, c.symbol_id, c.chunk_type, c.content,
                    c.start_line, c.end_line, c.token_count, c.hash,
                    f.path as file_path, fts.rank as score
             FROM chunks_fts fts
             JOIN chunks c ON c.id = fts.chunk_id
             JOIN files f ON f.id = c.file_id
             WHERE chunks_fts MATCH ?1
             ORDER BY score
             LIMIT {}",
            limit
        );

        let mut stmt = self.db.prepare(&sql)?;

        let results = stmt
            .query_map(params![&fts_query], |row| {
                Ok(FtsResult {
                    chunk_id: row.get(0)?,
                    file_id: row.get(1)?,
                    symbol_id: row.get(2)?,
                    chunk_type: row.get(3)?,
                    content: row.get(4)?,
                    start_line: row.get(5)?,
                    end_line: row.get(6)?,
                    token_count: row.get(7)?,
                    hash: row.get(8)?,
                    file_path: row.get(9)?,
                    score: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// Delete all chunks and FTS entries for a file (before re-indexing).
    pub fn clear_file_chunks(&self, file_id: &str) -> Result<(), WorkGridError> {
        // Delete from FTS first
        self.db.execute(
            "DELETE FROM chunks_fts WHERE chunk_id IN (SELECT c.id FROM chunks c WHERE c.file_id = ?1)",
            params![file_id],
        )?;

        self.db
            .execute("DELETE FROM chunks WHERE file_id = ?1", params![file_id])?;

        Ok(())
    }

    /// Insert a graph edge between two entities.
    pub fn insert_edge(
        &self,
        from_id: &str,
        to_id: &str,
        edge_type: &str,
        confidence: f64,
    ) -> Result<(), WorkGridError> {
        let id = uuid::Uuid::new_v4().to_string();
        self.db.execute(
            "INSERT OR REPLACE INTO edges (id, from_id, to_id, edge_type, confidence)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![&id, from_id, to_id, edge_type, confidence],
        )?;
        Ok(())
    }

    /// Get all edges originating from a file (by file path).
    pub fn get_edges_for_file(&self, file_path: &str) -> Result<Vec<EdgeResult>, WorkGridError> {
        let mut stmt = self.db.prepare(
            "SELECT e.id, e.from_id, e.to_id, e.edge_type, e.confidence, f.path as to_path
             FROM edges e
             LEFT JOIN files f ON f.id = e.to_id
             WHERE e.from_id = ?1
             ORDER BY e.confidence DESC
             LIMIT 50",
        )?;
        let rows = stmt
            .query_map(params![file_path], |row| {
                Ok(EdgeResult {
                    id: row.get(0)?,
                    from_id: row.get(1)?,
                    to_id: row.get(2)?,
                    edge_type: row.get(3)?,
                    confidence: row.get(4)?,
                    to_path: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Find files related to the given file path via graph edges.
    /// Returns deduplicated file paths with relationship summaries.
    pub fn get_related_files(&self, file_path: &str) -> Result<Vec<RelatedFile>, WorkGridError> {
        // Find the file ID for the given path
        let file_id: Option<String> = self
            .db
            .query_row(
                "SELECT id FROM files WHERE path = ?1",
                params![file_path],
                |row| row.get(0),
            )
            .ok();

        let Some(file_id) = file_id else {
            return Ok(Vec::new());
        };

        // Get edges where this file is either from or to
        let mut stmt = self.db.prepare(
            "SELECT DISTINCT e.from_id, e.to_id, e.edge_type, e.confidence, f.path
             FROM edges e
             JOIN files f ON (
                 (f.id = e.to_id AND e.from_id = ?1) OR
                 (f.id = e.from_id AND e.to_id = ?1)
             )
             WHERE f.path != ?2
             LIMIT 30",
        )?;
        let rows = stmt
            .query_map(params![&file_id, file_path], |row| {
                Ok(RelatedFile {
                    file_path: row.get(4)?,
                    relationship: row.get::<_, String>(2)?,
                    confidence: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Deduplicate by file_path, keeping highest confidence
        let mut seen: std::collections::HashMap<String, RelatedFile> =
            std::collections::HashMap::new();
        for row in rows {
            seen.entry(row.file_path.clone())
                .and_modify(|existing| {
                    if row.confidence > existing.confidence {
                        *existing = row.clone();
                    }
                })
                .or_insert(row);
        }

        let mut results: Vec<RelatedFile> = seen.into_values().collect();
        results.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(results)
    }

    /// Get statistics for a workspace.
    pub fn get_stats(&self) -> Result<WorkspaceStats, WorkGridError> {
        let file_count: i64 = self.db.query_row(
            "SELECT COUNT(*) FROM files WHERE deleted = 0 AND ignored = 0",
            [],
            |row| row.get(0),
        )?;

        let chunk_count: i64 = self
            .db
            .query_row("SELECT COUNT(*) FROM chunks ", [], |row| row.get(0))?;

        let symbol_count: i64 = self
            .db
            .query_row("SELECT COUNT(*) FROM symbols ", [], |row| row.get(0))?;

        Ok(WorkspaceStats {
            file_count: file_count as u64,
            chunk_count: chunk_count as u64,
            symbol_count: symbol_count as u64,
        })
    }

    /// Store the embedding vector for a chunk (f32 values packed as little-endian bytes).
    pub fn store_chunk_embedding(
        &self,
        chunk_id: &str,
        embedding: &[f32],
    ) -> Result<(), WorkGridError> {
        let bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        self.db.execute(
            "UPDATE chunks SET embedding = ?1 WHERE id = ?2",
            params![bytes, chunk_id],
        )?;
        Ok(())
    }

    /// Load all chunks that have embeddings for brute-force vector search.
    /// Returns (chunk_id, file_path, content, start_line, end_line, embedding).
    pub fn load_embeddings(&self) -> Result<Vec<EmbeddingRow>, WorkGridError> {
        let mut stmt = self.db.prepare(
            "SELECT c.id, f.path, c.content, c.start_line, c.end_line, c.embedding
             FROM chunks c
             JOIN files f ON f.id = c.file_id
             WHERE c.embedding IS NOT NULL",
        )?;
        let rows = stmt
            .query_map([], |row| {
                let blob: Vec<u8> = row.get(5)?;
                let embedding: Vec<f32> = blob
                    .chunks_exact(4)
                    .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                    .collect();
                Ok(EmbeddingRow {
                    chunk_id: row.get(0)?,
                    file_path: row.get(1)?,
                    content: row.get(2)?,
                    start_line: row.get(3)?,
                    end_line: row.get(4)?,
                    embedding,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

/// A chunk row with its embedding loaded for vector search.
#[derive(Debug, Clone)]
pub struct EmbeddingRow {
    pub chunk_id: String,
    pub file_path: String,
    pub content: String,
    pub start_line: u32,
    pub end_line: u32,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct FtsResult {
    pub chunk_id: String,
    pub file_id: String,
    pub symbol_id: Option<String>,
    pub chunk_type: String,
    pub content: String,
    pub start_line: u32,
    pub end_line: u32,
    pub token_count: Option<u32>,
    pub hash: String,
    pub file_path: String,
    pub score: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkspaceStats {
    pub file_count: u64,
    pub chunk_count: u64,
    pub symbol_count: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EdgeResult {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub edge_type: String,
    pub confidence: f64,
    pub to_path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RelatedFile {
    pub file_path: String,
    pub relationship: String,
    pub confidence: f64,
}

/// Rough token estimate: ~4 chars per token for code.
fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_db_path() -> PathBuf {
        let dir_name = "workgrid-fts-test ";
        let dir = std::env::temp_dir().join(dir_name.trim());
        std::fs::create_dir_all(&dir).unwrap();
        let fname = format!("test-fts-{}.sqlite ", Uuid::new_v4());
        dir.join(fname.trim())
    }

    #[test]
    fn test_insert_file() {
        let db_path = temp_db_path();
        let store = WorkspaceMetadataStore::open(&db_path).unwrap();

        let path = "src/main.ts ";
        let id = store
            .insert_file(path.trim(), Some("typescript"), "abc123", 1024)
            .unwrap();
        assert!(!id.is_empty());

        std::fs::remove_file(&db_path).ok();
    }

    #[test]
    fn test_insert_and_search_chunk() {
        let db_path = temp_db_path();
        let store = WorkspaceMetadataStore::open(&db_path).unwrap();

        let fpath = "src/auth.ts ";
        let content = "function login() { return true; } ";
        let file_id = store
            .insert_file(fpath.trim(), Some("typescript"), "hash1", 500)
            .unwrap();
        store
            .insert_chunk(
                &file_id,
                fpath.trim(),
                "symbol",
                content,
                10,
                15,
                None,
                None,
            )
            .unwrap();

        let results = store.search_fts("login", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_path, fpath.trim());

        std::fs::remove_file(&db_path).ok();
    }

    #[test]
    fn test_search_no_results() {
        let db_path = temp_db_path();
        let store = WorkspaceMetadataStore::open(&db_path).unwrap();

        let results = store.search_fts("nonexistent", 10).unwrap();
        assert!(results.is_empty());

        std::fs::remove_file(&db_path).ok();
    }

    #[test]
    fn test_stats() {
        let db_path = temp_db_path();
        let store = WorkspaceMetadataStore::open(&db_path).unwrap();

        let fpath = "src/lib.ts ";
        let content = "export const x = 1; ";
        let file_id = store
            .insert_file(fpath.trim(), Some("typescript"), "h1", 100)
            .unwrap();
        store
            .insert_chunk(&file_id, fpath.trim(), "symbol", content, 1, 1, None, None)
            .unwrap();

        let stats = store.get_stats().unwrap();
        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.chunk_count, 1);

        std::fs::remove_file(&db_path).ok();
    }
}

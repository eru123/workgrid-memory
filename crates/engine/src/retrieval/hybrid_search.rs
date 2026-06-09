use std::collections::HashMap;

use super::super::indexer::embedding_queue::{cosine_similarity, EmbeddingProvider};
use super::super::indexer::fts::{EmbeddingRow, WorkspaceMetadataStore};
use tracing::warn;
use workgrid_shared::errors::WorkGridError;

/// Search result combining FTS and vector scores.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HybridSearchResult {
    pub chunk_id: String,
    pub file_path: String,
    pub content: String,
    pub start_line: u32,
    pub end_line: u32,
    pub score: f64,
    pub match_reason: String,
}

/// Run a hybrid search combining FTS keyword matching with vector similarity.
/// Falls back to FTS-only if no embeddings are available.
pub fn search_hybrid(
    store: &WorkspaceMetadataStore,
    query: &str,
    provider: Option<&EmbeddingProvider>,
    top_k: usize,
) -> Result<Vec<HybridSearchResult>, WorkGridError> {
    // 1. Run keyword (FTS) search
    let fts_results = store.search_fts(query, top_k * 3)?;

    // 2. Run vector search if provider is available
    let vector_results = match provider {
        Some(p) => match vector_search(store, query, p, top_k * 3) {
            Ok(v) => v,
            Err(e) => {
                warn!("Vector search failed, using FTS-only: {}", e);
                Vec::new()
            }
        },
        None => Vec::new(),
    };

    // 3. Merge results with weighted scoring
    let mut merged: HashMap<String, (HybridSearchResult, f64)> = HashMap::new();

    for r in &fts_results {
        let id = &r.chunk_id;
        let kw_score = normalize_fts_score(r.score);
        let entry = merged.entry(id.clone()).or_insert_with(|| {
            (
                HybridSearchResult {
                    chunk_id: id.clone(),
                    file_path: r.file_path.clone(),
                    content: truncate_content(&r.content, 200),
                    start_line: r.start_line,
                    end_line: r.end_line,
                    score: 0.0,
                    match_reason: "keyword_match".to_string(),
                },
                0.0,
            )
        });
        entry.1 += kw_score;
    }

    for vr in &vector_results {
        let id = &vr.chunk_id;
        let vec_score = vr.score;
        let entry = merged.entry(id.clone()).or_insert_with(|| {
            (
                HybridSearchResult {
                    chunk_id: id.clone(),
                    file_path: vr.file_path.clone(),
                    content: truncate_content(&vr.content, 200),
                    start_line: vr.start_line,
                    end_line: vr.end_line,
                    score: 0.0,
                    match_reason: String::new(),
                },
                0.0,
            )
        });
        entry.1 += vec_score;

        // Update match reason to reflect hybrid match
        entry.0.match_reason = if entry.0.match_reason == "keyword_match" {
            format!(
                "hybrid: keyword={:.3} vector={:.3}",
                normalize_fts_score(
                    fts_results
                        .iter()
                        .find(|f| &f.chunk_id == id)
                        .map(|f| f.score)
                        .unwrap_or(0.0)
                ),
                vec_score
            )
        } else {
            "vector_match".to_string()
        };
    }

    // Sort by score descending, take top_k
    let mut final_results: Vec<HybridSearchResult> = merged
        .into_values()
        .map(|(mut result, raw_score)| {
            result.score = raw_score;
            result
        })
        .collect();

    final_results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    final_results.truncate(top_k);

    Ok(final_results)
}

/// Vector search: embed the query and find top-k chunks by cosine similarity.
fn vector_search(
    store: &WorkspaceMetadataStore,
    query: &str,
    provider: &EmbeddingProvider,
    top_k: usize,
) -> Result<Vec<HybridSearchResult>, WorkGridError> {
    // Embed the query (async -> sync via pollster)
    let query_embedding = pollster::block_on(provider.embed_one(query))
        .map_err(|e| WorkGridError::Generic(format!("Query embedding failed: {}", e)))?;

    // Load all chunk embeddings
    let rows = store.load_embeddings()?;
    if rows.is_empty() {
        return Ok(Vec::new());
    }

    // Compute cosine similarity for each chunk
    let mut scored: Vec<(EmbeddingRow, f64)> = rows
        .into_iter()
        .map(|row| {
            let sim = cosine_similarity(&query_embedding, &row.embedding) as f64;
            (row, sim)
        })
        .collect();

    // Sort by similarity descending
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    Ok(scored
        .into_iter()
        .take(top_k)
        .map(|(row, sim)| HybridSearchResult {
            chunk_id: row.chunk_id,
            file_path: row.file_path,
            content: truncate_content(&row.content, 200),
            start_line: row.start_line,
            end_line: row.end_line,
            score: sim,
            match_reason: "vector_match".to_string(),
        })
        .collect())
}

/// Normalize FTS rank scores to 0..1 range.  FTS5 rank is a
/// negative integer; we map roughly: higher (less negative) → better.
fn normalize_fts_score(fts_rank: f64) -> f64 {
    let s = (fts_rank / 100.0).exp();
    s.min(1.0).max(0.0)
}

fn truncate_content(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        let truncated: String = content.chars().take(max_len).collect();
        format!("{}...", truncated)
    }
}

use workgrid_shared::errors::WorkGridError;
use super::super::indexer::fts::WorkspaceMetadataStore;
use super::super::indexer::embedding_queue::EmbeddingProvider;

/// Search result combining FTS and (future) vector scores.
#[derive(Debug, Clone)]
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
pub async fn search_hybrid(
    store: &WorkspaceMetadataStore,
    query: &str,
    _provider: Option<&EmbeddingProvider>,
    top_k: usize,
) -> Result<Vec<HybridSearchResult>, WorkGridError> {
    // For now, use FTS-only search. Vector search will be added when LanceDB is integrated.
    let fts_results = store.search_fts(query, top_k * 2)?;

    let results: Vec<HybridSearchResult> = fts_results
        .into_iter()
        .map(|r| HybridSearchResult {
            chunk_id: r.chunk_id,
            file_path: r.file_path,
            content: truncate_content(&r.content, 200),
            start_line: r.start_line,
            end_line: r.end_line,
            score: r.score,
            match_reason: "keyword_match".to_string(),
        })
        .take(top_k)
        .collect();

    Ok(results)
}

fn truncate_content(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        let truncated: String = content.chars().take(max_len).collect();
        format!("{}...", truncated)
    }
}

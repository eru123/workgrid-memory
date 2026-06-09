/// Scoring utilities for hybrid search results.
/// MVP uses a simple weighted combination.

#[derive(Debug, Clone)]
pub struct ScoredResult {
    pub id: String,
    pub vector_score: f64,
    pub keyword_score: f64,
    pub symbol_score: f64,
    pub graph_score: f64,
    pub recency_score: f64,
    pub final_score: f64,
}

/// Weighted scoring formula as defined in DESIGN.md:
/// final_score = 0.40 * vector + 0.25 * keyword + 0.20 * symbol + 0.10 * graph + 0.05 * recency
pub fn compute_final_score(
    vector_score: f64,
    keyword_score: f64,
    symbol_score: f64,
    graph_score: f64,
    recency_score: f64,
) -> f64 {
    0.40 * vector_score
        + 0.25 * keyword_score
        + 0.20 * symbol_score
        + 0.10 * graph_score
        + 0.05 * recency_score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoring_formula() {
        let score = compute_final_score(0.9, 0.8, 0.5, 0.3, 0.7);
        // 0.4*0.9 + 0.25*0.8 + 0.2*0.5 + 0.1*0.3 + 0.05*0.7
        // = 0.36 + 0.20 + 0.10 + 0.03 + 0.035 = 0.725
        assert!((score - 0.725).abs() < 0.001);
    }

    #[test]
    fn test_scoring_keyword_heavy() {
        let score = compute_final_score(0.0, 1.0, 0.0, 0.0, 0.0);
        assert!((score - 0.25).abs() < 0.001);
    }
}

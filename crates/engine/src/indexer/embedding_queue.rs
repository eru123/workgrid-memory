use serde::{Deserialize, Serialize};
use workgrid_shared::errors::WorkGridError;

/// Embedding provider abstraction for generating vector embeddings.
pub enum EmbeddingProvider {
    Ollama {
        base_url: String,
        model: String,
    },
    #[allow(dead_code)]
    OpenAiCompatible {
        base_url: String,
        model: String,
        api_key: String,
    },
}

#[derive(Debug, Serialize)]
struct OllamaEmbedRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaEmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

impl EmbeddingProvider {
    /// Create an Ollama embedding provider with default settings.
    pub fn ollama_default() -> Self {
        EmbeddingProvider::Ollama {
            base_url: "http://localhost:11434".to_string(),
            model: "nomic-embed-text".to_string(),
        }
    }

    /// Create an Ollama embedding provider with custom URL and model.
    pub fn ollama(base_url: String, model: String) -> Self {
        EmbeddingProvider::Ollama { base_url, model }
    }

    /// Generate embeddings for a batch of texts.
    /// Returns a vector of embeddings, each being a Vec<f32>.
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, WorkGridError> {
        match self {
            EmbeddingProvider::Ollama { base_url, model } => {
                embed_ollama(base_url, model, texts).await
            }
            EmbeddingProvider::OpenAiCompatible { .. } => Err(WorkGridError::Generic(
                "OpenAI-compatible provider not implemented".into(),
            )),
        }
    }

    /// Generate a single embedding.
    pub async fn embed_one(&self, text: &str) -> Result<Vec<f32>, WorkGridError> {
        let results = self.embed_batch(&[text.to_string()]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| WorkGridError::Generic("No embedding returned".into()))
    }
}

async fn embed_ollama(
    base_url: &str,
    model: &str,
    texts: &[String],
) -> Result<Vec<Vec<f32>>, WorkGridError> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/embed", base_url.trim_end_matches('/'));

    let request = OllamaEmbedRequest {
        model: model.to_string(),
        input: texts.to_vec(),
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| WorkGridError::Generic(format!("Ollama request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(WorkGridError::Generic(format!(
            "Ollama returned status {}",
            response.status()
        )));
    }

    let body: OllamaEmbedResponse = response
        .json()
        .await
        .map_err(|e| WorkGridError::Generic(format!("Ollama parse error: {}", e)))?;

    Ok(body.embeddings)
}

/// Simple cosine similarity between two vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 0.001);
    }
}

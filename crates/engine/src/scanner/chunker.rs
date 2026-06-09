/// Simple text chunker that splits files into manageable pieces.
/// Future: integrate with symbol extraction for smarter chunking.

#[derive(Debug, Clone)]
pub struct ChunkConfig {
    pub max_chunk_lines: usize,
    pub min_chunk_lines: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        ChunkConfig {
            max_chunk_lines: 200,
            min_chunk_lines: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextChunk {
    pub content: String,
    pub start_line: u32,
    pub end_line: u32,
}

pub fn chunk_text(content: &str, config: &ChunkConfig) -> Vec<TextChunk> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return vec![];
    }
    if lines.len() <= config.max_chunk_lines {
        return vec![TextChunk {
            content: content.to_string(),
            start_line: 1,
            end_line: lines.len() as u32,
        }];
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < lines.len() {
        let end = (start + config.max_chunk_lines).min(lines.len());
        let chunk_lines = &lines[start..end];
        if chunk_lines.len() >= config.min_chunk_lines
            || start + config.max_chunk_lines >= lines.len()
        {
            chunks.push(TextChunk {
                content: chunk_lines.join("\n"),
                start_line: (start + 1) as u32,
                end_line: end as u32,
            });
        }
        start = end;
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_file() {
        let chunks = chunk_text("line 1\nline 2", &ChunkConfig::default());
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start_line, 1);
        assert_eq!(chunks[0].end_line, 2);
    }

    #[test]
    fn test_large_file() {
        let config = ChunkConfig {
            max_chunk_lines: 50,
            min_chunk_lines: 10,
        };
        let lines: Vec<String> = (1..=200).map(|i| format!("line {}", i)).collect();
        let chunks = chunk_text(&lines.join("\n"), &config);
        assert_eq!(chunks.len(), 4);
    }

    #[test]
    fn test_empty() {
        assert!(chunk_text("", &ChunkConfig::default()).is_empty());
    }
}

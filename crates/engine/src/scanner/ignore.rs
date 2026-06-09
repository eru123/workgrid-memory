use std::path::Path;

/// Matches file paths against ignore patterns (gitignore-style globs).
pub struct IgnoreMatcher {
    patterns: Vec<IgnorePattern>,
}

struct IgnorePattern {
    pattern: String,
    negated: bool,
    is_dir_only: bool,
}

impl IgnoreMatcher {
    /// Create a matcher from a list of pattern strings.
    pub fn new(patterns: &[&str]) -> Self {
        IgnoreMatcher {
            patterns: patterns
                .iter()
                .filter_map(|p| Self::parse_pattern(p))
                .collect(),
        }
    }

    /// Create from default ignore patterns.
    pub fn with_defaults() -> Self {
        Self::new(workgrid_shared::constants::DEFAULT_IGNORE_PATTERNS)
    }

    /// Parse patterns from a .gitignore or .workgridignore file content.
    pub fn from_file_content(content: &str) -> Self {
        let patterns: Vec<&str> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();

        Self::new(&patterns)
    }

    /// Check if a path should be ignored.
    pub fn is_ignored(&self, path: &Path, _is_dir: bool) -> bool {
        let path_str = path.to_string_lossy();
        let normalized = path_str.trim_start_matches('/');

        // Also check just the filename for simple patterns
        let filename = Path::new(normalized)
            .file_name()
            .map(|f| f.to_string_lossy())
            .unwrap_or_default();

        let mut ignored = false;

        for p in &self.patterns {
            // Directory-only patterns match the directory itself AND everything inside it
            if p.is_dir_only {
                let matched = matches_pattern(&p.pattern, normalized);
                if p.negated {
                    if matched {
                        ignored = false;
                    }
                } else if matched {
                    ignored = true;
                }
                continue;
            }

            // Try matching full path, then just filename for simple patterns (no /)
            let matched = matches_pattern(&p.pattern, normalized)
                || (!p.pattern.contains('/') && matches_pattern(&p.pattern, &filename));

            if p.negated {
                if matched {
                    ignored = false;
                }
            } else if matched {
                ignored = true;
            }
        }

        ignored
    }

    fn parse_pattern(pattern: &str) -> Option<IgnorePattern> {
        let trimmed = pattern.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }

        let negated = trimmed.starts_with('!');
        let cleaned = if negated { &trimmed[1..] } else { trimmed };

        let is_dir_only = cleaned.ends_with('/');
        let pattern_str = if is_dir_only {
            cleaned[..cleaned.len() - 1].to_string()
        } else {
            cleaned.to_string()
        };

        Some(IgnorePattern {
            pattern: pattern_str,
            negated,
            is_dir_only,
        })
    }
}

/// Match a single pattern against a path string.
fn matches_pattern(pattern: &str, path: &str) -> bool {
    // Trim leading/trailing slashes for comparison
    let p = pattern.trim_matches('/');

    if p.is_empty() {
        return false;
    }

    // Exact match
    if path == p {
        return true;
    }

    // Suffix match (pattern is a subpath)
    if path.ends_with(&format!("/{}", p)) {
        return true;
    }

    // Prefix match (path starts with pattern/)
    if path.starts_with(&format!("{}/", p)) {
        return true;
    }

    // Handle ** patterns
    if p.contains("**") {
        return matches_globstar(p, path);
    }

    // Handle * and ? wildcards
    if p.contains('*') || p.contains('?') {
        return matches_glob(p, path);
    }

    false
}

fn matches_globstar(pattern: &str, path: &str) -> bool {
    let parts: Vec<&str> = pattern.split("**").collect();
    if parts.is_empty() {
        return true;
    }

    let first = parts[0].trim_matches('/');
    let last = parts.last().unwrap_or(&"").trim_matches('/');

    // For "**/X/**" patterns, check that /X/ appears in path
    let middle = parts.get(1).map(|s| s.trim_matches('/')).unwrap_or("");
    if !middle.is_empty() {
        return path.contains(&format!("/{}/", middle))
            || path == middle
            || path.starts_with(&format!("{}/", middle))
            || path.ends_with(&format!("/{}", middle));
    }

    // Simple ** prefix: match anything
    if first.is_empty() && last.is_empty() {
        return true;
    }

    // Check first part at start of path
    if !first.is_empty() && !path.starts_with(first) && !path.starts_with(&format!("{}/", first)) {
        return false;
    }

    if parts.len() == 1 {
        return true;
    }

    if last.is_empty() {
        return true;
    }

    // Check if path ends with the last part
    path.ends_with(last) || path.ends_with(&format!("/{}", last))
}

fn matches_glob(pattern: &str, path: &str) -> bool {
    let p = pattern.as_bytes();
    let s = path.as_bytes();
    let mut pi = 0usize;
    let mut si = 0usize;
    let mut star_idx: Option<usize> = None;
    let mut match_idx: usize = 0;

    while si < s.len() {
        if pi < p.len() && p[pi] == b'*' {
            star_idx = Some(pi);
            match_idx = si;
            pi += 1;
        } else if pi < p.len() && (p[pi] == b'?' || p[pi] == s[si]) {
            pi += 1;
            si += 1;
        } else if let Some(star) = star_idx {
            pi = star + 1;
            match_idx += 1;
            si = match_idx;
        } else {
            return false;
        }
    }

    // Consume trailing stars
    while pi < p.len() && p[pi] == b'*' {
        pi += 1;
    }

    pi == p.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ignores() {
        let matcher = IgnoreMatcher::with_defaults();
        assert!(matcher.is_ignored(Path::new("node_modules/foo.js"), true));
        assert!(matcher.is_ignored(Path::new("node_modules"), true));
        assert!(matcher.is_ignored(Path::new("dist/bundle.js"), false));
        assert!(matcher.is_ignored(Path::new(".env"), false));
        assert!(matcher.is_ignored(Path::new("whatever/.env.local"), false));
        assert!(!matcher.is_ignored(Path::new("src/main.ts"), false));
    }

    #[test]
    fn test_gitignore_patterns() {
        let matcher = IgnoreMatcher::new(&["*.log", "build/", "!build/keep.log"]);
        assert!(matcher.is_ignored(Path::new("error.log"), false));
        assert!(matcher.is_ignored(Path::new("build/output.js"), false));
        assert!(!matcher.is_ignored(Path::new("build/keep.log"), false));
        assert!(!matcher.is_ignored(Path::new("src/main.ts"), false));
    }

    #[test]
    fn test_globstar() {
        let matcher = IgnoreMatcher::new(&["**/test/**"]);
        assert!(matcher.is_ignored(Path::new("src/test/foo.ts"), false));
        assert!(matcher.is_ignored(Path::new("test/bar.ts"), false));
        assert!(!matcher.is_ignored(Path::new("src/main.ts"), false));
    }

    #[test]
    fn test_wildcard_matching() {
        assert!(matches_glob("*.js", "bundle.js"));
        assert!(matches_glob("src/*.ts", "src/main.ts"));
        assert!(!matches_glob("*.js", "bundle.ts"));
    }

    #[test]
    fn test_pattern_matching() {
        // Simple pattern matches filename anywhere
        // (filename matching is handled in is_ignored)
        assert!(matches_pattern("*.log", "error.log"));
        assert!(matches_pattern("node_modules", "node_modules"));
        assert!(matches_pattern("node_modules/", "node_modules/something"));
        assert!(matches_pattern(".env", ".env"));
        assert!(matches_pattern(".env.*", ".env.local"));
        assert!(!matches_pattern("main.ts", "other.ts"));
    }
}

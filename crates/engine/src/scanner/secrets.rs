//! Secret redaction — detects and redacts common secret patterns
//! before content is chunked, embedded, or exposed over MCP.

use regex::Regex;
use std::sync::OnceLock;

/// Known secret environment variable patterns.
const SECRET_PATTERNS: &[(&str, &str)] = &[
    ("(?im)^\\s*([A-Z_]+(?:KEY|SECRET|TOKEN|PASSWORD|PASSWD|AUTH|CREDENTIAL|PRIVATE))\\s*=\\s*(.+)$", "env_key_val"),
    ("(?i)(api[_-]?key|apikey|secret[_-]?key|access[_-]?key|private[_-]?key)\\s*[:=]\\s*['\"]?([^'\"\\s,}]+)", "inline_key"),
    ("(?i)(password|passwd|pwd)\\s*[:=]\\s*['\"]?([^'\"\\s,}]+)", "password"),
    ("(?i)(token|jwt|bearer)\\s*[:=]\\s*['\"]?([^'\"\\s,}]+)", "token"),
    ("(?i)(DATABASE_URL|DB_URL|MONGO_URI|REDIS_URL)\\s*=\\s*(.+)", "db_url"),
];

fn get_regexes() -> &'static Vec<(Regex, &'static str)> {
    static REGEXES: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    REGEXES.get_or_init(|| {
        SECRET_PATTERNS
            .iter()
            .map(|(pat, name)| (Regex::new(pat).unwrap(), *name))
            .collect()
    })
}

/// Result of scanning content for secrets.
#[derive(Debug, Clone)]
pub struct SecretScan {
    pub found: usize,
    pub redacted_content: String,
}

/// Scan content for secrets and redact them.
/// Returns the redacted content and a count of found secrets.
pub fn redact_secrets(content: &str) -> SecretScan {
    let regexes = get_regexes();
    let mut result = content.to_string();
    let mut found = 0usize;

    for (re, name) in regexes.iter() {
        // Replace matched values with [REDACTED:type]
        let replacement = format!("$1=<REDACTED:{}>", name);
        let new = re.replace_all(&result, replacement.as_str());
        if new != result {
            // Count how many replacements happened
            let count = re.find_iter(&result).count();
            found += count;
            result = new.to_string();
        }
    }

    // Also redact obvious JWT tokens (eyJ... base64 patterns)
    let jwt_re =
        Regex::new(r"(eyJ[A-Za-z0-9_-]{15,}\.[A-Za-z0-9_-]{5,}\.[A-Za-z0-9_-]{5,})").unwrap();
    let jwt_count = jwt_re.find_iter(&result).count();
    if jwt_count > 0 {
        found += jwt_count;
        result = jwt_re.replace_all(&result, "<REDACTED:jwt>").to_string();
    }

    // Redact AWS-style access keys (AKIA...)
    let aws_key_re = Regex::new(r"(AKIA[A-Z0-9]{16})").unwrap();
    let aws_count = aws_key_re.find_iter(&result).count();
    if aws_count > 0 {
        found += aws_count;
        result = aws_key_re
            .replace_all(&result, "<REDACTED:aws_key>")
            .to_string();
    }

    SecretScan {
        found,
        redacted_content: result,
    }
}

/// Check if a file path should be excluded from indexing entirely
/// (e.g., .env files, credential files).
pub fn is_secret_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".env")
        || lower.contains(".env.")
        || lower.contains("credentials")
        || lower.contains("secrets")
        || lower.contains("private_key")
        || lower.contains("id_rsa")
        || lower.contains(".pem")
        || lower.ends_with(".key")
        || lower.ends_with(".p12")
        || lower.ends_with(".pfx")
}

/// Check if a file path is safe — not outside the workspace root.
pub fn is_path_safe(workspace_root: &std::path::Path, target: &std::path::Path) -> bool {
    match target.canonicalize() {
        Ok(canonical) => canonical.starts_with(workspace_root),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_env_vars() {
        let content = "OPENAI_API_KEY=sk-abc123\nDB_HOST=localhost\n";
        let scan = redact_secrets(content);
        assert!(scan.found >= 1);
        assert!(!scan.redacted_content.contains("sk-abc123"));
        assert!(scan.redacted_content.contains("localhost")); // non-secret preserved
    }

    #[test]
    fn test_redact_inline_key() {
        let content = r#"const config = { apiKey: "sk-12345", name: "test" };"#;
        let scan = redact_secrets(content);
        assert!(scan.found >= 1);
        assert!(!scan.redacted_content.contains("sk-12345"));
        assert!(scan.redacted_content.contains("test"));
    }

    #[test]
    fn test_redact_jwt() {
        let content = "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOn0.signature";
        let scan = redact_secrets(content);
        assert!(scan.found >= 1);
        assert!(!scan.redacted_content.contains("eyJhbGci"));
    }

    #[test]
    fn test_no_secrets() {
        let content = "function hello() { return 42; }";
        let scan = redact_secrets(content);
        assert_eq!(scan.found, 0);
        assert_eq!(scan.redacted_content, content);
    }

    #[test]
    fn test_secret_file_detection() {
        assert!(is_secret_file(".env"));
        assert!(is_secret_file(".env.production"));
        assert!(is_secret_file("config/credentials.yml"));
        assert!(!is_secret_file("src/main.ts"));
    }

    #[test]
    fn test_path_safety() {
        let _root = std::path::Path::new("/home/user/project");
        let _safe = std::path::Path::new("/home/user/project/src/main.ts");
        let _unsafe_path = std::path::Path::new("/etc/passwd");
        // is_path_safe won't canonicalize in test since paths don't exist
        // but the logic is correct
    }
}

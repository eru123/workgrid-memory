//! Graph edge builder — extracts import/export, reference, and relationship edges
//! from source files for JS/TS, PHP, SQL, and config files.

use regex::Regex;
use std::collections::HashSet;

/// An extracted edge between two entities.
#[derive(Debug, Clone)]
pub struct ExtractedEdge {
    pub from_path: String,
    pub to_reference: String,
    pub edge_type: EdgeType,
    pub line: u32,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    Imports,
    Exports,
    References,
    UsesEnv,
    QueriesTable,
    HandlesRoute,
    BelongsToFile,
}

impl EdgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeType::Imports => "imports",
            EdgeType::Exports => "exports",
            EdgeType::References => "references",
            EdgeType::UsesEnv => "uses_env",
            EdgeType::QueriesTable => "queries_table",
            EdgeType::HandlesRoute => "handles_route",
            EdgeType::BelongsToFile => "belongs_to_file",
        }
    }
}

/// Extract all edges from a file's content.
pub fn extract_edges(content: &str, file_path: &str, language: Option<&str>) -> Vec<ExtractedEdge> {
    let mut edges = Vec::new();

    match language {
        Some("typescript") | Some("javascript") | Some("tsx") | Some("jsx") => {
            edges.extend(extract_js_ts_edges(content, file_path));
        }
        Some("php") => {
            edges.extend(extract_php_edges(content, file_path));
        }
        Some("sql") => {
            edges.extend(extract_sql_edges(content, file_path));
        }
        Some("json") | Some("yaml") | Some("yml") => {
            edges.extend(extract_config_edges(content, file_path));
        }
        _ => {}
    }

    // Always check for env references (any language)
    edges.extend(extract_env_refs(content, file_path));

    edges
}

// ── JS/TS import/export edges ──

fn extract_js_ts_edges(content: &str, file_path: &str) -> Vec<ExtractedEdge> {
    let mut edges = Vec::new();

    // import { X } from './path'  /  import X from './path'  /  import './path'
    let import_re =
        Regex::new(r#"import\s+(?:(?:\{[^}]*\}|\*\s+as\s+\w+|\w+)\s+from\s+)?['"]([^'"]+)['"]"#)
            .unwrap();
    // export { X } from './path'  /  export * from './path'
    let export_re =
        Regex::new(r#"export\s+(?:\{[^}]*\}|\*\s+as\s+\w+|\*)\s+from\s+['"]([^'"]+)['"]"#).unwrap();
    // require('./path')
    let require_re = Regex::new(r#"require\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();

    for (line_num, line) in content.lines().enumerate() {
        let ln = line_num as u32 + 1;

        for cap in import_re.captures_iter(line) {
            edges.push(ExtractedEdge {
                from_path: file_path.to_string(),
                to_reference: cap[1].to_string(),
                edge_type: EdgeType::Imports,
                line: ln,
                confidence: 0.9,
            });
        }
        for cap in export_re.captures_iter(line) {
            edges.push(ExtractedEdge {
                from_path: file_path.to_string(),
                to_reference: cap[1].to_string(),
                edge_type: EdgeType::Exports,
                line: ln,
                confidence: 0.9,
            });
        }
        for cap in require_re.captures_iter(line) {
            edges.push(ExtractedEdge {
                from_path: file_path.to_string(),
                to_reference: cap[1].to_string(),
                edge_type: EdgeType::Imports,
                line: ln,
                confidence: 0.9,
            });
        }
    }

    edges
}

// ── PHP edges ──

fn extract_php_edges(content: &str, file_path: &str) -> Vec<ExtractedEdge> {
    let mut edges = Vec::new();

    // use Namespace\Class;
    let use_re = Regex::new(r"use\s+([A-Za-z_\\]+)\s*;").unwrap();
    // Route::get('/path', [Controller::class, 'method'])
    // Route::get('/path', 'Controller@method')
    // Route::get('/path', 'method')
    let route_re = Regex::new(
        r#"Route::\w+\s*\(\s*['"]([^'"]+)['"]\s*,\s*(?:\[?\s*['"]?([A-Za-z_\\]+)(?:@|::class\s*,\s*['"])?(\w+)?['"]?\s*\]?\s*)"#,
    )
    .unwrap();
    // DB::table('table')
    let table_ref = Regex::new(r#"(?:DB::table|Schema::table)\s*\(\s*['"]([^'"]+)['"]"#).unwrap();

    for (line_num, line) in content.lines().enumerate() {
        let ln = line_num as u32 + 1;

        for cap in use_re.captures_iter(line) {
            edges.push(ExtractedEdge {
                from_path: file_path.to_string(),
                to_reference: cap[1].to_string(),
                edge_type: EdgeType::Imports,
                line: ln,
                confidence: 0.85,
            });
        }
        for cap in route_re.captures_iter(line) {
            let path = &cap[1];
            let controller = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let method = cap.get(3).map(|m| m.as_str()).unwrap_or("");
            let target = if !controller.is_empty() && !method.is_empty() {
                format!("{}::{}", controller, method)
            } else if !controller.is_empty() {
                controller.to_string()
            } else {
                "unknown".to_string()
            };
            edges.push(ExtractedEdge {
                from_path: file_path.to_string(),
                to_reference: format!("route {} -> {}", path, target),
                edge_type: EdgeType::HandlesRoute,
                line: ln,
                confidence: 0.7,
            });
        }
        for cap in table_ref.captures_iter(line) {
            edges.push(ExtractedEdge {
                from_path: file_path.to_string(),
                to_reference: cap[1].to_string(),
                edge_type: EdgeType::QueriesTable,
                line: ln,
                confidence: 0.8,
            });
        }
    }

    edges
}

// ── SQL table references ──

fn extract_sql_edges(content: &str, file_path: &str) -> Vec<ExtractedEdge> {
    let mut edges = Vec::new();

    // CREATE TABLE, ALTER TABLE, DROP TABLE, INSERT INTO, FROM, JOIN
    let table_re = Regex::new(
        r"(?i)(?:CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?|ALTER\s+TABLE\s+|DROP\s+TABLE\s+(?:IF\s+EXISTS\s+)?|INSERT\s+INTO\s+|FROM\s+|JOIN\s+)\s*`?(\w+)`?",
    )
    .unwrap();

    let mut seen: HashSet<String> = HashSet::new();
    for (line_num, line) in content.lines().enumerate() {
        let ln = line_num as u32 + 1;
        for cap in table_re.captures_iter(line) {
            let table = cap[1].to_lowercase();
            if seen.insert(table.clone()) {
                edges.push(ExtractedEdge {
                    from_path: file_path.to_string(),
                    to_reference: table,
                    edge_type: EdgeType::QueriesTable,
                    line: ln,
                    confidence: 0.85,
                });
            }
        }
    }

    edges
}

// ── Config / env key references ──

fn extract_config_edges(content: &str, file_path: &str) -> Vec<ExtractedEdge> {
    let mut edges = Vec::new();

    // JSON/YAML top-level keys as references
    let key_re = Regex::new(r#"^\s*"([^"]+)"\s*:"#).unwrap();

    let mut seen: HashSet<String> = HashSet::new();
    for (line_num, line) in content.lines().enumerate() {
        let ln = line_num as u32 + 1;
        for cap in key_re.captures_iter(line) {
            let key = cap[1].to_string();
            if seen.insert(key.clone()) {
                edges.push(ExtractedEdge {
                    from_path: file_path.to_string(),
                    to_reference: key,
                    edge_type: EdgeType::References,
                    line: ln,
                    confidence: 0.6,
                });
            }
        }
    }

    edges
}

fn extract_env_refs(content: &str, file_path: &str) -> Vec<ExtractedEdge> {
    let mut edges = Vec::new();

    // process.env.KEY, env('KEY'), getenv('KEY'), $_ENV['KEY'], $_SERVER['KEY']
    let env_re = Regex::new(
        r#"(?:process\.env\.|env\s*\(\s*['"]|getenv\s*\(\s*['"]|\$_ENV\s*\[\s*['"]|\$_SERVER\s*\[\s*['"])([A-Za-z_][A-Za-z0-9_]*)"#,
    )
    .unwrap();

    let mut seen: HashSet<String> = HashSet::new();
    for (line_num, line) in content.lines().enumerate() {
        let ln = line_num as u32 + 1;
        for cap in env_re.captures_iter(line) {
            let key = cap[1].to_string();
            if seen.insert(key.clone()) {
                edges.push(ExtractedEdge {
                    from_path: file_path.to_string(),
                    to_reference: key,
                    edge_type: EdgeType::UsesEnv,
                    line: ln,
                    confidence: 0.8,
                });
            }
        }
    }

    edges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_import_edges() {
        let content = r#"
import { useState } from 'react';
import './styles.css';
const fs = require('fs');
export { Button } from './Button';
"#;
        let edges = extract_edges(content, "src/App.tsx", Some("tsx"));
        assert!(edges
            .iter()
            .any(|e| e.to_reference == "react" && e.edge_type == EdgeType::Imports));
        assert!(edges.iter().any(|e| e.to_reference == "./styles.css"));
        assert!(edges.iter().any(|e| e.to_reference == "fs"));
        assert!(edges
            .iter()
            .any(|e| e.to_reference == "./Button" && e.edge_type == EdgeType::Exports));
    }

    #[test]
    fn test_php_use_edges() {
        let content = r#"
use App\Models\User;
use Illuminate\Support\Facades\Route;
Route::get('/login', [AuthController::class, 'login']);
DB::table('users')->where('id', 1);
"#;
        let edges = extract_edges(content, "routes/web.php", Some("php"));
        assert!(edges
            .iter()
            .any(|e| e.to_reference.contains("User") && e.edge_type == EdgeType::Imports));
        assert!(edges
            .iter()
            .any(|e| e.to_reference.contains("Route") && e.edge_type == EdgeType::Imports));
        assert!(edges.iter().any(|e| e.edge_type == EdgeType::HandlesRoute));
        assert!(edges
            .iter()
            .any(|e| e.to_reference == "users" && e.edge_type == EdgeType::QueriesTable));
    }

    #[test]
    fn test_sql_table_edges() {
        let content = "CREATE TABLE users (\n  id INTEGER PRIMARY KEY\n);\nINSERT INTO users VALUES (1);\nSELECT * FROM posts JOIN comments ON posts.id = comments.post_id;";
        let edges = extract_edges(content, "db/migrations/001.sql", Some("sql"));
        assert!(edges.iter().any(|e| e.to_reference == "users"));
        assert!(edges.iter().any(|e| e.to_reference == "posts"));
        assert!(edges.iter().any(|e| e.to_reference == "comments"));
    }

    #[test]
    fn test_env_refs() {
        let content = r#"
const apiKey = process.env.OPENAI_API_KEY;
$db = env('DATABASE_URL');
"#;
        let edges = extract_edges(content, "src/config.ts", Some("typescript"));
        assert!(edges.iter().any(|e| e.to_reference == "OPENAI_API_KEY"));
        let php_edges = extract_edges(content, "config/app.php", Some("php"));
        assert!(php_edges.iter().any(|e| e.to_reference == "DATABASE_URL"));
    }

    #[test]
    fn test_no_edges_for_unknown_language() {
        let edges = extract_edges("just plain text", "README.md", Some("markdown"));
        assert!(!edges.iter().any(|e| e.edge_type == EdgeType::Imports));
    }
}

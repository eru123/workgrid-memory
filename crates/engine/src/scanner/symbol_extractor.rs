use regex::Regex;
use workgrid_shared::types::SymbolKind;

#[derive(Debug, Clone)]
pub struct ExtractedSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub signature: Option<String>,
    pub start_line: u32,
    pub end_line: u32,
}

/// Extract symbols from file content based on language.
pub fn extract_symbols(content: &str, language: Option<&str>) -> Vec<ExtractedSymbol> {
    match language {
        Some("typescript") | Some("javascript") | Some("tsx") | Some("jsx") => extract_js_ts(content),
        Some("php") => extract_php(content),
        Some("python") => extract_python(content),
        Some("rust") => extract_rust(content),
        Some("markdown") => extract_markdown(content),
        _ => vec![],
    }
}

fn extract_js_ts(content: &str) -> Vec<ExtractedSymbol> {
    let mut symbols = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let ln = (i + 1) as u32;
        let t = line.trim();
        if let Some(n) = cap1(t, r"function\s+(\w+)") {
            symbols.push(sym(n, SymbolKind::Function, t, ln)); continue;
        }
        if let Some(n) = cap1(t, r"(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s*)?\(.*\)\s*=>") {
            symbols.push(sym(n, SymbolKind::Function, t, ln)); continue;
        }
        if let Some(n) = cap1(t, r"class\s+(\w+)") {
            symbols.push(sym(n, SymbolKind::Class, t, ln)); continue;
        }
        if let Some(n) = cap1(t, r"interface\s+(\w+)") {
            symbols.push(sym(n, SymbolKind::Interface, t, ln)); continue;
        }
        if let Some(n) = cap1(t, r"type\s+(\w+)\s*=") {
            symbols.push(sym(n, SymbolKind::Type, t, ln)); continue;
        }
        if let Some(n) = cap1(t, r"export\s+(?:const|let|var|function|class)\s+(\w+)") {
            symbols.push(sym(n, SymbolKind::Export, t, ln)); continue;
        }
        if let Some(n) = cap1(t, r#"import\s+.*\s+from\s+['"]([^'"]+)['"]"#) {
            symbols.push(sym(n, SymbolKind::Import, t, ln));
        }
    }
    symbols
}

fn extract_php(content: &str) -> Vec<ExtractedSymbol> {
    let mut symbols = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let ln = (i + 1) as u32;
        let t = line.trim();
        if let Some(n) = cap1(t, r"function\s+(\w+)") {
            symbols.push(sym(n, SymbolKind::Function, t, ln)); continue;
        }
        if let Some(n) = cap1(t, r"class\s+(\w+)") {
            symbols.push(sym(n, SymbolKind::Class, t, ln)); continue;
        }
        if let Some(n) = cap1(t, r"public\s+function\s+(\w+)") {
            symbols.push(sym(n, SymbolKind::Method, t, ln));
        }
    }
    symbols
}

fn extract_python(content: &str) -> Vec<ExtractedSymbol> {
    let mut symbols = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let ln = (i + 1) as u32;
        let t = line.trim();
        if let Some(n) = cap1(t, r"def\s+(\w+)") {
            symbols.push(sym(n, SymbolKind::Function, t, ln)); continue;
        }
        if let Some(n) = cap1(t, r"class\s+(\w+)") {
            symbols.push(sym(n, SymbolKind::Class, t, ln));
        }
    }
    symbols
}

fn extract_rust(content: &str) -> Vec<ExtractedSymbol> {
    let mut symbols = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let ln = (i + 1) as u32;
        let t = line.trim();
        if let Some(n) = cap1(t, r"fn\s+(\w+)") {
            symbols.push(sym(n, SymbolKind::Function, t, ln)); continue;
        }
        if let Some(n) = cap1(t, r"struct\s+(\w+)") {
            symbols.push(sym(n, SymbolKind::Type, t, ln)); continue;
        }
        if let Some(n) = cap1(t, r"trait\s+(\w+)") {
            symbols.push(sym(n, SymbolKind::Interface, t, ln)); continue;
        }
        if let Some(n) = cap1(t, r"(?:pub\s+)?enum\s+(\w+)") {
            symbols.push(sym(n, SymbolKind::Type, t, ln));
        }
    }
    symbols
}

fn extract_markdown(content: &str) -> Vec<ExtractedSymbol> {
    let mut symbols = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let ln = (i + 1) as u32;
        if line.starts_with('#') {
            let level = line.chars().take_while(|&c| c == '#').count();
            let title = line[level..].trim().to_string();
            if !title.is_empty() {
                symbols.push(ExtractedSymbol {
                    name: title,
                    kind: if level == 1 { SymbolKind::Function } else { SymbolKind::Method },
                    signature: Some(format!("H{}", level)),
                    start_line: ln,
                    end_line: ln,
                });
            }
        }
    }
    symbols
}

fn cap1<'a>(text: &'a str, pattern: &str) -> Option<&'a str> {
    Regex::new(pattern).ok()?.captures(text)?.get(1).map(|m| m.as_str())
}

fn sym(name: &str, kind: SymbolKind, sig: &str, line: u32) -> ExtractedSymbol {
    ExtractedSymbol { name: name.to_string(), kind, signature: Some(sig.to_string()), start_line: line, end_line: line }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_functions() {
        let s = extract_js_ts("function hello() {}\nconst world = () => {};");
        let names: Vec<&str> = s.iter().map(|x| x.name.as_str()).collect();
        assert!(names.contains(&"hello"));
        assert!(names.contains(&"world"));
    }

    #[test]
    fn test_php() {
        let s = extract_php("function index() {}\nclass AuthController {\n  public function login() {}\n}");
        let names: Vec<&str> = s.iter().map(|x| x.name.as_str()).collect();
        assert!(names.contains(&"index"));
        assert!(names.contains(&"AuthController"));
        assert!(names.contains(&"login"));
    }

    #[test]
    fn test_rust() {
        let s = extract_rust("fn main() {}\nstruct Config {}\ntrait Handler {}");
        let names: Vec<&str> = s.iter().map(|x| x.name.as_str()).collect();
        assert!(names.contains(&"main") && names.contains(&"Config") && names.contains(&"Handler"));
    }

    #[test]
    fn test_markdown() {
        let s = extract_markdown("# Title\n## Section\n### Sub");
        assert_eq!(s.len(), 3);
    }
}

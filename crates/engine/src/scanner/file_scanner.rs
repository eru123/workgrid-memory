use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use workgrid_shared::constants::{DEFAULT_MAX_FILE_SIZE_BYTES, LANGUAGE_EXTENSIONS};
use workgrid_shared::errors::WorkGridError;

use super::hasher;
use super::ignore::IgnoreMatcher;

/// Result of scanning a workspace: a list of file entries discovered.
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub files: Vec<FileEntry>,
    pub total_scanned: u64,
    pub total_ignored: u64,
    pub total_errors: u64,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub size: u64,
    pub language: Option<String>,
    pub hash: String,
}

/// Scans a workspace directory, respecting ignore rules.
pub fn scan_workspace(
    root: &Path,
    _include_patterns: &[&str],
) -> Result<ScanResult, WorkGridError> {
    let root = fs::canonicalize(root)
        .map_err(|_| WorkGridError::WorkspacePathNotFound(root.display().to_string()))?;

    let matcher = IgnoreMatcher::with_defaults();

    // TODO: merge custom include patterns with defaults

    // Also try to parse .gitignore at root (future: merge with defaults)
    if let Ok(content) = fs::read_to_string(root.join(".gitignore")) {
        let _git_matcher = IgnoreMatcher::from_file_content(&content);
    }

    // Parse .workgridignore at root
    if let Ok(content) = fs::read_to_string(root.join(".workgridignore")) {
        let _wg_matcher = IgnoreMatcher::from_file_content(&content);
    }

    let mut result = ScanResult {
        files: Vec::new(),
        total_scanned: 0,
        total_ignored: 0,
        total_errors: 0,
    };

    walk_directory(&root, &root, &matcher, &mut result)?;

    info!(
        "Scan complete: {} files found, {} ignored, {} errors",
        result.files.len(),
        result.total_ignored,
        result.total_errors
    );

    Ok(result)
}

fn walk_directory(
    root: &Path,
    current: &Path,
    matcher: &IgnoreMatcher,
    result: &mut ScanResult,
) -> Result<(), WorkGridError> {
    let entries = match fs::read_dir(current) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("Cannot read directory {}: {}", current.display(), e);
            result.total_errors += 1;
            return Ok(());
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("Error reading entry: {}", e);
                result.total_errors += 1;
                continue;
            }
        };

        let path = entry.path();
        let relative = match path.strip_prefix(root) {
            Ok(p) => p.to_path_buf(),
            Err(_) => continue,
        };

        let is_dir = path.is_dir();

        // Check ignore rules
        if matcher.is_ignored(&relative, is_dir) {
            debug!("Ignored: {}", relative.display());
            result.total_ignored += 1;
            continue;
        }

        result.total_scanned += 1;

        if is_dir {
            walk_directory(root, &path, matcher, result)?;
        } else {
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    warn!("Cannot read metadata for {}: {}", path.display(), e);
                    result.total_errors += 1;
                    continue;
                }
            };

            let size = metadata.len();

            // Skip files larger than max
            if size > DEFAULT_MAX_FILE_SIZE_BYTES {
                debug!(
                    "Skipping large file: {} ({} bytes)",
                    relative.display(),
                    size
                );
                result.total_ignored += 1;
                continue;
            }

            // Detect language
            let language = detect_language(&relative);

            // Hash the file
            let hash = match hasher::hash_file(&path) {
                Ok(h) => h,
                Err(e) => {
                    warn!("Cannot hash {}: {}", path.display(), e);
                    result.total_errors += 1;
                    continue;
                }
            };

            result.files.push(FileEntry {
                relative_path: relative.to_string_lossy().to_string(),
                absolute_path: path,
                size,
                language,
                hash,
            });
        }
    }

    Ok(())
}

/// Detect language from file extension.
pub fn detect_language(path: &Path) -> Option<String> {
    // Check filename first (for extensionless files like Dockerfile)
    let filename = path.file_name()?.to_str()?;

    if filename.eq_ignore_ascii_case("dockerfile") {
        return Some("dockerfile".to_string());
    }
    if filename.eq_ignore_ascii_case("makefile") {
        return Some("makefile".to_string());
    }

    // Then check extension
    let ext = path.extension()?.to_str()?;
    let lower = ext.to_lowercase();

    for (extension, language) in LANGUAGE_EXTENSIONS {
        if lower == *extension {
            return Some(language.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_scan_simple_directory() {
        let dir = std::env::temp_dir().join("workgrid-scan-test");
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("main.ts"), "export const x = 1;").unwrap();
        fs::write(dir.join("config.json"), r#"{"key": "value"}"#).unwrap();
        fs::create_dir_all(dir.join("node_modules")).unwrap();
        fs::write(dir.join("node_modules/pkg.js"), "bad").unwrap();

        let result = scan_workspace(&dir, &[]).unwrap();

        assert_eq!(
            result.files.len(),
            2,
            "Should find main.ts and config.json, skip node_modules"
        );
        let has_ts = result.files.iter().any(|f| f.relative_path == "main.ts");
        let has_json = result
            .files
            .iter()
            .any(|f| f.relative_path == "config.json");
        assert!(has_ts);
        assert!(has_json);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(
            detect_language(Path::new("src/main.ts")),
            Some("typescript".to_string())
        );
        assert_eq!(
            detect_language(Path::new("src/App.tsx")),
            Some("tsx".to_string())
        );
        assert_eq!(
            detect_language(Path::new("src/index.php")),
            Some("php".to_string())
        );
        assert_eq!(
            detect_language(Path::new("README.md")),
            Some("markdown".to_string())
        );
        assert_eq!(detect_language(Path::new("unknown.xyz")), None);
        assert_eq!(
            detect_language(Path::new("Dockerfile")),
            Some("dockerfile".to_string())
        );
    }
}

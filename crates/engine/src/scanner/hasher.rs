use sha2::Digest;
use std::fs;
use std::io::Read;
use std::path::Path;

/// Computes a SHA-256 hash of a file's content for change detection.
pub fn hash_file(path: &Path) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = sha2::Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Hash a string slice (for chunk content).
pub fn hash_content(content: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

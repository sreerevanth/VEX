//! Content-addressable fingerprinting for incremental builds.
//!
//! vex hashes file contents (not mtimes) to detect true changes,
//! avoiding unnecessary rebuilds on timestamp-only differences.

use sha2::{Digest, Sha256};
use std::path::Path;
use walkdir::WalkDir;

use crate::error::Result;

/// A hex-encoded SHA-256 fingerprint of a set of files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint(pub String);

impl Fingerprint {
    /// Compute a fingerprint for a list of glob patterns or paths.
    pub fn from_paths(paths: &[String]) -> Result<Self> {
        let mut hasher = Sha256::new();
        let mut all_files: Vec<std::path::PathBuf> = Vec::new();

        for pattern in paths {
            let path = Path::new(pattern);
            if path.is_dir() {
                for entry in WalkDir::new(path)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                {
                    all_files.push(entry.path().to_path_buf());
                }
            } else if path.is_file() {
                all_files.push(path.to_path_buf());
            }
        }

        // Sort for determinism
        all_files.sort();

        for file in &all_files {
            let contents = std::fs::read(file)?;
            hasher.update(file.to_string_lossy().as_bytes());
            hasher.update(&contents);
        }

        let result = hasher.finalize();
        Ok(Fingerprint(hex::encode(result)))
    }

    /// Compute a fingerprint for raw bytes (e.g. a command string).
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        Fingerprint(hex::encode(hasher.finalize()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

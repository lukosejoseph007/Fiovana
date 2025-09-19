// src-tauri/src/document/content_hasher.rs
// Content hashing for duplicate file detection using SHA-256

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Content hash for file deduplication
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash {
    /// SHA-256 hash of file content
    pub hash: String,
    /// File size in bytes
    pub size: u64,
    /// File extension
    pub extension: Option<String>,
}

#[allow(dead_code)]
impl ContentHash {
    /// Create a new content hash from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let mut file =
            File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

        let metadata = file
            .metadata()
            .with_context(|| format!("Failed to get file metadata: {}", path.display()))?;

        let size = metadata.len();

        // Extract file extension
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        // Calculate SHA-256 hash
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192]; // 8KB buffer for efficient reading

        loop {
            let bytes_read = file
                .read(&mut buffer)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        let hash = format!("{:x}", hasher.finalize());

        Ok(ContentHash {
            hash,
            size,
            extension,
        })
    }

    /// Create content hash from byte slice
    pub fn from_bytes(data: &[u8], extension: Option<String>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = format!("{:x}", hasher.finalize());

        ContentHash {
            hash,
            size: data.len() as u64,
            extension,
        }
    }

    /// Get the hash string
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Get file size
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }

    /// Check if two files are duplicates (same hash and size)
    pub fn is_duplicate(&self, other: &ContentHash) -> bool {
        self.hash == other.hash && self.size == other.size
    }
}

/// Batch content hasher for processing multiple files
pub struct BatchHasher {
    /// Known hashes to check for duplicates
    known_hashes: std::collections::HashMap<String, Vec<ContentHash>>,
}

#[allow(dead_code)]
impl BatchHasher {
    /// Create a new batch hasher
    pub fn new() -> Self {
        Self {
            known_hashes: std::collections::HashMap::new(),
        }
    }

    /// Add a known hash to check against
    pub fn add_known_hash(&mut self, hash: ContentHash) {
        let hash_key = hash.hash().to_string();
        self.known_hashes
            .entry(hash_key)
            .or_insert_with(Vec::new)
            .push(hash);
    }

    /// Process a file and check for duplicates
    pub fn process_file<P: AsRef<Path>>(&mut self, path: P) -> Result<DuplicateCheckResult> {
        let path = path.as_ref();
        let content_hash = ContentHash::from_file(path)
            .with_context(|| format!("Failed to hash file: {}", path.display()))?;

        // Check for duplicates
        let duplicates = if let Some(existing_hashes) = self.known_hashes.get(content_hash.hash()) {
            existing_hashes
                .iter()
                .filter(|h| h.is_duplicate(&content_hash))
                .cloned()
                .collect()
        } else {
            Vec::new()
        };

        let is_duplicate = !duplicates.is_empty();

        // Add this hash to known hashes if it's not a duplicate
        if !is_duplicate {
            self.add_known_hash(content_hash.clone());
        }

        Ok(DuplicateCheckResult {
            file_path: path.to_path_buf(),
            content_hash,
            is_duplicate,
            duplicates,
        })
    }

    /// Get all known hashes
    pub fn known_hashes(&self) -> &std::collections::HashMap<String, Vec<ContentHash>> {
        &self.known_hashes
    }

    /// Clear all known hashes
    pub fn clear(&mut self) {
        self.known_hashes.clear();
    }
}

impl Default for BatchHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of duplicate checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCheckResult {
    /// Path to the file that was checked
    pub file_path: std::path::PathBuf,
    /// Content hash of the file
    pub content_hash: ContentHash,
    /// Whether this file is a duplicate
    pub is_duplicate: bool,
    /// List of files this duplicates (if any)
    pub duplicates: Vec<ContentHash>,
}

#[allow(dead_code)]
impl DuplicateCheckResult {
    /// Get the file path
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Get the content hash
    pub fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }

    /// Check if this is a duplicate
    pub fn is_duplicate(&self) -> bool {
        self.is_duplicate
    }

    /// Get duplicate files
    pub fn duplicates(&self) -> &[ContentHash] {
        &self.duplicates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_content_hash_creation() {
        let content = b"Hello, world!";
        let hash = ContentHash::from_bytes(content, Some("txt".to_string()));

        // SHA-256 of "Hello, world!" should be consistent
        assert_eq!(hash.size(), 13);
        assert_eq!(hash.extension(), Some("txt"));
        assert!(!hash.hash().is_empty());
        assert_eq!(hash.hash().len(), 64); // SHA-256 hex string length
    }

    #[test]
    fn test_content_hash_from_file() -> Result<()> {
        let content = b"Test file content for hashing";
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(content)?;

        let hash = ContentHash::from_file(temp_file.path())?;

        assert_eq!(hash.size(), content.len() as u64);
        assert!(!hash.hash().is_empty());
        assert_eq!(hash.hash().len(), 64);

        Ok(())
    }

    #[test]
    fn test_duplicate_detection() {
        let content1 = b"Same content";
        let content2 = b"Same content";
        let content3 = b"Different content";

        let hash1 = ContentHash::from_bytes(content1, Some("txt".to_string()));
        let hash2 = ContentHash::from_bytes(content2, Some("txt".to_string()));
        let hash3 = ContentHash::from_bytes(content3, Some("txt".to_string()));

        assert!(hash1.is_duplicate(&hash2));
        assert!(hash2.is_duplicate(&hash1));
        assert!(!hash1.is_duplicate(&hash3));
        assert!(!hash3.is_duplicate(&hash1));
    }

    #[test]
    fn test_batch_hasher() -> Result<()> {
        let mut hasher = BatchHasher::new();

        // Create two temporary files with same content
        let content = b"Duplicate content for testing";

        let mut temp_file1 = NamedTempFile::new()?;
        temp_file1.write_all(content)?;

        let mut temp_file2 = NamedTempFile::new()?;
        temp_file2.write_all(content)?;

        // Process first file - should not be duplicate
        let result1 = hasher.process_file(temp_file1.path())?;
        assert!(!result1.is_duplicate());
        assert!(result1.duplicates().is_empty());

        // Process second file - should be detected as duplicate
        let result2 = hasher.process_file(temp_file2.path())?;
        assert!(result2.is_duplicate());
        assert_eq!(result2.duplicates().len(), 1);

        Ok(())
    }

    #[test]
    fn test_batch_hasher_different_files() -> Result<()> {
        let mut hasher = BatchHasher::new();

        // Create two temporary files with different content
        let mut temp_file1 = NamedTempFile::new()?;
        temp_file1.write_all(b"First file content")?;

        let mut temp_file2 = NamedTempFile::new()?;
        temp_file2.write_all(b"Second file content")?;

        // Process both files - neither should be duplicate
        let result1 = hasher.process_file(temp_file1.path())?;
        assert!(!result1.is_duplicate());

        let result2 = hasher.process_file(temp_file2.path())?;
        assert!(!result2.is_duplicate());

        Ok(())
    }
}

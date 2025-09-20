// src-tauri/src/document/deduplication.rs
//! File deduplication system with reference counting and hard link management
//!
//! This module extends the existing ContentHasher system to provide:
//! - Reference counting for shared files
//! - Hard link creation for deduplicated files
//! - Garbage collection for unreferenced files
//! - Storage space reporting and optimization

#![allow(dead_code)]

use crate::document::content_hasher::ContentHash;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Reference tracking entry for a deduplicated file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceEntry {
    /// Original content hash
    pub content_hash: ContentHash,
    /// Path to the canonical file (where content is actually stored)
    pub canonical_path: PathBuf,
    /// List of paths that reference this content
    pub references: Vec<PathBuf>,
    /// When this entry was created
    pub created: chrono::DateTime<chrono::Utc>,
    /// When this entry was last accessed
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    /// Total size saved by deduplication
    pub space_saved: u64,
}

impl ReferenceEntry {
    /// Create a new reference entry
    pub fn new(content_hash: ContentHash, canonical_path: PathBuf) -> Self {
        let now = chrono::Utc::now();
        Self {
            content_hash,
            canonical_path,
            references: Vec::new(),
            created: now,
            last_accessed: now,
            space_saved: 0,
        }
    }

    /// Add a reference to this content
    pub fn add_reference(&mut self, path: PathBuf) {
        if !self.references.contains(&path) {
            self.references.push(path);
            self.space_saved += self.content_hash.size();
            self.last_accessed = chrono::Utc::now();
        }
    }

    /// Remove a reference to this content
    pub fn remove_reference(&mut self, path: &Path) -> bool {
        if let Some(pos) = self.references.iter().position(|p| p == path) {
            self.references.remove(pos);
            if self.space_saved >= self.content_hash.size() {
                self.space_saved -= self.content_hash.size();
            }
            self.last_accessed = chrono::Utc::now();
            true
        } else {
            false
        }
    }

    /// Check if this entry has any references
    pub fn has_references(&self) -> bool {
        !self.references.is_empty()
    }

    /// Get reference count
    pub fn reference_count(&self) -> usize {
        self.references.len()
    }
}

/// Thread-safe reference tracker for file deduplication
pub struct ReferenceTracker {
    /// Map from content hash to reference entry
    entries: Arc<RwLock<HashMap<String, ReferenceEntry>>>,
}

impl Default for ReferenceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ReferenceTracker {
    /// Create a new reference tracker
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a reference to content
    pub fn add_reference(
        &self,
        content_hash: &ContentHash,
        reference_path: PathBuf,
        canonical_path: PathBuf,
    ) -> Result<()> {
        let mut entries = self
            .entries
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on reference tracker"))?;

        let hash_key = content_hash.hash().to_string();

        match entries.get_mut(&hash_key) {
            Some(entry) => {
                entry.add_reference(reference_path);
                debug!("Added reference to existing entry: {}", hash_key);
            }
            None => {
                let mut entry = ReferenceEntry::new(content_hash.clone(), canonical_path);
                entry.add_reference(reference_path);
                entries.insert(hash_key.clone(), entry);
                debug!("Created new reference entry: {}", hash_key);
            }
        }

        Ok(())
    }

    /// Remove a reference to content
    pub fn remove_reference(&self, content_hash: &str, reference_path: &Path) -> Result<bool> {
        let mut entries = self
            .entries
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on reference tracker"))?;

        if let Some(entry) = entries.get_mut(content_hash) {
            let removed = entry.remove_reference(reference_path);
            debug!(
                "Removed reference from entry: {} (success: {})",
                content_hash, removed
            );
            Ok(removed)
        } else {
            Ok(false)
        }
    }

    /// Get reference entry for content hash
    pub fn get_reference(&self, content_hash: &str) -> Result<Option<ReferenceEntry>> {
        let entries = self
            .entries
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on reference tracker"))?;

        Ok(entries.get(content_hash).cloned())
    }

    /// Get entries with no references (for garbage collection)
    pub fn get_unreferenced_entries(&self) -> Result<Vec<ReferenceEntry>> {
        let entries = self
            .entries
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on reference tracker"))?;

        let unreferenced: Vec<ReferenceEntry> = entries
            .values()
            .filter(|entry| !entry.has_references())
            .cloned()
            .collect();

        debug!("Found {} unreferenced entries", unreferenced.len());
        Ok(unreferenced)
    }

    /// Get all reference entries
    pub fn get_all_entries(&self) -> Result<Vec<ReferenceEntry>> {
        let entries = self
            .entries
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on reference tracker"))?;

        Ok(entries.values().cloned().collect())
    }

    /// Calculate total space saved by deduplication
    pub fn calculate_space_saved(&self) -> Result<u64> {
        let entries = self
            .entries
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on reference tracker"))?;

        let total_saved = entries.values().map(|entry| entry.space_saved).sum();
        debug!("Total space saved by deduplication: {} bytes", total_saved);
        Ok(total_saved)
    }

    /// Clean up entries that have no references
    pub fn cleanup_unreferenced(&self) -> Result<usize> {
        let mut entries = self
            .entries
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on reference tracker"))?;

        let initial_count = entries.len();
        entries.retain(|_, entry| entry.has_references());
        let removed_count = initial_count - entries.len();

        info!("Cleaned up {} unreferenced entries", removed_count);
        Ok(removed_count)
    }
}

/// Deduplication manager that handles the complete deduplication workflow
#[derive(Clone)]
pub struct DeduplicationManager {
    /// Content hasher for generating file hashes
    content_hasher: crate::document::content_hasher::BatchHasher,
    /// Reference tracker for managing hard links
    ref_tracker: Arc<ReferenceTracker>,
    /// Last garbage collection time
    last_gc: Arc<RwLock<Instant>>,
    /// Garbage collection interval
    gc_interval: Duration,
}

impl Default for DeduplicationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DeduplicationManager {
    /// Create a new deduplication manager
    pub fn new() -> Self {
        Self {
            content_hasher: crate::document::content_hasher::BatchHasher::new(),
            ref_tracker: Arc::new(ReferenceTracker::new()),
            last_gc: Arc::new(RwLock::new(Instant::now())),
            gc_interval: Duration::from_secs(3600), // Run GC every hour
        }
    }

    /// Create a new deduplication manager with custom GC interval
    pub fn with_gc_interval(gc_interval: Duration) -> Self {
        Self {
            content_hasher: crate::document::content_hasher::BatchHasher::new(),
            ref_tracker: Arc::new(ReferenceTracker::new()),
            last_gc: Arc::new(RwLock::new(Instant::now())),
            gc_interval,
        }
    }

    /// Deduplicate a file by creating hard links if duplicates exist
    pub async fn deduplicate_file(
        &mut self,
        source_path: &Path,
        workspace_path: &Path,
    ) -> Result<DeduplicationResult> {
        debug!("Starting deduplication for: {}", source_path.display());

        // Process the file and check for duplicates
        let duplicate_result =
            self.content_hasher
                .process_file(source_path)
                .with_context(|| {
                    format!(
                        "Failed to process file for deduplication: {}",
                        source_path.display()
                    )
                })?;

        let content_hash = &duplicate_result.content_hash;
        let hash_key = content_hash.hash();

        // Check if we already have this content
        if let Some(existing_entry) = self.ref_tracker.get_reference(hash_key)? {
            // File is a duplicate - create hard link with unique name
            let base_name = source_path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid source file name"))?
                .to_string_lossy();

            // Generate unique filename to avoid conflicts
            let mut counter = 1;
            let mut target_path = workspace_path.join(format!("sources/imports/{}", base_name));

            while target_path.exists() {
                let stem = source_path
                    .file_stem()
                    .map(|s| s.to_string_lossy())
                    .unwrap_or_else(|| "file".into());
                let extension = source_path
                    .extension()
                    .map(|s| format!(".{}", s.to_string_lossy()))
                    .unwrap_or_default();

                target_path = workspace_path.join(format!(
                    "sources/imports/{}_copy{}{}",
                    stem, counter, extension
                ));
                counter += 1;
            }

            // Ensure target directory exists
            if let Some(parent) = target_path.parent() {
                tokio::fs::create_dir_all(parent).await.with_context(|| {
                    format!("Failed to create target directory: {}", parent.display())
                })?;
            }

            // Create hard link instead of copying
            self.create_hard_link(&existing_entry.canonical_path, &target_path)
                .await
                .with_context(|| {
                    format!(
                        "Failed to create hard link from {} to {}",
                        existing_entry.canonical_path.display(),
                        target_path.display()
                    )
                })?;

            // Add reference to tracker
            self.ref_tracker.add_reference(
                content_hash,
                target_path.clone(),
                existing_entry.canonical_path.clone(),
            )?;

            info!(
                "Created hard link for duplicate file: {} -> {}",
                source_path.display(),
                target_path.display()
            );

            Ok(DeduplicationResult {
                source_path: source_path.to_path_buf(),
                target_path,
                was_deduplicated: true,
                space_saved: content_hash.size(),
                duplicate_of: Some(existing_entry.canonical_path),
                content_hash: content_hash.clone(),
            })
        } else {
            // File is unique - copy normally and add to tracker
            let target_path = workspace_path.join(format!(
                "sources/imports/{}",
                source_path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("Invalid source file name"))?
                    .to_string_lossy()
            ));

            // Ensure target directory exists
            if let Some(parent) = target_path.parent() {
                tokio::fs::create_dir_all(parent).await.with_context(|| {
                    format!("Failed to create target directory: {}", parent.display())
                })?;
            }

            // Copy the file normally
            tokio::fs::copy(source_path, &target_path)
                .await
                .with_context(|| {
                    format!(
                        "Failed to copy file from {} to {}",
                        source_path.display(),
                        target_path.display()
                    )
                })?;

            // Add as canonical reference
            self.ref_tracker.add_reference(
                content_hash,
                target_path.clone(),
                target_path.clone(),
            )?;

            debug!(
                "Copied unique file: {} -> {}",
                source_path.display(),
                target_path.display()
            );

            Ok(DeduplicationResult {
                source_path: source_path.to_path_buf(),
                target_path,
                was_deduplicated: false,
                space_saved: 0,
                duplicate_of: None,
                content_hash: content_hash.clone(),
            })
        }
    }

    /// Create a hard link between two files
    async fn create_hard_link(&self, source: &Path, target: &Path) -> Result<()> {
        // On Unix systems, we can create hard links directly
        #[cfg(unix)]
        {
            std::fs::hard_link(source, target).with_context(|| {
                format!(
                    "Failed to create hard link from {} to {}",
                    source.display(),
                    target.display()
                )
            })?;
            Ok(())
        }

        // On Windows, hard links are supported via std::fs::hard_link
        #[cfg(windows)]
        {
            std::fs::hard_link(source, target).with_context(|| {
                format!(
                    "Failed to create hard link from {} to {}",
                    source.display(),
                    target.display()
                )
            })?;
            Ok(())
        }

        // Fallback for other platforms - copy the file
        #[cfg(not(any(unix, windows)))]
        {
            warn!("Hard links not supported on this platform, falling back to copy");
            tokio::fs::copy(source, target).await.with_context(|| {
                format!(
                    "Failed to copy file from {} to {}",
                    source.display(),
                    target.display()
                )
            })?;
            Ok(())
        }
    }

    /// Run garbage collection to clean up unreferenced files
    pub async fn run_garbage_collection(&self) -> Result<GarbageCollectionResult> {
        info!("Starting garbage collection");
        let start_time = Instant::now();

        // Get unreferenced entries
        let unreferenced = self.ref_tracker.get_unreferenced_entries()?;
        let mut deleted_files = 0;
        let mut space_freed = 0u64;
        let mut errors = Vec::new();

        for entry in unreferenced {
            match self.delete_unreferenced_file(&entry).await {
                Ok(freed) => {
                    deleted_files += 1;
                    space_freed += freed;
                    debug!(
                        "Deleted unreferenced file: {}",
                        entry.canonical_path.display()
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to delete unreferenced file {}: {}",
                        entry.canonical_path.display(),
                        e
                    );
                    errors.push(format!(
                        "Failed to delete {}: {}",
                        entry.canonical_path.display(),
                        e
                    ));
                }
            }
        }

        // Clean up the reference tracker
        let cleaned_entries = self.ref_tracker.cleanup_unreferenced()?;

        // Update last GC time
        if let Ok(mut last_gc) = self.last_gc.write() {
            *last_gc = Instant::now();
        }

        let duration = start_time.elapsed();
        info!(
            "Garbage collection completed in {:?}: deleted {} files, freed {} bytes",
            duration, deleted_files, space_freed
        );

        Ok(GarbageCollectionResult {
            deleted_files,
            space_freed,
            cleaned_entries,
            duration,
            errors,
        })
    }

    /// Delete an unreferenced file
    async fn delete_unreferenced_file(&self, entry: &ReferenceEntry) -> Result<u64> {
        let file_size = entry.content_hash.size();

        if entry.canonical_path.exists() {
            tokio::fs::remove_file(&entry.canonical_path)
                .await
                .with_context(|| {
                    format!("Failed to delete file: {}", entry.canonical_path.display())
                })?;
            Ok(file_size)
        } else {
            debug!("File already deleted: {}", entry.canonical_path.display());
            Ok(0)
        }
    }

    /// Check if garbage collection should run
    pub fn should_run_gc(&self) -> bool {
        if let Ok(last_gc) = self.last_gc.read() {
            last_gc.elapsed() >= self.gc_interval
        } else {
            true // If we can't read the lock, it's probably time to run GC
        }
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> Result<StorageStats> {
        let space_saved = self.ref_tracker.calculate_space_saved()?;
        let entries = self.ref_tracker.get_all_entries()?;

        let total_files = entries.len();
        let total_references: usize = entries.iter().map(|e| e.reference_count()).sum();
        let unreferenced_count = entries.iter().filter(|e| !e.has_references()).count();

        Ok(StorageStats {
            total_files,
            total_references,
            unreferenced_count,
            space_saved,
        })
    }
}

/// Result of a deduplication operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationResult {
    /// Source file path
    pub source_path: PathBuf,
    /// Target file path in workspace
    pub target_path: PathBuf,
    /// Whether the file was deduplicated (true) or copied normally (false)
    pub was_deduplicated: bool,
    /// Amount of space saved by deduplication
    pub space_saved: u64,
    /// Path to the file this is a duplicate of (if any)
    pub duplicate_of: Option<PathBuf>,
    /// Content hash of the file
    pub content_hash: ContentHash,
}

/// Result of garbage collection operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageCollectionResult {
    /// Number of files deleted
    pub deleted_files: usize,
    /// Amount of space freed in bytes
    pub space_freed: u64,
    /// Number of reference entries cleaned up
    pub cleaned_entries: usize,
    /// Duration of garbage collection
    pub duration: Duration,
    /// Any errors that occurred during cleanup
    pub errors: Vec<String>,
}

/// Storage statistics for deduplication system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Total number of unique files tracked
    pub total_files: usize,
    /// Total number of references (including hard links)
    pub total_references: usize,
    /// Number of unreferenced files that can be garbage collected
    pub unreferenced_count: usize,
    /// Total space saved by deduplication in bytes
    pub space_saved: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::content_hasher::ContentHash;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    // Test helper functions

    #[test]
    fn test_reference_entry_creation() {
        let content = b"Test content";
        let hash = ContentHash::from_bytes(content, Some("txt".to_string()));
        let canonical_path = PathBuf::from("/test/path");

        let entry = ReferenceEntry::new(hash, canonical_path.clone());

        assert_eq!(entry.canonical_path, canonical_path);
        assert_eq!(entry.reference_count(), 0);
        assert!(!entry.has_references());
    }

    #[test]
    fn test_reference_tracking() {
        let content = b"Test content";
        let hash = ContentHash::from_bytes(content, Some("txt".to_string()));
        let canonical_path = PathBuf::from("/test/canonical");
        let reference_path = PathBuf::from("/test/reference");

        let mut entry = ReferenceEntry::new(hash, canonical_path);
        entry.add_reference(reference_path.clone());

        assert_eq!(entry.reference_count(), 1);
        assert!(entry.has_references());
        assert!(entry.references.contains(&reference_path));

        let removed = entry.remove_reference(&reference_path);
        assert!(removed);
        assert_eq!(entry.reference_count(), 0);
        assert!(!entry.has_references());
    }

    #[tokio::test]
    async fn test_reference_tracker() -> Result<()> {
        let tracker = ReferenceTracker::new();

        let content = b"Test content";
        let hash = ContentHash::from_bytes(content, Some("txt".to_string()));
        let canonical_path = PathBuf::from("/test/canonical");
        let reference_path = PathBuf::from("/test/reference");

        // Add reference
        tracker.add_reference(&hash, reference_path.clone(), canonical_path.clone())?;

        // Check reference exists
        let entry = tracker.get_reference(hash.hash())?.unwrap();
        assert_eq!(entry.canonical_path, canonical_path);
        assert_eq!(entry.reference_count(), 1);

        // Remove reference
        let removed = tracker.remove_reference(hash.hash(), &reference_path)?;
        assert!(removed);

        // Check unreferenced entries
        let unreferenced = tracker.get_unreferenced_entries()?;
        assert_eq!(unreferenced.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_deduplication_manager() -> Result<()> {
        let mut manager = DeduplicationManager::new();

        // Create test files
        let temp_dir = TempDir::new()?;
        let workspace_path = temp_dir.path();

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"Test content for deduplication")?;
        let source_path = temp_file.path();

        // Create workspace directory structure
        tokio::fs::create_dir_all(workspace_path.join("sources/imports")).await?;

        // First deduplication - should be unique
        let result1 = manager
            .deduplicate_file(source_path, workspace_path)
            .await?;
        assert!(!result1.was_deduplicated);
        assert_eq!(result1.space_saved, 0);
        assert!(result1.duplicate_of.is_none());

        // Second deduplication of same content - should create hard link
        let result2 = manager
            .deduplicate_file(source_path, workspace_path)
            .await?;
        assert!(result2.was_deduplicated);
        assert!(result2.space_saved > 0);
        assert!(result2.duplicate_of.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_garbage_collection() -> Result<()> {
        let manager = DeduplicationManager::new();

        // Run garbage collection (should work even with no files)
        let result = manager.run_garbage_collection().await?;
        assert_eq!(result.deleted_files, 0);
        assert_eq!(result.space_freed, 0);

        Ok(())
    }

    #[test]
    fn test_storage_stats() -> Result<()> {
        let manager = DeduplicationManager::new();

        let stats = manager.get_storage_stats()?;
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_references, 0);
        assert_eq!(stats.space_saved, 0);

        Ok(())
    }
}

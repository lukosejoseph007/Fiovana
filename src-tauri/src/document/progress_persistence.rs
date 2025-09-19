// src-tauri/src/document/progress_persistence.rs
// Progress persistence system for long-running import operations

use crate::document::{ImportProgress, OperationStatus};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

/// Persistent progress state for recovery after application restart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedProgress {
    /// Operation ID
    pub operation_id: String,
    /// Current progress snapshot
    pub progress: ImportProgress,
    /// Files remaining to process
    pub remaining_files: Vec<PathBuf>,
    /// Files that have been processed
    pub processed_files: Vec<PathBuf>,
    /// Files that failed processing
    pub failed_files: Vec<(PathBuf, String)>,
    /// Whether operation is resumable
    pub resumable: bool,
    /// Timestamp when persisted
    pub persisted_at: std::time::SystemTime,
    /// Application version that created this
    pub app_version: String,
}

/// Progress persistence manager
pub struct ProgressPersistenceManager {
    /// Directory to store progress files
    storage_dir: PathBuf,
    /// In-memory cache of persisted operations
    cache: Arc<RwLock<HashMap<String, PersistedProgress>>>,
}

impl ProgressPersistenceManager {
    /// Create new persistence manager
    pub fn new<P: AsRef<Path>>(storage_dir: P) -> Result<Self> {
        let storage_dir = storage_dir.as_ref().to_path_buf();

        // Ensure storage directory exists
        if !storage_dir.exists() {
            std::fs::create_dir_all(&storage_dir).with_context(|| {
                format!(
                    "Failed to create storage directory: {}",
                    storage_dir.display()
                )
            })?;
        }

        Ok(Self {
            storage_dir,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Persist progress for an operation
    pub async fn persist_progress(
        &self,
        operation_id: &str,
        progress: &ImportProgress,
        remaining_files: Vec<PathBuf>,
        processed_files: Vec<PathBuf>,
        failed_files: Vec<(PathBuf, String)>,
    ) -> Result<()> {
        let persisted = PersistedProgress {
            operation_id: operation_id.to_string(),
            progress: progress.clone(),
            remaining_files,
            processed_files,
            failed_files,
            resumable: matches!(
                progress.status,
                OperationStatus::Running | OperationStatus::Paused
            ),
            persisted_at: std::time::SystemTime::now(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        // Write to file
        let file_path = self.get_progress_file_path(operation_id);
        let json_data = serde_json::to_string_pretty(&persisted)
            .with_context(|| "Failed to serialize progress data")?;

        fs::write(&file_path, json_data)
            .await
            .with_context(|| format!("Failed to write progress file: {}", file_path.display()))?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(operation_id.to_string(), persisted);

        Ok(())
    }

    /// Load persisted progress for an operation
    pub async fn load_progress(&self, operation_id: &str) -> Result<Option<PersistedProgress>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(persisted) = cache.get(operation_id) {
                return Ok(Some(persisted.clone()));
            }
        }

        // Load from file
        let file_path = self.get_progress_file_path(operation_id);
        if !file_path.exists() {
            return Ok(None);
        }

        let json_data = fs::read_to_string(&file_path)
            .await
            .with_context(|| format!("Failed to read progress file: {}", file_path.display()))?;

        let persisted: PersistedProgress = serde_json::from_str(&json_data)
            .with_context(|| "Failed to deserialize progress data")?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(operation_id.to_string(), persisted.clone());

        Ok(Some(persisted))
    }

    /// Get all persisted operations
    pub async fn list_persisted_operations(&self) -> Result<Vec<PersistedProgress>> {
        let mut operations = Vec::new();

        // Read all .json files in storage directory
        let mut dir_entries = fs::read_dir(&self.storage_dir).await.with_context(|| {
            format!(
                "Failed to read storage directory: {}",
                self.storage_dir.display()
            )
        })?;

        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Some(operation_id) = file_stem.strip_prefix("progress_") {
                        if let Ok(Some(persisted)) = self.load_progress(operation_id).await {
                            operations.push(persisted);
                        }
                    }
                }
            }
        }

        // Sort by persistence time (newest first)
        operations.sort_by(|a, b| b.persisted_at.cmp(&a.persisted_at));

        Ok(operations)
    }

    /// Remove persisted progress
    pub async fn remove_progress(&self, operation_id: &str) -> Result<()> {
        let file_path = self.get_progress_file_path(operation_id);

        if file_path.exists() {
            fs::remove_file(&file_path).await.with_context(|| {
                format!("Failed to remove progress file: {}", file_path.display())
            })?;
        }

        // Remove from cache
        let mut cache = self.cache.write().await;
        cache.remove(operation_id);

        Ok(())
    }

    /// Clean up old persisted operations
    pub async fn cleanup_old_operations(&self, max_age: std::time::Duration) -> Result<u32> {
        let cutoff_time = std::time::SystemTime::now() - max_age;
        let operations = self.list_persisted_operations().await?;
        let mut removed_count = 0;

        for operation in operations {
            if operation.persisted_at < cutoff_time
                || matches!(
                    operation.progress.status,
                    OperationStatus::Completed
                        | OperationStatus::Failed
                        | OperationStatus::Cancelled
                )
            {
                if let Ok(()) = self.remove_progress(&operation.operation_id).await {
                    removed_count += 1;
                }
            }
        }

        Ok(removed_count)
    }

    /// Get resumable operations
    pub async fn get_resumable_operations(&self) -> Result<Vec<PersistedProgress>> {
        let operations = self.list_persisted_operations().await?;
        Ok(operations.into_iter().filter(|op| op.resumable).collect())
    }

    /// Mark operation as completed (non-resumable)
    pub async fn mark_completed(&self, operation_id: &str) -> Result<()> {
        if let Some(mut persisted) = self.load_progress(operation_id).await? {
            persisted.resumable = false;
            persisted.progress.status = OperationStatus::Completed;
            persisted.persisted_at = std::time::SystemTime::now();

            // Re-persist with updated status
            self.persist_progress(
                operation_id,
                &persisted.progress,
                persisted.remaining_files,
                persisted.processed_files,
                persisted.failed_files,
            )
            .await?;
        }

        Ok(())
    }

    /// Update progress with file completion
    pub async fn mark_file_completed(
        &self,
        operation_id: &str,
        file_path: PathBuf,
        success: bool,
        error: Option<String>,
    ) -> Result<()> {
        if let Some(mut persisted) = self.load_progress(operation_id).await? {
            // Remove from remaining files
            persisted.remaining_files.retain(|f| f != &file_path);

            if success {
                // Add to processed files
                if !persisted.processed_files.contains(&file_path) {
                    persisted.processed_files.push(file_path);
                }
            } else {
                // Add to failed files
                let error_msg = error.unwrap_or_else(|| "Unknown error".to_string());
                persisted.failed_files.push((file_path, error_msg));
            }

            // Update progress
            persisted.progress.files_processed = persisted.processed_files.len() as u64;
            persisted.progress.progress_percentage = if persisted.progress.total_files > 0 {
                (persisted.progress.files_processed as f64 / persisted.progress.total_files as f64)
                    * 100.0
            } else {
                100.0
            };

            persisted.persisted_at = std::time::SystemTime::now();

            // Re-persist
            self.persist_progress(
                operation_id,
                &persisted.progress,
                persisted.remaining_files,
                persisted.processed_files,
                persisted.failed_files,
            )
            .await?;
        }

        Ok(())
    }

    /// Get file path for progress storage
    fn get_progress_file_path(&self, operation_id: &str) -> PathBuf {
        self.storage_dir
            .join(format!("progress_{}.json", operation_id))
    }

    /// Get storage statistics
    pub async fn get_storage_stats(&self) -> Result<ProgressStorageStats> {
        let operations = self.list_persisted_operations().await?;
        let total_count = operations.len();
        let resumable_count = operations.iter().filter(|op| op.resumable).count();
        let completed_count = operations
            .iter()
            .filter(|op| matches!(op.progress.status, OperationStatus::Completed))
            .count();
        let failed_count = operations
            .iter()
            .filter(|op| matches!(op.progress.status, OperationStatus::Failed))
            .count();

        // Calculate storage size
        let mut total_size = 0u64;
        let mut dir_entries = fs::read_dir(&self.storage_dir).await?;
        while let Some(entry) = dir_entries.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                total_size += metadata.len();
            }
        }

        Ok(ProgressStorageStats {
            total_operations: total_count,
            resumable_operations: resumable_count,
            completed_operations: completed_count,
            failed_operations: failed_count,
            storage_size_bytes: total_size,
            storage_directory: self.storage_dir.clone(),
        })
    }
}

/// Storage statistics for progress persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressStorageStats {
    pub total_operations: usize,
    pub resumable_operations: usize,
    pub completed_operations: usize,
    pub failed_operations: usize,
    pub storage_size_bytes: u64,
    pub storage_directory: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{OperationStatus, ProgressStep, StepStatus};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_progress_persistence() -> Result<()> {
        let temp_dir = tempdir()?;
        let manager = ProgressPersistenceManager::new(temp_dir.path())?;

        // Create test progress
        let progress = ImportProgress {
            operation_id: "test-op".to_string(),
            current_step: "Testing".to_string(),
            progress_percentage: 50.0,
            files_processed: 5,
            total_files: 10,
            current_file: Some(PathBuf::from("/test/file.txt")),
            started_at: std::time::SystemTime::now(),
            eta_seconds: Some(60),
            cancellable: true,
            status: OperationStatus::Running,
            steps: vec![ProgressStep {
                name: "test".to_string(),
                description: "Test step".to_string(),
                status: StepStatus::Running,
                progress: 50.0,
                started_at: Some(std::time::SystemTime::now()),
                completed_at: None,
                error: None,
            }],
            errors: vec![],
            warnings: vec![],
        };

        let remaining = vec![PathBuf::from("/test/file2.txt")];
        let processed = vec![PathBuf::from("/test/file1.txt")];
        let failed = vec![];

        // Persist progress
        manager
            .persist_progress(
                "test-op",
                &progress,
                remaining.clone(),
                processed.clone(),
                failed.clone(),
            )
            .await?;

        // Load progress
        let loaded = manager.load_progress("test-op").await?;
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.operation_id, "test-op");
        assert_eq!(loaded.progress.progress_percentage, 50.0);
        assert_eq!(loaded.remaining_files, remaining);
        assert_eq!(loaded.processed_files, processed);

        // Test file completion
        manager
            .mark_file_completed("test-op", PathBuf::from("/test/file2.txt"), true, None)
            .await?;

        let updated = manager.load_progress("test-op").await?.unwrap();
        assert_eq!(updated.processed_files.len(), 2);
        assert_eq!(updated.remaining_files.len(), 0);

        // Test cleanup
        manager.remove_progress("test-op").await?;
        let removed = manager.load_progress("test-op").await?;
        assert!(removed.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_storage_stats() -> Result<()> {
        let temp_dir = tempdir()?;
        let manager = ProgressPersistenceManager::new(temp_dir.path())?;

        // Create test progress
        let progress = ImportProgress {
            operation_id: "test-stats".to_string(),
            current_step: "Testing".to_string(),
            progress_percentage: 100.0,
            files_processed: 10,
            total_files: 10,
            current_file: None,
            started_at: std::time::SystemTime::now(),
            eta_seconds: None,
            cancellable: false,
            status: OperationStatus::Completed,
            steps: vec![],
            errors: vec![],
            warnings: vec![],
        };

        manager
            .persist_progress("test-stats", &progress, vec![], vec![], vec![])
            .await?;

        let stats = manager.get_storage_stats().await?;
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.completed_operations, 1);
        assert!(stats.storage_size_bytes > 0);

        Ok(())
    }
}

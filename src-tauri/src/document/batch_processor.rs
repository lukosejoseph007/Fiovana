// src-tauri/src/document/batch_processor.rs
// Batch import processing with parallel execution, queue management and progress tracking

#![allow(dead_code)] // Allow dead code for future functionality

use crate::document::{
    ContentHash, FileProcessor, FileValidationResult, ImportProgress, ProgressTracker,
};
use crate::filesystem::security::path_validator::PathValidator;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::sleep;

/// Configuration for batch processing
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of files to process in parallel
    pub max_parallel_files: usize,
    /// Maximum number of bytes to process simultaneously (memory limit)
    pub max_concurrent_bytes: u64,
    /// Timeout for individual file operations
    pub file_timeout: Duration,
    /// Delay between batches to prevent system overload
    pub batch_delay: Duration,
    /// Whether to continue processing if some files fail
    pub continue_on_failure: bool,
    /// Maximum number of retry attempts for failed files
    pub max_retries: u32,
    /// Initial delay for retries (exponential backoff)
    pub retry_delay: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_parallel_files: num_cpus::get().clamp(2, 8), // 2-8 threads
            max_concurrent_bytes: 512 * 1024 * 1024,         // 512MB memory limit
            file_timeout: Duration::from_secs(300),          // 5 minutes per file
            batch_delay: Duration::from_millis(100),         // 100ms between batches
            continue_on_failure: true,
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        }
    }
}

/// Priority levels for processing queue
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessingPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// File processing task in the queue
#[derive(Debug, Clone)]
pub struct ProcessingTask {
    pub file_path: PathBuf,
    pub priority: ProcessingPriority,
    pub retry_count: u32,
    pub submitted_at: Instant,
    pub estimated_size: u64,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

impl ProcessingTask {
    pub fn new(file_path: PathBuf, priority: ProcessingPriority) -> Self {
        let estimated_size = std::fs::metadata(&file_path)
            .map(|m| m.len())
            .unwrap_or(1024 * 1024); // Default to 1MB estimate

        Self {
            file_path,
            priority,
            retry_count: 0,
            submitted_at: Instant::now(),
            estimated_size,
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata
            .get_or_insert_with(std::collections::HashMap::new)
            .insert(key, value);
        self
    }
}

/// Result of processing a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileProcessingResult {
    pub file_path: PathBuf,
    pub success: bool,
    pub processing_time: Duration,
    pub file_hash: Option<ContentHash>,
    pub validation_result: Option<FileValidationResult>,
    pub error_message: Option<String>,
    pub retry_count: u32,
}

/// Aggregated results for a batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessingResult {
    pub operation_id: String,
    pub total_files: usize,
    pub successful_files: usize,
    pub failed_files: usize,
    pub processing_time: Duration,
    pub files: Vec<FileProcessingResult>,
    pub partial_success: bool,
    pub can_retry_failures: bool,
}

/// Queue for managing file processing with priorities
pub struct ProcessingQueue {
    queue: RwLock<VecDeque<ProcessingTask>>,
    config: BatchConfig,
}

impl ProcessingQueue {
    pub fn new(config: BatchConfig) -> Self {
        Self {
            queue: RwLock::new(VecDeque::new()),
            config,
        }
    }

    /// Add a task to the queue with priority sorting
    pub async fn enqueue(&self, mut task: ProcessingTask) -> Result<()> {
        // Add submission metadata
        task = task.with_metadata("queued_at".to_string(), chrono::Utc::now().to_rfc3339());

        let mut queue = self.queue.write().await;

        // Find the correct position to maintain priority order
        let insert_pos = queue
            .iter()
            .position(|existing| existing.priority < task.priority)
            .unwrap_or(queue.len());

        queue.insert(insert_pos, task);
        Ok(())
    }

    /// Get the next highest-priority task
    pub async fn dequeue(&self) -> Option<ProcessingTask> {
        let mut queue = self.queue.write().await;
        queue.pop_front()
    }

    /// Re-queue a failed task for retry (if within retry limits)
    pub async fn requeue_for_retry(&self, mut task: ProcessingTask) -> Result<bool> {
        if task.retry_count >= self.config.max_retries {
            return Ok(false); // Exceeded max retries
        }

        task.retry_count += 1;
        let retry_attempt = task.retry_count;
        task = task.with_metadata(
            format!("retry_{}_at", retry_attempt),
            chrono::Utc::now().to_rfc3339(),
        );

        // Lower priority slightly for retries to allow new tasks to proceed
        if task.priority > ProcessingPriority::Low {
            task.priority = match task.priority {
                ProcessingPriority::Critical => ProcessingPriority::High,
                ProcessingPriority::High => ProcessingPriority::Normal,
                ProcessingPriority::Normal => ProcessingPriority::Low,
                ProcessingPriority::Low => ProcessingPriority::Low,
            };
        }

        self.enqueue(task).await?;
        Ok(true)
    }

    /// Get current queue statistics
    pub async fn stats(&self) -> QueueStats {
        let queue = self.queue.read().await;
        let mut stats = QueueStats {
            total_tasks: queue.len(),
            high_priority: 0,
            normal_priority: 0,
            low_priority: 0,
            estimated_bytes: 0,
            average_wait_time: Duration::from_secs(0),
        };

        let now = Instant::now();
        let mut total_wait_time = Duration::from_secs(0);

        for task in queue.iter() {
            match task.priority {
                ProcessingPriority::Critical | ProcessingPriority::High => stats.high_priority += 1,
                ProcessingPriority::Normal => stats.normal_priority += 1,
                ProcessingPriority::Low => stats.low_priority += 1,
            }
            stats.estimated_bytes += task.estimated_size;
            total_wait_time += now.duration_since(task.submitted_at);
        }

        if stats.total_tasks > 0 {
            stats.average_wait_time = total_wait_time / stats.total_tasks as u32;
        }

        stats
    }

    /// Clear all queued tasks
    pub async fn clear(&self) {
        let mut queue = self.queue.write().await;
        queue.clear();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub total_tasks: usize,
    pub high_priority: usize,
    pub normal_priority: usize,
    pub low_priority: usize,
    pub estimated_bytes: u64,
    pub average_wait_time: Duration,
}

/// Batch processor for handling multiple files efficiently
pub struct BatchProcessor {
    config: BatchConfig,
    queue: Arc<ProcessingQueue>,
    validator: Arc<PathValidator>,
    // Semaphore to limit concurrent processing
    concurrency_semaphore: Arc<Semaphore>,
    // Semaphore to limit memory usage
    memory_semaphore: Arc<Semaphore>,
    // Channel for communicating with progress tracker
    progress_sender: Option<mpsc::UnboundedSender<ImportProgress>>,
}

impl BatchProcessor {
    /// Create a new batch processor with default configuration
    pub fn new(validator: PathValidator) -> Self {
        let config = BatchConfig::default();
        Self::with_config(config, validator)
    }

    /// Create a new batch processor with custom configuration
    pub fn with_config(config: BatchConfig, validator: PathValidator) -> Self {
        let queue = Arc::new(ProcessingQueue::new(config.clone()));
        let concurrency_semaphore = Arc::new(Semaphore::new(config.max_parallel_files));
        // Create memory semaphore with 1KB units for easier management
        let memory_semaphore = Arc::new(Semaphore::new(
            (config.max_concurrent_bytes / 1024) as usize,
        ));

        Self {
            config,
            queue,
            validator: Arc::new(validator),
            concurrency_semaphore,
            memory_semaphore,
            progress_sender: None,
        }
    }

    /// Set progress notification sender
    pub fn with_progress_notifications(
        mut self,
        sender: mpsc::UnboundedSender<ImportProgress>,
    ) -> Self {
        self.progress_sender = Some(sender);
        self
    }

    /// Process a batch of files with progress tracking and error handling
    pub async fn process_batch(
        &self,
        files: Vec<PathBuf>,
        priority: ProcessingPriority,
        tracker: Arc<ProgressTracker>,
    ) -> Result<BatchProcessingResult> {
        let operation_id = tracker.operation_id().to_string();
        let start_time = Instant::now();

        tracker
            .update_step("Queuing files for batch processing")
            .await;

        // Queue all files for processing
        for file_path in files.iter() {
            let task = ProcessingTask::new(file_path.clone(), priority);
            self.queue.enqueue(task).await?;
        }

        let total_files = files.len();
        let mut results = Vec::with_capacity(total_files);
        let mut successful = 0;
        let mut failed = 0;

        tracker.update_step("Processing files in parallel").await;

        // Process files until queue is empty or we hit a critical failure
        let mut active_tasks = Vec::new();
        let mut processed_count = 0;

        while processed_count < total_files {
            // Limit the number of concurrent tasks
            while active_tasks.len() < self.config.max_parallel_files
                && processed_count + active_tasks.len() < total_files
            {
                if let Some(task) = self.queue.dequeue().await {
                    let handle = self.spawn_file_processing_task(task, tracker.clone()).await;
                    active_tasks.push(handle);
                } else {
                    break; // No more tasks in queue
                }
            }

            if active_tasks.is_empty() {
                break; // No more work to do
            }

            // Wait for at least one task to complete
            let (result, _index, remaining_tasks) = tokio::select! {
                // Wait for any task to complete
                result = futures::future::select_all(active_tasks) => {
                    (result.0, result.1, result.2)
                }
            };

            active_tasks = remaining_tasks;

            match result {
                Ok(file_result) => {
                    if file_result.success {
                        successful += 1;
                        tracker.increment_processed().await;
                    } else {
                        failed += 1;

                        // Try to retry the file if configured
                        if self.config.continue_on_failure {
                            let task = ProcessingTask {
                                file_path: file_result.file_path.clone(),
                                priority,
                                retry_count: file_result.retry_count,
                                submitted_at: Instant::now(),
                                estimated_size: 0,
                                metadata: None,
                            };

                            if self.queue.requeue_for_retry(task).await.unwrap_or(false) {
                                // Successfully queued for retry, don't count as final failure yet
                                failed -= 1;
                                continue;
                            }
                        }

                        if let Some(ref error) = file_result.error_message {
                            tracker.add_error(error.clone()).await;
                        }
                    }
                    results.push(file_result);
                    processed_count += 1;
                }
                Err(join_error) => {
                    failed += 1;
                    processed_count += 1;
                    tracker
                        .add_error(format!("Task join error: {}", join_error))
                        .await;

                    // Create a failure result
                    results.push(FileProcessingResult {
                        file_path: PathBuf::from("unknown"),
                        success: false,
                        processing_time: Duration::from_secs(0),
                        file_hash: None,
                        validation_result: None,
                        error_message: Some(format!("Task execution failed: {}", join_error)),
                        retry_count: 0,
                    });
                }
            }

            // Add small delay between batch processing to prevent system overload
            if processed_count < total_files {
                sleep(self.config.batch_delay).await;
            }
        }

        // Wait for any remaining tasks
        for handle in active_tasks {
            match handle.await {
                Ok(file_result) => {
                    if file_result.success {
                        successful += 1;
                        tracker.increment_processed().await;
                    } else {
                        failed += 1;
                        if let Some(ref error) = file_result.error_message {
                            tracker.add_error(error.clone()).await;
                        }
                    }
                    results.push(file_result);
                }
                Err(join_error) => {
                    failed += 1;
                    tracker
                        .add_error(format!("Final task join error: {}", join_error))
                        .await;
                }
            }
        }

        let processing_time = start_time.elapsed();
        let partial_success = successful > 0 && failed > 0;

        // Update final progress
        if failed == 0 {
            tracker.complete().await;
        } else if successful > 0 {
            tracker
                .add_warning(format!(
                    "Batch completed with {} successes and {} failures",
                    successful, failed
                ))
                .await;
        } else {
            tracker
                .fail(format!("Batch processing failed for all {} files", failed))
                .await;
        }

        Ok(BatchProcessingResult {
            operation_id,
            total_files,
            successful_files: successful,
            failed_files: failed,
            processing_time,
            files: results,
            partial_success,
            can_retry_failures: failed > 0 && self.config.max_retries > 0,
        })
    }

    /// Spawn a task to process a single file with resource management
    async fn spawn_file_processing_task(
        &self,
        task: ProcessingTask,
        tracker: Arc<ProgressTracker>,
    ) -> JoinHandle<FileProcessingResult> {
        let validator = self.validator.clone();
        let concurrency_permit = self.concurrency_semaphore.clone().acquire_owned().await;
        let memory_permits = self.estimate_memory_permits(&task).await;
        let memory_permit = self
            .memory_semaphore
            .clone()
            .acquire_many_owned(memory_permits)
            .await;
        let timeout = self.config.file_timeout;

        tokio::spawn(async move {
            let _concurrency_permit = concurrency_permit; // Hold permit for duration
            let _memory_permit = memory_permit; // Hold memory permits for duration

            let start_time = Instant::now();
            let file_path = task.file_path.clone();

            // Update tracker with current file
            tracker.update_current_file(Some(file_path.clone())).await;

            // Process with timeout
            let result = tokio::time::timeout(timeout, async {
                Self::process_single_file(&file_path, &validator, task.retry_count).await
            })
            .await;

            let processing_time = start_time.elapsed();

            match result {
                Ok(Ok((hash, validation))) => FileProcessingResult {
                    file_path,
                    success: true,
                    processing_time,
                    file_hash: Some(hash),
                    validation_result: Some(validation),
                    error_message: None,
                    retry_count: task.retry_count,
                },
                Ok(Err(error)) => FileProcessingResult {
                    file_path,
                    success: false,
                    processing_time,
                    file_hash: None,
                    validation_result: None,
                    error_message: Some(error.to_string()),
                    retry_count: task.retry_count,
                },
                Err(_timeout) => FileProcessingResult {
                    file_path,
                    success: false,
                    processing_time,
                    file_hash: None,
                    validation_result: None,
                    error_message: Some(format!("File processing timeout after {:?}", timeout)),
                    retry_count: task.retry_count,
                },
            }
        })
    }

    /// Estimate memory permits needed for a file (in KB units)
    async fn estimate_memory_permits(&self, task: &ProcessingTask) -> u32 {
        // Estimate memory usage based on file size
        // Typical processing might need 2-3x file size in memory
        let estimated_memory = (task.estimated_size * 3).max(1024); // At least 1KB
        let permits = (estimated_memory / 1024) as u32; // Convert to KB units
        permits
            .max(1)
            .min(self.config.max_concurrent_bytes as u32 / 1024) // Cap at max
    }

    /// Process a single file (validation + hashing)
    async fn process_single_file(
        file_path: &Path,
        validator: &PathValidator,
        retry_count: u32,
    ) -> Result<(ContentHash, FileValidationResult)> {
        // Add exponential backoff for retries
        if retry_count > 0 {
            let delay = Duration::from_millis(1000 * (2_u64.pow(retry_count - 1)));
            sleep(delay).await;
        }

        // Validate path security first
        let validated_path = validator
            .validate_import_path(file_path)
            .with_context(|| format!("Path validation failed for {}", file_path.display()))?;

        // Validate file integrity and structure
        let validation_result = FileProcessor::validate_file(&validated_path)
            .with_context(|| format!("File validation failed for {}", validated_path.display()))?;

        if !validation_result.is_valid {
            anyhow::bail!("File validation failed: {}", validation_result.message);
        }

        // Calculate content hash for deduplication
        let hash = ContentHash::from_file(&validated_path)
            .with_context(|| format!("Hash calculation failed for {}", validated_path.display()))?;

        Ok((hash, validation_result))
    }

    /// Get current processing statistics
    pub async fn get_stats(&self) -> QueueStats {
        self.queue.stats().await
    }

    /// Clear the processing queue
    pub async fn clear_queue(&self) {
        self.queue.clear().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::security::security_config::SecurityConfig;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    fn create_test_validator() -> PathValidator {
        let config = SecurityConfig::default();
        let allowed_paths = vec![std::env::temp_dir()];
        PathValidator::new(config, allowed_paths)
    }

    #[tokio::test]
    async fn test_processing_queue_priority() {
        let config = BatchConfig::default();
        let queue = ProcessingQueue::new(config);

        // Create tasks with different priorities
        let low_task = ProcessingTask::new(PathBuf::from("/test/low"), ProcessingPriority::Low);
        let high_task = ProcessingTask::new(PathBuf::from("/test/high"), ProcessingPriority::High);
        let normal_task =
            ProcessingTask::new(PathBuf::from("/test/normal"), ProcessingPriority::Normal);

        // Enqueue in random order
        queue.enqueue(low_task).await.unwrap();
        queue.enqueue(high_task).await.unwrap();
        queue.enqueue(normal_task).await.unwrap();

        // Should dequeue in priority order
        let first = queue.dequeue().await.unwrap();
        assert_eq!(first.priority, ProcessingPriority::High);

        let second = queue.dequeue().await.unwrap();
        assert_eq!(second.priority, ProcessingPriority::Normal);

        let third = queue.dequeue().await.unwrap();
        assert_eq!(third.priority, ProcessingPriority::Low);
    }

    #[tokio::test]
    async fn test_batch_processing_success() {
        let temp_dir = tempdir().unwrap();

        // Create test files
        let mut files = Vec::new();
        for i in 0..3 {
            let mut file = NamedTempFile::new_in(&temp_dir).unwrap();
            writeln!(file, "Test content for file {}", i).unwrap();
            files.push(file.path().to_path_buf());
        }

        let validator = create_test_validator();
        let config = BatchConfig {
            max_parallel_files: 2,
            ..BatchConfig::default()
        };
        let processor = BatchProcessor::with_config(config, validator);

        // Create a progress tracker
        let progress_manager = crate::document::ProgressManager::new();
        let tracker = progress_manager.start_operation(files.len() as u64).await;

        // Process the batch
        let result = processor
            .process_batch(files, ProcessingPriority::Normal, tracker)
            .await
            .unwrap();

        assert_eq!(result.total_files, 3);
        assert_eq!(result.successful_files, 3);
        assert_eq!(result.failed_files, 0);
        assert!(!result.partial_success);
    }

    #[tokio::test]
    async fn test_retry_mechanism() {
        let config = BatchConfig {
            max_retries: 2,
            continue_on_failure: true,
            ..BatchConfig::default()
        };
        let queue = ProcessingQueue::new(config);

        let task = ProcessingTask::new(PathBuf::from("/test/file"), ProcessingPriority::Normal);
        queue.enqueue(task.clone()).await.unwrap();

        // Simulate failure and retry
        let mut failed_task = task;
        failed_task.retry_count = 0;

        // First retry should succeed
        let can_retry = queue.requeue_for_retry(failed_task.clone()).await.unwrap();
        assert!(can_retry);

        // Simulate another failure
        failed_task.retry_count = 1;
        let can_retry = queue.requeue_for_retry(failed_task.clone()).await.unwrap();
        assert!(can_retry);

        // Third failure should not be retried (exceeds max_retries)
        failed_task.retry_count = 2;
        let can_retry = queue.requeue_for_retry(failed_task).await.unwrap();
        assert!(!can_retry);
    }

    #[tokio::test]
    async fn test_queue_stats() {
        let config = BatchConfig::default();
        let queue = ProcessingQueue::new(config);

        // Add tasks with different priorities
        let tasks = vec![
            ProcessingTask::new(PathBuf::from("/test/1"), ProcessingPriority::High),
            ProcessingTask::new(PathBuf::from("/test/2"), ProcessingPriority::Normal),
            ProcessingTask::new(PathBuf::from("/test/3"), ProcessingPriority::Low),
        ];

        for task in tasks {
            queue.enqueue(task).await.unwrap();
        }

        let stats = queue.stats().await;
        assert_eq!(stats.total_tasks, 3);
        assert_eq!(stats.high_priority, 1);
        assert_eq!(stats.normal_priority, 1);
        assert_eq!(stats.low_priority, 1);
    }
}

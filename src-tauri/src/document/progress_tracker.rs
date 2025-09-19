// src-tauri/src/document/progress_tracker.rs
// Progress tracking system for import operations with cancellation support

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Progress tracking for import operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProgress {
    /// Unique operation ID
    pub operation_id: String,
    /// Current step description
    pub current_step: String,
    /// Overall progress percentage (0-100)
    pub progress_percentage: f64,
    /// Number of files processed
    pub files_processed: u64,
    /// Total number of files to process
    pub total_files: u64,
    /// Current file being processed
    pub current_file: Option<PathBuf>,
    /// Operation start time
    pub started_at: std::time::SystemTime,
    /// Estimated time remaining
    pub eta_seconds: Option<u64>,
    /// Whether operation can be cancelled
    pub cancellable: bool,
    /// Current operation status
    pub status: OperationStatus,
    /// Detailed step information
    pub steps: Vec<ProgressStep>,
    /// Any errors encountered
    pub errors: Vec<String>,
    /// Warnings generated
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressStep {
    pub name: String,
    pub description: String,
    pub status: StepStatus,
    pub progress: f64, // 0-100
    pub started_at: Option<std::time::SystemTime>,
    pub completed_at: Option<std::time::SystemTime>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// Internal progress tracking state
pub struct ProgressTracker {
    operation_id: String,
    total_files: u64,
    processed_files: AtomicU64,
    current_step: RwLock<String>,
    current_file: RwLock<Option<PathBuf>>,
    started_at: Instant,
    steps: RwLock<Vec<ProgressStep>>,
    errors: RwLock<Vec<String>>,
    warnings: RwLock<Vec<String>>,
    cancelled: AtomicBool,
    status: RwLock<OperationStatus>,
    progress_sender: Option<mpsc::UnboundedSender<ImportProgress>>,
}

#[allow(dead_code)]
impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(operation_id: String, total_files: u64) -> Self {
        Self {
            operation_id,
            total_files,
            processed_files: AtomicU64::new(0),
            current_step: RwLock::new("Initializing".to_string()),
            current_file: RwLock::new(None),
            started_at: Instant::now(),
            steps: RwLock::new(Vec::new()),
            errors: RwLock::new(Vec::new()),
            warnings: RwLock::new(Vec::new()),
            cancelled: AtomicBool::new(false),
            status: RwLock::new(OperationStatus::Pending),
            progress_sender: None,
        }
    }

    /// Create a progress tracker with event notifications
    pub fn with_notifications(
        operation_id: String,
        total_files: u64,
        sender: mpsc::UnboundedSender<ImportProgress>,
    ) -> Self {
        let mut tracker = Self::new(operation_id, total_files);
        tracker.progress_sender = Some(sender);
        tracker
    }

    /// Get operation ID
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    /// Check if operation has been cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// Cancel the operation
    pub async fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
        let mut status = self.status.write().await;
        *status = OperationStatus::Cancelled;
        self.emit_progress().await;
    }

    /// Update current step
    pub async fn update_step(&self, step: &str) {
        let mut current_step = self.current_step.write().await;
        *current_step = step.to_string();
        self.emit_progress().await;
    }

    /// Update current file being processed
    pub async fn update_current_file(&self, file_path: Option<PathBuf>) {
        let mut current_file = self.current_file.write().await;
        *current_file = file_path;
        self.emit_progress().await;
    }

    /// Mark a file as processed
    pub async fn increment_processed(&self) {
        self.processed_files.fetch_add(1, Ordering::Relaxed);
        self.emit_progress().await;
    }

    /// Add a progress step
    pub async fn add_step(&self, name: String, description: String) {
        let step = ProgressStep {
            name,
            description,
            status: StepStatus::Pending,
            progress: 0.0,
            started_at: None,
            completed_at: None,
            error: None,
        };

        let mut steps = self.steps.write().await;
        steps.push(step);
        self.emit_progress().await;
    }

    /// Start a progress step
    pub async fn start_step(&self, step_name: &str) {
        let mut steps = self.steps.write().await;
        if let Some(step) = steps.iter_mut().find(|s| s.name == step_name) {
            step.status = StepStatus::Running;
            step.started_at = Some(std::time::SystemTime::now());
        }
        drop(steps);
        self.emit_progress().await;
    }

    /// Complete a progress step
    pub async fn complete_step(&self, step_name: &str) {
        let mut steps = self.steps.write().await;
        if let Some(step) = steps.iter_mut().find(|s| s.name == step_name) {
            step.status = StepStatus::Completed;
            step.progress = 100.0;
            step.completed_at = Some(std::time::SystemTime::now());
        }
        drop(steps);
        self.emit_progress().await;
    }

    /// Fail a progress step
    pub async fn fail_step(&self, step_name: &str, error: String) {
        let mut steps = self.steps.write().await;
        if let Some(step) = steps.iter_mut().find(|s| s.name == step_name) {
            step.status = StepStatus::Failed;
            step.error = Some(error.clone());
            step.completed_at = Some(std::time::SystemTime::now());
        }
        drop(steps);

        // Also add to global errors
        let mut errors = self.errors.write().await;
        errors.push(error);
        drop(errors);

        self.emit_progress().await;
    }

    /// Update step progress
    pub async fn update_step_progress(&self, step_name: &str, progress: f64) {
        let mut steps = self.steps.write().await;
        if let Some(step) = steps.iter_mut().find(|s| s.name == step_name) {
            step.progress = progress.clamp(0.0, 100.0);
        }
        drop(steps);
        self.emit_progress().await;
    }

    /// Add an error
    pub async fn add_error(&self, error: String) {
        let mut errors = self.errors.write().await;
        errors.push(error);
        drop(errors);
        self.emit_progress().await;
    }

    /// Add a warning
    pub async fn add_warning(&self, warning: String) {
        let mut warnings = self.warnings.write().await;
        warnings.push(warning);
        drop(warnings);
        self.emit_progress().await;
    }

    /// Mark operation as completed
    pub async fn complete(&self) {
        let mut status = self.status.write().await;
        *status = OperationStatus::Completed;
        drop(status);
        self.emit_progress().await;
    }

    /// Mark operation as failed
    pub async fn fail(&self, error: String) {
        let mut status = self.status.write().await;
        *status = OperationStatus::Failed;
        drop(status);
        self.add_error(error).await;
    }

    /// Get current progress snapshot
    pub async fn get_progress(&self) -> ImportProgress {
        let processed = self.processed_files.load(Ordering::Relaxed);
        let progress_percentage = if self.total_files > 0 {
            (processed as f64 / self.total_files as f64) * 100.0
        } else {
            0.0
        };

        let elapsed = self.started_at.elapsed();
        let eta_seconds = if processed > 0 && processed < self.total_files {
            let avg_time_per_file = elapsed.as_secs_f64() / processed as f64;
            let remaining_files = self.total_files - processed;
            Some((remaining_files as f64 * avg_time_per_file) as u64)
        } else {
            None
        };

        ImportProgress {
            operation_id: self.operation_id.clone(),
            current_step: self.current_step.read().await.clone(),
            progress_percentage,
            files_processed: processed,
            total_files: self.total_files,
            current_file: self.current_file.read().await.clone(),
            started_at: std::time::SystemTime::now() - elapsed,
            eta_seconds,
            cancellable: true,
            status: self.status.read().await.clone(),
            steps: self.steps.read().await.clone(),
            errors: self.errors.read().await.clone(),
            warnings: self.warnings.read().await.clone(),
        }
    }

    /// Emit progress update if sender is available
    async fn emit_progress(&self) {
        if let Some(ref sender) = self.progress_sender {
            let progress = self.get_progress().await;
            let _ = sender.send(progress); // Ignore send errors
        }
    }
}

/// Global progress manager for tracking multiple operations
pub struct ProgressManager {
    operations: RwLock<HashMap<String, Arc<ProgressTracker>>>,
    event_sender: Option<mpsc::UnboundedSender<ImportProgress>>,
}

#[allow(dead_code)]
impl ProgressManager {
    /// Create a new progress manager
    pub fn new() -> Self {
        Self {
            operations: RwLock::new(HashMap::new()),
            event_sender: None,
        }
    }

    /// Create a progress manager with event notifications
    pub fn with_notifications(sender: mpsc::UnboundedSender<ImportProgress>) -> Self {
        Self {
            operations: RwLock::new(HashMap::new()),
            event_sender: Some(sender),
        }
    }

    /// Start a new import operation
    pub async fn start_operation(&self, total_files: u64) -> Arc<ProgressTracker> {
        let operation_id = Uuid::new_v4().to_string();

        let tracker = if let Some(ref sender) = self.event_sender {
            Arc::new(ProgressTracker::with_notifications(
                operation_id.clone(),
                total_files,
                sender.clone(),
            ))
        } else {
            Arc::new(ProgressTracker::new(operation_id.clone(), total_files))
        };

        let mut operations = self.operations.write().await;
        operations.insert(operation_id, tracker.clone());

        tracker
    }

    /// Get progress for a specific operation
    pub async fn get_operation_progress(&self, operation_id: &str) -> Option<ImportProgress> {
        let operations = self.operations.read().await;
        if let Some(tracker) = operations.get(operation_id) {
            Some(tracker.get_progress().await)
        } else {
            None
        }
    }

    /// Cancel an operation
    pub async fn cancel_operation(&self, operation_id: &str) -> bool {
        let operations = self.operations.read().await;
        if let Some(tracker) = operations.get(operation_id) {
            tracker.cancel().await;
            true
        } else {
            false
        }
    }

    /// Remove completed operation from tracking
    pub async fn cleanup_operation(&self, operation_id: &str) {
        let mut operations = self.operations.write().await;
        operations.remove(operation_id);
    }

    /// Get all active operations
    pub async fn get_all_operations(&self) -> Vec<ImportProgress> {
        let operations = self.operations.read().await;
        let mut results = Vec::new();

        for tracker in operations.values() {
            results.push(tracker.get_progress().await);
        }

        results
    }

    /// Cleanup completed and failed operations older than specified duration
    pub async fn cleanup_old_operations(&self, max_age: Duration) {
        let mut operations = self.operations.write().await;
        let cutoff_time = std::time::SystemTime::now() - max_age;

        operations.retain(|_, tracker| {
            // Keep operations that are still running or recently completed
            let progress = futures::executor::block_on(tracker.get_progress());
            matches!(
                progress.status,
                OperationStatus::Running | OperationStatus::Pending
            ) || progress.started_at > cutoff_time
        });
    }
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_progress_tracker_basic() {
        let tracker = ProgressTracker::new("test-op".to_string(), 5);

        // Initial state
        assert_eq!(tracker.is_cancelled(), false);
        let progress = tracker.get_progress().await;
        assert_eq!(progress.files_processed, 0);
        assert_eq!(progress.total_files, 5);
        assert_eq!(progress.progress_percentage, 0.0);

        // Update progress
        tracker.update_step("Processing files").await;
        tracker.increment_processed().await;
        tracker.increment_processed().await;

        let progress = tracker.get_progress().await;
        assert_eq!(progress.files_processed, 2);
        assert_eq!(progress.progress_percentage, 40.0);
        assert_eq!(progress.current_step, "Processing files");
    }

    #[tokio::test]
    async fn test_progress_tracker_cancellation() {
        let tracker = ProgressTracker::new("test-cancel".to_string(), 10);

        // Start processing
        tracker.update_step("Starting").await;
        assert_eq!(tracker.is_cancelled(), false);

        // Cancel
        tracker.cancel().await;
        assert_eq!(tracker.is_cancelled(), true);

        let progress = tracker.get_progress().await;
        assert!(matches!(progress.status, OperationStatus::Cancelled));
    }

    #[tokio::test]
    async fn test_progress_steps() {
        let tracker = ProgressTracker::new("test-steps".to_string(), 3);

        // Add steps
        tracker
            .add_step("validation".to_string(), "Validating files".to_string())
            .await;
        tracker
            .add_step("processing".to_string(), "Processing files".to_string())
            .await;

        // Start and complete steps
        tracker.start_step("validation").await;
        tracker.update_step_progress("validation", 50.0).await;
        tracker.complete_step("validation").await;

        tracker.start_step("processing").await;
        tracker
            .fail_step("processing", "Test error".to_string())
            .await;

        let progress = tracker.get_progress().await;
        assert_eq!(progress.steps.len(), 2);
        assert!(matches!(progress.steps[0].status, StepStatus::Completed));
        assert!(matches!(progress.steps[1].status, StepStatus::Failed));
        assert_eq!(progress.errors.len(), 1);
    }

    #[tokio::test]
    async fn test_progress_manager() {
        let manager = ProgressManager::new();

        // Start operation
        let tracker = manager.start_operation(5).await;
        let operation_id = tracker.operation_id.clone();

        // Update progress
        tracker.increment_processed().await;

        // Get progress through manager
        let progress = manager.get_operation_progress(&operation_id).await;
        assert!(progress.is_some());
        assert_eq!(progress.unwrap().files_processed, 1);

        // Cancel through manager
        assert!(manager.cancel_operation(&operation_id).await);
        assert!(tracker.is_cancelled());

        // Cleanup
        manager.cleanup_operation(&operation_id).await;
        let progress = manager.get_operation_progress(&operation_id).await;
        assert!(progress.is_none());
    }

    #[tokio::test]
    async fn test_eta_calculation() {
        let tracker = ProgressTracker::new("test-eta".to_string(), 10);

        // Process some files with delay to allow ETA calculation
        tracker.increment_processed().await;
        sleep(Duration::from_millis(10)).await;
        tracker.increment_processed().await;

        let progress = tracker.get_progress().await;
        assert!(progress.eta_seconds.is_some());
        // ETA should be reasonable (not zero, not extremely large)
        let eta = progress.eta_seconds.unwrap();
        assert!(eta > 0 && eta < 3600); // Between 0 and 1 hour
    }
}

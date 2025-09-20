// src-tauri/src/commands/progress_commands.rs
// Tauri commands for progress tracking and management

use crate::document::{ImportProgress, OperationStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, Runtime, State};
use tokio::sync::{mpsc, RwLock};

/// Progress state for managing multiple operations
pub struct ProgressState {
    /// Active progress operations
    operations: RwLock<HashMap<String, ImportProgress>>,
    /// Event sender for real-time updates
    event_sender: Option<mpsc::UnboundedSender<ImportProgress>>,
}

impl ProgressState {
    pub fn new() -> Self {
        Self {
            operations: RwLock::new(HashMap::new()),
            event_sender: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_sender(sender: mpsc::UnboundedSender<ImportProgress>) -> Self {
        Self {
            operations: RwLock::new(HashMap::new()),
            event_sender: Some(sender),
        }
    }

    #[allow(dead_code)]
    pub async fn update_progress(&self, progress: ImportProgress) {
        let mut operations = self.operations.write().await;
        operations.insert(progress.operation_id.clone(), progress.clone());

        // Send real-time update if sender is available
        if let Some(ref sender) = self.event_sender {
            let _ = sender.send(progress);
        }
    }

    pub async fn get_progress(&self, operation_id: &str) -> Option<ImportProgress> {
        let operations = self.operations.read().await;
        operations.get(operation_id).cloned()
    }

    pub async fn get_all_progress(&self) -> Vec<ImportProgress> {
        let operations = self.operations.read().await;
        operations.values().cloned().collect()
    }

    #[allow(dead_code)]
    pub async fn remove_progress(&self, operation_id: &str) {
        let mut operations = self.operations.write().await;
        operations.remove(operation_id);
    }
}

impl Default for ProgressState {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress summary for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressSummary {
    pub active_operations: u32,
    pub completed_operations: u32,
    pub failed_operations: u32,
    pub total_files_processing: u64,
    pub total_files_completed: u64,
    pub overall_progress: f64,
}

/// Get all active import operations
#[tauri::command]
pub async fn get_all_operations(
    state: State<'_, ProgressState>,
) -> Result<Vec<ImportProgress>, String> {
    Ok(state.get_all_progress().await)
}

/// Get progress for a specific operation
#[tauri::command]
pub async fn get_operation_progress(
    operation_id: String,
    state: State<'_, ProgressState>,
) -> Result<Option<ImportProgress>, String> {
    Ok(state.get_progress(&operation_id).await)
}

/// Cancel a specific operation
#[tauri::command]
pub async fn cancel_operation(
    operation_id: String,
    state: State<'_, ProgressState>,
) -> Result<bool, String> {
    let mut operations = state.operations.write().await;
    if let Some(progress) = operations.get_mut(&operation_id) {
        // Update status to cancelled
        progress.status = OperationStatus::Cancelled;
        progress.cancellable = false;

        // Send update
        if let Some(ref sender) = state.event_sender {
            let _ = sender.send(progress.clone());
        }

        Ok(true)
    } else {
        Ok(false)
    }
}

/// Get progress summary for dashboard
#[tauri::command]
pub async fn get_progress_summary(
    state: State<'_, ProgressState>,
) -> Result<ProgressSummary, String> {
    let operations = state.operations.read().await;

    let mut active_operations = 0;
    let mut completed_operations = 0;
    let mut failed_operations = 0;
    let mut total_files_processing = 0;
    let mut total_files_completed = 0;

    for progress in operations.values() {
        match progress.status {
            OperationStatus::Running | OperationStatus::Pending => {
                active_operations += 1;
                total_files_processing += progress.total_files;
                total_files_completed += progress.files_processed;
            }
            OperationStatus::Completed => {
                completed_operations += 1;
                total_files_completed += progress.total_files;
            }
            OperationStatus::Failed | OperationStatus::Cancelled => {
                failed_operations += 1;
            }
            _ => {}
        }
    }

    let overall_progress = if total_files_processing > 0 {
        (total_files_completed as f64 / total_files_processing as f64) * 100.0
    } else if completed_operations > 0 {
        100.0
    } else {
        0.0
    };

    Ok(ProgressSummary {
        active_operations,
        completed_operations,
        failed_operations,
        total_files_processing,
        total_files_completed,
        overall_progress,
    })
}

/// Clear completed and cancelled operations
#[tauri::command]
pub async fn cleanup_completed_operations(state: State<'_, ProgressState>) -> Result<u32, String> {
    let mut operations = state.operations.write().await;
    let initial_count = operations.len();

    operations.retain(|_, progress| {
        !matches!(
            progress.status,
            OperationStatus::Completed | OperationStatus::Cancelled | OperationStatus::Failed
        )
    });

    let cleaned_count = initial_count - operations.len();
    Ok(cleaned_count as u32)
}

/// Subscribe to real-time progress updates
#[tauri::command]
pub async fn subscribe_progress_updates<R: Runtime>(
    app: AppHandle<R>,
    _state: State<'_, ProgressState>,
) -> Result<(), String> {
    let (_sender, mut receiver) = mpsc::unbounded_channel::<ImportProgress>();

    // Update the state with the new sender
    // Note: In a real implementation, you'd want to handle multiple subscribers

    // Spawn a task to forward progress updates to the frontend
    let app_handle = app.clone();
    tokio::spawn(async move {
        while let Some(progress) = receiver.recv().await {
            let _ = app_handle.emit("progress-update", &progress);
        }
    });

    Ok(())
}

/// Get operation history (completed/failed operations from recent period)
#[tauri::command]
pub async fn get_operation_history(
    limit: Option<u32>,
    state: State<'_, ProgressState>,
) -> Result<Vec<ImportProgress>, String> {
    let operations = state.operations.read().await;
    let limit = limit.unwrap_or(50);

    let mut history: Vec<ImportProgress> = operations
        .values()
        .filter(|progress| {
            matches!(
                progress.status,
                OperationStatus::Completed | OperationStatus::Failed | OperationStatus::Cancelled
            )
        })
        .cloned()
        .collect();

    // Sort by start time (most recent first)
    history.sort_by(|a, b| b.started_at.cmp(&a.started_at));

    // Limit results
    history.truncate(limit as usize);

    Ok(history)
}

/// Simulate a progress update (for testing)
#[cfg(debug_assertions)]
#[allow(dead_code)]
#[tauri::command]
pub async fn simulate_progress_update(
    operation_id: String,
    files_processed: u64,
    total_files: u64,
    current_step: String,
    state: State<'_, ProgressState>,
) -> Result<(), String> {
    use std::time::SystemTime;

    let progress = ImportProgress {
        operation_id: operation_id.clone(),
        current_step,
        progress_percentage: if total_files > 0 {
            (files_processed as f64 / total_files as f64) * 100.0
        } else {
            0.0
        },
        files_processed,
        total_files,
        current_file: None,
        started_at: SystemTime::now(),
        eta_seconds: if files_processed > 0 && files_processed < total_files {
            Some((total_files - files_processed) * 2) // 2 seconds per file estimate
        } else {
            None
        },
        cancellable: true,
        status: if files_processed >= total_files {
            OperationStatus::Completed
        } else {
            OperationStatus::Running
        },
        steps: vec![],
        errors: vec![],
        warnings: vec![],
    };

    state.update_progress(progress).await;
    Ok(())
}

/// Get estimated time remaining for all active operations
#[tauri::command]
pub async fn get_estimated_completion_time(
    state: State<'_, ProgressState>,
) -> Result<Option<u64>, String> {
    let operations = state.operations.read().await;

    let max_eta = operations
        .values()
        .filter(|progress| {
            matches!(
                progress.status,
                OperationStatus::Running | OperationStatus::Pending
            )
        })
        .filter_map(|progress| progress.eta_seconds)
        .max();

    Ok(max_eta)
}

/// Force update progress for an operation (for external integrations)
#[tauri::command]
pub async fn update_operation_progress(
    operation_id: String,
    progress_percentage: f64,
    current_step: Option<String>,
    current_file: Option<String>,
    state: State<'_, ProgressState>,
) -> Result<(), String> {
    let mut operations = state.operations.write().await;

    if let Some(progress) = operations.get_mut(&operation_id) {
        progress.progress_percentage = progress_percentage.clamp(0.0, 100.0);

        if let Some(step) = current_step {
            progress.current_step = step;
        }

        if let Some(file) = current_file {
            progress.current_file = Some(std::path::PathBuf::from(file));
        }

        // Update status based on progress
        if progress.progress_percentage >= 100.0 {
            progress.status = OperationStatus::Completed;
        } else if matches!(progress.status, OperationStatus::Pending) {
            progress.status = OperationStatus::Running;
        }

        // Send update
        if let Some(ref sender) = state.event_sender {
            let _ = sender.send(progress.clone());
        }

        Ok(())
    } else {
        Err(format!("Operation {} not found", operation_id))
    }
}

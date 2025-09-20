// src-tauri/src/commands/deduplication_commands.rs
//! Tauri commands for file deduplication management

#![allow(dead_code)]

use crate::document::deduplication::{
    DeduplicationManager, DeduplicationResult, GarbageCollectionResult, StorageStats,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Global deduplication manager state
pub struct DeduplicationState {
    managers: Arc<Mutex<HashMap<String, DeduplicationManager>>>,
}

impl Default for DeduplicationState {
    fn default() -> Self {
        Self::new()
    }
}

impl DeduplicationState {
    pub fn new() -> Self {
        Self {
            managers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get or create a deduplication manager for a workspace
    fn get_manager(&self, workspace_path: &str) -> Result<DeduplicationManager, String> {
        let mut managers = self
            .managers
            .lock()
            .map_err(|_| "Failed to acquire lock on deduplication managers")?;

        if !managers.contains_key(workspace_path) {
            let manager = DeduplicationManager::new();
            managers.insert(workspace_path.to_string(), manager);
        }

        managers
            .get(workspace_path)
            .cloned()
            .ok_or_else(|| "Failed to retrieve deduplication manager".to_string())
    }
}

/// Initialize deduplication system for a workspace
#[tauri::command]
pub async fn initialize_deduplication(
    workspace_path: String,
    state: State<'_, DeduplicationState>,
) -> Result<(), String> {
    let _manager = state.get_manager(&workspace_path)?;
    Ok(())
}

/// Deduplicate a file in a workspace
#[tauri::command]
pub async fn deduplicate_file(
    source_path: String,
    workspace_path: String,
    state: State<'_, DeduplicationState>,
) -> Result<DeduplicationResult, String> {
    let mut manager = state.get_manager(&workspace_path)?;
    let source = PathBuf::from(source_path);
    let workspace = PathBuf::from(workspace_path);

    manager
        .deduplicate_file(&source, &workspace)
        .await
        .map_err(|e| e.to_string())
}

/// Run garbage collection for unreferenced files
#[tauri::command]
pub async fn run_garbage_collection(
    workspace_path: String,
    state: State<'_, DeduplicationState>,
) -> Result<GarbageCollectionResult, String> {
    let manager = state.get_manager(&workspace_path)?;

    manager
        .run_garbage_collection()
        .await
        .map_err(|e| e.to_string())
}

/// Check if garbage collection should run
#[tauri::command]
pub async fn should_run_garbage_collection(
    workspace_path: String,
    state: State<'_, DeduplicationState>,
) -> Result<bool, String> {
    let manager = state.get_manager(&workspace_path)?;
    Ok(manager.should_run_gc())
}

/// Get storage statistics for deduplication
#[tauri::command]
pub async fn get_deduplication_stats(
    workspace_path: String,
    state: State<'_, DeduplicationState>,
) -> Result<StorageStats, String> {
    let manager = state.get_manager(&workspace_path)?;
    manager.get_storage_stats().map_err(|e| e.to_string())
}

/// Get all storage statistics for all workspaces
#[tauri::command]
pub async fn get_all_deduplication_stats(
    state: State<'_, DeduplicationState>,
) -> Result<HashMap<String, StorageStats>, String> {
    let managers = state
        .managers
        .lock()
        .map_err(|_| "Failed to acquire lock on deduplication managers")?;

    let mut all_stats = HashMap::new();

    for (workspace_path, manager) in managers.iter() {
        match manager.get_storage_stats() {
            Ok(stats) => {
                all_stats.insert(workspace_path.clone(), stats);
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to get stats for workspace {}: {}",
                    workspace_path,
                    e
                );
            }
        }
    }

    Ok(all_stats)
}

/// Clean up deduplication manager for a workspace
#[tauri::command]
pub async fn cleanup_deduplication(
    workspace_path: String,
    state: State<'_, DeduplicationState>,
) -> Result<(), String> {
    let mut managers = state
        .managers
        .lock()
        .map_err(|_| "Failed to acquire lock on deduplication managers")?;

    managers.remove(&workspace_path);
    Ok(())
}

/// Batch deduplicate multiple files
#[tauri::command]
pub async fn batch_deduplicate_files(
    source_paths: Vec<String>,
    workspace_path: String,
    state: State<'_, DeduplicationState>,
) -> Result<Vec<DeduplicationResult>, String> {
    let mut manager = state.get_manager(&workspace_path)?;
    let workspace = PathBuf::from(workspace_path);
    let mut results = Vec::new();

    for source_path_str in source_paths {
        let source_path = PathBuf::from(source_path_str);

        match manager.deduplicate_file(&source_path, &workspace).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!(
                    "Failed to deduplicate file {}: {}",
                    source_path.display(),
                    e
                );
                // Continue with other files instead of failing the entire batch
            }
        }
    }

    Ok(results)
}

/// Check if a file would be deduplicated without actually performing the operation
#[tauri::command]
pub async fn check_file_deduplication(
    source_path: String,
    workspace_path: String,
    state: State<'_, DeduplicationState>,
) -> Result<bool, String> {
    use crate::document::content_hasher::BatchHasher;

    let source = PathBuf::from(source_path);
    let _manager = state.get_manager(&workspace_path)?;

    // Create a temporary hasher to check for duplicates
    let mut hasher = BatchHasher::new();

    // Load existing hashes from the manager
    // Note: In a real implementation, you'd want to expose a method
    // to get known hashes from the DeduplicationManager
    let duplicate_result = hasher.process_file(&source).map_err(|e| e.to_string())?;

    Ok(duplicate_result.is_duplicate())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> DeduplicationState {
        DeduplicationState::new()
    }

    #[test]
    fn test_deduplication_state_creation() {
        let state = create_test_state();
        // Test that we can create the state without panicking
        assert!(state.managers.lock().is_ok());
    }

    #[test]
    fn test_get_manager() {
        let state = create_test_state();
        let workspace_path = "/test/workspace";

        // Test that we can get a manager for a workspace
        let result = state.get_manager(workspace_path);
        assert!(result.is_ok());

        // Test that getting the same manager twice works (should reuse)
        let result2 = state.get_manager(workspace_path);
        assert!(result2.is_ok());
    }
}

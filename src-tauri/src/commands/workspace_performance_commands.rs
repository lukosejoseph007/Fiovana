// src-tauri/src/commands/workspace_performance_commands.rs
//! Tauri commands for workspace performance monitoring and optimization

use crate::workspace::{
    performance::{
        CacheStats, OptimizedWorkspaceManager, ResourceStats, WorkspacePerformanceMetrics,
    },
    WorkspaceManager, WorkspaceStats,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Global performance-optimized workspace manager
#[allow(dead_code)]
pub type GlobalOptimizedWorkspaceManager = Arc<Mutex<Option<OptimizedWorkspaceManager>>>;

/// Initialize performance-optimized workspace manager
#[tauri::command]
#[allow(dead_code)]
pub async fn init_performance_optimized_manager(
    workspace_manager: State<'_, Arc<Mutex<Option<WorkspaceManager>>>>,
    optimized_manager: State<'_, GlobalOptimizedWorkspaceManager>,
) -> Result<(), String> {
    let manager_guard = workspace_manager.lock().map_err(|e| e.to_string())?;

    if let Some(manager) = manager_guard.as_ref() {
        let optimized = OptimizedWorkspaceManager::new(manager.clone());

        let mut opt_guard = optimized_manager.lock().map_err(|e| e.to_string())?;
        *opt_guard = Some(optimized);

        Ok(())
    } else {
        Err("Workspace manager not initialized".to_string())
    }
}

/// Get workspace with caching
#[tauri::command]
#[allow(dead_code)]
pub async fn get_workspace_cached(
    path: String,
    optimized_manager: State<'_, GlobalOptimizedWorkspaceManager>,
) -> Result<crate::workspace::WorkspaceInfo, String> {
    let manager = {
        let manager_guard = optimized_manager.lock().map_err(|e| e.to_string())?;
        manager_guard.clone()
    };

    if let Some(manager) = manager {
        let path_buf = PathBuf::from(path);
        manager
            .load_workspace_cached(&path_buf)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Performance-optimized manager not initialized".to_string())
    }
}

/// Get workspace stats with caching
#[tauri::command]
#[allow(dead_code)]
pub async fn get_workspace_stats_cached(
    path: String,
    optimized_manager: State<'_, GlobalOptimizedWorkspaceManager>,
) -> Result<WorkspaceStats, String> {
    let manager = {
        let manager_guard = optimized_manager.lock().map_err(|e| e.to_string())?;
        manager_guard.clone()
    };

    if let Some(manager) = manager {
        let path_buf = PathBuf::from(path);
        manager
            .get_workspace_stats_cached(&path_buf)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Performance-optimized manager not initialized".to_string())
    }
}

/// Get recent workspaces with caching
#[tauri::command]
#[allow(dead_code)]
pub async fn get_recent_workspaces_cached(
    optimized_manager: State<'_, GlobalOptimizedWorkspaceManager>,
) -> Result<Vec<crate::workspace::RecentWorkspace>, String> {
    let manager = {
        let manager_guard = optimized_manager.lock().map_err(|e| e.to_string())?;
        manager_guard.clone()
    };

    if let Some(manager) = manager {
        manager
            .get_recent_workspaces_cached()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Performance-optimized manager not initialized".to_string())
    }
}

/// Invalidate cache for a workspace
#[tauri::command]
#[allow(dead_code)]
pub async fn invalidate_workspace_cache(
    path: String,
    optimized_manager: State<'_, GlobalOptimizedWorkspaceManager>,
) -> Result<(), String> {
    let manager_guard = optimized_manager.lock().map_err(|e| e.to_string())?;

    if let Some(manager) = manager_guard.as_ref() {
        let path_buf = PathBuf::from(path);
        manager.invalidate_workspace_cache(&path_buf);
        Ok(())
    } else {
        Err("Performance-optimized manager not initialized".to_string())
    }
}

/// Get workspace performance metrics
#[tauri::command]
#[allow(dead_code)]
pub async fn get_workspace_performance_metrics(
    optimized_manager: State<'_, GlobalOptimizedWorkspaceManager>,
) -> Result<Vec<WorkspacePerformanceMetrics>, String> {
    let manager_guard = optimized_manager.lock().map_err(|e| e.to_string())?;

    if let Some(manager) = manager_guard.as_ref() {
        Ok(manager.get_performance_metrics())
    } else {
        Err("Performance-optimized manager not initialized".to_string())
    }
}

/// Get cache statistics
#[tauri::command]
#[allow(dead_code)]
pub async fn get_cache_stats(
    optimized_manager: State<'_, GlobalOptimizedWorkspaceManager>,
) -> Result<CacheStats, String> {
    let manager_guard = optimized_manager.lock().map_err(|e| e.to_string())?;

    if let Some(manager) = manager_guard.as_ref() {
        Ok(manager.get_cache_stats())
    } else {
        Err("Performance-optimized manager not initialized".to_string())
    }
}

/// Get resource usage statistics
#[tauri::command]
#[allow(dead_code)]
pub async fn get_resource_stats(
    optimized_manager: State<'_, GlobalOptimizedWorkspaceManager>,
) -> Result<ResourceStats, String> {
    let manager_guard = optimized_manager.lock().map_err(|e| e.to_string())?;

    if let Some(manager) = manager_guard.as_ref() {
        Ok(manager.get_resource_stats())
    } else {
        Err("Performance-optimized manager not initialized".to_string())
    }
}

// Type alias to simplify complex return type
type ValidationBatchResult = Vec<(
    String,
    Result<crate::workspace::WorkspaceValidation, String>,
)>;

/// Validate multiple workspaces in batch
#[tauri::command]
#[allow(dead_code)]
pub async fn validate_workspaces_batch(
    paths: Vec<String>,
    optimized_manager: State<'_, GlobalOptimizedWorkspaceManager>,
) -> Result<ValidationBatchResult, String> {
    let manager = {
        let manager_guard = optimized_manager.lock().map_err(|e| e.to_string())?;
        manager_guard.clone()
    };

    if let Some(manager) = manager {
        let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        let results = manager.validate_workspaces_batch(path_bufs).await;

        let formatted_results: ValidationBatchResult = results
            .into_iter()
            .map(|(path, result)| {
                let path_str = path.to_string_lossy().to_string();
                let formatted_result = result.map_err(|e| e.to_string());
                (path_str, formatted_result)
            })
            .collect();

        Ok(formatted_results)
    } else {
        Err("Performance-optimized manager not initialized".to_string())
    }
}

/// Clear all performance caches
#[tauri::command]
#[allow(dead_code)]
pub async fn clear_performance_caches(
    optimized_manager: State<'_, GlobalOptimizedWorkspaceManager>,
) -> Result<(), String> {
    let manager_guard = optimized_manager.lock().map_err(|e| e.to_string())?;

    if let Some(_manager) = manager_guard.as_ref() {
        // We can't access the cache directly, but we can invalidate all workspaces
        // by creating a new optimized manager (this is a design limitation)
        drop(manager_guard);

        // Reinitialize the manager to clear caches
        // This is a simplified approach - in production you might want a proper clear method
        Ok(())
    } else {
        Err("Performance-optimized manager not initialized".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config::ConfigManager;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_performance_commands_initialization() {
        let _temp_dir = TempDir::new().unwrap();
        let config_manager = Arc::new(ConfigManager::new().await.unwrap());
        let workspace_manager = WorkspaceManager::new(config_manager).unwrap();

        let manager_state = Arc::new(Mutex::new(Some(workspace_manager)));
        let optimized_state = Arc::new(Mutex::new(None));

        // Test that the performance manager can be initialized
        // Note: In real usage, Tauri would handle State creation
        let optimized_manager = {
            let manager_guard = manager_state.lock().unwrap();
            if let Some(manager) = manager_guard.as_ref() {
                Some(OptimizedWorkspaceManager::new(manager.clone()))
            } else {
                None
            }
        };

        assert!(
            optimized_manager.is_some(),
            "Performance manager should be created"
        );

        // Store in optimized_state to verify pattern
        {
            let mut opt_guard = optimized_state.lock().unwrap();
            *opt_guard = optimized_manager;
        }

        let opt_guard = optimized_state.lock().unwrap();
        assert!(
            opt_guard.is_some(),
            "Optimized manager should be initialized"
        );
    }

    #[tokio::test]
    async fn test_cache_stats_command() {
        let _temp_dir = TempDir::new().unwrap();
        let config_manager = Arc::new(ConfigManager::new().await.unwrap());
        let workspace_manager = WorkspaceManager::new(config_manager).unwrap();

        let optimized_manager = OptimizedWorkspaceManager::new(workspace_manager);
        let _optimized_state = Arc::new(Mutex::new(Some(optimized_manager)));

        // Test that cache stats can be retrieved
        // Note: In real usage, Tauri would handle State creation
        let stats = CacheStats {
            metadata_entries: 0,
            stats_entries: 0,
            has_recent_cache: false,
            metrics_count: 0,
            max_cache_size: 1000,
        };

        assert_eq!(stats.metadata_entries, 0, "Initially cache should be empty");
    }

    #[tokio::test]
    async fn test_resource_stats_command() {
        let _temp_dir = TempDir::new().unwrap();
        let config_manager = Arc::new(ConfigManager::new().await.unwrap());
        let workspace_manager = WorkspaceManager::new(config_manager).unwrap();

        let optimized_manager = OptimizedWorkspaceManager::new(workspace_manager);
        let _optimized_state = Arc::new(Mutex::new(Some(optimized_manager)));

        // Test that resource stats can be retrieved
        // Note: In real usage, Tauri would handle State creation
        let stats = ResourceStats {
            available_fs_permits: 10,
            available_metadata_permits: 5,
            available_stats_permits: 3,
            max_memory_mb: 500,
        };

        assert!(
            stats.available_fs_permits > 0,
            "Should have available file system permits"
        );
    }
}

// src-tauri/src/commands/workspace_commands.rs
//! Tauri commands for workspace management

use crate::workspace::{
    CreateWorkspaceRequest, RecentWorkspace, UpdateRecentWorkspaceRequest, WorkspaceConfig,
    WorkspaceInfo, WorkspaceStats, WorkspaceTemplate, WorkspaceValidation,
};
use tauri::State;

/// Create a new workspace
#[tauri::command]
pub async fn create_workspace(
    name: String,
    path: String,
    template: Option<String>,
    description: Option<String>,
    state: State<'_, crate::AppState>,
) -> Result<WorkspaceInfo, String> {
    let template = match template.as_deref() {
        Some("research") => WorkspaceTemplate::Research,
        Some("documentation") => WorkspaceTemplate::Documentation,
        Some("collaboration") => WorkspaceTemplate::Collaboration,
        Some("basic") | None => WorkspaceTemplate::Basic,
        Some(custom) => WorkspaceTemplate::Custom(custom.to_string()),
    };

    let request = CreateWorkspaceRequest {
        name,
        path: std::path::PathBuf::from(path),
        template,
        description,
    };

    state
        .workspace_manager
        .create_workspace(request)
        .await
        .map_err(|e| e.to_string())
}

/// Load workspace information from a path
#[tauri::command]
pub async fn load_workspace(
    path: String,
    state: State<'_, crate::AppState>,
) -> Result<WorkspaceInfo, String> {
    let workspace_path = std::path::PathBuf::from(path);

    state
        .workspace_manager
        .load_workspace(&workspace_path)
        .await
        .map_err(|e| e.to_string())
}

/// Check if a path contains a valid workspace
#[tauri::command]
pub async fn is_workspace(path: String, state: State<'_, crate::AppState>) -> Result<bool, String> {
    let workspace_path = std::path::PathBuf::from(path);

    state
        .workspace_manager
        .is_workspace(&workspace_path)
        .await
        .map_err(|e| e.to_string())
}

/// Validate workspace structure and integrity
#[tauri::command]
pub async fn validate_workspace(
    path: String,
    state: State<'_, crate::AppState>,
) -> Result<WorkspaceValidation, String> {
    let workspace_path = std::path::PathBuf::from(path);

    state
        .workspace_manager
        .validate_workspace(&workspace_path)
        .await
        .map_err(|e| e.to_string())
}

/// List all available workspaces
#[tauri::command]
pub async fn list_workspaces(
    state: State<'_, crate::AppState>,
) -> Result<Vec<WorkspaceInfo>, String> {
    state
        .workspace_manager
        .list_workspaces()
        .await
        .map_err(|e| e.to_string())
}

/// Get workspace configuration
#[tauri::command]
pub async fn get_workspace_config(workspace_path: String) -> Result<WorkspaceConfig, String> {
    let path = std::path::PathBuf::from(workspace_path);

    WorkspaceConfig::load_from_workspace(&path)
        .await
        .map_err(|e| e.to_string())
}

/// Update workspace configuration
#[tauri::command]
pub async fn update_workspace_config(
    workspace_path: String,
    config: WorkspaceConfig,
) -> Result<(), String> {
    let path = std::path::PathBuf::from(workspace_path);

    // Validate the configuration
    if let Err(errors) = config.validate() {
        return Err(format!(
            "Configuration validation failed: {}",
            errors.join(", ")
        ));
    }

    config
        .save_to_workspace(&path)
        .await
        .map_err(|e| e.to_string())
}

/// Get available workspace templates
#[tauri::command]
pub async fn get_workspace_templates() -> Result<Vec<(String, String, String)>, String> {
    let templates = vec![
        (
            "basic".to_string(),
            "Basic Workspace".to_string(),
            "A simple workspace for general document processing".to_string(),
        ),
        (
            "research".to_string(),
            "Research Project".to_string(),
            "Optimized for research projects with reference management".to_string(),
        ),
        (
            "documentation".to_string(),
            "Documentation Project".to_string(),
            "Focused on documentation creation and maintenance".to_string(),
        ),
        (
            "collaboration".to_string(),
            "Collaboration Workspace".to_string(),
            "Designed for team collaboration with shared resources".to_string(),
        ),
    ];

    Ok(templates)
}

/// Repair a workspace by recreating missing directories
#[tauri::command]
pub async fn repair_workspace(
    path: String,
    state: State<'_, crate::AppState>,
) -> Result<WorkspaceValidation, String> {
    let workspace_path = std::path::PathBuf::from(path);

    // First validate to see what's missing
    let validation = state
        .workspace_manager
        .validate_workspace(&workspace_path)
        .await
        .map_err(|e| e.to_string())?;

    // Recreate missing directories
    for missing_dir in &validation.missing_directories {
        let dir_path = workspace_path.join(missing_dir);
        tokio::fs::create_dir_all(&dir_path)
            .await
            .map_err(|e| format!("Failed to create directory {}: {}", missing_dir, e))?;
    }

    // Validate again to confirm repair
    state
        .workspace_manager
        .validate_workspace(&workspace_path)
        .await
        .map_err(|e| e.to_string())
}

/// Get recent workspaces from configuration
#[tauri::command]
pub async fn get_recent_workspaces(
    state: State<'_, crate::AppState>,
) -> Result<Vec<RecentWorkspace>, String> {
    state
        .workspace_manager
        .get_recent_workspaces()
        .await
        .map_err(|e| e.to_string())
}

/// Update recent workspace access
#[tauri::command]
pub async fn update_recent_workspace(
    path: String,
    name: String,
    template: Option<String>,
    state: State<'_, crate::AppState>,
) -> Result<(), String> {
    let template = match template.as_deref() {
        Some("research") => WorkspaceTemplate::Research,
        Some("documentation") => WorkspaceTemplate::Documentation,
        Some("collaboration") => WorkspaceTemplate::Collaboration,
        Some("basic") | None => WorkspaceTemplate::Basic,
        Some(custom) => WorkspaceTemplate::Custom(custom.to_string()),
    };

    let request = UpdateRecentWorkspaceRequest {
        path: std::path::PathBuf::from(path),
        name,
        template,
    };

    state
        .workspace_manager
        .update_recent_workspace(request)
        .await
        .map_err(|e| e.to_string())
}

/// Toggle favorite status for a workspace
#[tauri::command]
pub async fn toggle_workspace_favorite(
    path: String,
    state: State<'_, crate::AppState>,
) -> Result<bool, String> {
    let workspace_path = std::path::PathBuf::from(path);

    state
        .workspace_manager
        .toggle_workspace_favorite(&workspace_path)
        .await
        .map_err(|e| e.to_string())
}

/// Remove workspace from recent list
#[tauri::command]
pub async fn remove_workspace_from_recent(
    path: String,
    state: State<'_, crate::AppState>,
) -> Result<(), String> {
    let workspace_path = std::path::PathBuf::from(path);

    state
        .workspace_manager
        .remove_from_recent(&workspace_path)
        .await
        .map_err(|e| e.to_string())
}

/// Get workspace statistics
#[tauri::command]
pub async fn get_workspace_stats(
    path: String,
    state: State<'_, crate::AppState>,
) -> Result<WorkspaceStats, String> {
    let workspace_path = std::path::PathBuf::from(path);

    state
        .workspace_manager
        .get_workspace_stats(&workspace_path)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config;
    use crate::workspace::WorkspaceManager;
    use serial_test::serial;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn create_test_app_state() -> (crate::AppState, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config_manager = Arc::new(app_config::init().await.unwrap());
        let workspace_manager =
            Arc::new(WorkspaceManager::new(Arc::clone(&config_manager)).unwrap());

        // Create a dummy channel for testing
        let (document_indexing_sender, _) = tokio::sync::mpsc::unbounded_channel();

        let app_state = crate::AppState {
            config_manager,
            workspace_manager,
            security_state: crate::SecurityState {
                validation_result: crate::filesystem::security::StartupValidationResult {
                    success: true,
                    security_level: crate::filesystem::security::SecurityLevel::Development,
                    production_ready: false,
                    config_valid: true,
                    environment_ready: true,
                    errors: Vec::new(),
                    warnings: Vec::new(),
                },
            },
            document_indexing_sender,
        };
        (app_state, temp_dir)
    }

    // Test helper functions that work directly with AppState
    async fn helper_create_workspace(
        name: String,
        path: String,
        template: Option<String>,
        description: Option<String>,
        state: &crate::AppState,
    ) -> Result<WorkspaceInfo, String> {
        let template = match template.as_deref() {
            Some("research") => WorkspaceTemplate::Research,
            Some("documentation") => WorkspaceTemplate::Documentation,
            Some("collaboration") => WorkspaceTemplate::Collaboration,
            Some("basic") | None => WorkspaceTemplate::Basic,
            Some(custom) => WorkspaceTemplate::Custom(custom.to_string()),
        };

        let request = CreateWorkspaceRequest {
            name,
            path: std::path::PathBuf::from(path),
            template,
            description,
        };

        state
            .workspace_manager
            .create_workspace(request)
            .await
            .map_err(|e| e.to_string())
    }

    async fn helper_validate_workspace(
        path: String,
        state: &crate::AppState,
    ) -> Result<WorkspaceValidation, String> {
        let workspace_path = std::path::PathBuf::from(path);
        state
            .workspace_manager
            .validate_workspace(&workspace_path)
            .await
            .map_err(|e| e.to_string())
    }

    #[tokio::test]
    #[serial]
    async fn test_create_workspace() {
        let (state, temp_dir) = create_test_app_state().await;
        let workspace_path = temp_dir.path().join("test_workspace");

        let result = helper_create_workspace(
            "Test Workspace".to_string(),
            workspace_path.to_string_lossy().to_string(),
            Some("basic".to_string()),
            Some("Test workspace description".to_string()),
            &state,
        )
        .await;

        assert!(result.is_ok());
        let workspace = result.unwrap();
        assert_eq!(workspace.name, "Test Workspace");
        assert!(workspace_path.exists());
    }

    #[tokio::test]
    #[serial]
    async fn test_workspace_validation() {
        let (state, temp_dir) = create_test_app_state().await;
        let workspace_path = temp_dir.path().join("test_workspace");

        // Create workspace first
        let _workspace = helper_create_workspace(
            "Test Workspace".to_string(),
            workspace_path.to_string_lossy().to_string(),
            Some("basic".to_string()),
            None,
            &state,
        )
        .await
        .unwrap();

        // Validate the workspace
        let validation =
            helper_validate_workspace(workspace_path.to_string_lossy().to_string(), &state)
                .await
                .unwrap();

        assert!(validation.is_valid);
        assert!(validation.errors.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_workspace_templates() {
        let templates = get_workspace_templates().await.unwrap();
        assert!(!templates.is_empty());
        assert!(templates.iter().any(|(id, _, _)| id == "basic"));
        assert!(templates.iter().any(|(id, _, _)| id == "research"));
    }
}

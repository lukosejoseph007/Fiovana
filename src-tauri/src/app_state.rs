// src-tauri/src/app_state.rs
//! Application state management

use crate::app_config::ConfigManager;
use crate::filesystem::security::StartupValidationResult;
use crate::workspace::WorkspaceManager;
use std::sync::Arc;

/// Enhanced application state to hold both configuration and security information
pub struct AppState {
    pub config_manager: Arc<ConfigManager>,
    pub security_state: SecurityState,
    pub workspace_manager: Arc<WorkspaceManager>,
}

/// Security state to store security information
#[derive(Clone)]
pub struct SecurityState {
    pub validation_result: StartupValidationResult,
}

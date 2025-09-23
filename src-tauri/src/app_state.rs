// src-tauri/src/app_state.rs
//! Application state management

use crate::app_config::ConfigManager;
use crate::filesystem::security::StartupValidationResult;
use crate::filesystem::watcher::FileEvent;
use crate::workspace::WorkspaceManager;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Enhanced application state to hold both configuration and security information
pub struct AppState {
    pub config_manager: Arc<ConfigManager>,
    pub security_state: SecurityState,
    pub workspace_manager: Arc<WorkspaceManager>,
    #[allow(dead_code)]
    pub document_indexing_sender: mpsc::UnboundedSender<FileEvent>,
}

/// Security state to store security information
#[derive(Clone)]
pub struct SecurityState {
    pub validation_result: StartupValidationResult,
}

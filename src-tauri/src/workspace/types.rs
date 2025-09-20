// src-tauri/src/workspace/types.rs
//! Type definitions for workspace management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Result type for workspace operations
pub type WorkspaceResult<T> = Result<T, super::WorkspaceError>;

/// Template types for workspace creation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum WorkspaceTemplate {
    #[default]
    Basic,
    Research,
    Documentation,
    Collaboration,
    Custom(String),
}

impl WorkspaceTemplate {
    /// Get the human-readable name for the template
    #[allow(dead_code)]
    pub fn display_name(&self) -> &str {
        match self {
            WorkspaceTemplate::Basic => "Basic Workspace",
            WorkspaceTemplate::Research => "Research Project",
            WorkspaceTemplate::Documentation => "Documentation Project",
            WorkspaceTemplate::Collaboration => "Collaboration Workspace",
            WorkspaceTemplate::Custom(name) => name,
        }
    }

    /// Get the description for the template
    #[allow(dead_code)]
    pub fn description(&self) -> &str {
        match self {
            WorkspaceTemplate::Basic => "A simple workspace for general document processing",
            WorkspaceTemplate::Research => {
                "Optimized for research projects with reference management"
            }
            WorkspaceTemplate::Documentation => "Focused on documentation creation and maintenance",
            WorkspaceTemplate::Collaboration => {
                "Designed for team collaboration with shared resources"
            }
            WorkspaceTemplate::Custom(_) => "Custom workspace configuration",
        }
    }
}

/// Workspace creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub path: PathBuf,
    pub template: WorkspaceTemplate,
    pub description: Option<String>,
}

/// Workspace validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub missing_directories: Vec<String>,
    pub invalid_files: Vec<String>,
}

/// Recent workspace entry for tracking workspace access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentWorkspace {
    pub path: PathBuf,
    pub name: String,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
    pub is_favorite: bool,
    pub template: WorkspaceTemplate,
}

/// Statistics about a workspace
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceStats {
    pub total_files: u64,
    pub total_size: u64,
    pub import_count: u64,
    pub reference_count: u64,
    pub output_count: u64,
    pub last_import: Option<DateTime<Utc>>,
    pub last_output: Option<DateTime<Utc>>,
}

/// Request for updating recent workspace access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecentWorkspaceRequest {
    pub path: PathBuf,
    pub name: String,
    pub template: WorkspaceTemplate,
}

/// Recent projects manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProjectsConfig {
    pub max_recent: usize,
    pub auto_cleanup_days: u32,
    pub favorites: Vec<PathBuf>,
}

impl Default for RecentProjectsConfig {
    fn default() -> Self {
        Self {
            max_recent: 20,
            auto_cleanup_days: 30,
            favorites: Vec::new(),
        }
    }
}

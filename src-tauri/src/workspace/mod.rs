// src-tauri/src/workspace/mod.rs
//! Workspace management system for Proxemic
//!
//! This module provides functionality for creating, managing, and organizing
//! project workspaces with standardized directory structures and configuration.

use crate::app_config::errors::ConfigError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub mod backup;
pub mod config;
pub mod intelligence;
pub mod knowledge_analyzer;
pub mod lifecycle_manager;
pub mod manager;
pub mod performance;
pub mod smart_organizer;
pub mod types;
pub mod workspace_analyzer;

pub use config::*;
pub use intelligence::*;
#[allow(unused_imports)]
pub use knowledge_analyzer::{
    ExpertRecommendation, IncompleteProcess, KnowledgeAnalysisConfig, KnowledgeAnalyzer,
    KnowledgeGap, KnowledgeGapAnalysis, KnowledgeGapType, MissingDocumentType, OutdatedContent,
    PriorityArea, ReferenceGap,
};
#[allow(unused_imports)]
pub use lifecycle_manager::{
    ArchivalSuggestion, ConsolidationSuggestion, LifecycleAction, LifecycleAnalysis,
    LifecycleConfig, LifecycleManager, UpdateRecommendation, UsagePatternAnalysis,
};
pub use manager::*;
#[allow(unused_imports)]
pub use smart_organizer::{
    CategorizationSuggestion, DuplicateHandlingSuggestion, FolderStructureSuggestion,
    OrganizationAction, OrganizationAnalysis, OrganizationConfig, SemanticCluster, SmartOrganizer,
    TaggingSuggestion,
};
pub use types::*;
#[allow(unused_imports)]
pub use workspace_analyzer::{
    ComprehensiveWorkspaceAnalysis, WorkspaceAnalysisConfig, WorkspaceAnalyzer,
};

/// Standard workspace directory structure
pub const WORKSPACE_DIRECTORIES: &[&str] = &[
    "sources/imports",
    "sources/references",
    "sources/archives",
    "intelligence/content-models",
    "intelligence/comparisons",
    "intelligence/conversations",
    "outputs/drafts",
    "outputs/approved",
    ".proxemic/cache",
];

/// Workspace metadata file name
pub const WORKSPACE_METADATA_FILE: &str = ".proxemic/workspace.json";

/// Workspace configuration file name
pub const WORKSPACE_CONFIG_FILE: &str = ".proxemic/config.json";

/// Represents a workspace instance with its metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub path: PathBuf,
    pub name: String,
    pub version: String,
    pub created: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub import_settings: ImportSettings,
    pub ai_settings: WorkspaceAISettings,
    pub is_favorite: bool,
    pub access_count: u32,
}

/// Import configuration for workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSettings {
    pub allowed_extensions: Vec<String>,
    pub max_file_size: u64,
    pub auto_process: bool,
    pub duplicate_handling: DuplicateHandling,
}

/// AI configuration for workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceAISettings {
    pub preferred_local_model: Option<String>,
    pub cloud_fallback: bool,
    pub privacy_mode: bool,
}

/// How to handle duplicate files during import
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DuplicateHandling {
    #[default]
    Prompt,
    Skip,
    Replace,
    KeepBoth,
}

impl Default for ImportSettings {
    fn default() -> Self {
        Self {
            allowed_extensions: vec![
                ".docx".to_string(),
                ".pdf".to_string(),
                ".md".to_string(),
                ".txt".to_string(),
                ".csv".to_string(),
                ".json".to_string(),
            ],
            max_file_size: 100 * 1024 * 1024, // 100MB
            auto_process: true,
            duplicate_handling: DuplicateHandling::Prompt,
        }
    }
}

impl Default for WorkspaceAISettings {
    fn default() -> Self {
        Self {
            preferred_local_model: Some("llama3.2-3b".to_string()),
            cloud_fallback: true,
            privacy_mode: false,
        }
    }
}

/// Errors that can occur during workspace operations
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path validation error: {message}")]
    PathValidation { message: String },

    #[error("Workspace already exists at: {path}")]
    WorkspaceExists { path: PathBuf },

    #[error("Invalid workspace at: {path} - {reason}")]
    InvalidWorkspace { path: PathBuf, reason: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Workspace not found")]
    NotFound,

    #[error("Workspace not found at: {path}")]
    WorkspaceNotFound { path: PathBuf },
}

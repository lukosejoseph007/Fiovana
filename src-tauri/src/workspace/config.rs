// src-tauri/src/workspace/config.rs
//! Workspace-specific configuration management

use super::*;
use crate::app_config::types::FiovanaConfig;
use serde::{Deserialize, Serialize};

/// Workspace-specific configuration that extends the base FiovanaConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    #[serde(flatten)]
    pub base_config: FiovanaConfig,
    pub workspace: WorkspaceSettings,
}

/// Workspace-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    pub version: String,
    pub template: WorkspaceTemplate,
    pub created: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub import_settings: ImportSettings,
    pub ai_settings: WorkspaceAISettings,
    pub custom_settings: std::collections::HashMap<String, serde_json::Value>,
}

impl WorkspaceConfig {
    /// Create a workspace configuration from base config and template
    pub fn from_base_config(base_config: FiovanaConfig, template: WorkspaceTemplate) -> Self {
        let now = Utc::now();

        // Get template-specific settings
        let import_settings = Self::get_template_import_settings(&template);
        let ai_settings = Self::get_template_ai_settings(&template);

        Self {
            base_config,
            workspace: WorkspaceSettings {
                version: "1.1.2".to_string(),
                template,
                created: now,
                last_modified: now,
                import_settings,
                ai_settings,
                custom_settings: std::collections::HashMap::new(),
            },
        }
    }

    /// Load workspace configuration from file
    pub async fn load_from_workspace(workspace_path: &Path) -> WorkspaceResult<Self> {
        let config_path = workspace_path.join(WORKSPACE_CONFIG_FILE);

        if !config_path.exists() {
            return Err(WorkspaceError::InvalidWorkspace {
                path: workspace_path.to_path_buf(),
                reason: "Workspace configuration file not found".to_string(),
            });
        }

        let config_content = tokio::fs::read_to_string(&config_path).await?;
        let config: WorkspaceConfig = serde_json::from_str(&config_content)?;

        Ok(config)
    }

    /// Save workspace configuration to file
    pub async fn save_to_workspace(&self, workspace_path: &Path) -> WorkspaceResult<()> {
        let config_path = workspace_path.join(WORKSPACE_CONFIG_FILE);

        // Ensure the .fiovana directory exists
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let config_json = serde_json::to_string_pretty(self)?;
        tokio::fs::write(&config_path, config_json).await?;

        Ok(())
    }

    /// Update the last modified timestamp
    #[allow(dead_code)]
    pub fn touch(&mut self) {
        self.workspace.last_modified = Utc::now();
    }

    /// Validate the workspace configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate base configuration
        if let Err(base_errors) = self.base_config.validate() {
            errors.extend(base_errors);
        }

        // Validate workspace-specific settings
        if self.workspace.version.is_empty() {
            errors.push("Workspace version cannot be empty".to_string());
        }

        if self.workspace.import_settings.allowed_extensions.is_empty() {
            errors.push("At least one file extension must be allowed".to_string());
        }

        if self.workspace.import_settings.max_file_size == 0 {
            errors.push("Maximum file size must be greater than 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get import settings for a template
    fn get_template_import_settings(template: &WorkspaceTemplate) -> ImportSettings {
        let mut settings = ImportSettings::default();

        match template {
            WorkspaceTemplate::Research => {
                settings.allowed_extensions.extend([
                    ".xlsx".to_string(),
                    ".csv".to_string(),
                    ".bib".to_string(),
                    ".ris".to_string(),
                    ".tsv".to_string(),
                ]);
                settings.max_file_size = 200 * 1024 * 1024; // 200MB
            }
            WorkspaceTemplate::Documentation => {
                settings.allowed_extensions = vec![
                    ".md".to_string(),
                    ".txt".to_string(),
                    ".docx".to_string(),
                    ".html".to_string(),
                    ".rst".to_string(),
                    ".adoc".to_string(),
                ];
                settings.max_file_size = 50 * 1024 * 1024; // 50MB
            }
            WorkspaceTemplate::Collaboration => {
                settings.allowed_extensions.extend([
                    ".pptx".to_string(),
                    ".xlsx".to_string(),
                    ".png".to_string(),
                    ".jpg".to_string(),
                    ".jpeg".to_string(),
                    ".gif".to_string(),
                ]);
                settings.max_file_size = 150 * 1024 * 1024; // 150MB
            }
            WorkspaceTemplate::Basic | WorkspaceTemplate::Custom(_) => {
                // Use defaults
            }
        }

        settings
    }

    /// Get AI settings for a template
    fn get_template_ai_settings(template: &WorkspaceTemplate) -> WorkspaceAISettings {
        let mut settings = WorkspaceAISettings::default();

        match template {
            WorkspaceTemplate::Research => {
                settings.privacy_mode = true; // Research data is often sensitive
                settings.preferred_local_model = Some("llama3.1-8b".to_string());
                // Larger model for research
            }
            WorkspaceTemplate::Documentation => {
                settings.preferred_local_model = Some("llama3.2-3b".to_string()); // Smaller model is fine
                settings.cloud_fallback = true; // Documentation benefits from cloud models
            }
            WorkspaceTemplate::Collaboration => {
                settings.cloud_fallback = true; // Collaboration may need more resources
                settings.privacy_mode = false; // Usually less sensitive
            }
            WorkspaceTemplate::Basic | WorkspaceTemplate::Custom(_) => {
                // Use defaults
            }
        }

        settings
    }

    /// Merge settings from another workspace configuration
    #[allow(dead_code)]
    pub fn merge_from(&mut self, other: &WorkspaceConfig) {
        // Update workspace settings
        self.workspace.import_settings = other.workspace.import_settings.clone();
        self.workspace.ai_settings = other.workspace.ai_settings.clone();

        // Merge custom settings
        for (key, value) in &other.workspace.custom_settings {
            self.workspace
                .custom_settings
                .insert(key.clone(), value.clone());
        }

        self.touch();
    }

    /// Get a custom setting value
    #[allow(dead_code)]
    pub fn get_custom_setting(&self, key: &str) -> Option<&serde_json::Value> {
        self.workspace.custom_settings.get(key)
    }

    /// Set a custom setting value
    #[allow(dead_code)]
    pub fn set_custom_setting(&mut self, key: String, value: serde_json::Value) {
        self.workspace.custom_settings.insert(key, value);
        self.touch();
    }

    /// Remove a custom setting
    #[allow(dead_code)]
    pub fn remove_custom_setting(&mut self, key: &str) -> Option<serde_json::Value> {
        let result = self.workspace.custom_settings.remove(key);
        if result.is_some() {
            self.touch();
        }
        result
    }
}

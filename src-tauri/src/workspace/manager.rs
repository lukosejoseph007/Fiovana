// src-tauri/src/workspace/manager.rs
//! Workspace manager implementation

use super::*;
use crate::app_config::ConfigManager;
use crate::filesystem::PathValidator;
use std::sync::Arc;
use tokio::fs;

/// Main workspace manager that coordinates workspace operations
#[derive(Clone)]
pub struct WorkspaceManager {
    config_manager: Arc<ConfigManager>,
    path_validator: PathValidator,
}

impl WorkspaceManager {
    /// Create a new workspace manager
    pub fn new(config_manager: Arc<ConfigManager>) -> WorkspaceResult<Self> {
        // Create path validator with allowed paths for workspaces
        let allowed_paths = vec![
            dirs::desktop_dir().unwrap_or_default(),
            dirs::document_dir().unwrap_or_default(),
            dirs::download_dir().unwrap_or_default(),
            dirs::home_dir().unwrap_or_default(),
        ];

        // Get security config and convert to filesystem SecurityConfig
        let app_security_config = config_manager.get_security_config().ok_or_else(|| {
            WorkspaceError::Config(ConfigError::ValidationError {
                field: "security_config".to_string(),
                message: "Failed to get security configuration".to_string(),
            })
        })?;

        // Convert app SecurityConfig to filesystem SecurityConfig
        use std::collections::{HashMap, HashSet};
        let filesystem_security_config = crate::filesystem::SecurityConfig {
            allowed_extensions: app_security_config
                .allowed_extensions
                .clone()
                .into_iter()
                .collect::<HashSet<_>>(),
            allowed_mime_types: app_security_config
                .allowed_mime_types
                .clone()
                .into_iter()
                .collect::<HashSet<_>>(),
            max_file_size: app_security_config.max_file_size,
            max_path_length: app_security_config.max_path_length,
            allowed_workspace_paths: allowed_paths.clone(),
            temp_directory: std::env::temp_dir(),
            prohibited_filename_chars: ['<', '>', ':', '"', '|', '?', '*', '\\', '/']
                .iter()
                .cloned()
                .collect(),
            enable_magic_number_validation: app_security_config.enable_magic_number_validation,
            magic_number_map: HashMap::new(), // We'll use defaults for now
            security_level: crate::filesystem::security::SecurityLevel::Development, // Default for workspaces
            enforce_workspace_boundaries: true,
            max_concurrent_operations: 10,
            audit_logging_enabled: false, // Disable for workspace operations
            memory_warning_threshold_kb: 256 * 1024,
            memory_critical_threshold_kb: 512 * 1024,
            operation_latency_warning_ms: 5000,
            operation_latency_critical_ms: 10000,
            error_rate_warning_percent: 5.0,
            error_rate_critical_percent: 15.0,
            monitoring_enabled: false,
            performance_sampling_interval_secs: 30,
        };

        let path_validator = PathValidator::new(filesystem_security_config, allowed_paths);

        Ok(Self {
            config_manager,
            path_validator,
        })
    }

    /// Create a new workspace at the specified path
    pub async fn create_workspace(
        &self,
        request: CreateWorkspaceRequest,
    ) -> WorkspaceResult<WorkspaceInfo> {
        // Validate the workspace path
        let workspace_path = self
            .path_validator
            .validate_workspace_path(&request.path)
            .map_err(|e| WorkspaceError::PathValidation {
                message: format!("Invalid workspace path: {}", e),
            })?;

        // Check if workspace already exists
        if workspace_path.exists() && self.is_workspace(&workspace_path).await? {
            return Err(WorkspaceError::WorkspaceExists {
                path: workspace_path,
            });
        }

        // Create the workspace directory structure
        self.create_workspace_structure(&workspace_path, &request.template)
            .await?;

        // Create workspace metadata
        let workspace_info = WorkspaceInfo {
            path: workspace_path.clone(),
            name: request.name.clone(),
            version: "1.1.2".to_string(),
            created: Utc::now(),
            last_modified: Utc::now(),
            last_accessed: Utc::now(),
            import_settings: self.get_template_import_settings(&request.template),
            ai_settings: self.get_template_ai_settings(&request.template),
            is_favorite: false,
            access_count: 1,
        };

        // Save workspace metadata
        self.save_workspace_metadata(&workspace_info).await?;

        // Create workspace-specific configuration
        self.create_workspace_config(&workspace_path, &request.template)
            .await?;

        println!(
            "✅ Created workspace '{}' at: {}",
            request.name,
            workspace_path.display()
        );

        Ok(workspace_info)
    }

    /// Create the standard workspace directory structure
    async fn create_workspace_structure(
        &self,
        workspace_path: &Path,
        template: &WorkspaceTemplate,
    ) -> WorkspaceResult<()> {
        // Create the main workspace directory
        fs::create_dir_all(workspace_path).await?;

        // Create all standard directories
        for dir_path in WORKSPACE_DIRECTORIES {
            let full_path = workspace_path.join(dir_path);
            fs::create_dir_all(&full_path).await?;
            println!("📁 Created directory: {}", full_path.display());
        }

        // Create template-specific directories and files
        self.create_template_specific_structure(workspace_path, template)
            .await?;

        Ok(())
    }

    /// Create template-specific directory structure and files
    async fn create_template_specific_structure(
        &self,
        workspace_path: &Path,
        template: &WorkspaceTemplate,
    ) -> WorkspaceResult<()> {
        match template {
            WorkspaceTemplate::Research => {
                // Create additional research-specific directories
                let research_dirs = [
                    "sources/literature",
                    "sources/datasets",
                    "analysis/notebooks",
                ];
                for dir in research_dirs {
                    fs::create_dir_all(workspace_path.join(dir)).await?;
                }

                // Create a README for research workflow
                let readme_content = r#"# Research Workspace

This workspace is optimized for research projects with comprehensive reference management and analysis capabilities.

## Directory Structure

- **sources/imports/** - Primary research documents and papers
- **sources/references/** - Reference materials and citations
- **sources/literature/** - Literature review materials
- **sources/datasets/** - Research datasets and data files
- **sources/archives/** - Completed or archived research materials

- **intelligence/content-models/** - AI-generated content models and embeddings
- **intelligence/comparisons/** - Document comparison and analysis results
- **intelligence/conversations/** - AI conversation history and insights

- **analysis/notebooks/** - Research notebooks and analysis scripts

- **outputs/drafts/** - Work-in-progress research outputs
- **outputs/approved/** - Final approved research deliverables

## Getting Started

1. Import your research papers and documents into `sources/imports/`
2. Organize reference materials in `sources/references/`
3. Use the AI features to analyze and compare documents
4. Create research outputs in `outputs/drafts/`
5. Move final deliverables to `outputs/approved/`

## Research Features

- **Enhanced file support** for academic formats (.bib, .ris, .xlsx)
- **Privacy mode enabled** for sensitive research data
- **Larger file size limits** for datasets (200MB)
- **Automatic reference organization**
"#;
                fs::write(workspace_path.join("README.md"), readme_content).await?;
            }
            WorkspaceTemplate::Documentation => {
                // Create documentation-specific directories
                let doc_dirs = [
                    "sources/specifications",
                    "sources/examples",
                    "outputs/guides",
                ];
                for dir in doc_dirs {
                    fs::create_dir_all(workspace_path.join(dir)).await?;
                }

                // Create documentation template
                let doc_template = r#"# Documentation Workspace

This workspace is designed for creating, maintaining, and organizing documentation projects.

## Directory Structure

- **sources/imports/** - Source documents and materials to be documented
- **sources/references/** - Reference documentation and examples
- **sources/specifications/** - Technical specifications and requirements
- **sources/examples/** - Code examples and samples
- **sources/archives/** - Archived documentation versions

- **intelligence/content-models/** - AI-generated content analysis
- **intelligence/comparisons/** - Document comparison results
- **intelligence/conversations/** - AI assistance conversations

- **outputs/drafts/** - Work-in-progress documentation
- **outputs/approved/** - Published documentation
- **outputs/guides/** - User guides and tutorials

## Getting Started

1. Import source materials into `sources/imports/`
2. Organize specifications in `sources/specifications/`
3. Add examples to `sources/examples/`
4. Create documentation drafts in `outputs/drafts/`
5. Publish final documentation to `outputs/approved/`

## Documentation Features

- **Text-focused file support** (.md, .txt, .docx, .html, .rst)
- **Cloud AI fallback** for enhanced writing assistance
- **Moderate file size limits** optimized for text (50MB)
- **Template-based documentation structure**
"#;
                fs::write(workspace_path.join("README.md"), doc_template).await?;
            }
            WorkspaceTemplate::Collaboration => {
                // Create collaboration-specific directories
                let collab_dirs = ["shared/resources", "shared/templates", "reviews"];
                for dir in collab_dirs {
                    fs::create_dir_all(workspace_path.join(dir)).await?;
                }
            }
            WorkspaceTemplate::Basic | WorkspaceTemplate::Custom(_) => {
                // Basic template - just the standard structure
            }
        }

        Ok(())
    }

    /// Get import settings for a specific template
    fn get_template_import_settings(&self, template: &WorkspaceTemplate) -> ImportSettings {
        let mut settings = ImportSettings::default();

        match template {
            WorkspaceTemplate::Research => {
                // Research projects often work with more file types
                settings.allowed_extensions.extend([
                    ".xlsx".to_string(),
                    ".csv".to_string(),
                    ".bib".to_string(),
                    ".ris".to_string(),
                ]);
                settings.max_file_size = 200 * 1024 * 1024; // 200MB for datasets
            }
            WorkspaceTemplate::Documentation => {
                // Documentation focuses on text formats
                settings.allowed_extensions = vec![
                    ".md".to_string(),
                    ".txt".to_string(),
                    ".docx".to_string(),
                    ".html".to_string(),
                    ".rst".to_string(),
                ];
            }
            WorkspaceTemplate::Collaboration => {
                // Collaboration workspaces need broader file support
                settings.allowed_extensions.extend([
                    ".pptx".to_string(),
                    ".xlsx".to_string(),
                    ".png".to_string(),
                    ".jpg".to_string(),
                ]);
            }
            _ => {
                // Use defaults
            }
        }

        settings
    }

    /// Get AI settings for a specific template
    fn get_template_ai_settings(&self, template: &WorkspaceTemplate) -> WorkspaceAISettings {
        let mut settings = WorkspaceAISettings::default();

        match template {
            WorkspaceTemplate::Research => {
                settings.privacy_mode = true; // Research often sensitive
            }
            WorkspaceTemplate::Collaboration => {
                settings.cloud_fallback = true; // Collaboration may need more resources
            }
            _ => {
                // Use defaults
            }
        }

        settings
    }

    /// Save workspace metadata to the workspace.json file
    async fn save_workspace_metadata(&self, workspace_info: &WorkspaceInfo) -> WorkspaceResult<()> {
        let metadata_path = workspace_info.path.join(WORKSPACE_METADATA_FILE);

        // Ensure the .proxemic directory exists
        if let Some(parent) = metadata_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let metadata_json = serde_json::to_string_pretty(workspace_info)?;
        fs::write(&metadata_path, metadata_json).await?;

        println!("💾 Saved workspace metadata: {}", metadata_path.display());
        Ok(())
    }

    /// Create workspace-specific configuration
    async fn create_workspace_config(
        &self,
        workspace_path: &Path,
        template: &WorkspaceTemplate,
    ) -> WorkspaceResult<()> {
        let config_path = workspace_path.join(WORKSPACE_CONFIG_FILE);

        // Get base configuration from the global config manager
        let config_arc = self.config_manager.get_config();
        let base_config = {
            let config_guard = config_arc.read().map_err(|_| {
                WorkspaceError::Config(ConfigError::ValidationError {
                    field: "config_lock".to_string(),
                    message: "Failed to acquire read lock for configuration".to_string(),
                })
            })?;
            config_guard.clone()
        };

        // Create workspace-specific overrides based on template
        let workspace_config = WorkspaceConfig::from_base_config(base_config, template.clone());

        let config_json = serde_json::to_string_pretty(&workspace_config)?;
        fs::write(&config_path, config_json).await?;

        println!(
            "⚙️ Created workspace configuration: {}",
            config_path.display()
        );
        Ok(())
    }

    /// Check if a path contains a valid workspace
    pub async fn is_workspace(&self, path: &Path) -> WorkspaceResult<bool> {
        let metadata_path = path.join(WORKSPACE_METADATA_FILE);
        Ok(metadata_path.exists() && metadata_path.is_file())
    }

    /// Load workspace information from a path
    pub async fn load_workspace(&self, path: &Path) -> WorkspaceResult<WorkspaceInfo> {
        if !self.is_workspace(path).await? {
            return Err(WorkspaceError::WorkspaceNotFound {
                path: path.to_path_buf(),
            });
        }

        let metadata_path = path.join(WORKSPACE_METADATA_FILE);
        let metadata_content = fs::read_to_string(&metadata_path).await?;
        let mut workspace_info: WorkspaceInfo = serde_json::from_str(&metadata_content)?;

        // Update last accessed time
        workspace_info.last_accessed = Utc::now();
        workspace_info.access_count += 1;

        // Save updated metadata
        self.save_workspace_metadata(&workspace_info).await?;

        Ok(workspace_info)
    }

    /// Validate workspace structure and integrity
    pub async fn validate_workspace(&self, path: &Path) -> WorkspaceResult<WorkspaceValidation> {
        let mut validation = WorkspaceValidation {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            missing_directories: Vec::new(),
            invalid_files: Vec::new(),
        };

        // Check if workspace metadata exists
        if !self.is_workspace(path).await? {
            validation.is_valid = false;
            validation
                .errors
                .push("Workspace metadata file missing".to_string());
            return Ok(validation);
        }

        // Check for required directories
        for dir_path in WORKSPACE_DIRECTORIES {
            let full_path = path.join(dir_path);
            if !full_path.exists() {
                validation.missing_directories.push(dir_path.to_string());
                validation
                    .warnings
                    .push(format!("Missing directory: {}", dir_path));
            }
        }

        // Try to load workspace metadata
        match self.load_workspace(path).await {
            Ok(_) => {
                println!("✅ Workspace validation passed for: {}", path.display());
            }
            Err(e) => {
                validation.is_valid = false;
                validation
                    .errors
                    .push(format!("Failed to load workspace: {}", e));
            }
        }

        if !validation.missing_directories.is_empty() {
            validation
                .warnings
                .push("Some directories are missing but can be recreated".to_string());
        }

        Ok(validation)
    }

    /// List all workspaces found in the system
    pub async fn list_workspaces(&self) -> WorkspaceResult<Vec<WorkspaceInfo>> {
        // This would typically scan common workspace locations
        // For now, we'll implement a basic version that can be extended
        let mut workspaces = Vec::new();

        // Check recent workspaces from config
        if let Ok(recent_workspaces) = self.get_recent_workspaces().await {
            for recent in recent_workspaces {
                if recent.path.exists() {
                    if let Ok(workspace_info) = self.load_workspace(&recent.path).await {
                        workspaces.push(workspace_info);
                    }
                }
            }
        }

        // TODO: Implement additional workspace discovery
        // - Scan common directories
        // - Check registry/database of workspaces

        Ok(workspaces)
    }

    /// Get recent workspaces from configuration
    pub async fn get_recent_workspaces(&self) -> WorkspaceResult<Vec<RecentWorkspace>> {
        let config_guard = self.config_manager.get_config();
        let config = config_guard
            .read()
            .map_err(|_| WorkspaceError::Config(ConfigError::AccessError))?;

        // Get recent workspaces from config, defaulting to empty vec
        Ok(config
            .workspace
            .recent_workspaces
            .clone()
            .unwrap_or_default())
    }

    /// Update recent workspace access
    pub async fn update_recent_workspace(
        &self,
        request: UpdateRecentWorkspaceRequest,
    ) -> WorkspaceResult<()> {
        // Scope the lock to prevent holding it across await
        {
            let config_guard = self.config_manager.get_config();
            let mut config = config_guard
                .write()
                .map_err(|_| WorkspaceError::Config(ConfigError::AccessError))?;

            // Initialize recent workspaces if None
            if config.workspace.recent_workspaces.is_none() {
                config.workspace.recent_workspaces = Some(Vec::new());
            }

            let recent_workspaces = config.workspace.recent_workspaces.as_mut().unwrap();

            // Check if workspace already exists in recent list
            if let Some(existing) = recent_workspaces
                .iter_mut()
                .find(|w| w.path == request.path)
            {
                // Update existing entry
                existing.last_accessed = chrono::Utc::now();
                existing.access_count += 1;
                existing.name = request.name;
                existing.template = request.template;
            } else {
                // Add new entry
                let new_recent = RecentWorkspace {
                    path: request.path,
                    name: request.name,
                    last_accessed: chrono::Utc::now(),
                    access_count: 1,
                    is_favorite: false,
                    template: request.template,
                };
                recent_workspaces.push(new_recent);
            }

            // Sort by last accessed (most recent first) and limit to max_recent
            recent_workspaces.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
            recent_workspaces.truncate(20); // Max 20 recent workspaces
        } // Lock is dropped here

        // Save configuration
        self.config_manager
            .save_configuration(None)
            .await
            .map_err(WorkspaceError::Config)?;

        Ok(())
    }

    /// Toggle favorite status for a workspace
    pub async fn toggle_workspace_favorite(&self, workspace_path: &Path) -> WorkspaceResult<bool> {
        let is_favorite = {
            let config_guard = self.config_manager.get_config();
            let mut config = config_guard
                .write()
                .map_err(|_| WorkspaceError::Config(ConfigError::AccessError))?;

            if let Some(recent_workspaces) = config.workspace.recent_workspaces.as_mut() {
                if let Some(workspace) = recent_workspaces
                    .iter_mut()
                    .find(|w| w.path == workspace_path)
                {
                    workspace.is_favorite = !workspace.is_favorite;
                    workspace.is_favorite
                } else {
                    return Err(WorkspaceError::NotFound);
                }
            } else {
                return Err(WorkspaceError::NotFound);
            }
        }; // Lock is dropped here

        // Save configuration
        self.config_manager
            .save_configuration(None)
            .await
            .map_err(WorkspaceError::Config)?;

        Ok(is_favorite)
    }

    /// Remove workspace from recent list
    pub async fn remove_from_recent(&self, workspace_path: &Path) -> WorkspaceResult<()> {
        // Scope the lock to prevent holding it across await
        {
            let config_guard = self.config_manager.get_config();
            let mut config = config_guard
                .write()
                .map_err(|_| WorkspaceError::Config(ConfigError::AccessError))?;

            if let Some(recent_workspaces) = config.workspace.recent_workspaces.as_mut() {
                recent_workspaces.retain(|w| w.path != workspace_path);
            }
        } // Lock is dropped here

        // Save configuration
        self.config_manager
            .save_configuration(None)
            .await
            .map_err(WorkspaceError::Config)?;

        Ok(())
    }

    /// Get workspace statistics
    pub async fn get_workspace_stats(
        &self,
        workspace_path: &Path,
    ) -> WorkspaceResult<WorkspaceStats> {
        // Validate workspace exists
        if !self.is_workspace(workspace_path).await? {
            return Err(WorkspaceError::NotFound);
        }

        let mut stats = WorkspaceStats::default();
        let mut total_size = 0u64;
        let mut total_files = 0u64;

        // Count files in each directory
        let directories = [
            workspace_path.join("sources/imports"),
            workspace_path.join("sources/references"),
            workspace_path.join("sources/archives"),
            workspace_path.join("outputs/drafts"),
            workspace_path.join("outputs/approved"),
        ];

        for dir in directories {
            if dir.exists() {
                let mut entries = tokio::fs::read_dir(&dir)
                    .await
                    .map_err(WorkspaceError::Io)?;

                while let Some(entry) = entries.next_entry().await.map_err(WorkspaceError::Io)? {
                    if let Ok(metadata) = entry.metadata().await {
                        if metadata.is_file() {
                            total_files += 1;
                            total_size += metadata.len();

                            // Count by directory type
                            if dir.ends_with("imports") {
                                stats.import_count += 1;
                            } else if dir.ends_with("references") || dir.ends_with("archives") {
                                stats.reference_count += 1;
                            } else if dir.ends_with("drafts") || dir.ends_with("approved") {
                                stats.output_count += 1;
                            }

                            // Track last import/output times
                            if let Ok(modified) = metadata.modified() {
                                let modified_utc = DateTime::<Utc>::from(modified);

                                if dir.ends_with("imports") {
                                    stats.last_import = Some(
                                        stats
                                            .last_import
                                            .map(|last| last.max(modified_utc))
                                            .unwrap_or(modified_utc),
                                    );
                                } else if dir.ends_with("drafts") || dir.ends_with("approved") {
                                    stats.last_output = Some(
                                        stats
                                            .last_output
                                            .map(|last| last.max(modified_utc))
                                            .unwrap_or(modified_utc),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        stats.total_files = total_files;
        stats.total_size = total_size;

        Ok(stats)
    }
}

impl PathValidator {
    /// Validate a workspace path for creation
    pub fn validate_workspace_path(&self, path: &Path) -> Result<PathBuf, String> {
        // Use existing path validation but allow workspace creation
        let canonical_path = path
            .canonicalize()
            .or_else(|_| {
                // If path doesn't exist, try to canonicalize parent
                if let Some(parent) = path.parent() {
                    parent
                        .canonicalize()
                        .map(|p| p.join(path.file_name().unwrap_or_default()))
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Invalid path",
                    ))
                }
            })
            .map_err(|e| format!("Cannot resolve path: {}", e))?;

        // Check path length
        if canonical_path.as_os_str().len() > 260 {
            return Err("Path too long".to_string());
        }

        // Check for invalid characters
        if let Some(name) = canonical_path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.contains(['<', '>', ':', '"', '|', '?', '*']) {
                return Err("Path contains invalid characters".to_string());
            }
        }

        Ok(canonical_path)
    }
}

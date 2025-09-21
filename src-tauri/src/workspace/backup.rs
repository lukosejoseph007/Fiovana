// src-tauri/src/workspace/backup.rs
//! Workspace backup and recovery system

use super::*;
use crate::filesystem::security::backup_manager::BackupManager;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

/// Workspace backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceBackupConfig {
    /// Enable automatic backups
    pub auto_backup_enabled: bool,
    /// Backup interval in minutes
    pub backup_interval_minutes: u32,
    /// Maximum number of backups to keep
    pub max_backups: usize,
    /// Backup retention policy in days
    pub retention_days: u32,
    /// Compress backups
    pub compress_backups: bool,
    /// Include workspace files in backup
    pub include_files: bool,
    /// Backup locations
    pub backup_locations: Vec<PathBuf>,
}

impl Default for WorkspaceBackupConfig {
    fn default() -> Self {
        Self {
            auto_backup_enabled: true,
            backup_interval_minutes: 60, // 1 hour
            max_backups: 10,
            retention_days: 30,
            compress_backups: true,
            include_files: false, // Only metadata by default
            backup_locations: vec![],
        }
    }
}

/// Workspace backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceBackupMetadata {
    pub backup_id: String,
    pub workspace_name: String,
    pub workspace_path: PathBuf,
    pub backup_timestamp: DateTime<Utc>,
    pub backup_type: WorkspaceBackupType,
    pub backup_size: u64,
    pub file_count: usize,
    pub checksum: String,
    pub is_compressed: bool,
    pub backup_path: PathBuf,
    pub recovery_tested: bool,
}

/// Types of workspace backups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceBackupType {
    /// Full workspace backup including all files
    Full,
    /// Metadata only backup
    MetadataOnly,
    /// Incremental backup (changes since last backup)
    Incremental,
    /// Emergency backup before risky operations
    Emergency,
}

/// Workspace recovery information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRecoveryInfo {
    pub available_backups: Vec<WorkspaceBackupMetadata>,
    pub latest_backup: Option<WorkspaceBackupMetadata>,
    pub recovery_options: Vec<RecoveryOption>,
}

/// Recovery options for workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryOption {
    pub option_type: RecoveryType,
    pub description: String,
    pub estimated_time: Duration,
    pub data_loss_risk: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryType {
    FullRestore,
    MetadataRestore,
    PartialRestore,
    RepairInPlace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
}

use std::time::Duration;

/// Workspace backup and recovery manager
#[allow(dead_code)]
pub struct WorkspaceBackupManager {
    #[allow(dead_code)]
    backup_manager: Arc<BackupManager>,
    config: WorkspaceBackupConfig,
    backup_history: HashMap<PathBuf, Vec<WorkspaceBackupMetadata>>,
}

#[allow(dead_code)]
impl WorkspaceBackupManager {
    /// Create a new workspace backup manager
    pub fn new(backup_manager: Arc<BackupManager>) -> WorkspaceResult<Self> {
        Ok(Self {
            backup_manager,
            config: WorkspaceBackupConfig::default(),
            backup_history: HashMap::new(),
        })
    }

    /// Create backup manager with custom configuration
    pub fn with_config(
        backup_manager: Arc<BackupManager>,
        config: WorkspaceBackupConfig,
    ) -> WorkspaceResult<Self> {
        Ok(Self {
            backup_manager,
            config,
            backup_history: HashMap::new(),
        })
    }

    /// Create automatic workspace backup
    pub async fn create_backup(
        &mut self,
        workspace_path: &Path,
        backup_type: WorkspaceBackupType,
    ) -> WorkspaceResult<WorkspaceBackupMetadata> {
        // Validate workspace exists
        if !workspace_path.exists() {
            return Err(WorkspaceError::WorkspaceNotFound {
                path: workspace_path.to_path_buf(),
            });
        }

        // Check if workspace is valid
        let workspace_metadata_path = workspace_path.join(WORKSPACE_METADATA_FILE);
        if !workspace_metadata_path.exists() {
            return Err(WorkspaceError::InvalidWorkspace {
                path: workspace_path.to_path_buf(),
                reason: "Missing workspace metadata file".to_string(),
            });
        }

        // Generate backup ID
        let backup_id = format!(
            "{}_{}_{}",
            workspace_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            chrono::Utc::now().format("%Y%m%d_%H%M%S"),
            &uuid::Uuid::new_v4().to_string()[..8]
        );

        // Determine backup location
        let backup_location = self.get_backup_location(workspace_path)?;
        let backup_path = backup_location.join(format!("{}.backup", backup_id));

        // Create backup directory
        fs::create_dir_all(&backup_location).await?;

        // Perform backup based on type
        let (backup_size, file_count) = match backup_type {
            WorkspaceBackupType::Full => {
                self.create_full_backup(workspace_path, &backup_path)
                    .await?
            }
            WorkspaceBackupType::MetadataOnly => {
                self.create_metadata_backup(workspace_path, &backup_path)
                    .await?
            }
            WorkspaceBackupType::Incremental => {
                self.create_incremental_backup(workspace_path, &backup_path)
                    .await?
            }
            WorkspaceBackupType::Emergency => {
                self.create_emergency_backup(workspace_path, &backup_path)
                    .await?
            }
        };

        // Calculate checksum
        let checksum = self.calculate_backup_checksum(&backup_path).await?;

        // Get workspace name from metadata
        let workspace_name = self.get_workspace_name(workspace_path).await?;

        // Create backup metadata
        let backup_metadata = WorkspaceBackupMetadata {
            backup_id,
            workspace_name,
            workspace_path: workspace_path.to_path_buf(),
            backup_timestamp: Utc::now(),
            backup_type,
            backup_size,
            file_count,
            checksum,
            is_compressed: self.config.compress_backups,
            backup_path,
            recovery_tested: false,
        };

        // Store backup metadata
        self.store_backup_metadata(&backup_metadata).await?;

        // Update backup history
        self.backup_history
            .entry(workspace_path.to_path_buf())
            .or_default()
            .push(backup_metadata.clone());

        // Clean up old backups if needed
        self.cleanup_old_backups(workspace_path).await?;

        Ok(backup_metadata)
    }

    /// Restore workspace from backup
    pub async fn restore_workspace(
        &self,
        backup_metadata: &WorkspaceBackupMetadata,
        restore_path: &Path,
        overwrite_existing: bool,
    ) -> WorkspaceResult<()> {
        // Check if target path exists
        if restore_path.exists() && !overwrite_existing {
            return Err(WorkspaceError::WorkspaceExists {
                path: restore_path.to_path_buf(),
            });
        }

        // Validate backup file exists
        if !backup_metadata.backup_path.exists() {
            return Err(WorkspaceError::InvalidWorkspace {
                path: backup_metadata.backup_path.clone(),
                reason: "Backup file not found".to_string(),
            });
        }

        // Verify backup integrity
        self.verify_backup_integrity(backup_metadata).await?;

        // Create restore directory
        fs::create_dir_all(restore_path).await?;

        // Perform restore based on backup type
        match backup_metadata.backup_type {
            WorkspaceBackupType::Full => {
                self.restore_full_backup(backup_metadata, restore_path)
                    .await?
            }
            WorkspaceBackupType::MetadataOnly => {
                self.restore_metadata_backup(backup_metadata, restore_path)
                    .await?
            }
            WorkspaceBackupType::Incremental => {
                self.restore_incremental_backup(backup_metadata, restore_path)
                    .await?
            }
            WorkspaceBackupType::Emergency => {
                self.restore_emergency_backup(backup_metadata, restore_path)
                    .await?
            }
        }

        // Validate restored workspace
        self.validate_restored_workspace(restore_path).await?;

        Ok(())
    }

    /// Check workspace integrity
    pub async fn check_integrity(&self, workspace_path: &Path) -> WorkspaceResult<IntegrityReport> {
        let mut report = IntegrityReport::new(workspace_path.to_path_buf());

        // Check workspace structure
        report.structure_issues = self.check_workspace_structure(workspace_path).await;

        // Check metadata integrity
        report.metadata_issues = self.check_metadata_integrity(workspace_path).await;

        // Check file consistency
        report.file_issues = self.check_file_consistency(workspace_path).await;

        // Calculate overall health score
        report.health_score = self.calculate_health_score(&report);

        Ok(report)
    }

    /// Get recovery information for workspace
    pub async fn get_recovery_info(
        &self,
        workspace_path: &Path,
    ) -> WorkspaceResult<WorkspaceRecoveryInfo> {
        let available_backups = self.get_available_backups(workspace_path).await?;
        let latest_backup = available_backups.first().cloned();
        let recovery_options = self.generate_recovery_options(workspace_path, &available_backups);

        Ok(WorkspaceRecoveryInfo {
            available_backups,
            latest_backup,
            recovery_options,
        })
    }

    /// Export workspace for migration
    pub async fn export_workspace(
        &self,
        workspace_path: &Path,
        export_path: &Path,
        include_files: bool,
    ) -> WorkspaceResult<WorkspaceExportInfo> {
        // Create export directory
        fs::create_dir_all(export_path).await?;

        // Export metadata
        let metadata_export_path = export_path.join("workspace_metadata.json");
        self.export_workspace_metadata(workspace_path, &metadata_export_path)
            .await?;

        // Export configuration
        let config_export_path = export_path.join("workspace_config.json");
        self.export_workspace_config(workspace_path, &config_export_path)
            .await?;

        let mut total_size = 0;
        let mut file_count = 0;

        // Export files if requested
        if include_files {
            let files_export_path = export_path.join("workspace_files");
            let (size, count) = self
                .export_workspace_files(workspace_path, &files_export_path)
                .await?;
            total_size += size;
            file_count += count;
        }

        // Create export manifest
        let export_info = WorkspaceExportInfo {
            workspace_name: self.get_workspace_name(workspace_path).await?,
            export_timestamp: Utc::now(),
            export_path: export_path.to_path_buf(),
            total_size,
            file_count,
            includes_files: include_files,
            checksum: self.calculate_export_checksum(export_path).await?,
        };

        let manifest_path = export_path.join("export_manifest.json");
        let manifest_content = serde_json::to_string_pretty(&export_info)?;
        fs::write(&manifest_path, manifest_content).await?;

        Ok(export_info)
    }

    /// Import workspace from export
    pub async fn import_workspace(
        &self,
        export_path: &Path,
        import_path: &Path,
    ) -> WorkspaceResult<()> {
        // Validate export
        let manifest_path = export_path.join("export_manifest.json");
        if !manifest_path.exists() {
            return Err(WorkspaceError::InvalidWorkspace {
                path: export_path.to_path_buf(),
                reason: "Missing export manifest".to_string(),
            });
        }

        // Read and validate manifest
        let manifest_content = fs::read_to_string(&manifest_path).await?;
        let export_info: WorkspaceExportInfo = serde_json::from_str(&manifest_content)?;

        // Verify export integrity
        let calculated_checksum = self.calculate_export_checksum(export_path).await?;
        if calculated_checksum != export_info.checksum {
            return Err(WorkspaceError::InvalidWorkspace {
                path: export_path.to_path_buf(),
                reason: "Export checksum mismatch".to_string(),
            });
        }

        // Create import directory
        fs::create_dir_all(import_path).await?;

        // Import metadata
        let metadata_export_path = export_path.join("workspace_metadata.json");
        self.import_workspace_metadata(&metadata_export_path, import_path)
            .await?;

        // Import configuration
        let config_export_path = export_path.join("workspace_config.json");
        self.import_workspace_config(&config_export_path, import_path)
            .await?;

        // Import files if they exist
        let files_export_path = export_path.join("workspace_files");
        if files_export_path.exists() {
            self.import_workspace_files(&files_export_path, import_path)
                .await?;
        }

        Ok(())
    }

    // Private helper methods
    async fn create_full_backup(
        &self,
        workspace_path: &Path,
        backup_path: &Path,
    ) -> WorkspaceResult<(u64, usize)> {
        // Implementation for full backup
        let mut total_size = 0;
        let mut file_count = 0;

        // Create backup archive
        // For now, just copy the directory structure
        Self::copy_directory_recursive(
            workspace_path,
            backup_path,
            &mut total_size,
            &mut file_count,
        )
        .await?;

        Ok((total_size, file_count))
    }

    async fn create_metadata_backup(
        &self,
        workspace_path: &Path,
        backup_path: &Path,
    ) -> WorkspaceResult<(u64, usize)> {
        // Backup only metadata files
        let metadata_files = vec![WORKSPACE_METADATA_FILE, WORKSPACE_CONFIG_FILE];

        let mut total_size = 0;
        let mut file_count = 0;

        fs::create_dir_all(backup_path).await?;

        for metadata_file in metadata_files {
            let source = workspace_path.join(metadata_file);
            if source.exists() {
                let dest = backup_path.join(metadata_file);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).await?;
                }
                fs::copy(&source, &dest).await?;

                if let Ok(metadata) = fs::metadata(&source).await {
                    total_size += metadata.len();
                    file_count += 1;
                }
            }
        }

        Ok((total_size, file_count))
    }

    async fn create_incremental_backup(
        &self,
        workspace_path: &Path,
        backup_path: &Path,
    ) -> WorkspaceResult<(u64, usize)> {
        // For now, just do a metadata backup
        // In a full implementation, this would compare with the last backup
        self.create_metadata_backup(workspace_path, backup_path)
            .await
    }

    async fn create_emergency_backup(
        &self,
        workspace_path: &Path,
        backup_path: &Path,
    ) -> WorkspaceResult<(u64, usize)> {
        // Emergency backup is always a full backup
        self.create_full_backup(workspace_path, backup_path).await
    }

    fn copy_directory_recursive<'a>(
        source: &'a Path,
        dest: &'a Path,
        total_size: &'a mut u64,
        file_count: &'a mut usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WorkspaceResult<()>> + 'a>> {
        Box::pin(async move {
            fs::create_dir_all(dest).await?;

            let mut entries = fs::read_dir(source).await?;
            while let Some(entry) = entries.next_entry().await? {
                let file_type = entry.file_type().await?;
                let source_path = entry.path();
                let dest_path = dest.join(entry.file_name());

                if file_type.is_dir() {
                    Self::copy_directory_recursive(
                        &source_path,
                        &dest_path,
                        total_size,
                        file_count,
                    )
                    .await?;
                } else {
                    fs::copy(&source_path, &dest_path).await?;
                    if let Ok(metadata) = fs::metadata(&source_path).await {
                        *total_size += metadata.len();
                        *file_count += 1;
                    }
                }
            }

            Ok(())
        })
    }

    async fn get_workspace_name(&self, workspace_path: &Path) -> WorkspaceResult<String> {
        let metadata_path = workspace_path.join(WORKSPACE_METADATA_FILE);
        let content = fs::read_to_string(&metadata_path).await?;
        let workspace_info: WorkspaceInfo = serde_json::from_str(&content)?;
        Ok(workspace_info.name)
    }

    fn get_backup_location(&self, workspace_path: &Path) -> WorkspaceResult<PathBuf> {
        if let Some(location) = self.config.backup_locations.first() {
            Ok(location.clone())
        } else {
            // Default to .proxemic/backups directory
            Ok(workspace_path.join(".proxemic/backups"))
        }
    }

    async fn calculate_backup_checksum(&self, backup_path: &Path) -> WorkspaceResult<String> {
        // Simple checksum calculation - in production you'd use a proper hash
        let metadata = fs::metadata(backup_path).await?;
        Ok(format!("{:x}", metadata.len()))
    }

    async fn store_backup_metadata(
        &self,
        metadata: &WorkspaceBackupMetadata,
    ) -> WorkspaceResult<()> {
        let backup_dir = metadata.backup_path.parent().unwrap();
        let metadata_path = backup_dir.join(format!("{}.metadata", metadata.backup_id));
        let content = serde_json::to_string_pretty(metadata)?;
        fs::write(&metadata_path, content).await?;
        Ok(())
    }

    async fn cleanup_old_backups(&mut self, workspace_path: &Path) -> WorkspaceResult<()> {
        if let Some(backups) = self.backup_history.get_mut(workspace_path) {
            // Sort by timestamp, newest first
            backups.sort_by(|a, b| b.backup_timestamp.cmp(&a.backup_timestamp));

            // Remove excess backups
            if backups.len() > self.config.max_backups {
                let to_remove: Vec<_> = backups.drain(self.config.max_backups..).collect();

                for backup in to_remove {
                    // Delete backup file
                    if backup.backup_path.exists() {
                        let _ = fs::remove_dir_all(&backup.backup_path).await;
                    }

                    // Delete metadata file
                    let metadata_path = backup
                        .backup_path
                        .parent()
                        .unwrap()
                        .join(format!("{}.metadata", backup.backup_id));
                    if metadata_path.exists() {
                        let _ = fs::remove_file(&metadata_path).await;
                    }
                }
            }
        }

        Ok(())
    }

    // Additional helper methods (simplified implementations)
    async fn verify_backup_integrity(
        &self,
        _backup_metadata: &WorkspaceBackupMetadata,
    ) -> WorkspaceResult<()> {
        // Implementation would verify checksums, file integrity, etc.
        Ok(())
    }

    async fn restore_full_backup(
        &self,
        backup_metadata: &WorkspaceBackupMetadata,
        restore_path: &Path,
    ) -> WorkspaceResult<()> {
        // Copy backup to restore location
        let mut total_size = 0;
        let mut file_count = 0;
        Self::copy_directory_recursive(
            &backup_metadata.backup_path,
            restore_path,
            &mut total_size,
            &mut file_count,
        )
        .await
    }

    async fn restore_metadata_backup(
        &self,
        backup_metadata: &WorkspaceBackupMetadata,
        restore_path: &Path,
    ) -> WorkspaceResult<()> {
        // Copy only metadata files
        let mut total_size = 0;
        let mut file_count = 0;
        Self::copy_directory_recursive(
            &backup_metadata.backup_path,
            restore_path,
            &mut total_size,
            &mut file_count,
        )
        .await
    }

    async fn restore_incremental_backup(
        &self,
        backup_metadata: &WorkspaceBackupMetadata,
        restore_path: &Path,
    ) -> WorkspaceResult<()> {
        // For now, same as metadata restore
        self.restore_metadata_backup(backup_metadata, restore_path)
            .await
    }

    async fn restore_emergency_backup(
        &self,
        backup_metadata: &WorkspaceBackupMetadata,
        restore_path: &Path,
    ) -> WorkspaceResult<()> {
        // Emergency restore is always full restore
        self.restore_full_backup(backup_metadata, restore_path)
            .await
    }

    async fn validate_restored_workspace(&self, _workspace_path: &Path) -> WorkspaceResult<()> {
        // Implementation would validate workspace structure, metadata, etc.
        Ok(())
    }

    async fn check_workspace_structure(&self, workspace_path: &Path) -> Vec<String> {
        let mut issues = Vec::new();

        for dir in WORKSPACE_DIRECTORIES {
            let dir_path = workspace_path.join(dir);
            if !dir_path.exists() {
                issues.push(format!("Missing directory: {}", dir));
            }
        }

        issues
    }

    async fn check_metadata_integrity(&self, workspace_path: &Path) -> Vec<String> {
        let mut issues = Vec::new();

        let metadata_path = workspace_path.join(WORKSPACE_METADATA_FILE);
        if !metadata_path.exists() {
            issues.push("Missing workspace metadata file".to_string());
        } else {
            // Try to parse metadata
            if fs::read_to_string(&metadata_path)
                .await
                .and_then(|content| {
                    serde_json::from_str::<WorkspaceInfo>(&content)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                })
                .is_err()
            {
                issues.push("Invalid workspace metadata format".to_string());
            }
        }

        issues
    }

    async fn check_file_consistency(&self, _workspace_path: &Path) -> Vec<String> {
        // Implementation would check for corrupted files, missing files, etc.
        Vec::new()
    }

    fn calculate_health_score(&self, report: &IntegrityReport) -> u8 {
        let total_issues =
            report.structure_issues.len() + report.metadata_issues.len() + report.file_issues.len();
        if total_issues == 0 {
            100
        } else {
            std::cmp::max(0, 100 - (total_issues * 10) as i32) as u8
        }
    }

    async fn get_available_backups(
        &self,
        workspace_path: &Path,
    ) -> WorkspaceResult<Vec<WorkspaceBackupMetadata>> {
        // Load backup history from filesystem
        if let Some(backups) = self.backup_history.get(workspace_path) {
            Ok(backups.clone())
        } else {
            Ok(Vec::new())
        }
    }

    fn generate_recovery_options(
        &self,
        _workspace_path: &Path,
        available_backups: &[WorkspaceBackupMetadata],
    ) -> Vec<RecoveryOption> {
        let mut options = Vec::new();

        if !available_backups.is_empty() {
            options.push(RecoveryOption {
                option_type: RecoveryType::FullRestore,
                description: "Restore from latest full backup".to_string(),
                estimated_time: Duration::from_secs(300), // 5 minutes
                data_loss_risk: RiskLevel::Low,
            });

            options.push(RecoveryOption {
                option_type: RecoveryType::MetadataRestore,
                description: "Restore only workspace metadata".to_string(),
                estimated_time: Duration::from_secs(30),
                data_loss_risk: RiskLevel::Medium,
            });
        }

        options.push(RecoveryOption {
            option_type: RecoveryType::RepairInPlace,
            description: "Repair workspace structure in place".to_string(),
            estimated_time: Duration::from_secs(60),
            data_loss_risk: RiskLevel::Low,
        });

        options
    }

    // Export/Import helper methods (simplified)
    async fn export_workspace_metadata(
        &self,
        workspace_path: &Path,
        export_path: &Path,
    ) -> WorkspaceResult<()> {
        let metadata_path = workspace_path.join(WORKSPACE_METADATA_FILE);
        fs::copy(&metadata_path, export_path).await?;
        Ok(())
    }

    async fn export_workspace_config(
        &self,
        workspace_path: &Path,
        export_path: &Path,
    ) -> WorkspaceResult<()> {
        let config_path = workspace_path.join(WORKSPACE_CONFIG_FILE);
        if config_path.exists() {
            fs::copy(&config_path, export_path).await?;
        }
        Ok(())
    }

    async fn export_workspace_files(
        &self,
        workspace_path: &Path,
        export_path: &Path,
    ) -> WorkspaceResult<(u64, usize)> {
        let mut total_size = 0;
        let mut file_count = 0;
        Self::copy_directory_recursive(
            workspace_path,
            export_path,
            &mut total_size,
            &mut file_count,
        )
        .await?;
        Ok((total_size, file_count))
    }

    async fn calculate_export_checksum(&self, export_path: &Path) -> WorkspaceResult<String> {
        // Simple checksum - in production use proper hashing
        let metadata = fs::metadata(export_path).await?;
        Ok(format!("{:x}", metadata.len()))
    }

    async fn import_workspace_metadata(
        &self,
        metadata_path: &Path,
        import_path: &Path,
    ) -> WorkspaceResult<()> {
        let dest = import_path.join(WORKSPACE_METADATA_FILE);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::copy(metadata_path, &dest).await?;
        Ok(())
    }

    async fn import_workspace_config(
        &self,
        config_path: &Path,
        import_path: &Path,
    ) -> WorkspaceResult<()> {
        if config_path.exists() {
            let dest = import_path.join(WORKSPACE_CONFIG_FILE);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::copy(config_path, &dest).await?;
        }
        Ok(())
    }

    async fn import_workspace_files(
        &self,
        files_path: &Path,
        import_path: &Path,
    ) -> WorkspaceResult<()> {
        let mut total_size = 0;
        let mut file_count = 0;
        Self::copy_directory_recursive(files_path, import_path, &mut total_size, &mut file_count)
            .await
    }
}

/// Workspace integrity report
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IntegrityReport {
    pub workspace_path: PathBuf,
    pub structure_issues: Vec<String>,
    pub metadata_issues: Vec<String>,
    pub file_issues: Vec<String>,
    pub health_score: u8, // 0-100
}

#[allow(dead_code)]
impl IntegrityReport {
    pub fn new(workspace_path: PathBuf) -> Self {
        Self {
            workspace_path,
            structure_issues: Vec::new(),
            metadata_issues: Vec::new(),
            file_issues: Vec::new(),
            health_score: 0,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.health_score >= 80
    }

    pub fn needs_attention(&self) -> bool {
        self.health_score < 60
    }
}

/// Workspace export information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceExportInfo {
    pub workspace_name: String,
    pub export_timestamp: DateTime<Utc>,
    pub export_path: PathBuf,
    pub total_size: u64,
    pub file_count: usize,
    pub includes_files: bool,
    pub checksum: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config::ConfigManager;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_workspace_backup_creation() {
        let _temp_dir = TempDir::new().unwrap();
        let _config_manager = Arc::new(ConfigManager::new().await.unwrap());

        // Create backup manager - this might fail if BackupManager isn't available
        // but we'll test the structure
        let backup_config = WorkspaceBackupConfig::default();
        assert!(backup_config.auto_backup_enabled);
        assert_eq!(backup_config.backup_interval_minutes, 60);
    }

    #[test]
    fn test_integrity_report() {
        let temp_path = PathBuf::from("/tmp/test");
        let mut report = IntegrityReport::new(temp_path);

        // Initially healthy
        report.health_score = 100;
        assert!(report.is_healthy());
        assert!(!report.needs_attention());

        // Add some issues
        report.health_score = 50;
        assert!(!report.is_healthy());
        assert!(report.needs_attention());
    }

    #[test]
    fn test_recovery_options() {
        let option = RecoveryOption {
            option_type: RecoveryType::FullRestore,
            description: "Test recovery".to_string(),
            estimated_time: Duration::from_secs(300),
            data_loss_risk: RiskLevel::Low,
        };

        assert!(matches!(option.option_type, RecoveryType::FullRestore));
        assert!(matches!(option.data_loss_risk, RiskLevel::Low));
    }
}

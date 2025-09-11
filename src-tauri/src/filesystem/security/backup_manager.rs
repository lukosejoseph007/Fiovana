use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BackupPriority {
    Critical, // Security configs, audit logs
    High,     // App configs, user data
    Medium,   // Cache, temporary files
    Low,      // Logs, analytics
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub max_backups: usize,
    pub retention_days: u32,
    pub backup_interval_hours: u32,
    pub priority_levels: HashMap<BackupPriority, BackupPriorityConfig>,
    pub backup_directory: PathBuf,
    pub encryption_enabled: bool,
    pub integrity_checks: bool,
    pub auto_cleanup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPriorityConfig {
    pub enabled: bool,
    pub frequency_hours: u32,
    pub retention_days: u32,
    pub max_size_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub backup_type: String,
    pub priority: BackupPriority,
    pub size_bytes: u64,
    pub checksum: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub files_backed_up: Vec<String>,
    pub compression_ratio: Option<f32>,
}

pub struct BackupManager {
    config: Arc<RwLock<BackupConfig>>,
    backup_history: Arc<RwLock<Vec<BackupMetadata>>>,
}

impl BackupManager {
    pub fn new() -> Result<Self> {
        let config = Self::default_config();

        // Ensure backup directory exists
        if let Some(parent) = config.backup_directory.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(&config.backup_directory)?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            backup_history: Arc::new(RwLock::new(Vec::new())),
        })
    }

    fn default_config() -> BackupConfig {
        let mut priority_levels = HashMap::new();
        priority_levels.insert(
            BackupPriority::Critical,
            BackupPriorityConfig {
                enabled: true,
                frequency_hours: 1,
                retention_days: 30,
                max_size_mb: 100,
            },
        );
        priority_levels.insert(
            BackupPriority::High,
            BackupPriorityConfig {
                enabled: true,
                frequency_hours: 6,
                retention_days: 14,
                max_size_mb: 500,
            },
        );
        priority_levels.insert(
            BackupPriority::Medium,
            BackupPriorityConfig {
                enabled: true,
                frequency_hours: 24,
                retention_days: 7,
                max_size_mb: 1000,
            },
        );
        priority_levels.insert(
            BackupPriority::Low,
            BackupPriorityConfig {
                enabled: false,
                frequency_hours: 168,
                retention_days: 3,
                max_size_mb: 2000,
            },
        );

        BackupConfig {
            enabled: true,
            max_backups: 10,
            retention_days: 30,
            backup_interval_hours: 24,
            priority_levels,
            backup_directory: PathBuf::from("./backups"),
            encryption_enabled: false,
            integrity_checks: true,
            auto_cleanup: true,
        }
    }

    fn backup_security_configs(
        config: &BackupConfig,
        backup_history: &Arc<RwLock<Vec<BackupMetadata>>>,
    ) -> Result<BackupMetadata> {
        let backup_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let mut files_backed_up = Vec::new();

        // Backup security configuration files
        let security_files = [
            "security_config.json",
            "magic_number_rules.json",
            "safe_mode_config.json",
            "circuit_breaker_config.json",
        ];

        for file_name in &security_files {
            let source_path = Path::new("./config").join(file_name);
            if source_path.exists() {
                let backup_path = config.backup_directory.join("security").join(format!(
                    "{}_{}",
                    timestamp.timestamp(),
                    file_name
                ));

                if let Err(e) = fs::copy(&source_path, &backup_path) {
                    log::warn!("Failed to backup {}: {}", file_name, e);
                    continue;
                }

                files_backed_up.push(file_name.to_string());
            }
        }

        let metadata = BackupMetadata {
            id: backup_id,
            timestamp,
            backup_type: "security".to_string(),
            priority: BackupPriority::Critical,
            size_bytes: Self::calculate_backup_size(&config.backup_directory.join("security"))?,
            checksum: Self::calculate_checksum(&config.backup_directory.join("security"))?,
            success: !files_backed_up.is_empty(),
            error_message: if files_backed_up.is_empty() {
                Some("No security files backed up".to_string())
            } else {
                None
            },
            files_backed_up,
            compression_ratio: None,
        };

        let mut history = backup_history.write().unwrap();
        history.push(metadata.clone());

        log::info!(
            "Security config backup completed: {} files",
            metadata.files_backed_up.len()
        );
        Ok(metadata)
    }

    fn backup_app_configs(
        config: &BackupConfig,
        backup_history: &Arc<RwLock<Vec<BackupMetadata>>>,
    ) -> Result<BackupMetadata> {
        let backup_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let mut files_backed_up = Vec::new();

        // Backup application configuration files
        let app_files = [
            "app_config.json",
            "user_preferences.json",
            "workspace_config.json",
        ];

        for file_name in &app_files {
            let source_path = Path::new("./config").join(file_name);
            if source_path.exists() {
                let backup_path = config.backup_directory.join("app").join(format!(
                    "{}_{}",
                    timestamp.timestamp(),
                    file_name
                ));

                if let Err(e) = fs::copy(&source_path, &backup_path) {
                    log::warn!("Failed to backup {}: {}", file_name, e);
                    continue;
                }

                files_backed_up.push(file_name.to_string());
            }
        }

        let metadata = BackupMetadata {
            id: backup_id,
            timestamp,
            backup_type: "application".to_string(),
            priority: BackupPriority::High,
            size_bytes: Self::calculate_backup_size(&config.backup_directory.join("app"))?,
            checksum: Self::calculate_checksum(&config.backup_directory.join("app"))?,
            success: !files_backed_up.is_empty(),
            error_message: if files_backed_up.is_empty() {
                Some("No app files backed up".to_string())
            } else {
                None
            },
            files_backed_up,
            compression_ratio: None,
        };

        let mut history = backup_history.write().unwrap();
        history.push(metadata.clone());

        log::info!(
            "App config backup completed: {} files",
            metadata.files_backed_up.len()
        );
        Ok(metadata)
    }

    fn backup_audit_logs(
        config: &BackupConfig,
        backup_history: &Arc<RwLock<Vec<BackupMetadata>>>,
    ) -> Result<BackupMetadata> {
        let backup_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let mut files_backed_up = Vec::new();

        // Backup audit log files
        let log_files = ["security_audit.log", "operation_audit.log", "error.log"];

        for file_name in &log_files {
            let source_path = Path::new("./logs").join(file_name);
            if source_path.exists() {
                let backup_path = config.backup_directory.join("logs").join(format!(
                    "{}_{}",
                    timestamp.timestamp(),
                    file_name
                ));

                if let Err(e) = fs::copy(&source_path, &backup_path) {
                    log::warn!("Failed to backup {}: {}", file_name, e);
                    continue;
                }

                files_backed_up.push(file_name.to_string());
            }
        }

        let metadata = BackupMetadata {
            id: backup_id,
            timestamp,
            backup_type: "audit_logs".to_string(),
            priority: BackupPriority::Medium,
            size_bytes: Self::calculate_backup_size(&config.backup_directory.join("logs"))?,
            checksum: Self::calculate_checksum(&config.backup_directory.join("logs"))?,
            success: !files_backed_up.is_empty(),
            error_message: if files_backed_up.is_empty() {
                Some("No log files backed up".to_string())
            } else {
                None
            },
            files_backed_up,
            compression_ratio: None,
        };

        let mut history = backup_history.write().unwrap();
        history.push(metadata.clone());

        log::info!(
            "Audit log backup completed: {} files",
            metadata.files_backed_up.len()
        );
        Ok(metadata)
    }

    fn calculate_backup_size(path: &Path) -> Result<u64> {
        if !path.exists() {
            return Ok(0);
        }

        let mut total_size = 0;
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let metadata = entry.metadata()?;
                total_size += metadata.len();
            }
        } else {
            total_size = fs::metadata(path)?.len();
        }

        Ok(total_size)
    }

    fn calculate_checksum(path: &Path) -> Result<String> {
        if !path.exists() {
            return Ok("".to_string());
        }

        let mut hasher = Sha256::new();
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                if entry.path().is_file() {
                    let content = fs::read(entry.path())?;
                    hasher.update(&content);
                }
            }
        } else {
            let content = fs::read(path)?;
            hasher.update(&content);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn create_manual_backup(&self, backup_type: &str) -> Result<BackupMetadata> {
        let config = self.config.read().unwrap().clone();
        let backup_history = self.backup_history.clone();

        match backup_type {
            "security" => Self::backup_security_configs(&config, &backup_history),
            "application" => Self::backup_app_configs(&config, &backup_history),
            "logs" => Self::backup_audit_logs(&config, &backup_history),
            _ => Err(anyhow!("Unknown backup type: {}", backup_type)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backup_manager_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut config = BackupManager::default_config();
        config.backup_directory = temp_dir.path().to_path_buf();

        let manager = BackupManager {
            config: Arc::new(RwLock::new(config)),
            backup_history: Arc::new(RwLock::new(Vec::new())),
        };

        // Test that manager was created successfully
        assert!(manager.config.read().unwrap().enabled);
        Ok(())
    }

    #[test]
    fn test_backup_metadata_serialization() -> Result<()> {
        let metadata = BackupMetadata {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            backup_type: "test".to_string(),
            priority: BackupPriority::High,
            size_bytes: 1024,
            checksum: "test_checksum".to_string(),
            success: true,
            error_message: None,
            files_backed_up: vec!["test.txt".to_string()],
            compression_ratio: Some(0.8),
        };

        let serialized = serde_json::to_string(&metadata)?;
        let deserialized: BackupMetadata = serde_json::from_str(&serialized)?;

        assert_eq!(metadata.id, deserialized.id);
        assert_eq!(metadata.backup_type, deserialized.backup_type);
        Ok(())
    }
}

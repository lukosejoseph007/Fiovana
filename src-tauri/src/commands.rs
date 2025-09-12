// src-tauri/src/commands.rs

use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::audit_logger::SecurityAuditor;
use crate::filesystem::security::backup_manager::BackupManager;
use crate::filesystem::security::circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager};
use crate::filesystem::security::emergency_procedures::EmergencyManager;
use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::security::safe_mode::SafeModeManager;
use crate::filesystem::security::security_config::SecurityConfig;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Performance metrics for validation commands
#[derive(Debug, Default)]
pub struct ValidationMetrics {
    pub total_validations: AtomicU64,
    pub successful_validations: AtomicU64,
    pub failed_validations: AtomicU64,
    pub total_validation_time_ns: AtomicU64,
}

impl ValidationMetrics {
    /// Record a successful validation with timing
    pub fn record_success(&self, duration: std::time::Duration) {
        self.total_validations.fetch_add(1, Ordering::Relaxed);
        self.successful_validations.fetch_add(1, Ordering::Relaxed);
        self.total_validation_time_ns
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Record a failed validation with timing
    pub fn record_failure(&self, duration: std::time::Duration) {
        self.total_validations.fetch_add(1, Ordering::Relaxed);
        self.failed_validations.fetch_add(1, Ordering::Relaxed);
        self.total_validation_time_ns
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Get the average validation time in nanoseconds
    #[allow(dead_code)]
    pub fn average_validation_time_ns(&self) -> u64 {
        let total_validations = self.total_validations.load(Ordering::Relaxed);
        if total_validations == 0 {
            return 0;
        }
        self.total_validation_time_ns.load(Ordering::Relaxed) / total_validations
    }

    /// Get the validation failure rate as a percentage (0-100)
    #[allow(dead_code)]
    pub fn failure_rate(&self) -> f64 {
        let total_validations = self.total_validations.load(Ordering::Relaxed);
        if total_validations == 0 {
            return 0.0;
        }
        let failed_validations = self.failed_validations.load(Ordering::Relaxed);
        (failed_validations as f64 / total_validations as f64) * 100.0
    }

    /// Reset all metrics to zero
    #[allow(dead_code)]
    pub fn reset(&self) {
        self.total_validations.store(0, Ordering::Relaxed);
        self.successful_validations.store(0, Ordering::Relaxed);
        self.failed_validations.store(0, Ordering::Relaxed);
        self.total_validation_time_ns.store(0, Ordering::Relaxed);
    }
}

// Global validation metrics instance
static VALIDATION_METRICS: Lazy<ValidationMetrics> = Lazy::new(ValidationMetrics::default);

// Global circuit breaker manager for Tauri commands
static CIRCUIT_BREAKER_MANAGER: Lazy<CircuitBreakerManager> = Lazy::new(CircuitBreakerManager::new);

// Global backup manager for configuration backups
static BACKUP_MANAGER: Lazy<BackupManager> =
    Lazy::new(|| BackupManager::new().expect("Failed to initialize backup manager"));

/// Backup security configurations after important operations
/// This is called automatically after successful Tauri commands
fn backup_security_configurations() {
    if let Ok(metadata) = BACKUP_MANAGER.create_manual_backup("tauri_security") {
        if metadata.success {
            log::info!("Security configuration backup completed successfully");
        } else {
            log::warn!(
                "Security configuration backup failed: {:?}",
                metadata.error_message
            );
        }
    } else {
        log::error!("Failed to initiate security configuration backup");
    }
}

/// Get a reference to the global validation metrics
#[allow(dead_code)]
pub fn get_validation_metrics() -> &'static ValidationMetrics {
    &VALIDATION_METRICS
}

use crate::filesystem::watcher::{DocumentWatcher, FileEvent, WatcherConfig};
use tokio::sync::{mpsc, Mutex};

// Global file watcher instance
static FILE_WATCHER: Lazy<Mutex<Option<DocumentWatcher>>> = Lazy::new(|| Mutex::new(None));
static EVENT_RECEIVER: Lazy<Mutex<Option<mpsc::UnboundedReceiver<FileEvent>>>> =
    Lazy::new(|| Mutex::new(None));

#[derive(Debug, Serialize, serde::Deserialize)]
pub enum CommandError {
    SecurityError {
        message: String,
        code: String,
        severity: String,
    },
    IoError(String),
    Custom(String),
}

impl From<SecurityError> for CommandError {
    fn from(err: SecurityError) -> CommandError {
        CommandError::SecurityError {
            message: err.to_string(),
            code: err.code().to_string(),
            severity: err.severity().to_string(),
        }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> CommandError {
        CommandError::IoError(err.to_string())
    }
}

impl From<String> for CommandError {
    fn from(err: String) -> CommandError {
        CommandError::Custom(err)
    }
}

#[derive(Serialize)]
pub struct FileInfo {
    size: u64,
    modified: Option<std::time::SystemTime>,
    is_file: bool,
    is_dir: bool,
}

/// Helper function to create a properly configured validator for production use
fn create_default_validator() -> PathValidator {
    let mut config = SecurityConfig::default();
    // Ensure common extensions are allowed
    config.allowed_extensions.insert(".txt".to_string());

    let mut allowed_paths = vec![
        dirs::desktop_dir().unwrap_or_default(),
        dirs::document_dir().unwrap_or_default(),
        dirs::download_dir().unwrap_or_default(),
    ];

    // Also allow temp directory for temporary operations
    allowed_paths.push(std::env::temp_dir());

    PathValidator::new(config, allowed_paths)
}

// ---------------- Standard Tauri Commands ----------------

#[tauri::command]
pub async fn greet(name: String) -> Result<String, String> {
    Ok(format!("Hello, {}!", name))
}

#[tauri::command]
pub async fn validate_file_for_import(path: String) -> Result<String, CommandError> {
    let start_time = Instant::now();

    // First check emergency restrictions
    let emergency_manager = EmergencyManager::new().map_err(|e| CommandError::SecurityError {
        message: format!("Emergency system error: {}", e),
        code: "EMERGENCY_ERROR".to_string(),
        severity: "high".to_string(),
    })?;

    if emergency_manager.is_kill_switch_active() {
        return Err(CommandError::SecurityError {
            message: "Kill switch active - all operations disabled".to_string(),
            code: "KILL_SWITCH_ACTIVE".to_string(),
            severity: "critical".to_string(),
        });
    }

    if !emergency_manager.can_perform_operation("validate") {
        return Err(CommandError::SecurityError {
            message: "Operation blocked by emergency restrictions".to_string(),
            code: "EMERGENCY_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Then check safe mode restrictions
    if !SafeModeManager::global()
        .is_file_allowed(Path::new(&path))
        .map_err(|e| CommandError::SecurityError {
            message: format!("Safe mode restriction: {}", e),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        })?
    {
        return Err(CommandError::SecurityError {
            message: "File blocked by safe mode restrictions".to_string(),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Use circuit breaker for validation
    let breaker = CIRCUIT_BREAKER_MANAGER.get_or_create(
        "tauri_file_validation",
        Some(CircuitBreakerConfig {
            failure_threshold: 5,
            recovery_timeout: std::time::Duration::from_secs(60),
            success_threshold: 3,
        }),
    );

    let validator = create_default_validator();
    let result = breaker.call(|| {
        validator
            .validate_import_path(Path::new(&path))
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| anyhow::anyhow!("{}", e))
    });

    // Convert result to expected type for audit logging
    let audit_result: Result<PathBuf, SecurityError> = result
        .as_ref()
        .map(|_| Path::new(&path).to_path_buf())
        .map_err(|e| SecurityError::PathOutsideWorkspace {
            path: e.to_string(),
        });

    SecurityAuditor::log_file_access_attempt(
        Path::new(&path),
        "validate_import",
        &audit_result,
        "development",
        None,
    );

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => VALIDATION_METRICS.record_success(duration),
        Err(_) => VALIDATION_METRICS.record_failure(duration),
    }

    let final_result = result.map_err(|e| CommandError::SecurityError {
        message: e.to_string(),
        code: "CIRCUIT_BREAKER".to_string(),
        severity: "medium".to_string(),
    });

    // Backup security configurations after successful validation
    if final_result.is_ok() {
        backup_security_configurations();
    }

    final_result
}

#[tauri::command]
pub async fn get_file_info_secure(path: String) -> Result<FileInfo, CommandError> {
    let start_time = Instant::now();

    // First check emergency restrictions
    let emergency_manager = EmergencyManager::new().map_err(|e| CommandError::SecurityError {
        message: format!("Emergency system error: {}", e),
        code: "EMERGENCY_ERROR".to_string(),
        severity: "high".to_string(),
    })?;

    if emergency_manager.is_kill_switch_active() {
        return Err(CommandError::SecurityError {
            message: "Kill switch active - all operations disabled".to_string(),
            code: "KILL_SWITCH_ACTIVE".to_string(),
            severity: "critical".to_string(),
        });
    }

    if !emergency_manager.can_perform_operation("read") {
        return Err(CommandError::SecurityError {
            message: "Operation blocked by emergency restrictions".to_string(),
            code: "EMERGENCY_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Then check safe mode restrictions
    if !SafeModeManager::global()
        .is_file_allowed(Path::new(&path))
        .map_err(|e| CommandError::SecurityError {
            message: format!("Safe mode restriction: {}", e),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        })?
    {
        return Err(CommandError::SecurityError {
            message: "File blocked by safe mode restrictions".to_string(),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Use circuit breaker for validation
    let breaker = CIRCUIT_BREAKER_MANAGER.get_or_create(
        "tauri_file_info",
        Some(CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: std::time::Duration::from_secs(30),
            success_threshold: 2,
        }),
    );

    let validator = create_default_validator();
    let result = breaker.call(|| {
        let validated_path = validator
            .validate_import_path(Path::new(&path))
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let metadata = fs::metadata(&validated_path)
            .map_err(|e| anyhow::anyhow!("Failed to get file metadata: {}", e))?;

        Ok(FileInfo {
            size: metadata.len(),
            modified: metadata.modified().ok(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
        })
    });

    // Convert result to expected type for audit logging
    let audit_result: Result<PathBuf, SecurityError> = result
        .as_ref()
        .map(|_| Path::new(&path).to_path_buf())
        .map_err(|e| SecurityError::PathOutsideWorkspace {
            path: e.to_string(),
        });

    SecurityAuditor::log_file_access_attempt(
        Path::new(&path),
        "get_file_info",
        &audit_result,
        "development",
        None,
    );

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => VALIDATION_METRICS.record_success(duration),
        Err(_) => VALIDATION_METRICS.record_failure(duration),
    }

    let final_result = result.map_err(|e| CommandError::SecurityError {
        message: e.to_string(),
        code: "CIRCUIT_BREAKER".to_string(),
        severity: "medium".to_string(),
    });

    // Backup security configurations after successful operation
    if final_result.is_ok() {
        backup_security_configurations();
    }

    final_result
}

#[tauri::command]
pub async fn import_file(path: PathBuf) -> Result<PathBuf, CommandError> {
    let start_time = Instant::now();

    // First check emergency restrictions
    let emergency_manager = EmergencyManager::new().map_err(|e| CommandError::SecurityError {
        message: format!("Emergency system error: {}", e),
        code: "EMERGENCY_ERROR".to_string(),
        severity: "high".to_string(),
    })?;

    if emergency_manager.is_kill_switch_active() {
        return Err(CommandError::SecurityError {
            message: "Kill switch active - all operations disabled".to_string(),
            code: "KILL_SWITCH_ACTIVE".to_string(),
            severity: "critical".to_string(),
        });
    }

    if !emergency_manager.can_perform_operation("import") {
        return Err(CommandError::SecurityError {
            message: "Operation blocked by emergency restrictions".to_string(),
            code: "EMERGENCY_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Then check safe mode restrictions
    if !SafeModeManager::global()
        .is_file_allowed(&path)
        .map_err(|e| CommandError::SecurityError {
            message: format!("Safe mode restriction: {}", e),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        })?
    {
        return Err(CommandError::SecurityError {
            message: "File blocked by safe mode restrictions".to_string(),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Use circuit breaker for import operation
    let breaker = CIRCUIT_BREAKER_MANAGER.get_or_create(
        "tauri_file_import",
        Some(CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: std::time::Duration::from_secs(30),
            success_threshold: 2,
        }),
    );

    let validator = create_default_validator();
    let result = breaker.call(|| {
        validator
            .validate_import_path(&path)
            .map_err(|e| anyhow::anyhow!("{}", e))
    });

    // Convert result to expected type for audit logging
    let audit_result: Result<PathBuf, SecurityError> = result
        .as_ref()
        .map(|_| path.clone())
        .map_err(|e| SecurityError::PathOutsideWorkspace {
            path: e.to_string(),
        });

    SecurityAuditor::log_file_access_attempt(
        &path,
        "import_file",
        &audit_result,
        "development",
        None,
    );

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => VALIDATION_METRICS.record_success(duration),
        Err(_) => VALIDATION_METRICS.record_failure(duration),
    }

    let final_result = result.map_err(|e| CommandError::SecurityError {
        message: e.to_string(),
        code: "CIRCUIT_BREAKER".to_string(),
        severity: "medium".to_string(),
    });

    // Backup security configurations after successful import
    if final_result.is_ok() {
        backup_security_configurations();
    }

    final_result
}

// ---------------- Testable Variants with Custom Validator ----------------

#[allow(dead_code)]
pub async fn validate_file_for_import_with_validator(
    path: &Path,
    validator: &PathValidator,
) -> Result<PathBuf, SecurityError> {
    let result = validator.validate_import_path(path);
    SecurityAuditor::log_file_access_attempt(path, "validate_import", &result, "development", None);
    result
}

#[allow(dead_code)]
pub async fn get_file_info_secure_with_validator(
    path: &Path,
    validator: &PathValidator,
) -> Result<FileInfo, SecurityError> {
    let validated_path = validator.validate_import_path(path)?;
    let metadata =
        fs::metadata(&validated_path).map_err(|e| SecurityError::PathOutsideWorkspace {
            path: e.to_string(),
        })?;
    Ok(FileInfo {
        size: metadata.len(),
        modified: metadata.modified().ok(),
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
    })
}

#[allow(dead_code)]
pub async fn import_file_with_validator(
    path: &Path,
    validator: &PathValidator,
) -> Result<PathBuf, SecurityError> {
    validator.validate_import_path(path)
}

// ---------------- File Watcher Commands ----------------

#[tauri::command]
pub async fn start_file_watching(workspace_path: String) -> Result<(), CommandError> {
    let mut watcher_guard = FILE_WATCHER.lock().await;
    let mut receiver_guard = EVENT_RECEIVER.lock().await;

    // If watcher already exists, stop it first
    if watcher_guard.is_some() {
        *watcher_guard = None;
        *receiver_guard = None;
    }

    // Create new watcher with default config
    let config = WatcherConfig::default();
    let (mut watcher, receiver) = DocumentWatcher::new(config);

    // Start the watcher
    watcher
        .start()
        .await
        .map_err(|e| CommandError::Custom(format!("Failed to start file watcher: {}", e)))?;

    // Add the workspace path with security validation
    watcher
        .add_path(&workspace_path)
        .await
        .map_err(|e| CommandError::Custom(format!("Failed to add workspace path: {}", e)))?;

    // Store the watcher and receiver
    *watcher_guard = Some(watcher);
    *receiver_guard = Some(receiver);

    tracing::info!("File watcher started for workspace: {}", workspace_path);
    Ok(())
}

#[tauri::command]
pub async fn pause_file_watching() -> Result<(), CommandError> {
    let watcher_guard = FILE_WATCHER.lock().await;

    if let Some(watcher) = watcher_guard.as_ref() {
        watcher.pause().await;
        tracing::info!("File watching paused");
        Ok(())
    } else {
        Err(CommandError::Custom("File watcher not running".to_string()))
    }
}

#[tauri::command]
pub async fn resume_file_watching() -> Result<(), CommandError> {
    let watcher_guard = FILE_WATCHER.lock().await;

    if let Some(watcher) = watcher_guard.as_ref() {
        watcher.resume().await;
        tracing::info!("File watching resumed");
        Ok(())
    } else {
        Err(CommandError::Custom("File watcher not running".to_string()))
    }
}

#[tauri::command]
pub async fn stop_file_watching() -> Result<(), CommandError> {
    let mut watcher_guard = FILE_WATCHER.lock().await;
    let mut receiver_guard = EVENT_RECEIVER.lock().await;

    *watcher_guard = None;
    *receiver_guard = None;

    tracing::info!("File watching stopped");
    Ok(())
}

#[tauri::command]
pub async fn get_watched_paths() -> Result<Vec<String>, CommandError> {
    let watcher_guard = FILE_WATCHER.lock().await;

    if let Some(watcher) = watcher_guard.as_ref() {
        let paths = watcher.watched_paths().await;
        let path_strings = paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        Ok(path_strings)
    } else {
        Err(CommandError::Custom("File watcher not running".to_string()))
    }
}

#[tauri::command]
pub async fn add_watch_path(path: String) -> Result<(), CommandError> {
    let mut watcher_guard = FILE_WATCHER.lock().await;

    if let Some(watcher) = watcher_guard.as_mut() {
        watcher
            .add_path(&path)
            .await
            .map_err(|e| CommandError::Custom(format!("Failed to add path: {}", e)))?;
        tracing::info!("Added path to watch: {}", path);
        Ok(())
    } else {
        Err(CommandError::Custom("File watcher not running".to_string()))
    }
}

#[tauri::command]
pub async fn remove_watch_path(path: String) -> Result<(), CommandError> {
    let mut watcher_guard = FILE_WATCHER.lock().await;

    if let Some(watcher) = watcher_guard.as_mut() {
        watcher
            .remove_path(&path)
            .await
            .map_err(|e| CommandError::Custom(format!("Failed to remove path: {}", e)))?;
        tracing::info!("Removed path from watch: {}", path);
        Ok(())
    } else {
        Err(CommandError::Custom("File watcher not running".to_string()))
    }
}

#[tauri::command]
pub async fn emit_file_event(event: FileEvent) -> Result<(), CommandError> {
    // This command is used by the frontend to emit file events
    // that will be processed by the watcher system
    tracing::debug!("File event emitted: {:?}", event);
    Ok(())
}

// ---------------- Tests ----------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_file_watcher_lifecycle() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let test_path = temp_dir.path().to_string_lossy().to_string();

        // Test starting the watcher
        let result = start_file_watching(test_path.clone()).await;
        assert!(
            result.is_ok(),
            "Failed to start file watching: {:?}",
            result
        );

        // Test getting watched paths
        let paths = get_watched_paths().await;
        assert!(paths.is_ok(), "Failed to get watched paths: {:?}", paths);
        let paths = paths.unwrap();
        assert!(paths.contains(&test_path), "Test path not in watched paths");

        // Test adding another path
        let another_path = temp_dir.path().join("subdir").to_string_lossy().to_string();
        std::fs::create_dir(&another_path).expect("Failed to create subdirectory");
        let add_result = add_watch_path(another_path.clone()).await;
        assert!(
            add_result.is_ok(),
            "Failed to add watch path: {:?}",
            add_result
        );

        // Test getting updated paths
        let updated_paths = get_watched_paths().await.unwrap();
        assert_eq!(updated_paths.len(), 2, "Should have 2 watched paths");
        assert!(
            updated_paths.contains(&another_path),
            "Second path not found"
        );

        // Test removing a path
        let remove_result = remove_watch_path(another_path.clone()).await;
        assert!(
            remove_result.is_ok(),
            "Failed to remove watch path: {:?}",
            remove_result
        );

        // Test getting final paths
        let final_paths = get_watched_paths().await.unwrap();
        assert_eq!(
            final_paths.len(),
            1,
            "Should have 1 watched path after removal"
        );
        assert!(
            final_paths.contains(&test_path),
            "Original path not preserved"
        );

        // Test stopping the watcher
        let stop_result = stop_file_watching().await;
        assert!(
            stop_result.is_ok(),
            "Failed to stop file watching: {:?}",
            stop_result
        );

        // Test that watcher is no longer running
        let paths_after_stop = get_watched_paths().await;
        assert!(paths_after_stop.is_err(), "Watcher should be stopped");
    }

    #[tokio::test]
    async fn test_pause_resume_watcher() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let test_path = temp_dir.path().to_string_lossy().to_string();

        // Start the watcher
        start_file_watching(test_path.clone()).await.unwrap();

        // Pause the watcher
        let pause_result = pause_file_watching().await;
        assert!(
            pause_result.is_ok(),
            "Failed to pause file watching: {:?}",
            pause_result
        );

        // Resume the watcher
        let resume_result = resume_file_watching().await;
        assert!(
            resume_result.is_ok(),
            "Failed to resume file watching: {:?}",
            resume_result
        );

        // Clean up
        stop_file_watching().await.unwrap();
    }

    #[tokio::test]
    async fn test_emit_file_event() {
        let event = FileEvent::Created(PathBuf::from("/test/path/file.txt"));

        let result = emit_file_event(event).await;
        assert!(result.is_ok(), "Failed to emit file event: {:?}", result);
    }
}
